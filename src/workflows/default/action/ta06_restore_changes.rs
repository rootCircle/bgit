use crate::{
    bgit_error::BGitError,
    events::{
        git_restore::{GitRestore, RestoreMode},
        AtomicEvent,
    },
    step::{ActionStep, Step},
};

use super::ta07_has_uncommitted::HasUncommitted;
use crate::step::Task::ActionStepTask;
pub(crate) struct RestoreChanges {
    name: String,
    restore_mode: Option<RestoreMode>,
}

impl RestoreChanges {
    pub fn with_mode(mode: RestoreMode) -> Self {
        RestoreChanges {
            name: "restore_changes".to_owned(),
            restore_mode: Some(mode),
        }
    }
}

impl ActionStep for RestoreChanges {
    fn new() -> Self
    where
        Self: Sized,
    {
        RestoreChanges {
            name: "restore_changes".to_owned(),
            restore_mode: None,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        let git_restore = if let Some(mode) = &self.restore_mode {
            GitRestore::with_mode(mode.clone())
        } else {
            return Err(Box::new(BGitError::new(
                "BGitError",
                "Restore mode not specified for restore changes operation",
                crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                crate::bgit_error::NO_EVENT,
                &self.name,
                crate::bgit_error::NO_RULE,
            )));
        };

        git_restore.execute()?;

        match &self.restore_mode {
            Some(RestoreMode::RestoreAllUnstaged) => {
                println!("Unstaged changes restored successfully.");
            }
            Some(RestoreMode::UnstageAll) => {
                println!("All files unstaged successfully.");
            }
            None => {
                println!("Restore operation completed successfully.");
            }
        }

        Ok(Step::Task(ActionStepTask(Box::new(HasUncommitted::new()))))
    }
}
