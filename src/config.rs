use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;
use git2::Repository;
use crate::bgit_error::BGitError;
use crate::rules::RuleLevel;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BGitConfig {
    #[serde(default)]
    pub rules: RuleConfig,
    #[serde(default)]
    pub workflow: WorkflowConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct RuleConfig {
    /// Workflow-specific rule settings - maps workflow name to its rules
    #[serde(flatten)]
    pub workflows: HashMap<String, WorkflowRules>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct WorkflowRules {
    /// Rule settings for a specific workflow - maps rule name to its level
    #[serde(flatten)]
    pub rule_levels: HashMap<String, RuleLevel>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct WorkflowConfig {
    /// Workflow configurations - maps workflow name to its configuration
    #[serde(flatten)]
    pub workflows: HashMap<String, WorkflowSteps>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct WorkflowSteps {
    /// Step configurations - maps step name to its flag overrides
    #[serde(flatten)]
    pub steps: HashMap<String, StepFlags>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct StepFlags {
    /// Flag overrides - maps flag name to its value
    #[serde(flatten)]
    pub flags: HashMap<String, serde_json::Value>,
}

impl Default for BGitConfig {
    fn default() -> Self {
        Self {
            rules: RuleConfig::default(),
            workflow: WorkflowConfig::default(),
        }
    }
}

impl BGitConfig {
    /// Load config from .bgit/config.toml at repository root
    pub fn load() -> Result<Self, Box<BGitError>> {
        let config_path = Self::find_config_path()?;
        
        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let config_content = fs::read_to_string(&config_path)
            .map_err(|e| Box::new(BGitError::new(
                "Failed to read config file",
                &format!("Could not read {}: {}", config_path.display(), e),
                crate::bgit_error::BGitErrorWorkflowType::Config,
                crate::bgit_error::NO_STEP,
                crate::bgit_error::NO_EVENT,
                crate::bgit_error::NO_RULE,
            )))?;

        let config: BGitConfig = toml::from_str(&config_content)
            .map_err(|e| Box::new(BGitError::new(
                "Failed to parse config file",
                &format!("Invalid TOML in {}: {}", config_path.display(), e),
                crate::bgit_error::BGitErrorWorkflowType::Config,
                crate::bgit_error::NO_STEP,
                crate::bgit_error::NO_EVENT,
                crate::bgit_error::NO_RULE,
            )))?;

        Ok(config)
    }

    /// Find the config file path, looking for .bgit/config.toml at repository root
    fn find_config_path() -> Result<PathBuf, Box<BGitError>> {
        let cwd = env::current_dir()
            .map_err(|e| Box::new(BGitError::new(
                "Failed to get current directory",
                &e.to_string(),
                crate::bgit_error::BGitErrorWorkflowType::Config,
                crate::bgit_error::NO_STEP,
                crate::bgit_error::NO_EVENT,
                crate::bgit_error::NO_RULE,
            )))?;

        // Try to find git repository root
        match Repository::discover(&cwd) {
            Ok(repo) => {
                let repo_root = repo.path()
                    .parent()
                    .ok_or_else(|| Box::new(BGitError::new(
                        "Failed to find repository root",
                        "Could not determine repository root directory",
                        crate::bgit_error::BGitErrorWorkflowType::Config,
                        crate::bgit_error::NO_STEP,
                        crate::bgit_error::NO_EVENT,
                        crate::bgit_error::NO_RULE,
                    )))?;
                Ok(repo_root.join(".bgit").join("config.toml"))
            }
            Err(_) => {
                // If not in a git repo, use current directory
                Ok(cwd.join(".bgit").join("config.toml"))
            }
        }
    }

    /// Get rule level for a specific rule in a workflow, returns None if not configured
    pub fn get_rule_level(&self, workflow_name: &str, rule_name: &str) -> Option<&RuleLevel> {
        self.rules
            .workflows
            .get(workflow_name)?
            .rule_levels
            .get(rule_name)
    }

    /// Get rule level with default workflow fallback
    pub fn get_rule_level_or_default(&self, workflow_name: &str, rule_name: &str) -> Option<&RuleLevel> {
        // Try specific workflow first, then fall back to "default" workflow
        self.get_rule_level(workflow_name, rule_name)
            .or_else(|| self.get_rule_level("default", rule_name))
    }

    /// Get flag value for a specific workflow step
    pub fn get_workflow_flag<T>(&self, workflow_name: &str, step_name: &str, flag_name: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.workflow
            .workflows
            .get(workflow_name)?
            .steps
            .get(step_name)?
            .flags
            .get(flag_name)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    /// Get flag value with default fallback
    pub fn get_workflow_flag_or_default<T>(&self, workflow_name: &str, step_name: &str, flag_name: &str, default: T) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        self.get_workflow_flag(workflow_name, step_name, flag_name)
            .unwrap_or(default)
    }

    /// Check if a workflow step has a specific flag set
    pub fn has_workflow_flag(&self, workflow_name: &str, step_name: &str, flag_name: &str) -> bool {
        self.workflow
            .workflows
            .get(workflow_name)
            .and_then(|w| w.steps.get(step_name))
            .and_then(|s| s.flags.get(flag_name))
            .is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BGitConfig::default();
        assert!(config.rules.workflows.is_empty());
        assert!(config.workflow.workflows.is_empty());
    }

    #[test]
    fn test_config_parsing() {
        let toml_content = r#"
[rules.default]
a01_git_install = "Warning"
a02_git_name_email_setup = "Error"
a12_no_secrets_staged = "Skip"

[rules.git_commit]
a01_git_install = "Error"
a17_conventional_commit_message = "Warning"

[workflow.default.is_sole_contributor]
overrideCheckForAuthors = ["Lab Rat <dev.frolics@gmail.com>"]
skipAddAll = true
force = false

[workflow.git_commit.git_add]
skipAddAll = true
includeUntracked = false
maxFileSize = 100

[workflow.git_push.pre_push_checks]
skipLinting = true
timeout = 30
"#;

        let config: BGitConfig = toml::from_str(toml_content).unwrap();
        
        // Test workflow-specific rule parsing
        assert_eq!(config.get_rule_level("default", "a01_git_install"), Some(&RuleLevel::Warning));
        assert_eq!(config.get_rule_level("default", "a02_git_name_email_setup"), Some(&RuleLevel::Error));
        assert_eq!(config.get_rule_level("default", "a12_no_secrets_staged"), Some(&RuleLevel::Skip));
        
        assert_eq!(config.get_rule_level("git_commit", "a01_git_install"), Some(&RuleLevel::Error));
        assert_eq!(config.get_rule_level("git_commit", "a17_conventional_commit_message"), Some(&RuleLevel::Warning));
        
        assert_eq!(config.get_rule_level("nonexistent", "a01_git_install"), None);
        assert_eq!(config.get_rule_level("default", "nonexistent_rule"), None);
        
        // Test fallback to default workflow
        assert_eq!(config.get_rule_level_or_default("some_workflow", "a01_git_install"), Some(&RuleLevel::Warning));
        assert_eq!(config.get_rule_level_or_default("git_commit", "a01_git_install"), Some(&RuleLevel::Error));
        
        // Test workflow flag parsing
        let authors: Option<Vec<String>> = config.get_workflow_flag("default", "is_sole_contributor", "overrideCheckForAuthors");
        assert_eq!(authors, Some(vec!["Lab Rat <dev.frolics@gmail.com>".to_string()]));
        
        assert_eq!(config.get_workflow_flag::<bool>("default", "is_sole_contributor", "skipAddAll"), Some(true));
        assert_eq!(config.get_workflow_flag::<bool>("default", "is_sole_contributor", "force"), Some(false));
        
        assert_eq!(config.get_workflow_flag::<bool>("git_commit", "git_add", "skipAddAll"), Some(true));
        assert_eq!(config.get_workflow_flag::<bool>("git_commit", "git_add", "includeUntracked"), Some(false));
        assert_eq!(config.get_workflow_flag::<i32>("git_commit", "git_add", "maxFileSize"), Some(100));
        
        assert_eq!(config.get_workflow_flag::<bool>("git_push", "pre_push_checks", "skipLinting"), Some(true));
        assert_eq!(config.get_workflow_flag::<i32>("git_push", "pre_push_checks", "timeout"), Some(30));
        
        // Test default fallback
        assert_eq!(config.get_workflow_flag_or_default("nonexistent", "step", "flag", true), true);
        assert_eq!(config.get_workflow_flag_or_default("git_commit", "git_add", "nonexistent", 42), 42);
        
        // Test flag existence
        assert!(config.has_workflow_flag("git_commit", "git_add", "skipAddAll"));
        assert!(!config.has_workflow_flag("git_commit", "git_add", "nonexistent"));
    }
}