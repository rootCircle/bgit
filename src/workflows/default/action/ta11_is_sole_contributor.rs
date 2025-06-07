use crate::config::{StepFlags, WorkflowRules};
use crate::events::git_log::GitLog;
use crate::events::AtomicEvent;
use crate::step::PromptStep;
use crate::step::Task::PromptStepTask;
use crate::workflows::default::prompt::pa08_ask_commit::AskCommit;
use crate::workflows::default::prompt::pa09_ask_branch_name::AskBranchName;
use crate::{
    bgit_error::BGitError,
    step::{ActionStep, Step},
};
pub(crate) struct IsSoleContributor {
    name: String,
}

impl ActionStep for IsSoleContributor {
    fn new() -> Self
    where
        Self: Sized,
    {
        IsSoleContributor {
            name: "is_sole_contributor".to_owned(),
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
        let git_log = GitLog::check_sole_contributor();
        match git_log.execute() {
            Ok(true) => Ok(Step::Task(PromptStepTask(Box::new(AskCommit::new())))),
            Ok(false) => Ok(Step::Task(PromptStepTask(Box::new(AskBranchName::new())))),
            Err(e) => Err(e),
        }
    }
}
