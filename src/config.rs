#![allow(dead_code)]

use crate::bgit_error::BGitError;
use crate::rules::RuleLevel;
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default, Serialize, Clone)]
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

impl BGitConfig {
    /// Load config from .bgit/config.toml at repository root
    pub fn load() -> Result<Self, Box<BGitError>> {
        let config_path = Self::find_config_path()?;

        if !config_path.exists() {
            // Return default config if file doesn't exist
            return Ok(Self::default());
        }

        let config_content = fs::read_to_string(&config_path).map_err(|e| {
            Box::new(BGitError::new(
                "Failed to read config file",
                &format!("Could not read {}: {}", config_path.display(), e),
                crate::bgit_error::BGitErrorWorkflowType::Config,
                crate::bgit_error::NO_STEP,
                crate::bgit_error::NO_EVENT,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        let config: BGitConfig = toml::from_str(&config_content).map_err(|e| {
            Box::new(BGitError::new(
                "Failed to parse config file",
                &format!("Invalid TOML in {}: {}", config_path.display(), e),
                crate::bgit_error::BGitErrorWorkflowType::Config,
                crate::bgit_error::NO_STEP,
                crate::bgit_error::NO_EVENT,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        Ok(config)
    }

    /// Find the config file path, looking for .bgit/config.toml at repository root
    fn find_config_path() -> Result<PathBuf, Box<BGitError>> {
        let cwd = env::current_dir().map_err(|e| {
            Box::new(BGitError::new(
                "Failed to get current directory",
                &e.to_string(),
                crate::bgit_error::BGitErrorWorkflowType::Config,
                crate::bgit_error::NO_STEP,
                crate::bgit_error::NO_EVENT,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        // Try to find git repository root
        match Repository::discover(&cwd) {
            Ok(repo) => {
                let repo_root = repo.path().parent().ok_or_else(|| {
                    Box::new(BGitError::new(
                        "Failed to find repository root",
                        "Could not determine repository root directory",
                        crate::bgit_error::BGitErrorWorkflowType::Config,
                        crate::bgit_error::NO_STEP,
                        crate::bgit_error::NO_EVENT,
                        crate::bgit_error::NO_RULE,
                    ))
                })?;
                Ok(repo_root.join(".bgit").join("config.toml"))
            }
            Err(_) => {
                // If not in a git repo, use current directory
                Ok(cwd.join(".bgit").join("config.toml"))
            }
        }
    }

    /// Get workflow rules for a specific workflow
    pub fn get_workflow_rules(&self, workflow_name: &str) -> Option<&WorkflowRules> {
        self.rules.workflows.get(workflow_name)
    }

    /// Get workflow rules with default fallback
    pub fn get_workflow_rules_or_default(&self, workflow_name: &str) -> Option<&WorkflowRules> {
        self.get_workflow_rules(workflow_name)
            .or_else(|| self.get_workflow_rules("default"))
    }

    /// Get workflow steps for a specific workflow
    pub fn get_workflow_steps(&self, workflow_name: &str) -> Option<&WorkflowSteps> {
        self.workflow.workflows.get(workflow_name)
    }

    /// Get workflow steps with default fallback
    pub fn get_workflow_steps_or_default(&self, workflow_name: &str) -> Option<&WorkflowSteps> {
        self.get_workflow_steps(workflow_name)
            .or_else(|| self.get_workflow_steps("default"))
    }
}

impl WorkflowRules {
    /// Get rule level for a specific rule
    pub fn get_rule_level(&self, rule_name: &str) -> Option<&RuleLevel> {
        self.rule_levels.get(rule_name)
    }

    /// Check if a rule is configured
    pub fn has_rule(&self, rule_name: &str) -> bool {
        self.rule_levels.contains_key(rule_name)
    }

    /// Get all rule names
    pub fn get_rule_names(&self) -> Vec<&String> {
        self.rule_levels.keys().collect()
    }
}

impl WorkflowSteps {
    /// Get step flags for a specific step
    pub fn get_step_flags(&self, step_name: &str) -> Option<&StepFlags> {
        self.steps.get(step_name)
    }

    /// Check if a step is configured
    pub fn has_step(&self, step_name: &str) -> bool {
        self.steps.contains_key(step_name)
    }
}

impl StepFlags {
    /// Get flag value for a specific flag
    pub fn get_flag<T>(&self, flag_name: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.flags
            .get(flag_name)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    /// Get flag value with default fallback
    pub fn get_flag_or_default<T>(&self, flag_name: &str, default: T) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        self.get_flag(flag_name).unwrap_or(default)
    }

    /// Check if a flag exists
    pub fn has_flag(&self, flag_name: &str) -> bool {
        self.flags.contains_key(flag_name)
    }

    /// Get all flag names
    pub fn get_flag_names(&self) -> Vec<&String> {
        self.flags.keys().collect()
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

        // Test workflow rules access
        let default_rules = config.get_workflow_rules("default").unwrap();
        assert_eq!(
            default_rules.get_rule_level("a01_git_install"),
            Some(&RuleLevel::Warning)
        );
        assert_eq!(
            default_rules.get_rule_level("a02_git_name_email_setup"),
            Some(&RuleLevel::Error)
        );
        assert_eq!(
            default_rules.get_rule_level("a12_no_secrets_staged"),
            Some(&RuleLevel::Skip)
        );
        assert!(default_rules.has_rule("a01_git_install"));
        assert!(!default_rules.has_rule("nonexistent_rule"));

        let git_commit_rules = config.get_workflow_rules("git_commit").unwrap();
        assert_eq!(
            git_commit_rules.get_rule_level("a01_git_install"),
            Some(&RuleLevel::Error)
        );
        assert_eq!(
            git_commit_rules.get_rule_level("a17_conventional_commit_message"),
            Some(&RuleLevel::Warning)
        );

        // Test fallback to default workflow
        let fallback_rules = config
            .get_workflow_rules_or_default("some_workflow")
            .unwrap();
        assert_eq!(
            fallback_rules.get_rule_level("a01_git_install"),
            Some(&RuleLevel::Warning)
        );

        // Test workflow steps access
        let default_steps = config.get_workflow_steps("default").unwrap();
        assert!(default_steps.has_step("is_sole_contributor"));

        // Test step flags direct access
        let step_flags = default_steps.get_step_flags("is_sole_contributor").unwrap();
        assert_eq!(step_flags.get_flag::<bool>("skipAddAll"), Some(true));
        assert!(!step_flags.get_flag_or_default::<bool>("nonexistent", false));
        assert!(step_flags.has_flag("skipAddAll"));
        assert!(!step_flags.has_flag("nonexistent"));
    }

    #[test]
    fn test_workflow_structure_methods() {
        let toml_content = r#"
[rules.default]
a01_git_install = "Warning"
a02_git_name_email_setup = "Error"

[workflow.default.is_sole_contributor]
overrideCheckForAuthors = ["Test User"]
skipAddAll = true

[workflow.default.git_add]
includeUntracked = false
maxFileSize = 100
"#;

        let config: BGitConfig = toml::from_str(toml_content).unwrap();

        let default_rules = config.get_workflow_rules("default").unwrap();
        let rule_names = default_rules.get_rule_names();
        assert!(rule_names.contains(&&"a01_git_install".to_string()));
        assert!(rule_names.contains(&&"a02_git_name_email_setup".to_string()));

        let default_steps = config.get_workflow_steps("default").unwrap();

        let step_flags = default_steps.get_step_flags("is_sole_contributor").unwrap();
        let flag_names = step_flags.get_flag_names();
        assert!(flag_names.contains(&&"overrideCheckForAuthors".to_string()));
        assert!(flag_names.contains(&&"skipAddAll".to_string()));
    }
}
