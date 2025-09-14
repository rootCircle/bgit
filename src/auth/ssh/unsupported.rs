use log::debug;

use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE, NO_STEP};

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
