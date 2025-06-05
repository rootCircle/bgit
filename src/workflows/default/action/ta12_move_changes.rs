use crate::events::git_branch::GitBranch;
use crate::events::AtomicEvent;
use crate::step::PromptStep;
use crate::step::Task::PromptStepTask;
use crate::workflows::default::prompt::pa08_ask_commit::AskCommit;
use crate::{
    bgit_error::BGitError,
    step::{ActionStep, Step},
};
pub(crate) struct MoveChanges {
    name: String,
    target_branch_name: Option<String>,
    stash_message: Option<String>,
}

impl ActionStep for MoveChanges {
    fn new() -> Self
    where
        Self: Sized,
    {
        MoveChanges {
            name: "move_changes".to_owned(),
            target_branch_name: None,
            stash_message: None,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        // Check if target branch name is provided
        let target_branch = match &self.target_branch_name {
            Some(name) => name.clone(),
            None => {
                return Err(Box::new(BGitError::new(
                    "BGitError",
                    "Target branch name not provided for move changes operation",
                    crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                    crate::bgit_error::NO_EVENT,
                    &self.name,
                    crate::bgit_error::NO_RULE,
                )));
            }
        };

        // Create GitBranch instance with MoveChanges operation
        let mut git_branch = GitBranch::move_changes_to_branch(target_branch);

        // Set custom stash message if provided
        if let Some(ref message) = self.stash_message {
            git_branch.with_stash_message(message.clone());
        }

        // Execute move changes operation
        match git_branch.execute() {
            Ok(_) => Ok(Step::Task(PromptStepTask(Box::new(AskCommit::new())))),
            Err(e) => Err(e),
        }
    }
}

impl MoveChanges {
    /// Create a new MoveChanges instance with a target branch name
    pub fn with_target_branch(target_branch_name: String) -> Self {
        MoveChanges {
            name: "move_changes".to_owned(),
            target_branch_name: Some(target_branch_name),
            stash_message: None,
        }
    }
}
