use crate::{
    bgit_error::BGitError,
    config::{StepFlags, WorkflowRules},
    step::{
        ActionStep, PromptStep, Step,
        Task::{ActionStepTask, PromptStepTask},
    },
    workflows::default::prompt::pa01_ask_to_init_clone_git::AskToInitCloneGit,
};
use git2::Repository;
use std::env;

use super::ta02_has_stash::HasStash;

pub(crate) struct IsGitRepo {
    name: String,
}

impl ActionStep for IsGitRepo {
    fn new() -> Self
    where
        Self: Sized,
    {
        IsGitRepo {
            name: "is_git_repo".to_owned(),
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
        if Repository::discover(cwd).is_ok() {
            Ok(Step::Task(ActionStepTask(Box::new(HasStash::new()))))
        } else {
            Ok(Step::Task(PromptStepTask(Box::new(
                AskToInitCloneGit::new(),
            ))))
        }
    }
}
