use crate::step::Task::PromptStepTask;
use crate::workflows::default::prompt::pa09_ask_branch_name::AskBranchName;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    step::{PromptStep, Step},
    workflows::default::prompt::pa08_ask_commit::AskCommit,
};
use dialoguer::theme::ColorfulTheme;
use dialoguer::Select;
pub(crate) struct AskIfSameFeat {
    name: String,
}

impl PromptStep for AskIfSameFeat {
    fn new() -> Self
    where
        Self: Sized,
    {
        AskIfSameFeat {
            name: "ask_if_same_feat".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        let options = vec!["yes", "no"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Are you working on the same feature as older?")
            .default(1) // Default to "no" (index 1)
            .items(&options)
            .interact()
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

        let is_same_feature = selection == 0;

        if is_same_feature {
            println!("Continuing with same feature workflow...");
            Ok(Step::Task(PromptStepTask(Box::new(AskCommit::new()))))
        } else {
            println!("Working on different feature - will move changes to new branch...");
            Ok(Step::Task(PromptStepTask(Box::new(AskBranchName::new()))))
        }
    }
}
