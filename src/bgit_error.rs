use colored::Colorize;

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct BGitError {
    name: String,
    message: String,
    workflow_type: BGitErrorWorkflowType,
    step_name: String,
    event_name: String,
    rule_name: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) enum BGitErrorWorkflowType {
    Rules,
    AtomicEvent,
    RawExecutor,
    HookExecutor,
    Config,
    WorkflowQueue,
    ActionStep,
    PromptStep,
}

pub(crate) const EMPTY_STRING: &str = "";
pub(crate) const NO_STEP: &str = EMPTY_STRING;
pub(crate) const NO_EVENT: &str = EMPTY_STRING;
pub(crate) const NO_RULE: &str = EMPTY_STRING;

impl BGitError {
    pub(crate) fn new(
        name: &str,
        message: &str,
        workflow_type: BGitErrorWorkflowType,
        step_name: &str,
        event_name: &str,
        rule_name: &str,
    ) -> Self {
        Self {
            name: name.to_owned(),
            message: message.to_owned(),
            workflow_type,
            step_name: step_name.to_owned(),
            event_name: event_name.to_owned(),
            rule_name: rule_name.to_owned(),
        }
    }

    pub(crate) fn print_error(&self) {
        let mut context = format!("[{:?}", self.workflow_type);
        if self.step_name != NO_STEP {
            context.push_str(&format!("::{}", self.step_name));
        }
        if self.event_name != NO_EVENT {
            context.push_str(&format!("::{}", self.event_name));
        }
        if self.rule_name != NO_RULE {
            context.push_str(&format!("::{}", self.rule_name));
        }
        context.push(']');

        eprintln!(
            "{} {} {}",
            "ERROR".red().bold(),
            context.yellow(),
            self.name.bright_red().bold()
        );
        eprintln!("{}: {}", "Message".bright_blue(), self.message);
    }
}
