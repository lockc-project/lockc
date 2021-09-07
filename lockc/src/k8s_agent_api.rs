use serde::{Deserialize, Serialize};

use super::bpfstructs;

pub static DEFAULT_SOCKET_PATH: &str = "/run/lockc/lockc-k8s-agent.sock";

/// JSON object for returning errors in a response body.
#[derive(Deserialize, Serialize)]
pub struct ErrorObj {
    pub error: String,
}

/// Server health check status.
#[derive(Deserialize, Serialize)]
pub struct ServerStatus {
    pub status: String,
}

/// JSON representation of policies.
#[derive(Deserialize, Serialize)]
pub struct Policies {
    pub enforce: bpfstructs::container_policy_level,
    pub enforce_version: String,
    pub audit: bpfstructs::container_policy_level,
    pub audit_version: String,
    pub warn: bpfstructs::container_policy_level,
    pub warn_version: String,
}
