use crate::{
    bgit_error::BGitError,
    events::{git_add::GitAdd, AtomicEvent},
    step::{ActionStep, Step},
};

pub(crate) struct AddToStaging {
    name: String,
    add_all: bool,
}

impl AddToStaging {
    #[allow(dead_code)]
    pub fn with_all() -> Self {
        AddToStaging {
            name: "add_to_staging".to_owned(),
            add_all: true,
        }
    }

    #[allow(dead_code)]
    pub fn with_selective() -> Self {
        AddToStaging {
            name: "add_to_staging".to_owned(),
            add_all: false,
        }
    }

    #[allow(dead_code)]
    pub fn set_add_all(&mut self, add_all: bool) {
        self.add_all = add_all;
    }
}

impl ActionStep for AddToStaging {
    fn new() -> Self
    where
        Self: Sized,
    {
        AddToStaging {
            name: "add_to_staging".to_owned(),
            add_all: false,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        let git_add = GitAdd::new();
        git_add.execute()?;
        // CHANGE
        Ok(Step::Stop)
    }
}
