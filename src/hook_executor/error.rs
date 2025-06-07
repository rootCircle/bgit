use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_RULE, NO_STEP};

pub fn create_hook_error(message: &str, details: &str, event_name: &str) -> Box<BGitError> {
    Box::new(BGitError::new(
        message,
        &format!("{}\nTo debug, run bgit with -vvv flag", details),
        BGitErrorWorkflowType::HookExecutor,
        NO_STEP,
        event_name,
        NO_RULE,
    ))
}
