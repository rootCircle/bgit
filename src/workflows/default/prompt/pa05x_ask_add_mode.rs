use crate::config::{StepFlags, WorkflowRules};
use crate::events::git_add::{AddMode, GitAdd};
use crate::events::{AtomicEvent, git_status};
use crate::rules::Rule;
use crate::rules::a12_no_secrets_staged::NoSecretsStaged;
use crate::rules::a12b_no_secret_files_staged::NoSecretFilesStaged;
use crate::rules::a16_no_large_file::NoLargeFile;
use crate::step::ActionStep;
use crate::step::Task::ActionStepTask;
use crate::workflows::default::action::ta07_has_uncommitted::HasUncommitted;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    step::{PromptStep, Step},
};

use dialoguer::{MultiSelect, Select, theme::ColorfulTheme};
pub(crate) struct AskAddMode {
    name: String,
}

impl PromptStep for AskAddMode {
    fn new() -> Self
    where
        Self: Sized,
    {
        AskAddMode {
            name: "ask_add_mode".to_owned(),
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
        let options = vec!["Add all unstaged files", "Select specific files to add"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose add mode:")
            .default(0)
            .items(&options)
            .interact()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get user selection: {e}"),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        let add_mode = match selection {
            0 => AddMode::All,
            1 => {
                // Handle selective mode with file selection prompt
                let selected_files = self.prompt_file_selection()?;
                if selected_files.is_empty() {
                    println!("No files selected.");
                    return Ok(Step::Stop);
                }
                AddMode::Selective(selected_files)
            }
            _ => AddMode::All,
        };

        // Create GitAdd instance with the selected mode and execute
        let mut git_add = GitAdd::new().with_add_mode(add_mode);

        git_add.add_pre_check_rule(Box::new(NoSecretsStaged::new(workflow_rules_config)));
        git_add.add_pre_check_rule(Box::new(NoSecretFilesStaged::new(workflow_rules_config)));
        git_add.add_pre_check_rule(Box::new(NoLargeFile::new(workflow_rules_config)));

        git_add.execute()?;

        Ok(Step::Task(ActionStepTask(Box::new(HasUncommitted::new()))))
    }
}

impl AskAddMode {
    fn prompt_file_selection(&self) -> Result<Vec<String>, Box<BGitError>> {
        let unstaged_files = git_status::get_unstaged_files_list()?;

        if unstaged_files.is_empty() {
            println!("No unstaged files found.");
            return Ok(vec![]);
        }

        let file_display: Vec<String> = unstaged_files
            .iter()
            .map(|file| format!("{} ({})", file.path, file.status_type))
            .collect();

        if file_display.is_empty() {
            println!("No files to select.");
            return Ok(vec![]);
        }

        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select files to add (use Space to select, Enter to confirm):")
            .items(&file_display)
            .interact()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get file selections: {e}"),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        if selections.is_empty() {
            return Ok(vec![]);
        }

        let selected_files: Vec<String> = selections
            .iter()
            .map(|&i| unstaged_files[i].path.clone())
            .collect();

        Ok(selected_files)
    }
}
