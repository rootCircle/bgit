use crate::config::{StepFlags, WorkflowRules};
use crate::events::git_log::GitLog;
use crate::events::{AtomicEvent, git_config};
use crate::flags::config_flag;
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
        step_config_flags: Option<&StepFlags>,
        _workflow_rules_config: Option<&WorkflowRules>,
    ) -> Result<Step, Box<BGitError>> {
        let override_check_for_authors_list = step_config_flags
            .and_then(|flags| flags.get_flag::<Vec<String>>(config_flag::workflows::default::is_sole_contributor::OVERRIDE_CHECK_FOR_AUTHORS))
            .and_then(|author_emails| {
                if author_emails.is_empty() {
                    None
                } else {
                    Some(author_emails)
                }
            });
        let skip_author_ownership_check =
            if let Some(author_emails) = override_check_for_authors_list {
                let git_config_event = git_config::GitConfig::new()
                    .with_operation(git_config::ConfigOperation::Get)
                    .with_key("user.email".to_owned());

                git_config_event.execute()?;
                let current_author_email = git_config_event.get_value()?;

                author_emails.contains(&current_author_email)
            } else {
                false
            };

        let git_log = GitLog::check_sole_contributor();
        let is_sole_contributor = skip_author_ownership_check || git_log.execute()?;
        match is_sole_contributor {
            true => Ok(Step::Task(PromptStepTask(Box::new(AskCommit::new())))),
            false => Ok(Step::Task(PromptStepTask(Box::new(AskBranchName::new())))),
        }
    }
}
