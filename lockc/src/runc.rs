use std::{collections::HashMap, fs, io, os::unix::fs::PermissionsExt, path::Path, string::String};

use fanotify::{
    high_level::{Event, Fanotify, FanotifyMode, FanotifyResponse},
    low_level::FAN_OPEN_EXEC_PERM,
};
use k8s_openapi::api::core::v1;
use nix::poll::{poll, PollFd, PollFlags};
use procfs::{process::Process, ProcError};
use scopeguard::defer;
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;
use tokio::{
    runtime::Builder,
    sync::{mpsc, oneshot},
};
use tracing::{debug, error};

use crate::{
    bpfstructs::{
        container_policy_level, container_policy_level_POLICY_LEVEL_BASELINE,
        container_policy_level_POLICY_LEVEL_PRIVILEGED,
        container_policy_level_POLICY_LEVEL_RESTRICTED,
    },
    communication::EbpfCommand,
    maps::MapOperationError,
};

// static LABEL_NAMESPACE: &str = "io.kubernetes.pod.namespace";
static LABEL_POLICY_ENFORCE: &str = "pod-security.kubernetes.io/enforce";
// static LABEL_POLICY_AUDIT: &str = "pod-security.kubernetes.io/audit";
// static LABEL_POLICY_WARN: &str = "pod-security.kubernetes.io/warn";

static ANNOTATION_CONTAINERD_LOG_DIRECTORY: &str = "io.kubernetes.cri.sandbox-log-directory";
static ANNOTATION_CONTAINERD_SANDBOX_ID: &str = "io.kubernetes.cri.sandbox-id";

/// Type of Kubernetes container determined by annotations.
enum KubernetesContainerType {
    /// Containerd CRI, main container with own log directory.
    ContainerdMain,
    /// Containerd CRI, part of another sandbox which has its own log
    /// directory.
    ContainerdPartOfSandbox,
    /// Unknown type of Kubernetes annotations.
    Unknown,
}

fn kubernetes_type(annotations: HashMap<String, String>) -> KubernetesContainerType {
    if annotations.contains_key(ANNOTATION_CONTAINERD_LOG_DIRECTORY) {
        return KubernetesContainerType::ContainerdMain;
    } else if annotations.contains_key(ANNOTATION_CONTAINERD_SANDBOX_ID) {
        return KubernetesContainerType::ContainerdPartOfSandbox;
    }
    KubernetesContainerType::Unknown
}

/// Type of container by engine/runtime.
enum ContainerType {
    Docker,
    KubernetesContainerd,
    Unknown,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Mount {
    source: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContainerConfig {
    mounts: Vec<Mount>,
    annotations: Option<HashMap<String, String>>,
}

#[derive(Error, Debug)]
pub enum ContainerError {
    #[error(transparent)]
    Status(#[from] io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("could not get the file name of container log file")]
    LogFileName,

    #[error("could not parse k8s namespace")]
    K8sNamespace,
}

fn container_type_data<P: AsRef<std::path::Path>>(
    container_bundle: P,
) -> Result<(ContainerType, Option<std::string::String>), ContainerError> {
    let bundle_path = container_bundle.as_ref();
    let config_path = bundle_path.join("config.json");
    let f = fs::File::open(config_path.clone())?;
    let r = io::BufReader::new(f);

    let config: ContainerConfig = serde_json::from_reader(r)?;

    // Kubernetes
    if let Some(annotations) = config.annotations {
        debug!(
            bundle = ?bundle_path,
            config = ?config_path,
            "detected kubernetes container",
        );
        match kubernetes_type(annotations.clone()) {
            KubernetesContainerType::ContainerdMain => {
                // containerd doesn't expose k8s namespaces directly. They have
                // to be parsed from the log directory path, where the first
                // part of the filename is the namespace.
                let log_directory = &annotations[ANNOTATION_CONTAINERD_LOG_DIRECTORY];
                debug!(
                    log_directory = log_directory.as_str(),
                    "detected k8s+containerd container",
                );
                let log_path = std::path::PathBuf::from(log_directory);
                let file_name = log_path
                    .file_name()
                    .ok_or(ContainerError::LogFileName)?
                    .to_str()
                    .ok_or(ContainerError::LogFileName)?;
                let mut splitter = file_name.split('_');
                let namespace = splitter
                    .next()
                    .ok_or(ContainerError::K8sNamespace)?
                    .to_string();

                return Ok((ContainerType::KubernetesContainerd, Some(namespace)));
            }
            KubernetesContainerType::ContainerdPartOfSandbox => {
                // When a container is running as a part of a previously created
                // pod, the log directory path has to be retrieved from the
                // sandbox container.
                let sandbox_id = &annotations[ANNOTATION_CONTAINERD_SANDBOX_ID];
                debug!(
                    sandbox_id = sandbox_id.as_str(),
                    "detected k8s+containerd container",
                );

                // Go one directory up from the current bundle.
                let mut ancestors = bundle_path.ancestors();
                ancestors.next();
                if let Some(v) = ancestors.next() {
                    // Then go to sandbox_id directory (sandbox's bundle).
                    let new_bundle = v.join(sandbox_id);
                    return container_type_data(new_bundle);
                }
            }
            KubernetesContainerType::Unknown => {}
        }
        // TODO(vadorovsky): Support more Kubernetes CRI implementations.
        // They all come with their own annotations, so we will have to
        // handle more keys here.
    }

    // Docker
    for mount in config.mounts {
        let source: Vec<&str> = mount.source.split('/').collect();
        if source.len() > 1 && source[source.len() - 1] == "hostname" {
            let config_v2 = str::replace(&mount.source, "hostname", "config.v2.json");
            debug!(
                config_path = config_v2.as_str(),
                "detected docker container"
            );
            return Ok((ContainerType::Docker, Some(config_v2)));
        }
    }

    Ok((ContainerType::Unknown, None))
}

/// Finds the policy for the given Kubernetes namespace. If none, the baseline
/// policy is returned. Otherwise checks the Kubernetes namespace labels.
async fn policy_kubernetes(namespace: String) -> Result<container_policy_level, kube::Error> {
    // Apply the privileged policy for kube-system containers immediately.
    // Otherwise the core k8s components (apiserver, scheduler) won't be able
    // to run.
    // If container has no k8s namespace, apply the baseline policy.
    if namespace.as_str() == "kube-system" {
        return Ok(container_policy_level_POLICY_LEVEL_PRIVILEGED);
    }

    let client = kube::Client::try_default().await?;

    let namespaces: kube::api::Api<v1::Namespace> = kube::api::Api::all(client);
    let namespace = namespaces.get(&namespace).await?;

    match namespace.metadata.labels {
        Some(v) => match v.get(LABEL_POLICY_ENFORCE) {
            Some(v) => match v.as_str() {
                "restricted" => Ok(container_policy_level_POLICY_LEVEL_RESTRICTED),
                "baseline" => Ok(container_policy_level_POLICY_LEVEL_BASELINE),
                "privileged" => Ok(container_policy_level_POLICY_LEVEL_PRIVILEGED),
                _ => Ok(container_policy_level_POLICY_LEVEL_BASELINE),
            },
            None => Ok(container_policy_level_POLICY_LEVEL_BASELINE),
        },
        None => Ok(container_policy_level_POLICY_LEVEL_BASELINE),
    }
}

#[derive(Error, Debug)]
pub enum PolicyKubernetesSyncError {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    Kube(#[from] kube::Error),
}

/// Makes the `policy_label_sync` function synchronous. We use it together with
/// poll(2) syscall, which is definitely not meant for multithreaded code.
fn policy_kubernetes_sync(
    namespace: String,
) -> Result<container_policy_level, PolicyKubernetesSyncError> {
    match Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(policy_kubernetes(namespace))
    {
        Ok(p) => Ok(p),
        Err(e) => Err(PolicyKubernetesSyncError::from(e)),
    }
}

fn policy_docker<P: AsRef<Path>>(
    docker_bundle: P,
) -> Result<container_policy_level, ContainerError> {
    let config_path = docker_bundle.as_ref();
    let f = std::fs::File::open(config_path)?;
    let r = std::io::BufReader::new(f);

    let l: Value = serde_json::from_reader(r)?;

    let x = l["Config"]["Labels"]["org.lockc.policy"].as_str();

    match x {
        Some(x) => match x {
            "restricted" => Ok(container_policy_level_POLICY_LEVEL_RESTRICTED),
            "baseline" => Ok(container_policy_level_POLICY_LEVEL_BASELINE),
            "privileged" => Ok(container_policy_level_POLICY_LEVEL_PRIVILEGED),
            _ => Ok(container_policy_level_POLICY_LEVEL_BASELINE),
        },
        None => Ok(container_policy_level_POLICY_LEVEL_BASELINE),
    }
}

enum ShimOptParsingAction {
    NoPositional,
    Skip,
    ContainerId,
}

enum ShimContainerAction {
    Other,
    Delete,
}

/// Types of options (prepositioned by `--`).
enum OptParsingAction {
    /// Option not followed by a positional argument.
    NoPositional,
    /// Option followed by a positional argument we don't want to store.
    Skip,
    /// --bundle option which we want to store.
    Bundle,
}

/// Types of positional arguments.
enum ArgParsingAction {
    /// Argument we don't want to store.
    None,
    /// Container ID which we want to store.
    ContainerId,
}

/// Types of actions performed on the container, defined by a runc subcommand.
enum ContainerAction {
    /// Types we don't explicitly handle, except of registering the process as
    /// containerized.
    Other,
    /// Action of creating the container, when we want to register the new
    /// container.
    Create,
    /// Action of deleting the container, when we want to remove the registered
    /// container.
    Delete,
}

pub struct RuncWatcher {
    bootstrap_rx: oneshot::Receiver<()>,
    ebpf_tx: mpsc::Sender<EbpfCommand>,
    fd: Fanotify,
}

#[derive(Error, Debug)]
pub enum HandleRuncEventError {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    Errno(#[from] nix::errno::Errno),

    #[error(transparent)]
    CommandSend(#[from] mpsc::error::SendError<EbpfCommand>),

    #[error(transparent)]
    CommandRecv(#[from] oneshot::error::RecvError),

    #[error(transparent)]
    BootstrapTryRecv(#[from] oneshot::error::TryRecvError),

    #[error(transparent)]
    Proc(#[from] ProcError),

    #[error(transparent)]
    Container(#[from] ContainerError),

    #[error(transparent)]
    PolicyKubernetes(#[from] PolicyKubernetesSyncError),

    #[error(transparent)]
    MapOperation(#[from] MapOperationError),

    #[error("container data missing")]
    ContainerData,

    #[error("container ID missing")]
    ContainerID,
}

impl RuncWatcher {
    pub fn new(
        bootstrap_rx: oneshot::Receiver<()>,
        ebpf_tx: mpsc::Sender<EbpfCommand>,
    ) -> Result<Self, io::Error> {
        let runc_paths = vec![
            "/usr/bin/runc",
            "/usr/sbin/runc",
            "/usr/local/bin/runc",
            "/usr/local/sbin/runc",
            "/host/usr/bin/runc",
            "/host/usr/sbin/runc",
            "/host/usr/local/bin/runc",
            "/host/usr/local/sbin/runc",
        ];
        let fd = Fanotify::new_with_blocking(FanotifyMode::CONTENT);

        for runc_path in runc_paths {
            debug!(path = runc_path, "checking runc");
            let p = Path::new(&runc_path);
            if p.exists() {
                let metadata = p.metadata()?;

                // When the source for host mount in Kubernetes does not
                // exists, an empty directory is created. Also, directories
                // contain an executable bit. Skip directories before any other
                // checks.
                if metadata.is_dir() {
                    continue;
                }

                // If the file is executable.
                if metadata.permissions().mode() & 0o111 != 0 {
                    debug!(path = runc_path, "excecutable runc binary found");
                    fd.add_path(FAN_OPEN_EXEC_PERM, runc_path)?;
                    debug!(path = runc_path, "added runc to fanotify");
                }
            }
        }

        Ok(RuncWatcher {
            bootstrap_rx,
            ebpf_tx,
            fd,
        })
    }

    async fn add_container(
        &self,
        container_id: String,
        pid: i32,
        policy_level: container_policy_level,
    ) -> Result<(), HandleRuncEventError> {
        let (responder_tx, responder_rx) = oneshot::channel();

        self.ebpf_tx
            .send(EbpfCommand::AddContainer {
                container_id,
                pid,
                policy_level,
                responder_tx,
            })
            .await?;
        responder_rx.await??;

        Ok(())
    }

    fn add_container_sync(
        &self,
        container_id: String,
        pid: i32,
        policy_level: container_policy_level,
    ) -> Result<(), HandleRuncEventError> {
        debug!(container_id = container_id.as_str(), "adding container");

        Builder::new_current_thread()
            .build()?
            .block_on(self.add_container(container_id, pid, policy_level))
    }

    async fn delete_container(&self, container_id: String) -> Result<(), HandleRuncEventError> {
        let (responder_tx, responder_rx) = oneshot::channel();

        self.ebpf_tx
            .send(EbpfCommand::DeleteContainer {
                container_id,
                responder_tx,
            })
            .await?;
        responder_rx.await??;

        Ok(())
    }

    fn delete_container_sync(&self, container_id: String) -> Result<(), HandleRuncEventError> {
        debug!(container_id = container_id.as_str(), "deleting container");

        Builder::new_current_thread()
            .build()?
            .block_on(self.delete_container(container_id))
    }

    async fn add_process(
        &self,
        container_id: String,
        pid: i32,
    ) -> Result<(), HandleRuncEventError> {
        let (responder_tx, responder_rx) = oneshot::channel();

        self.ebpf_tx
            .send(EbpfCommand::AddProcess {
                container_id,
                pid,
                responder_tx,
            })
            .await?;
        responder_rx.await??;

        Ok(())
    }

    fn add_process_sync(&self, container_id: String, pid: i32) -> Result<(), HandleRuncEventError> {
        debug!(
            container = container_id.as_str(),
            pid = pid,
            "adding process"
        );

        Builder::new_current_thread()
            .build()?
            .block_on(self.add_process(container_id, pid))
    }

    fn handle_containerd_shim_event(
        &self,
        containerd_shim_process: Process,
    ) -> Result<(), HandleRuncEventError> {
        let mut opt_parsing_action = ShimOptParsingAction::NoPositional;
        let mut container_action = ShimContainerAction::Other;

        let mut container_id_o: Option<String> = None;

        for arg in containerd_shim_process.cmdline()? {
            debug!(argument = arg.as_str(), "containerd-shim");
            match arg.as_str() {
                "-address" => opt_parsing_action = ShimOptParsingAction::Skip,
                "-bundle" => opt_parsing_action = ShimOptParsingAction::Skip,
                "-id" => opt_parsing_action = ShimOptParsingAction::ContainerId,
                "-namespace" => opt_parsing_action = ShimOptParsingAction::Skip,
                "-publish-binary" => opt_parsing_action = ShimOptParsingAction::Skip,
                _ => {}
            }
            if arg.starts_with('-') {
                continue;
            }

            match opt_parsing_action {
                ShimOptParsingAction::NoPositional => {}
                ShimOptParsingAction::Skip => {
                    opt_parsing_action = ShimOptParsingAction::NoPositional;
                    continue;
                }
                ShimOptParsingAction::ContainerId => {
                    container_id_o = Some(arg);
                    opt_parsing_action = ShimOptParsingAction::NoPositional;
                    continue;
                }
            }

            if arg.as_str() == "delete" {
                container_action = ShimContainerAction::Delete
            }
        }

        match container_action {
            ShimContainerAction::Other => {}
            ShimContainerAction::Delete => {
                let container_id = container_id_o.ok_or(HandleRuncEventError::ContainerID)?;
                debug!(container = container_id.as_str(), "deleting container");

                self.delete_container_sync(container_id)?;
            }
        }

        Ok(())
    }

    fn handle_runc_event(&self, runc_process: Process) -> Result<(), HandleRuncEventError> {
        let mut opt_parsing_action = OptParsingAction::NoPositional;
        let mut arg_parsing_action = ArgParsingAction::None;
        let mut container_action = ContainerAction::Other;

        let mut container_bundle_o: Option<String> = None;
        let mut container_id_o: Option<String> = None;

        // for arg in cmdline.split(CMDLINE_DELIMITER) {
        for arg in runc_process.cmdline()? {
            debug!(argument = arg.as_str(), "runc");
            match arg.as_str() {
                // Options which are followed with a positional arguments we don't
                // want to store.
                "--log" => opt_parsing_action = OptParsingAction::Skip,
                "--log-format" => opt_parsing_action = OptParsingAction::Skip,
                "--pid-file" => opt_parsing_action = OptParsingAction::Skip,
                "--process" => opt_parsing_action = OptParsingAction::Skip,
                "--console-socket" => opt_parsing_action = OptParsingAction::Skip,
                "--root" => opt_parsing_action = OptParsingAction::Skip,
                // We want to explicitly store the value of --bundle and --root
                // options.
                "--bundle" => opt_parsing_action = OptParsingAction::Bundle,
                _ => {}
            }
            if arg.starts_with('-') {
                // After handling the option, start parsing the next argument.
                continue;
            }

            match opt_parsing_action {
                OptParsingAction::NoPositional => {}
                OptParsingAction::Skip => {
                    opt_parsing_action = OptParsingAction::NoPositional;
                    continue;
                }
                OptParsingAction::Bundle => {
                    container_bundle_o = Some(arg);
                    opt_parsing_action = OptParsingAction::NoPositional;
                    continue;
                }
            }
            match arg_parsing_action {
                ArgParsingAction::None => {}
                ArgParsingAction::ContainerId => {
                    container_id_o = Some(arg);
                    arg_parsing_action = ArgParsingAction::None;
                    continue;
                }
            }

            match arg.as_str() {
                "checkpoint" => arg_parsing_action = ArgParsingAction::ContainerId,
                "create" => {
                    arg_parsing_action = ArgParsingAction::ContainerId;
                    container_action = ContainerAction::Create;
                }
                "delete" => {
                    arg_parsing_action = ArgParsingAction::ContainerId;
                    container_action = ContainerAction::Delete;
                }
                "events" => arg_parsing_action = ArgParsingAction::ContainerId,
                "exec" => arg_parsing_action = ArgParsingAction::ContainerId,
                "kill" => arg_parsing_action = ArgParsingAction::ContainerId,
                "pause" => arg_parsing_action = ArgParsingAction::ContainerId,
                "ps" => arg_parsing_action = ArgParsingAction::ContainerId,
                "restore" => arg_parsing_action = ArgParsingAction::ContainerId,
                "resume" => arg_parsing_action = ArgParsingAction::ContainerId,
                "run" => arg_parsing_action = ArgParsingAction::ContainerId,
                "start" => {
                    arg_parsing_action = ArgParsingAction::ContainerId;
                }
                "state" => arg_parsing_action = ArgParsingAction::ContainerId,
                "update" => arg_parsing_action = ArgParsingAction::ContainerId,
                _ => {}
            }
        }

        match container_action {
            ContainerAction::Other => {
                debug!("other container action");
                if let Some(container_id) = container_id_o {
                    self.add_process_sync(container_id, runc_process.pid)?;
                }
            }
            ContainerAction::Create => {
                let container_id = container_id_o.ok_or(HandleRuncEventError::ContainerID)?;
                let container_bundle = match container_bundle_o {
                    Some(v) => std::path::PathBuf::from(v),
                    None => std::env::current_dir()?,
                };

                // let policy;
                let (container_type, container_data) = container_type_data(container_bundle)?;
                let policy: container_policy_level = match container_type {
                    ContainerType::Docker => {
                        policy_docker(container_data.ok_or(HandleRuncEventError::ContainerData)?)?
                    }
                    ContainerType::KubernetesContainerd => policy_kubernetes_sync(
                        container_data.ok_or(HandleRuncEventError::ContainerData)?,
                    )?,
                    ContainerType::Unknown => container_policy_level_POLICY_LEVEL_BASELINE,
                };

                self.add_container_sync(container_id, runc_process.pid, policy)?;
            }
            ContainerAction::Delete => {
                let container_id = container_id_o.ok_or(HandleRuncEventError::ContainerID)?;
                self.delete_container_sync(container_id)?;
            }
        }

        Ok(())
    }

    fn handle_event(&self, event: Event) -> Result<(), HandleRuncEventError> {
        // Let the process execute again
        defer!(self.fd.send_response(event.fd, FanotifyResponse::Allow));

        debug!(
            path = event.path.as_str(),
            pid = event.pid,
            "received fanotify event"
        );

        let p = Process::new(event.pid)?;

        // Usually fanotify receives two notifications about executing runc:
        // 1) from containerd-shim (or similar)
        // 2) from runc
        // We are interested in parsing only runc arguments rather than
        // containerd-shim.
        let comm = p.stat()?.comm;
        match comm.as_str() {
            "runc" => {
                self.handle_runc_event(p)?;
            }
            "containerd-shim" => {
                self.handle_containerd_shim_event(p)?;
            }
            _ => {}
        }

        Ok(())
    }

    pub fn work_loop(&mut self) -> Result<(), HandleRuncEventError> {
        // Wait for the bootstrap request from the main, asynchronous part of
        // lockc.
        loop {
            match self.bootstrap_rx.try_recv() {
                Ok(_) => {
                    break;
                }
                Err(oneshot::error::TryRecvError::Empty) => {
                    // Keep waiting.
                }
                Err(e) => return Err(HandleRuncEventError::from(e)),
            }
        }

        debug!("starting work loop");

        let mut fds = [PollFd::new(self.fd.as_raw_fd(), PollFlags::POLLIN)];
        loop {
            let poll_num = poll(&mut fds, -1)?;
            if poll_num > 0 {
                for event in self.fd.read_event() {
                    match self.handle_event(event) {
                        Ok(_) => {}
                        Err(e) => error!(error = e.to_string().as_str(), "failed to handle event"),
                    };
                }
            } else {
                debug!("poll_num <= 0!");
                break;
            }
        }

        Ok(())
    }
}
