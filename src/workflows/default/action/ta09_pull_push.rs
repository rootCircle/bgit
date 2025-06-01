use crate::events::git_pull::GitPull;
use crate::events::git_push::GitPush;
use crate::events::AtomicEvent;

use crate::{
    bgit_error::BGitError,
    step::{ActionStep, Step},
};

pub(crate) struct PullAndPush {
    name: String,
}

impl ActionStep for PullAndPush {
    fn new() -> Self
    where
        Self: Sized,
    {
        PullAndPush {
            name: "pull_and_push".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        // Create GitPull instance with rebase flag enabled
        let mut git_pull = GitPull::new();
        git_pull.set_rebase(true);

        // Execute pull with rebase
        match git_pull.raw_execute() {
            Ok(_) => {
                // Pull successful, now attempt push
                let mut git_push = GitPush::new();
                // Configure push options - you can customize these as needed
                git_push.set_force(false).set_upstream_flag(false);

                match git_push.raw_execute() {
                    Ok(_) => {
                        // Both pull and push successful
                        Ok(Step::Stop)
                    }
                    Err(e) => {
                        // Push failed, return error
                        Err(e)
                    }
                }
            }
            Err(e) => {
                // Pull failed, return error
                Err(e)
            }
        }
    }
}
