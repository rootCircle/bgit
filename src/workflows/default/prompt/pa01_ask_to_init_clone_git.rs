use crate::config::{StepFlags, WorkflowRules};
use crate::{
    bgit_error::BGitError,
    step::{
        PromptStep, Step,
        Task::{self},
    },
    workflows::default::{
        prompt::pa02_ask_to_clone_git::CloneGitRepo, prompt::pa03_init_git_repo::InitGitRepo,
    },
};
use dialoguer::{Input, Select, theme::ColorfulTheme};

pub(crate) struct AskToInitCloneGit {
    name: String,
}

impl PromptStep for AskToInitCloneGit {
    fn new() -> Self
    where
        Self: Sized,
    {
        AskToInitCloneGit {
            name: "ask_to_init_git".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(
        &self,
        _step_config_flags: Option<&StepFlags>,
        _workflow_rules_config: Option<&WorkflowRules>,
    ) -> Result<Step, Box<BGitError>> {
        let options = vec![
            "Initialize a new Git repository",
            "Clone an existing repository",
            "Cancel",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("What would you like to do?")
            .default(0)
            .items(&options)
            .interact()
            .unwrap();

        match selection {
            // Initialize a new repository
            0 => {
                let path: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter path (. for current path)")
                    .default(".".into())
                    .interact_text()
                    .unwrap();

                println!("Initializing Git repository at: {path}");

                // Create InitGitRepo with path

                let mut init_git_repo = InitGitRepo::new();

                init_git_repo.set_path(&path);

                Ok(Step::Task(Task::PromptStepTask(Box::new(init_git_repo))))
            }
            //  Clone an existing repository
            1 => {
                // Clone an existing repository - redirect to clone_repo action
                println!("Preparing to clone a repository...");
                Ok(Step::Task(Task::PromptStepTask(Box::new(
                    CloneGitRepo::new(),
                ))))
            }
            _ => Ok(Step::Stop),
        }
    }
}
