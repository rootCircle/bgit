use crate::{
    bgit_error::BGitError,
    config::{StepFlags, WorkflowRules},
};

#[derive(PartialEq)]
pub(crate) enum Step {
    Start(Task),
    Stop,
    Task(Task),
}

pub(crate) enum Task {
    ActionStepTask(Box<dyn ActionStep>),
    PromptStepTask(Box<dyn PromptStep>),
}

impl std::cmp::PartialEq for Task {
    fn eq(&self, other: &Task) -> bool {
        // TODO: Improve the core matching logic for Task
        match (self, other) {
            (Task::ActionStepTask(a), Task::ActionStepTask(b)) => a.get_name() == b.get_name(),
            (Task::PromptStepTask(a), Task::PromptStepTask(b)) => a.get_name() == b.get_name(),
            (Task::ActionStepTask(_), Task::PromptStepTask(_)) => false,
            (Task::PromptStepTask(_), Task::ActionStepTask(_)) => false,
        }
    }
}

pub(crate) trait ActionStep {
    fn new() -> Self
    where
        Self: Sized;
    fn get_name(&self) -> &str;
    fn execute(
        &self,
        step_config_flags: Option<&StepFlags>,
        workflow_rules_config: Option<&WorkflowRules>,
    ) -> Result<Step, Box<BGitError>>;
}

pub(crate) trait PromptStep {
    fn new() -> Self
    where
        Self: Sized;
    fn get_name(&self) -> &str;
    fn execute(
        &self,
        step_config_flags: Option<&StepFlags>,
        workflow_rules_config: Option<&WorkflowRules>,
    ) -> Result<Step, Box<BGitError>>;
}
