use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    step::{PromptStep, Step},
};
use dialoguer::{theme::ColorfulTheme, Select};

pub(crate) struct AskCommit {
    name: String,
}

impl PromptStep for AskCommit {
    fn new() -> Self
    where
        Self: Sized,
    {
        AskCommit {
            name: "ask_commit".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        let selection: usize = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to commit changes?")
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
            0 => Ok(Step::Stop),
            1 => Ok(Step::Stop),
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
