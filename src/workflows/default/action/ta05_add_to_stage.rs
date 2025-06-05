use crate::{
    bgit_error::BGitError,
    events::{git_add::GitAdd, AtomicEvent},
    step::{ActionStep, Step},
};

use super::ta07_has_uncommitted::HasUncommitted;
use crate::step::Task::ActionStepTask;
pub(crate) struct AddToStaging {
    name: String,
}

impl ActionStep for AddToStaging {
    fn new() -> Self
    where
        Self: Sized,
    {
        AddToStaging {
            name: "add_to_staging".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        let git_add = GitAdd::new();
        git_add.execute()?;
        Ok(Step::Task(ActionStepTask(Box::new(HasUncommitted::new()))))
    }
}
