use crate::config::{StepFlags, WorkflowRules};

use super::ta07_has_uncommitted::HasUncommitted;
use crate::events::AtomicEvent;
use crate::step::Task::ActionStepTask;
use crate::workflows::default::prompt::pa05_ask_to_add::AskToAdd;
use crate::{
    bgit_error::BGitError,
    events::git_status::GitStatus,
    step::{ActionStep, PromptStep, Step, Task::PromptStepTask},
};
pub(crate) struct HasUnstaged {
    name: String,
}

impl ActionStep for HasUnstaged {
    fn new() -> Self
    where
        Self: Sized,
    {
        HasUnstaged {
            name: "has_unstaged".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(
        &self,
        _step_config_flags: Option<&StepFlags>,
        _workflow_rules_config: Option<&WorkflowRules>,
    ) -> Result<Step, Box<BGitError>> {
        let git_status = GitStatus::new();
        match git_status.execute() {
            Ok(true) => Ok(Step::Task(PromptStepTask(Box::new(AskToAdd::new())))),
            Ok(false) => Ok(Step::Task(ActionStepTask(Box::new(HasUncommitted::new())))),
            Err(e) => Err(e),
        }
    }
}
