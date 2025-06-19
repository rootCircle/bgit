use crate::config::{StepFlags, WorkflowRules};
use crate::events::AtomicEvent;
use crate::events::git_pull::GitPull;
use crate::events::git_push::GitPush;

use crate::rules::Rule;
use crate::rules::a14_big_repo_size::IsRepoSizeTooBig;
use crate::step::PromptStep;
use crate::{bgit_error::BGitError, step::Step};

pub(crate) struct PullAndPush {
    name: String,
}

impl PromptStep for PullAndPush {
    fn new() -> Self
    where
        Self: Sized,
    {
        PullAndPush {
            name: "pull_and_push".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(
        &self,
        _step_config_flags: Option<&StepFlags>,
        workflow_rules_config: Option<&WorkflowRules>,
    ) -> Result<Step, Box<BGitError>> {
        let git_pull = GitPull::new().with_rebase(true);

        match git_pull.execute() {
            Ok(_) => {
                let mut git_push = GitPush::new();

                git_push.add_pre_check_rule(Box::new(IsRepoSizeTooBig::new(workflow_rules_config)));

                git_push
                    .with_force_with_lease(false)
                    .with_upstream_flag(false);

                match git_push.execute() {
                    Ok(_) => Ok(Step::Stop),
                    Err(e) => {
                        // Push failed, return error
                        Err(e)
                    }
                }
            }
            Err(e) => {
                // Pull failed, return error
                Err(e)
            }
        }
    }
}
