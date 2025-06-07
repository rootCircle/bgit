use super::ta11_is_sole_contributor::IsSoleContributor;
use crate::config::{StepFlags, WorkflowRules};
use crate::events::git_branch::GitBranch;
use crate::events::AtomicEvent;
use crate::step::PromptStep;
use crate::step::Task::ActionStepTask;
use crate::step::Task::PromptStepTask;
use crate::workflows::default::prompt::pa10_ask_same_feat::AskIfSameFeat;
use crate::{
    bgit_error::BGitError,
    step::{ActionStep, Step},
};

pub(crate) struct IsBranchMain {
    name: String,
}

impl ActionStep for IsBranchMain {
    fn new() -> Self
    where
        Self: Sized,
    {
        IsBranchMain {
            name: "is_branch_main".to_owned(),
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
        let git_branch = GitBranch::check_current_branch();
        match git_branch.execute() {
            Ok(true) => Ok(Step::Task(ActionStepTask(Box::new(
                IsSoleContributor::new(),
            )))),
            Ok(false) => Ok(Step::Task(PromptStepTask(Box::new(AskIfSameFeat::new())))),
            Err(e) => Err(e),
        }
    }
}
