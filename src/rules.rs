use serde::{Deserialize, Serialize};

use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_STEP},
    config::local::WorkflowRules,
};

pub(crate) mod a01_git_install;
pub(crate) mod a02_git_name_email_setup;
mod a03_github_username;
mod a04_gitlab_username;
mod a05_github_credentials_http;
mod a06_gitlab_credentials_http;
mod a07_github_credentials_ssh;
mod a08_gitlab_credentials_ssh;
mod a09_commit_gpg_sign;
mod a11_git_remote_http_ssh;
pub(crate) mod a12_no_secrets_staged;
pub(crate) mod a12b_no_secret_files_staged;
mod a13_git_lfs;
pub(crate) mod a14_big_repo_size;
mod a15_file_not_gitignored;
pub(crate) mod a16_no_large_file;
pub(crate) mod a17_conventional_commit_message;
pub(crate) mod a18_remote_exists;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub(crate) enum RuleLevel {
    /// Skip the rule check
    Skip,
    /// Emit a warning if the rule is not satisfied and try to fix it, but continue
    Warning,
    /// Emit an error if the rule is not satisfied and try to fix it, but stop if not fixable
    Error,
}

pub(crate) enum RuleOutput {
    /// If Rule check has failed!
    Exception(String),
    /// If Rule check is passed!
    Success,
}

/// Sample struct for Rule
/// pub(crate) struct SampleRule {
///     name: String,
///     description: String,
///     level: RuleLevel
/// }
pub(crate) trait Rule {
    /// Sample constructor for Rule
    /// This function should be implemented to create a new instance of the rule.
    /// It should take an optional `WorkflowRules` configuration to determine the rule level.
    /// This is from `.bgit/config.toml` file, if a given rule needs to be overriden.
    ///
    /// Ensure that you do override the rule level as is provided in the implementation below.
    /// ```rust
    /// fn new(workflow_rule_config: Option<&WorkflowRules>) -> Self {
    ///     let default_rule_level = RuleLevel::Error; // or anthing else you want for this rule
    ///     let name = "IsGitInstalledLocally";
    ///     let rule_level = workflow_rule_config
    ///         .and_then(|config| config.get_rule_level(name))
    ///         .cloned()
    ///         .unwrap_or(default_rule_level); // MUST DO
    ///
    ///     // Can add other fields as needed
    ///     Self {
    ///         name: name.to_string(),
    ///         description: "Check if Git is installed".to_string(), // or anything else you want for this rule
    ///         level: rule_level,
    ///     }
    /// }
    /// ```
    fn new(config_rule_level: Option<&WorkflowRules>) -> Self
    where
        Self: Sized;
    fn get_name(&self) -> &str;
    fn get_description(&self) -> &str;
    fn get_level(&self) -> RuleLevel;

    /// Implement logic to check the rule
    fn check(&self) -> Result<RuleOutput, Box<BGitError>>;

    /// Implement logic to fix the rule if broken
    fn try_fix(&self) -> Result<bool, Box<BGitError>>;

    fn execute(&self) -> Result<bool, Box<BGitError>> {
        if self.get_level() == RuleLevel::Skip {
            return Ok(true);
        }
        let check_report = self.check()?;
        match check_report {
            RuleOutput::Success => Ok(true),
            RuleOutput::Exception(exception) => {
                let fix_report = self.try_fix()?;
                if self.get_level() == RuleLevel::Warning {
                    // No need to verify as it's a warning level!
                    Ok(true)
                } else if fix_report {
                    let verify_report = self.verify()?;
                    if verify_report {
                        Ok(true)
                    } else {
                        Err(Box::new(BGitError::new(
                            "Failed to verify the rule",
                            &exception,
                            BGitErrorWorkflowType::Rules,
                            NO_STEP,
                            NO_EVENT,
                            self.get_name(),
                        )))
                    }
                } else {
                    Err(Box::new(BGitError::new(
                        "Failed to fix the rule",
                        &exception,
                        BGitErrorWorkflowType::Rules,
                        NO_STEP,
                        NO_EVENT,
                        self.get_name(),
                    )))
                }
            }
        }
    }

    fn verify(&self) -> Result<bool, Box<BGitError>> {
        match self.check()? {
            RuleOutput::Success => Ok(true),
            RuleOutput::Exception(_) => Ok(false),
        }
    }
}
