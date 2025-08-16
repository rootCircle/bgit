use crate::config::global::BGitGlobalConfig;
use crate::config::local::{StepFlags, WorkflowRules};
use crate::step::PromptStep;
use crate::step::Task::PromptStepTask;
use crate::workflows::default::prompt::pa05_ask_to_add::AskToAdd;
use crate::{
    bgit_error::BGitError,
    events::{AtomicEvent, git_stash::GitStash},
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

    fn execute(
        &self,
        _step_config_flags: Option<&StepFlags>,
        _workflow_rules_config: Option<&WorkflowRules>,
        global_config: &BGitGlobalConfig,
    ) -> Result<Step, Box<BGitError>> {
        let git_stash = GitStash::pop_stash(global_config, self.stash_index);

        git_stash.execute()?;
        println!("Stash popped successfully.");
        Ok(Step::Task(PromptStepTask(Box::new(AskToAdd::new()))))
    }
}
