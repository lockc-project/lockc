use std::{env, fs, path, thread};

use aya_log::BpfLogger;
use clap::Parser;
use thiserror::Error;
use tokio::{
    runtime::Runtime,
    sync::{mpsc, oneshot},
};
use tracing::{debug, error, Level};
use tracing_log::LogTracer;
use tracing_subscriber::FmtSubscriber;

mod communication;
mod load;
mod maps;
mod runc;
mod sysutils;

use communication::EbpfCommand;
use load::{attach_programs, load_bpf};
use maps::{add_container, add_process, delete_container};
// use runc::{attach_runc_nsexec, handle_events, mark_runc_binaries};
use runc::RuncWatcher;
use sysutils::check_bpf_lsm_enabled;

#[derive(Error, Debug)]
enum FanotifyError {
    #[error("could not send the message")]
    Send,
}

/// Runs an fanotify-based runc watcher, which registers containers every time
/// they are created or deleted.
fn fanotify(
    fanotify_bootstrap_rx: oneshot::Receiver<()>,
    ebpf_tx: mpsc::Sender<EbpfCommand>,
) -> Result<(), anyhow::Error> {
    RuncWatcher::new(fanotify_bootstrap_rx, ebpf_tx)?.work_loop()?;
    Ok(())
}

/// Loads and attaches eBPF programs, then fetches logs and events from them.
async fn ebpf(
    fanotify_bootstrap_tx: oneshot::Sender<()>,
    mut ebpf_rx: mpsc::Receiver<EbpfCommand>,
) -> Result<(), anyhow::Error> {
    // Check whether BPF LSM is enabled in the kernel. That check should be
    // omitted in Kubernetes (where lockc runs in a container) or nested
    // containers, because sysctls inside containers might hide the fact
    // that BPF LSM is enabled.
    if env::var("LOCKC_CHECK_LSM_SKIP").is_err() {
        let sys_lsm_path = path::Path::new("/sys")
            .join("kernel")
            .join("security")
            .join("lsm");
        check_bpf_lsm_enabled(sys_lsm_path)?;
    }

    // let config = new_config().await?;

    let path_base = std::path::Path::new("/sys")
        .join("fs")
        .join("bpf")
        .join("lockc");
    fs::create_dir_all(&path_base)?;

    let mut bpf = load_bpf(&path_base)?;
    BpfLogger::init(&mut bpf)?;

    // init_allowed_paths(&mut bpf, &config)?;
    debug!("allowed paths initialized");
    attach_programs(&mut bpf)?;
    debug!("attached programs");

    // Bootstrap the fanotify thread.
    fanotify_bootstrap_tx
        .send(())
        .map_err(|_| FanotifyError::Send)?;

    while let Some(cmd) = ebpf_rx.recv().await {
        match cmd {
            EbpfCommand::AddContainer {
                container_id,
                pid,
                policy_level,
                responder_tx,
            } => {
                let res = add_container(&mut bpf, container_id, pid, policy_level);
                match responder_tx.send(res) {
                    Ok(_) => {}
                    Err(_) => error!(
                        command = "add_container",
                        "could not send eBPF command result although the operation was succeessful"
                    ),
                }
            }
            EbpfCommand::DeleteContainer {
                container_id,
                responder_tx,
            } => {
                let res = delete_container(&mut bpf, container_id);
                match responder_tx.send(res) {
                    Ok(_) => {}
                    Err(_) => error!(
                        command = "delete_container",
                        "could not send eBPF command result although the operation was succeessful"
                    ),
                }
            }
            EbpfCommand::AddProcess {
                container_id,
                pid,
                responder_tx,
            } => {
                let res = add_process(&mut bpf, container_id, pid);
                match responder_tx.send(res) {
                    Ok(_) => {}
                    Err(_) => error!(
                        command = "add_proceess",
                        "could not send eBPF command result although the operation was succeessful"
                    ),
                }
            }
        }
    }

    Ok(())
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Opt {
    #[cfg_attr(
        debug_assertions,
        clap(
            long,
            env="LOCKC_LOG_LEVEL",
            default_value = "debug",
            possible_values = &["trace", "debug", "info", "warn", "error"]
        )
    )]
    #[cfg_attr(
        not(debug_assertions),
        clap(
            long,
            env="LOCKC_LOG_LEVEL",
            default_value = "info",
            possible_values = &["trace", "debug", "info", "warn", "error"]
        )
    )]
    log_level: String,

    #[clap(long, env="LOCKC_LOG_FMT", default_value = "text", possible_values = &["json", "text"])]
    log_fmt: String,
}

#[derive(Error, Debug)]
enum SetupTracingError {
    #[error(transparent)]
    SetLogger(#[from] log::SetLoggerError),

    #[error(transparent)]
    SetGlobalDefault(#[from] tracing_core::dispatcher::SetGlobalDefaultError),

    #[error("unknown log level")]
    UnknownLogLevel,

    #[error("unknown log message format")]
    UnknownLogFormat,
}

fn setup_tracing(opt: &Opt) -> Result<(), SetupTracingError> {
    let (level_tracing, level_log) = match opt.log_level.as_str() {
        "trace" => (Level::TRACE, log::LevelFilter::Trace),
        "debug" => (Level::DEBUG, log::LevelFilter::Debug),
        "info" => (Level::INFO, log::LevelFilter::Info),
        "warn" => (Level::WARN, log::LevelFilter::Warn),
        "error" => (Level::ERROR, log::LevelFilter::Error),
        _ => return Err(SetupTracingError::UnknownLogLevel),
    };

    let builder = FmtSubscriber::builder().with_max_level(level_tracing);
    match opt.log_fmt.as_str() {
        "json" => {
            let subscriber = builder.json().finish();
            tracing::subscriber::set_global_default(subscriber)?;
        }
        "text" => {
            let subscriber = builder.finish();
            tracing::subscriber::set_global_default(subscriber)?;
        }
        _ => return Err(SetupTracingError::UnknownLogFormat),
    };

    LogTracer::builder().with_max_level(level_log).init()?;

    Ok(())
}

fn main() -> Result<(), anyhow::Error> {
    let opt = Opt::parse();
    setup_tracing(&opt)?;

    // Step 1: Create a synchronous thread which takes care of fanotify
    // polling on runc binaries. We monitor all possible runc binaries to get
    // all runc execution events (and therefore - all operations on
    // containers).
    // This thread has to be synchronous and cannot be a part of Tokio runtime,
    // because it:
    // * uses the poll() function
    // * blocks the filesystem operations on monitored files
    // * in case of monitoring runc, we have to be sure that we register a new
    //   container exactly before we allow runc to be actually executed;
    //   otherwise we cannot guarantee that lockc will actually enforce
    //   anything on that container.

    // Fanotify thread bootstrap channel - used later to start the real bootstrap
    // of the thread. We want to bootstrap it later, after loading eBPF
    // programs (which happens in async code in Tokio runtime).
    let (fanotify_bootstrap_tx, fanotify_bootstrap_rx) = oneshot::channel::<()>();

    // eBPF thread channel - used by fanotify thread to request eBFP operations
    // from the async eBPF thread.
    let (ebpf_tx, ebpf_rx) = mpsc::channel::<EbpfCommand>(100);

    // Start the thread (but it's going to wait for bootstrap).
    let fanotify_thread = thread::spawn(move || fanotify(fanotify_bootstrap_rx, ebpf_tx));

    // Step 2: Setup a Tokio runtime for asynchronous part of lockc, which
    // takes care of:
    // * loading and attaching of eBPF programs
    // * fetching events/logs from eBPF programs
    // After initializing the eBPF world, the thread from the step 1 is going
    // to be bootstraped.

    let rt = Runtime::new()?;

    rt.block_on(ebpf(fanotify_bootstrap_tx, ebpf_rx))?;

    if let Err(e) = fanotify_thread.join() {
        error!("failed to join the fanotify thread: {:?}", e);
    }

    Ok(())
}
