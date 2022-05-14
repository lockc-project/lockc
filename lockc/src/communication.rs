use tokio::sync::oneshot;

use lockc_common::ContainerPolicyLevel;

use crate::maps::MapOperationError;

/// Set of commands that the other tokio threads can use to request eBPF map
/// operations.
#[derive(Debug)]
pub enum EbpfCommand {
    AddContainer {
        container_id: String,
        pid: i32,
        policy_level: ContainerPolicyLevel,
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
