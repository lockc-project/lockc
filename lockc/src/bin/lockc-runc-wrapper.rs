use std::{
    convert::TryFrom,
    fs, io, path,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    body::Body,
    http::{Method, Request, StatusCode, Uri},
};
use hyper::client::connect::{Connected, Connection};
use log::{error, info, LevelFilter, SetLoggerError};
use log4rs::append::file::FileAppender;
use log4rs::config::{runtime::ConfigErrors, Appender, Config, Root};
use serde_json::json;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::UnixStream,
};
use uuid::Uuid;

use lockc::k8s_agent_api::DEFAULT_SOCKET_PATH;

// TODO: To be used for cri-o.
// static ANNOTATION_K8S_LABELS: &str = "io.kubernetes.cri-o.Labels";

static ANNOTATION_CONTAINERD_LOG_DIRECTORY: &str = "io.kubernetes.cri.sandbox-log-directory";
static ANNOTATION_CONTAINERD_SANDBOX_ID: &str = "io.kubernetes.cri.sandbox-id";

#[derive(thiserror::Error, Debug)]
enum ContainerError {
    #[error("could not retrieve the runc status")]
    Status(#[from] std::io::Error),

    #[error("could not format")]
    Format(#[from] std::fmt::Error),

    #[error("could not convert bytes to utf-8 string")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("could not parse JSON")]
    Json(#[from] serde_json::Error),

    #[error("could not find sandbox container bundle directory")]
    BundleDirError,
}

fn container_namespace<P: AsRef<std::path::Path>>(
    container_bundle: P,
) -> Result<Option<std::string::String>, ContainerError> {
    let bundle_path = container_bundle.as_ref();
    let config_path = bundle_path.join("config.json");
    let f = std::fs::File::open(config_path)?;
    let r = std::io::BufReader::new(f);

    let config: serde_json::Value = serde_json::from_reader(r)?;
    let annotations_o = config["annotations"].as_object();

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
                let log_path = std::path::PathBuf::from(log_directory);
                let file_name = log_path.file_name().unwrap().to_str().unwrap();
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

                // Go one directory up from the current bundle.
                let mut ancestors = bundle_path.ancestors();
                ancestors.next();
                match ancestors.next() {
                    Some(v) => {
                        // Then go to sandbox_id directory (sandbox's bundle).
                        let new_bundle = v.join(sandbox_id);
                        return container_namespace(new_bundle);
                    }
                    None => return Err(ContainerError::BundleDirError),
                }
            }
            Ok(None)
        }
        None => Ok(None),
    }
}

struct ClientConnection {
    stream: UnixStream,
}

impl AsyncWrite for ClientConnection {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        Pin::new(&mut self.stream).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.stream).poll_flush(cx)
    }

    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }
}

impl AsyncRead for ClientConnection {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_read(cx, buf)
    }
}

impl Connection for ClientConnection {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

#[derive(thiserror::Error, Debug)]
enum PolicyLabelError {
    #[error(transparent)]
    Axum(#[from] axum::http::Error),

    #[error(transparent)]
    FromUtf8(#[from] std::string::FromUtf8Error),

    #[error(transparent)]
    Hyper(#[from] hyper::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
}

/// Finds the policy for the given Kubernetes namespace. If none, the baseline
/// policy is returned. Otherwise checks the Kubernetes namespace labels.
async fn policy_label(
    namespace_o: Option<std::string::String>,
) -> Result<lockc::bpfstructs::container_policy_level, PolicyLabelError> {
    let namespace_s = match namespace_o {
        Some(v) => v,
        None => return Ok(lockc::bpfstructs::container_policy_level_POLICY_LEVEL_BASELINE),
    };

    info!("creating connector");
    let connector = tower::service_fn(move |_: Uri| {
        Box::pin(async move {
            let stream = UnixStream::connect(DEFAULT_SOCKET_PATH).await?;
            Ok::<_, io::Error>(ClientConnection { stream })
        })
    });
    info!("creating client");
    let client = hyper::Client::builder().build(connector);

    info!("creating request");
    let request = Request::builder()
        .header("Content-Type", "application/json")
        .method(Method::GET)
        .uri(hyperlocal::Uri::new(DEFAULT_SOCKET_PATH, "/policies"))
        .body(Body::from(serde_json::to_vec(&json!({
            "namespace": namespace_s
        }))?))?;

    info!("creating response");
    let response = client.request(request).await?;
    info!("got response {}", response.status());

    assert_eq!(response.status(), StatusCode::OK);
    info!("response ok");

    let body = hyper::body::to_bytes(response.into_body()).await?;
    info!("to bytes");
    let body: Value = serde_json::from_slice(&body)?;
    info!("to json");

    let policy: lockc::bpfstructs::container_policy_level =
        body["enforce"].as_i64().unwrap() as i32;
    info!("got policy: {}", policy);
    Ok(policy)
}

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Mount {
    destination: String,
    r#type: String,
    source: String,
    options: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Mounts {
    mounts: Vec<Mount>,
}

fn docker_config<P: AsRef<std::path::Path>>(
    container_bundle: P,
) -> Result<std::path::PathBuf, ContainerError> {
    let bundle_path = container_bundle.as_ref();
    let config_path = bundle_path.join("config.json");
    let f = std::fs::File::open(config_path)?;
    let r = std::io::BufReader::new(f);

    let m: Mounts = serde_json::from_reader(r)?;

    for test in m.mounts {
        let source: Vec<&str> = test.source.split('/').collect();
        if source.len() > 1 && source[source.len() - 1] == "hostname" {
            let config_v2 = str::replace(&test.source, "hostname", "config.v2.json");
            return Ok(std::path::PathBuf::from(config_v2));
        }
    }

    Err(ContainerError::BundleDirError)
}

use serde_json::Value;

fn docker_label<P: AsRef<std::path::Path>>(
    docker_bundle: P,
) -> Result<lockc::bpfstructs::container_policy_level, ContainerError> {
    let config_path = docker_bundle.as_ref();
    let f = std::fs::File::open(config_path)?;
    let r = std::io::BufReader::new(f);

    let l: Value = serde_json::from_reader(r)?;

    let x = l["Config"]["Labels"]["org.lockc.policy"].as_str();

    match x {
        Some(x) => match x {
            "restricted" => Ok(lockc::bpfstructs::container_policy_level_POLICY_LEVEL_RESTRICTED),
            "baseline" => Ok(lockc::bpfstructs::container_policy_level_POLICY_LEVEL_BASELINE),
            "privileged" => Ok(lockc::bpfstructs::container_policy_level_POLICY_LEVEL_PRIVILEGED),
            _ => Ok(lockc::bpfstructs::container_policy_level_POLICY_LEVEL_BASELINE),
        },
        None => Ok(lockc::bpfstructs::container_policy_level_POLICY_LEVEL_BASELINE),
    }
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
    /// Action of starting the container, when we want to detect and apply a
    /// policy.
    Start,
}

#[derive(thiserror::Error, Debug)]
enum SetupLoggingError {
    #[error(transparent)]
    Config(#[from] ConfigErrors),

    #[error(transparent)]
    IO(#[from] io::Error),

    #[error(transparent)]
    SetLogger(#[from] SetLoggerError),
}

fn setup_logging() -> Result<(), SetupLoggingError> {
    let log_dir = path::Path::new("/var")
        .join("log")
        .join("lockc-runc-wrapper");

    fs::create_dir_all(log_dir.clone())?;
    let log_file = FileAppender::builder()
        .build(log_dir.join(format!("{}.log", Uuid::new_v4())))
        .unwrap();
    let config = Config::builder()
        .appender(Appender::builder().build("log_file", Box::new(log_file)))
        .build(
            Root::builder()
                .appender("log_file")
                .build(LevelFilter::Info),
        )?;
    log4rs::init_config(config)?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging()?;

    let pid = nix::unistd::getpid();
    let pid_u = u32::try_from(libc::pid_t::from(pid))?;

    let mut opt_parsing_action = OptParsingAction::NoPositional;
    let mut arg_parsing_action = ArgParsingAction::None;
    let mut container_action = ContainerAction::Other;

    let mut container_bundle_o: Option<std::string::String> = None;
    let mut container_id_o: Option<std::string::String> = None;

    let mut cmd = tokio::process::Command::new("runc");
    for arg in std::env::args().skip(1) {
        info!("argument: {}", arg.clone());
        cmd.arg(arg.clone());

        match arg.as_str() {
            // Options which are followed with a positional arguments we don't
            // want to store.
            "--console-socket" => opt_parsing_action = OptParsingAction::Skip,
            "--criu" => opt_parsing_action = OptParsingAction::Skip,
            "--log" => opt_parsing_action = OptParsingAction::Skip,
            "--log-format" => opt_parsing_action = OptParsingAction::Skip,
            "--pid-file" => opt_parsing_action = OptParsingAction::Skip,
            "--preserve-fds" => opt_parsing_action = OptParsingAction::Skip,
            "--process" => opt_parsing_action = OptParsingAction::Skip,
            "--root" => opt_parsing_action = OptParsingAction::Skip,
            "--rootless" => opt_parsing_action = OptParsingAction::Skip,
            // We want to explicitly store the value of --bundle option.
            "--bundle" => opt_parsing_action = OptParsingAction::Bundle,
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
            OptParsingAction::Bundle => {
                container_bundle_o = Some(arg.clone());
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
                    cmd.status().await?;
                }
            }
        }
        ContainerAction::Create => {
            let container_key = lockc::hash(&container_id_o.unwrap())?;
            let container_bundle = match container_bundle_o {
                Some(v) => std::path::PathBuf::from(v),
                None => std::env::current_dir()?,
            };

            let policy;
            let runc_bundle = container_bundle.clone();
            let namespace = container_namespace(container_bundle);
            match namespace {
                Ok(n) => {
                    policy = policy_label(n).await?;
                }
                Err(_) => {
                    let docker_conf = docker_config(runc_bundle)?;
                    policy = docker_label(docker_conf)?;
                }
            };
            info!("found policy");
            lockc::add_container(container_key, pid_u, policy)?;
            info!("added container");
        }
        ContainerAction::Delete => {
            let container_key = lockc::hash(&container_id_o.unwrap())?;
            lockc::delete_container(container_key)?;
            cmd.status().await?;
        }
        ContainerAction::Start => {
            cmd.status().await?;
        }
    }
    info!("success");

    Ok(())
}
