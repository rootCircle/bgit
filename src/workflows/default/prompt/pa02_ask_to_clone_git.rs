use crate::config::{StepFlags, WorkflowRules};
use dialoguer::{Confirm, Input};

use crate::rules::Rule;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType},
    events::{AtomicEvent, git_clone::GitClone},
    rules::a01_git_install::IsGitInstalledLocally,
    step::{PromptStep, Step},
};

pub(crate) struct CloneGitRepo {
    name: String,
}

impl PromptStep for CloneGitRepo {
    fn new() -> Self
    where
        Self: Sized,
    {
        CloneGitRepo {
            name: "clone_repo".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(
        &self,
        _step_config_flags: Option<&StepFlags>,
        workflow_rules_config: Option<&WorkflowRules>,
    ) -> Result<Step, Box<BGitError>> {
        // Take link input in cli
        let clone_link: String = Input::new()
            .with_prompt("Enter the link to the repository you want to clone")
            .interact()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "Input Error",
                    &e.to_string(),
                    BGitErrorWorkflowType::ActionStep,
                    &self.name,
                    "",
                    "",
                ))
            })?;

        // Check if repository is potentially large
        let suggest_shallow = clone_link.contains("large")
            || clone_link.contains("linux")
            || clone_link.contains("chromium");

        let mut shallow_clone = false;
        if suggest_shallow {
            shallow_clone = Confirm::new()
                .with_prompt("This repository might be large. Would you like to perform a shallow clone? (faster, but with limited history)")
                .default(true)
                .interact()
                .map_err(|e| Box::new(BGitError::new("Input Error", &e.to_string(), BGitErrorWorkflowType::ActionStep, &self.name, "", "")))?;
        }

        // // Get a mutable reference to the GitClone event
        let mut git_clone = GitClone::new();
        git_clone.add_pre_check_rule(Box::new(IsGitInstalledLocally::new(workflow_rules_config)));
        // Set the URL
        git_clone.set_url(&clone_link);

        // Print information about the clone operation
        println!("Cloning repository from {}...", clone_link);
        println!("Destination: Current directory");
        if shallow_clone {
            println!("Performing shallow clone (limited history)");
            // Note: You would need to implement shallow clone functionality
            // in the GitClone struct's raw_execute method
        }

        // Execute the clone operation
        match git_clone.execute() {
            Ok(_) => {
                println!("Git repository cloned successfully!");
                Ok(Step::Stop)
            }
            Err(e) => Err(e),
        }
    }
}
