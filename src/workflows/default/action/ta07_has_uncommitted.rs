use super::ta08_is_pulled_pushed::IsPushedPulled;
use super::ta10_is_branch_main::IsBranchMain;
use crate::config::{StepFlags, WorkflowRules};
use crate::events::git_status;
use crate::step::Task::ActionStepTask;
use crate::{
    bgit_error::BGitError,
    step::{ActionStep, Step},
};

pub(crate) struct HasUncommitted {
    name: String,
}

impl ActionStep for HasUncommitted {
    fn new() -> Self
    where
        Self: Sized,
    {
        HasUncommitted {
            name: "has_uncommitted".to_owned(),
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
        // Check for both unstaged/new files and staged files
        let has_unstaged = git_status::has_unstaged_or_new_files()?;
        let has_staged = git_status::has_staged_files()?;

        if has_unstaged || has_staged {
            Ok(Step::Task(ActionStepTask(Box::new(IsBranchMain::new()))))
        } else {
            println!("No uncommitted changes found.");
            Ok(Step::Task(ActionStepTask(Box::new(IsPushedPulled::new()))))
        }
    }
}
