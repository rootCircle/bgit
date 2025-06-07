use super::pa12_ask_commit_msg::AskHumanCommitMessage;
use crate::config::{StepFlags, WorkflowRules};
use crate::step::ActionStep;
use crate::step::Task::ActionStepTask;
use crate::step::Task::PromptStepTask;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    step::{PromptStep, Step},
    workflows::default::action::ta13_ai_commit_msg::AICommit,
};
use dialoguer::{theme::ColorfulTheme, Select};

pub(crate) struct AskAICommitMessage {
    name: String,
}

impl PromptStep for AskAICommitMessage {
    fn new() -> Self
    where
        Self: Sized,
    {
        AskAICommitMessage {
            name: "ask_ai_commit_message".to_owned(),
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
            .with_prompt("Do you want your commit message written by AI?")
            .default(0)
            .items(&[
                "Yes, generate AI commit message",
                "No, I'll write it myself",
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
            0 => Ok(Step::Task(ActionStepTask(Box::new(AICommit::new())))),
            1 => Ok(Step::Task(PromptStepTask(Box::new(
                AskHumanCommitMessage::new(),
            )))),
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
