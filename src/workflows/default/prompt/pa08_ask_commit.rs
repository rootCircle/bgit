use super::pa11_ask_ai_commit_msg::AskAICommitMessage;
use crate::config::global::BGitGlobalConfig;
use crate::config::local::{StepFlags, WorkflowRules};
use crate::step::ActionStep;
use crate::step::Task::ActionStepTask;
use crate::step::Task::PromptStepTask;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    step::{PromptStep, Step},
    workflows::default::action::ta08_is_pulled_pushed::IsPushedPulled,
};
use dialoguer::{Select, theme::ColorfulTheme};
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

    fn execute(
        &self,
        _step_config_flags: Option<&StepFlags>,
        _workflow_rules_config: Option<&WorkflowRules>,
        _global_config: &BGitGlobalConfig,
    ) -> Result<Step, Box<BGitError>> {
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
            0 => Ok(Step::Task(PromptStepTask(Box::new(
                AskAICommitMessage::new(),
            )))),
            1 => Ok(Step::Task(ActionStepTask(Box::new(IsPushedPulled::new())))),
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
