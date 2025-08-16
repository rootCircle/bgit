use crate::config::global::BGitGlobalConfig;
use crate::config::local::{StepFlags, WorkflowRules};
use crate::rules::Rule;
use crate::{
    bgit_error::BGitError,
    events::{AtomicEvent, git_init::GitInit},
    rules::a01_git_install::IsGitInstalledLocally,
    step::{PromptStep, Step},
};
pub(crate) struct InitGitRepo {
    name: String,
    path: String,
}

impl InitGitRepo {
    pub fn set_path(&mut self, path: &str) {
        self.path = path.to_owned();
    }
}

impl PromptStep for InitGitRepo {
    fn new() -> Self {
        InitGitRepo {
            name: "init_git_repo".to_owned(),
            path: ".".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(
        &self,
        _step_config_flags: Option<&StepFlags>,
        workflow_rules_config: Option<&WorkflowRules>,
        global_config: &BGitGlobalConfig,
    ) -> Result<Step, Box<BGitError>> {
        let mut git_init = GitInit::new(global_config).with_path(&self.path);
        git_init.add_pre_check_rule(Box::new(IsGitInstalledLocally::new(workflow_rules_config)));
        git_init.execute()?;
        Ok(Step::Stop)
    }
}
