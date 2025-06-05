use crate::events::git_commit::GitCommit;
use crate::events::AtomicEvent;
use crate::rules::a12_no_secrets_staged::NoSecretsStaged;
use crate::rules::a14_big_repo_size::IsRepoSizeTooBig;
use crate::rules::Rule;
use crate::step::ActionStep;
use crate::step::Task::ActionStepTask;
use crate::workflows::default::action::ta08_is_pulled_pushed::IsPushedPulled;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    step::{PromptStep, Step},
};
use dialoguer::{theme::ColorfulTheme, Input};
pub(crate) struct AskHumanCommitMessage {
    name: String,
}

impl PromptStep for AskHumanCommitMessage {
    fn new() -> Self
    where
        Self: Sized,
    {
        AskHumanCommitMessage {
            name: "ask_human_commit_message".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        let commit_message: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your commit message")
            .interact_text()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "Input Error",
                    &e.to_string(),
                    BGitErrorWorkflowType::PromptStep,
                    &self.name,
                    NO_EVENT,
                    NO_RULE,
                ))
            })?;

        // Check if commit message is empty
        if commit_message.trim().is_empty() {
            return Err(Box::new(BGitError::new(
                "Empty commit message",
                "Commit message cannot be empty.",
                BGitErrorWorkflowType::PromptStep,
                &self.name,
                NO_EVENT,
                NO_RULE,
            )));
        }

        let mut git_commit = GitCommit::new().with_commit_message(commit_message);
        git_commit.add_pre_check_rule(Box::new(NoSecretsStaged::new()));
        git_commit.add_pre_check_rule(Box::new(IsRepoSizeTooBig::new()));

        git_commit.execute()?;

        // Return to next step (IsPushedPulled)
        Ok(Step::Task(ActionStepTask(Box::new(IsPushedPulled::new()))))
    }
}
