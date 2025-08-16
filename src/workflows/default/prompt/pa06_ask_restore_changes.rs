use crate::config::global::BGitGlobalConfig;
use crate::config::local::{StepFlags, WorkflowRules};
use crate::events::AtomicEvent;
use crate::workflows::default::action::ta07_has_uncommitted::HasUncommitted;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    events::git_restore::{GitRestore, RestoreMode},
    step::{ActionStep, PromptStep, Step, Task::ActionStepTask},
};
use dialoguer::{MultiSelect, theme::ColorfulTheme};
pub(crate) struct AskToRestore {
    name: String,
}

impl PromptStep for AskToRestore {
    fn new() -> Self
    where
        Self: Sized,
    {
        AskToRestore {
            name: "ask_to_restore".to_owned(),
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(
        &self,
        _step_config_flags: Option<&StepFlags>,
        _workflow_rules_config: Option<&WorkflowRules>,
        global_config: &BGitGlobalConfig,
    ) -> Result<Step, Box<BGitError>> {
        let selections = MultiSelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select restore options (Space to select, Enter to confirm, or press Enter with nothing selected to cancel)")            
            .items(&[
                "Restore unstaged changes (git restore .)",
                "Unstage all files (git restore --staged .)",
            ])
            .interact()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "Input Error",
                    &e.to_string(),
                    BGitErrorWorkflowType::PromptStep,
                    &self.name,
                    NO_EVENT,
                    NO_RULE,
                ))
            })?;

        if selections.is_empty() {
            return Ok(Step::Task(ActionStepTask(Box::new(HasUncommitted::new()))));
        }

        for &selection in &selections {
            match selection {
                0 => {
                    let git_restore =
                        GitRestore::new(global_config).with_mode(RestoreMode::RestoreAllUnstaged);
                    git_restore.execute()?;
                }
                1 => {
                    let git_restore =
                        GitRestore::new(global_config).with_mode(RestoreMode::UnstageAll);
                    git_restore.execute()?;
                }
                _ => {
                    return Err(Box::new(BGitError::new(
                        "Invalid selection",
                        "Unexpected selection index in MultiSelect prompt.",
                        BGitErrorWorkflowType::PromptStep,
                        &self.name,
                        NO_EVENT,
                        NO_RULE,
                    )));
                }
            }
        }
        Ok(Step::Task(ActionStepTask(Box::new(HasUncommitted::new()))))
    }
}
