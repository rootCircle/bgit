use crate::step::PromptStep;
use crate::step::Task::PromptStepTask;
use crate::workflows::default::prompt::pa04_ask_pop_stash::AskPopStash;
use crate::{
    bgit_error::BGitError,
    events::{git_add::GitAdd, AtomicEvent},
    rules::{a01_git_install::IsGitInstalledLocally, Rule},
    step::{ActionStep, Step},
};
use git2::Repository;
use std::env;
pub(crate) struct HasStash {
    name: String,
}

impl ActionStep for HasStash {
    fn new() -> Self
    where
        Self: Sized,
    {
        HasStash {
            name: "has_stash".to_owned(),
        }
    }
    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        let cwd = env::current_dir().expect("Failed to get current directory");
        if Repository::discover(&cwd).is_ok() {
            let mut repo = Repository::discover(cwd).unwrap();
            let mut has_stash: bool = false;

            let _ = repo
                .stash_foreach(|_, _, _| {
                    has_stash = true;
                    false
                })
                .is_ok();

            let mut git_add_event = GitAdd::new();
            git_add_event.add_pre_check_rule(Box::new(IsGitInstalledLocally::new()));
            git_add_event.execute()?;

            if has_stash {
                println!("Stash exists in the repository.");
                Ok(Step::Task(PromptStepTask(Box::new(AskPopStash::new()))))
            } else {
                println!("No stash found in the repository.");
                // Prompt step:  has unstaged files
                Ok(Step::Stop)
            }
        } else {
            Ok(Step::Stop)
        }
    }
}
