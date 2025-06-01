use crate::workflows::default::prompt::pa07_ask_pull_push::AskPushPull;
use crate::{
    bgit_error::BGitError,
    step::{ActionStep, PromptStep, Step, Task::PromptStepTask},
};

pub(crate) struct IsPushedPulled {
    name: String,
}

impl ActionStep for IsPushedPulled {
    fn new() -> Self
    where
        Self: Sized,
    {
        IsPushedPulled {
            name: "is_pushed_pulled".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        // Check for unpushed commits (ahead of remote)
        // let _has_unpushed = crate::events::git_status::has_unpushed_commits()?;

        // Check for unpulled commits (behind remote)
        // let has_unpulled = crate::events::git_status::has_unpulled_commits()?;

        Ok(Step::Task(PromptStepTask(Box::new(AskPushPull::new()))))
    }
}
