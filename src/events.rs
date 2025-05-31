use colored::Colorize;
use git2::Repository;
use std::env;

use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_RULE, NO_STEP},
    hook_executor::execute_hook_util,
    rules::Rule,
    util::find_hook_with_extension,
};
pub mod git_add;
mod git_branch;
mod git_checkout;
mod git_clean;
pub mod git_clone;
mod git_commit;
mod git_filter_repo;
pub mod git_init;
mod git_pull;
mod git_push;
mod git_restore;
pub mod git_stash;
pub mod git_status;

const PENGUIN_EMOJI: &str = "🐧";

pub(crate) enum HookType {
    PreEvent,
    PostEvent,
}

/// Sample struct
/// struct GitAdd {
///     name: String,
///     action_description: String,
///     pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>
/// }
/// List of various Git Events to be called with git2-rs library
pub(crate) trait AtomicEvent {
    fn new() -> Self
    where
        Self: Sized;
    fn get_name(&self) -> &str;

    #[allow(unused)]
    fn get_action_description(&self) -> &str;
    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>);
    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>>;
    // Plain execute the event, without any checks and hook
    fn raw_execute(&self) -> Result<bool, Box<BGitError>>;

    // Hooks
    fn pre_execute_hook(&self) -> Result<bool, Box<BGitError>> {
        let event_hook_file_name: String = format!("pre_{}", self.get_name());
        self.execute_hook(&event_hook_file_name, HookType::PreEvent)
    }

    fn post_execute_hook(&self) -> Result<bool, Box<BGitError>> {
        let post_event_hook_file_name: String = format!("post_{}", self.get_name());
        self.execute_hook(&post_event_hook_file_name, HookType::PostEvent)
    }

    /// Run hooks inside `{RepositoryBase}/.bgit/hooks/[pre|post]-{hook_name}`
    /// TODO: Implement for Windows and other OS using custom toml for custom runtime like
    /// shell and languages
    fn execute_hook(
        &self,
        event_hook_file_name: &str,
        hook_type: HookType,
    ) -> Result<bool, Box<BGitError>> {
        let cwd = env::current_dir().expect("Failed to get current directory");
        let git_repo = Repository::discover(&cwd);
        let bgit_hooks_path = match git_repo.is_ok() {
            true => {
                let git_repo = git_repo.unwrap();
                let git_repo_path = git_repo
                    .path()
                    .parent()
                    .expect("Failed to crawl to parent directory of .git folder");
                git_repo_path.join(".bgit").join("hooks")
            }
            false => cwd.join(".bgit").join("hooks"),
        };

        let event_hook_path = bgit_hooks_path.join(event_hook_file_name);
        match find_hook_with_extension(&event_hook_path) {
            None => Ok(true),
            Some(hook_path) => {
                let hook_type_str = match hook_type {
                    HookType::PreEvent => "pre",
                    HookType::PostEvent => "post",
                };
                eprintln!(
                    "{} Running {}-event hook for {}",
                    PENGUIN_EMOJI,
                    hook_type_str,
                    self.get_name().cyan().bold()
                );
                execute_hook_util(&hook_path, self.get_name())
            }
        }
    }

    // Check against set of rules before running the event
    fn check_rules(&self) -> Result<bool, Box<BGitError>> {
        let rules = self.get_pre_check_rule();
        if rules.is_empty() {
            return Ok(true);
        }
        eprintln!(
            "{} Running pre-check rules for {}",
            PENGUIN_EMOJI,
            self.get_name().cyan().bold()
        );
        for rule in rules.iter() {
            let rule_passed = rule.execute()?;
            if !rule_passed {
                return Err(Box::new(BGitError::new(
                    "Pre-check Rule failed",
                    rule.get_description(),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    rule.get_name(),
                )));
            }
        }
        Ok(true)
    }

    fn execute(&self) -> Result<bool, Box<BGitError>> {
        eprintln!("Running event: {}", self.get_name());
        let rule_check_status = self.check_rules()?;
        if !rule_check_status {
            return Ok(false);
        }
        let event_hook_status = self.pre_execute_hook()?;
        if !event_hook_status {
            return Err(Box::new(BGitError::new(
                "Pre-event hook failed",
                "Pre-event hook failed!",
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            )));
        }

        eprintln!(
            "{} Running executor for event {}",
            PENGUIN_EMOJI,
            self.get_name().cyan().bold()
        );
        let raw_executor_status = self.raw_execute()?;
        if !raw_executor_status {
            return Err(Box::new(BGitError::new(
                "Raw executor failed",
                "Raw executor failed!",
                BGitErrorWorkflowType::RawExecutor,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            )));
        }

        let post_event_hook_status = self.post_execute_hook()?;
        if !post_event_hook_status {
            return Err(Box::new(BGitError::new(
                "Post-event hook failed",
                "Post-event hook failed!",
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            )));
        }
        Ok(true)
    }
}
