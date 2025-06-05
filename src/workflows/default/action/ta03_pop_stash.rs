use crate::step::PromptStep;
use crate::step::Task::PromptStepTask;
use crate::workflows::default::prompt::pa05_ask_to_add::AskToAdd;
use crate::{
    bgit_error::BGitError,
    events::{git_stash::GitStash, AtomicEvent},
    step::{ActionStep, Step},
};
pub(crate) struct PopStash {
    name: String,
    stash_index: Option<usize>,
}

impl ActionStep for PopStash {
    fn new() -> Self
    where
        Self: Sized,
    {
        PopStash {
            name: "pop_stash".to_owned(),
            stash_index: None,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        let git_stash = GitStash::pop_stash(self.stash_index);

        git_stash.raw_execute()?;
        println!("Stash popped successfully.");
        Ok(Step::Task(PromptStepTask(Box::new(AskToAdd::new()))))
    }
}
