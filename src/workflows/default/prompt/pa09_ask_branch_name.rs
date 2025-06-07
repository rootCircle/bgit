use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    step::{ActionStep, PromptStep, Step, Task::ActionStepTask},
    workflows::default::action::ta12_move_changes::MoveChanges,
};
use dialoguer::{theme::ColorfulTheme, Input};

pub(crate) struct AskBranchName {
    name: String,
}

impl PromptStep for AskBranchName {
    fn new() -> Self
    where
        Self: Sized,
    {
        AskBranchName {
            name: "ask_branch_name".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        let branch_name: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter branch name")
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

        // Validate branch name is not empty
        if branch_name.trim().is_empty() {
            return Err(Box::new(BGitError::new(
                "Invalid branch name",
                "Branch name cannot be empty.",
                BGitErrorWorkflowType::PromptStep,
                &self.name,
                NO_EVENT,
                NO_RULE,
            )));
        }

        let move_changes = MoveChanges::new().with_target_branch(branch_name);
        Ok(Step::Task(ActionStepTask(Box::new(move_changes))))
    }
}
