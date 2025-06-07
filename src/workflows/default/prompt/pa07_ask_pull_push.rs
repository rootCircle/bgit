use crate::config::{StepFlags, WorkflowRules};
use crate::workflows::default::action::ta09_pull_push::PullAndPush;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    step::{ActionStep, PromptStep, Step, Task::ActionStepTask},
};
use dialoguer::{theme::ColorfulTheme, Select};

pub(crate) struct AskPushPull {
    name: String,
}

impl PromptStep for AskPushPull {
    fn new() -> Self
    where
        Self: Sized,
    {
        AskPushPull {
            name: "ask_push_pull".to_owned(),
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
        let selection: usize = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to pull/push commits?")
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
            0 => Ok(Step::Task(ActionStepTask(Box::new(PullAndPush::new())))),
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
