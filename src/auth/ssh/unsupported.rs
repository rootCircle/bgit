use git2::Cred;
use log::debug;
use std::path::{Path, PathBuf};

use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE, NO_STEP};
use crate::config::global::BGitGlobalConfig;

/// SSH agent functionality is not supported on this platform
pub fn ensure_agent_ready() -> Result<(), Box<BGitError>> {
    debug!("SSH agent not supported on this platform");
    Err(Box::new(BGitError::new(
        "SSH agent unsupported",
        "SSH agent not supported on this platform",
        BGitErrorWorkflowType::Authentication,
        NO_STEP,
        NO_EVENT,
        NO_RULE,
    )))
}

/// SSH agent start not supported on this platform
pub fn start_agent_detached(_socket_path: Option<&Path>) -> Result<(), Box<BGitError>> {
    Err(Box::new(BGitError::new(
        "SSH agent start unsupported",
        "SSH agent start not supported on this platform",
        BGitErrorWorkflowType::Authentication,
        NO_STEP,
        NO_EVENT,
        NO_RULE,
    )))
}

/// SSH key addition not supported on this platform
pub fn add_all_ssh_keys(_cfg: &BGitGlobalConfig) -> Result<Option<PathBuf>, Box<BGitError>> {
    debug!("SSH key addition not supported on this platform");
    Err(Box::new(BGitError::new(
        "SSH key addition unsupported",
        "SSH key addition not supported on this platform",
        BGitErrorWorkflowType::Authentication,
        NO_STEP,
        NO_EVENT,
        NO_RULE,
    )))
}

/// Direct SSH key authentication not supported on this platform
pub fn try_ssh_key_files_directly(_username: &str) -> Result<Cred, Box<BGitError>> {
    debug!("Direct SSH key authentication not supported on this platform");
    Err(Box::new(BGitError::new(
        "Direct SSH key auth unsupported",
        "Direct SSH key authentication not supported on this platform",
        BGitErrorWorkflowType::Authentication,
        NO_STEP,
        NO_EVENT,
        NO_RULE,
    )))
}

/// SSH agent identity count not supported on this platform
pub fn agent_identities_count() -> Result<usize, Box<BGitError>> {
    Err(Box::new(BGitError::new(
        "SSH agent identity count unsupported",
        "SSH agent identity count not supported on this platform",
        BGitErrorWorkflowType::Authentication,
        NO_STEP,
        NO_EVENT,
        NO_RULE,
    )))
}

/// Interactive key addition not supported on this platform
pub fn add_key_interactive(_key_path: &Path, _key_name: &str) -> Result<bool, Box<BGitError>> {
    debug!("Interactive key addition not supported on this platform");
    Err(Box::new(BGitError::new(
        "Interactive key addition unsupported",
        "Interactive key addition not supported on this platform",
        BGitErrorWorkflowType::Authentication,
        NO_STEP,
        NO_EVENT,
        NO_RULE,
    )))
}
