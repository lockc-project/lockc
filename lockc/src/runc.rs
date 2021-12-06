use std::{collections::HashMap, fs, io, os::unix::fs::PermissionsExt, path::Path, string::String};

use fanotify::{
    high_level::{Event, Fanotify, FanotifyMode, FanotifyResponse},
    low_level::FAN_OPEN_EXEC_PERM,
};
use k8s_openapi::api::core::v1;
use log::{debug, error};
use nix::poll::{poll, PollFd, PollFlags};
use procfs::{process::Process, ProcError};
use scopeguard::defer;
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;
use tokio::runtime::Builder;

use crate::{
    bpfstructs::{
        container_policy_level, container_policy_level_POLICY_LEVEL_BASELINE,
        container_policy_level_POLICY_LEVEL_PRIVILEGED,
        container_policy_level_POLICY_LEVEL_RESTRICTED,
    },
    hash, HashError,
};
use lockc_uprobes::{add_container, add_process, delete_container};

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

// NOTE(vadorovsky): Rust complains about `mounts` field not being used, but it
// IS FRICKING USED in the `container_type_data` function. Bug in the compiler?
#[allow(dead_code)]
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
            "detected kubernetes container with bundle {}, config {}",
            bundle_path.display(),
            config_path.display(),
        );
        match kubernetes_type(annotations.clone()) {
            KubernetesContainerType::ContainerdMain => {
                // containerd doesn't expose k8s namespaces directly. They have
                // to be parsed from the log directory path, where the first
                // part of the filename is the namespace.
                let log_directory = &annotations[ANNOTATION_CONTAINERD_LOG_DIRECTORY];
                debug!(
                    "detected k8s+containerd container with log directory {}",
                    log_directory
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
                    "detected k8s+containerd container with sandbox id {}",
                    sandbox_id
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
            debug!("detected docker container with config path {}", config_v2);
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Mounts {
    mounts: Vec<Mount>,
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

#[derive(Error, Debug)]
pub enum UprobeError {
    #[error("failed to call into uprobe, BPF programs are most likely not running")]
    Call,

    #[error("BPF program error")]
    BPF,

    #[error("unknown uprobe error")]
    Unknown,
}

fn check_uprobe_ret(ret: i32) -> Result<(), UprobeError> {
    match ret {
        0 => Ok(()),
        n if n == -libc::EAGAIN => Err(UprobeError::Call),
        n if n == -libc::EINVAL => Err(UprobeError::BPF),
        _ => Err(UprobeError::Unknown),
    }
}

pub struct RuncWatcher {
    fd: Fanotify,
}

#[derive(Error, Debug)]
pub enum HandleRuncEventError {
    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    Errno(#[from] nix::errno::Errno),

    #[error(transparent)]
    Proc(#[from] ProcError),

    #[error(transparent)]
    Container(#[from] ContainerError),

    #[error(transparent)]
    Hash(#[from] HashError),

    #[error(transparent)]
    PolicyKubernetes(#[from] PolicyKubernetesSyncError),

    #[error(transparent)]
    Uprobe(#[from] UprobeError),

    #[error("container data missing")]
    ContainerData,

    #[error("container ID missing")]
    ContainerID,
}

impl RuncWatcher {
    pub fn new() -> Result<Self, io::Error> {
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
        let fd = Fanotify::new_with_nonblocking(FanotifyMode::CONTENT);

        for runc_path in runc_paths {
            debug!("checking runc path {}", runc_path);
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
                    debug!(
                        "runc path {} exists and is an excecutable binary",
                        runc_path
                    );
                    fd.add_path(FAN_OPEN_EXEC_PERM, runc_path)?;
                    debug!("added runc path {} to fanotify", runc_path);
                }
            }
        }

        Ok(RuncWatcher { fd })
    }

    fn handle_containerd_shim_event(
        &self,
        containerd_shim_process: Process,
    ) -> Result<(), HandleRuncEventError> {
        let mut opt_parsing_action = ShimOptParsingAction::NoPositional;
        let mut container_action = ShimContainerAction::Other;

        let mut container_id_o: Option<String> = None;

        for arg in containerd_shim_process.cmdline()? {
            debug!("containerd-shim argument: {}", arg);
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
                let container_key =
                    hash(&container_id_o.ok_or(HandleRuncEventError::ContainerID)?)?;
                debug!("deleting container with key {}", container_key);

                let mut ret: i32 = -libc::EAGAIN;
                delete_container(&mut ret as *mut i32, container_key);
                check_uprobe_ret(ret)?;
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
            debug!("runc argument: {}", arg);
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
                if let Some(v) = container_id_o {
                    let container_key = hash(&v)?;

                    let mut ret: i32 = -libc::EAGAIN;
                    add_process(&mut ret as *mut i32, container_key, runc_process.pid);
                    check_uprobe_ret(ret)?;
                }
            }
            ContainerAction::Create => {
                let container_id = container_id_o.ok_or(HandleRuncEventError::ContainerID)?;
                let container_key = hash(&container_id)?;
                debug!(
                    "creating containerd with id {} key {}",
                    container_id, container_key
                );
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

                let mut ret: i32 = -libc::EAGAIN;
                add_container(
                    &mut ret as *mut i32,
                    container_key,
                    runc_process.pid,
                    policy,
                );
                check_uprobe_ret(ret)?;
            }
            ContainerAction::Delete => {
                let container_id = container_id_o.ok_or(HandleRuncEventError::ContainerID)?;
                let container_key = hash(&container_id)?;
                debug!(
                    "deleting container with id {} key {}",
                    container_id, container_key
                );

                let mut ret: i32 = -libc::EAGAIN;
                delete_container(&mut ret as *mut i32, container_key);
                check_uprobe_ret(ret)?;
            }
        }

        Ok(())
    }

    fn handle_event(&self, event: Event) -> Result<(), HandleRuncEventError> {
        // Let the process execute again
        defer!(self.fd.send_response(event.fd, FanotifyResponse::Allow));

        debug!("received fanotify event: {:#?}", event);

        let p = Process::new(event.pid)?;

        // Usually fanotify receives two notifications about executing runc:
        // 1) from containerd-shim (or similar)
        // 2) from runc
        // We are interested in parsing only runc arguments rather than
        // containerd-shim.
        let comm = p.stat()?.comm;
        debug!("event's process comm: {}", comm);
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

    pub fn work_loop(&self) -> Result<(), HandleRuncEventError> {
        let mut fds = [PollFd::new(self.fd.as_raw_fd(), PollFlags::POLLIN)];
        loop {
            let poll_num = poll(&mut fds, -1)?;
            if poll_num > 0 {
                for event in self.fd.read_event() {
                    match self.handle_event(event) {
                        Ok(_) => {}
                        Err(e) => {
                            error!("failed to handle event: {}", e);
                        }
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
