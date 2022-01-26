use tokio::sync::oneshot;

use crate::{bpfstructs::container_policy_level, maps::MapOperationError};

/// Set of commands that the fanotify thread can send to the eBPF thread
/// to request eBPF map operations.
#[derive(Debug)]
pub enum EbpfCommand {
    AddContainer {
        container_id: String,
        pid: i32,
        policy_level: container_policy_level,
        responder_tx: oneshot::Sender<Result<(), MapOperationError>>,
    },
    DeleteContainer {
        container_id: String,
        responder_tx: oneshot::Sender<Result<(), MapOperationError>>,
    },
    AddProcess {
        container_id: String,
        pid: i32,
        responder_tx: oneshot::Sender<Result<(), MapOperationError>>,
    },
}
