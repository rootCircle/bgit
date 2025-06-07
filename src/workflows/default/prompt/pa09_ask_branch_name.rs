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
        let branch_name = branch_name.trim();

        // Validate branch name is not empty
        if branch_name.is_empty() {
            return Err(Box::new(BGitError::new(
                "Invalid branch name",
                "Branch name cannot be empty.",
                BGitErrorWorkflowType::PromptStep,
                &self.name,
                NO_EVENT,
                NO_RULE,
            )));
        }

        // Convert spaces to hyphens for multi-word branch names
        let branch_name = branch_name.replace(' ', "_");

        // Validate git branch name rules
        if branch_name.starts_with('-') || branch_name.ends_with('.') || branch_name.ends_with('/')
        {
            return Err(Box::new(BGitError::new(
                "Invalid branch name",
                "Branch name cannot start with '-' or end with '.' or '/'.",
                BGitErrorWorkflowType::PromptStep,
                &self.name,
                NO_EVENT,
                NO_RULE,
            )));
        }

        // Check for invalid characters
        if branch_name.contains("..")
            || branch_name.chars().any(|c| {
                matches!(
                    c,
                    '~' | '^' | ':' | '?' | '*' | '[' | '\\' | '\x00'..='\x1f' | '\x7f'
                )
            })
        {
            return Err(Box::new(BGitError::new(
                "Invalid branch name",
                "Branch name contains invalid characters.",
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
