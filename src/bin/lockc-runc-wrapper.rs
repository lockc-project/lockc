use k8s_openapi::api::core::v1;
use std::convert::TryFrom;

// TODO: To be used for cri-o.
// static ANNOTATION_K8S_LABELS: &str = "io.kubernetes.cri-o.Labels";

// static LABEL_NAMESPACE: &str = "io.kubernetes.pod.namespace";
static LABEL_POLICY_ENFORCE: &str = "pod-security.kubernetes.io/enforce";
// static LABEL_POLICY_AUDIT: &str = "pod-security.kubernetes.io/audit";
// static LABEL_POLICY_WARN: &str = "pod-security.kubernetes.io/warn";

static ANNOTATION_CONTAINERD_LOG_DIRECTORY: &str = "io.kubernetes.cri.sandbox-log-directory";
static ANNOTATION_CONTAINERD_SANDBOX_ID: &str = "io.kubernetes.cri.sandbox-id";

#[derive(thiserror::Error, Debug)]
enum NewContainerError {
    #[error("could not retrieve the runc status")]
    Status(#[from] std::io::Error),

    #[error("could not format")]
    Format(#[from] std::fmt::Error),

    #[error("could not convert bytes to utf-8 string")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("could not parse JSON")]
    Json(#[from] serde_json::Error),
}

/// Checks whether the container is a part of Kubernetes infrastracture and if
/// yes, looks up for the Kubernetes namespace.
fn container_namespace(
    container_id: &str,
    container_root: Option<std::string::String>,
) -> Result<Option<std::string::String>, NewContainerError> {
    let mut state_cmd = std::process::Command::new("runc");
    if let Some(ref v) = container_root {
        state_cmd.arg("--root").arg(v);
    }

    state_cmd.arg("state").arg(container_id);
    let state_raw = String::from_utf8(state_cmd.output()?.stdout)?;

    let state: serde_json::Value = serde_json::from_str(&state_raw)?;
    let annotations_o = state["annotations"].as_object();

    match annotations_o {
        Some(annotations) => {
            // containerd
            if annotations.contains_key(ANNOTATION_CONTAINERD_LOG_DIRECTORY) {
                // containerd doesn't expose k8s namespaces directly. They have
                // to be parsed from the log directory path, where the first
                // part of the filename is the namespace.
                let log_directory = annotations[ANNOTATION_CONTAINERD_LOG_DIRECTORY]
                    .as_str()
                    .unwrap();
                let p = std::path::PathBuf::from(log_directory);
                let file_name = p.file_name().unwrap().to_str().unwrap();
                let mut splitter = file_name.split('_');
                let namespace = splitter.next().unwrap().to_string();

                return Ok(Some(namespace));
            } else if annotations.contains_key(ANNOTATION_CONTAINERD_SANDBOX_ID) {
                // When a container is running as a part of a previously created
                // pod, the log directory path has to be retrieved from the
                // sandbox container.
                let sandbox_id = annotations[ANNOTATION_CONTAINERD_SANDBOX_ID]
                    .as_str()
                    .unwrap();
                return container_namespace(sandbox_id, container_root);
            }
            Ok(None)
        }
        None => Ok(None),
    }
}

/// Finds the policy for the given Kubernetes namespace. If none, the baseline
/// policy is returned. Otherwise checks the Kubernetes namespace labels.
async fn policy_label(
    namespace_o: Option<std::string::String>,
) -> Result<lockc::ContainerPolicyLevel, kube::Error> {
    // Apply the privileged policy for kube-system containers immediately.
    // Otherwise the core k8s components (apiserver, scheduler) won't be able
    // to run.
    // If container has no k8s namespace, apply the baseline policy.
    let namespace_s = match namespace_o {
        Some(v) if v.as_str() == "kube-system" => {
            return Ok(lockc::ContainerPolicyLevel::Privileged)
        }
        Some(v) => v,
        None => return Ok(lockc::ContainerPolicyLevel::Baseline),
    };

    let kubeconfig =
        kube::config::Kubeconfig::read_from(std::path::Path::new("/etc/kubernetes/admin.conf"))?;
    let options = kube::config::KubeConfigOptions::default();
    let config = kube::config::Config::from_custom_kubeconfig(kubeconfig, &options).await?;
    let client = kube::Client::try_from(config)?;

    let namespaces: kube::api::Api<v1::Namespace> = kube::api::Api::all(client);
    let namespace = namespaces.get(&namespace_s).await?;

    match namespace.metadata.labels.get(LABEL_POLICY_ENFORCE) {
        Some(s) => match &s[..] {
            "restricted" => Ok(lockc::ContainerPolicyLevel::Restricted),
            "baseline" => Ok(lockc::ContainerPolicyLevel::Baseline),
            "privileged" => Ok(lockc::ContainerPolicyLevel::Privileged),
            _ => Ok(lockc::ContainerPolicyLevel::Baseline),
        },
        None => Ok(lockc::ContainerPolicyLevel::Baseline),
    }
}

/// Types of options (prepositioned by `--`).
enum OptParsingAction {
    /// Option not followed by a positional argument.
    NoPositional,
    /// Option followed by a positional argument we don't want to store.
    Skip,
    /// --root option which we want to store.
    Root,
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
    /// Action of starting the container, when we want to detect and apply a
    /// policy.
    Start,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pid = nix::unistd::getpid();
    let pid_u = u32::try_from(libc::pid_t::from(pid))?;

    let mut opt_parsing_action = OptParsingAction::NoPositional;
    let mut arg_parsing_action = ArgParsingAction::None;
    let mut container_action = ContainerAction::Other;

    let mut container_root: Option<std::string::String> = None;
    let mut container_id_o: Option<std::string::String> = None;

    let mut cmd = tokio::process::Command::new("runc");
    for arg in std::env::args().skip(1) {
        cmd.arg(arg.clone());

        match arg.as_str() {
            // Options which are followed with a positional arguments we don't
            // want to store.
            "--bundle" => opt_parsing_action = OptParsingAction::Skip,
            "--log" => opt_parsing_action = OptParsingAction::Skip,
            "--log-format" => opt_parsing_action = OptParsingAction::Skip,
            "--pid-file" => opt_parsing_action = OptParsingAction::Skip,
            // We want to explicitly store the value of the --root option.
            "--root" => opt_parsing_action = OptParsingAction::Root,
            _ => {}
        }
        if arg.as_str().starts_with('-') {
            // After handling the option, start parsing the next argument.
            continue;
        }

        match opt_parsing_action {
            OptParsingAction::NoPositional => {}
            OptParsingAction::Skip => {
                opt_parsing_action = OptParsingAction::NoPositional;
                continue;
            }
            OptParsingAction::Root => {
                container_root = Some(arg.clone());
                opt_parsing_action = OptParsingAction::NoPositional;
                continue;
            }
        }
        match arg_parsing_action {
            ArgParsingAction::None => {}
            ArgParsingAction::ContainerId => {
                container_id_o = Some(arg.clone());
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
                container_action = ContainerAction::Start;
            }
            "state" => arg_parsing_action = ArgParsingAction::ContainerId,
            "update" => arg_parsing_action = ArgParsingAction::ContainerId,
            _ => {}
        }
    }

    match container_action {
        ContainerAction::Other => {
            match container_id_o {
                Some(v) => {
                    let container_key = lockc::hash(&v)?;
                    lockc::add_process(container_key, pid_u)?;
                    cmd.status().await?;
                    lockc::delete_process(pid_u)?;
                }
                None => {
                    // The purpose of this fake "container" is only to allow the runc
                    // subcommand to be detected as wrapped and thus allowed by
                    // the LSM program to execute. It's only to handle subcommands
                    // like `init`, `list` or `spec`, so we make it restricted.
                    lockc::add_container(0, pid_u, lockc::ContainerPolicyLevel::Restricted)?;
                    cmd.status().await?;
                    lockc::delete_container(0)?;
                }
            }
        }
        ContainerAction::Create => {
            let container_key = lockc::hash(&container_id_o.unwrap())?;
            // Initialize the container with the baseline policy.
            lockc::add_container(container_key, pid_u, lockc::ContainerPolicyLevel::Baseline)?;
            cmd.status().await?;
        }
        ContainerAction::Delete => {
            let container_key = lockc::hash(&container_id_o.unwrap())?;
            lockc::delete_container(container_key)?;
            cmd.status().await?;
        }
        ContainerAction::Start => {
            let container_id = container_id_o.unwrap();
            let container_key = lockc::hash(&container_id)?;
            lockc::add_process(container_key, pid_u)?;

            let namespace = container_namespace(&container_id, container_root)?;

            // Apply the appropriate policy.
            let policy = policy_label(namespace).await?;
            lockc::write_policy(&container_id, policy)?;

            cmd.status().await?;
        }
    }

    Ok(())
}
