use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    events::git_restore::RestoreMode,
    step::{PromptStep, Step, Task::ActionStepTask},
    workflows::default::action::ta06_restore_changes::RestoreChanges,
};
use dialoguer::{theme::ColorfulTheme, Select};

pub(crate) struct AskToRestore {
    name: String,
}

impl PromptStep for AskToRestore {
    fn new() -> Self
    where
        Self: Sized,
    {
        AskToRestore {
            name: "ask_to_restore".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What do you want to restore?")
            .default(0)
            .items(&[
                "Restore unstaged changes (git restore .)",
                "Unstage all files (git restore --staged .)",
                "Cancel",
            ])
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
            0 => Ok(Step::Task(ActionStepTask(Box::new(
                RestoreChanges::with_mode(RestoreMode::RestoreAllUnstaged),
            )))),
            1 => Ok(Step::Task(ActionStepTask(Box::new(
                RestoreChanges::with_mode(RestoreMode::UnstageAll),
            )))),
            2 => Ok(Step::Stop),
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
