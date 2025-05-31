use crate::events::AtomicEvent;
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

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        let git_status = GitStatus::new();
        git_status.raw_execute()?;
        // CHANGE : "no" step if left to implement
        Ok(Step::Task(PromptStepTask(Box::new(AskToAdd::new()))))
    }
}
