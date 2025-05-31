use crate::{
    bgit_error::BGitError,
    events::{git_stash::GitStash, AtomicEvent},
    step::{ActionStep, Step},
};

pub(crate) struct PopStash {
    name: String,
    stash_index: Option<usize>,
}

impl PopStash {
    #[allow(dead_code)]
    pub fn with_index(index: usize) -> Self {
        PopStash {
            name: "pop_stash".to_owned(),
            stash_index: Some(index),
        }
    }
    #[allow(dead_code)]
    pub fn set_index(&mut self, index: usize) {
        self.stash_index = Some(index);
    }
}

impl ActionStep for PopStash {
    fn new() -> Self
    where
        Self: Sized,
    {
        PopStash {
            name: "pop_stash".to_owned(),
            stash_index: None,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        let git_stash = GitStash::pop_stash(self.stash_index);

        git_stash.raw_execute()?;
        // change thissssssss to ask add to add file
        println!("Stash popped successfully.");
        Ok(Step::Stop)
    }
}
