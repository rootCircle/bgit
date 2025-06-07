use crate::config::{StepFlags, WorkflowRules};
use crate::step::Task::PromptStepTask;
use crate::workflows::default::prompt::pa05x_ask_add_mode::AskAddMode;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    step::{PromptStep, Step},
};
use dialoguer::{theme::ColorfulTheme, Select};

use super::pa06_ask_restore_changes::AskToRestore;
pub(crate) struct AskToAdd {
    name: String,
}

impl PromptStep for AskToAdd {
    fn new() -> Self
    where
        Self: Sized,
    {
        AskToAdd {
            name: "ask_to_add".to_owned(),
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
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to add the unstaged files?")
            .default(0)
            .items(&["Yes", "No"])
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

        match selection {
            0 => Ok(Step::Task(PromptStepTask(Box::new(AskAddMode::new())))),
            1 => Ok(Step::Task(PromptStepTask(Box::new(AskToRestore::new())))),
            _ => Err(Box::new(BGitError::new(
                "Invalid selection",
                "Unexpected selection index in Select prompt.",
                BGitErrorWorkflowType::PromptStep,
                &self.name,
                NO_EVENT,
                NO_RULE,
            ))),
        }
    }
}
