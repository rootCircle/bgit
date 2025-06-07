use crate::config::{StepFlags, WorkflowRules};
use crate::step::PromptStep;
use crate::step::Task::ActionStepTask;
use crate::step::Task::PromptStepTask;
use crate::workflows::default::action::ta04_has_unstaged::HasUnstaged;
use crate::workflows::default::prompt::pa04_ask_pop_stash::AskPopStash;
use crate::{
    bgit_error::BGitError,
    step::{ActionStep, Step},
};
use git2::Repository;
use std::env;
pub(crate) struct HasStash {
    name: String,
}

impl ActionStep for HasStash {
    fn new() -> Self
    where
        Self: Sized,
    {
        HasStash {
            name: "has_stash".to_owned(),
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
        let cwd = env::current_dir().expect("Failed to get current directory");
        if Repository::discover(&cwd).is_ok() {
            let mut repo = Repository::discover(cwd).unwrap();
            let mut has_stash: bool = false;

            let _ = repo
                .stash_foreach(|_, _, _| {
                    has_stash = true;
                    false
                })
                .is_ok();

            if has_stash {
                println!("Stash exists in the repository.");
                Ok(Step::Task(PromptStepTask(Box::new(AskPopStash::new()))))
            } else {
                println!("No stash found in the repository.");
                Ok(Step::Task(ActionStepTask(Box::new(HasUnstaged::new()))))
            }
        } else {
            Ok(Step::Stop)
        }
    }
}
