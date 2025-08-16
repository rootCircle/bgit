use crate::bgit_error::BGitError;
use crate::config::local::WorkflowRules;
use crate::rules::{Rule, RuleLevel, RuleOutput};
use git2::Config;

pub(crate) struct GitNameEmailSetup {
    name: String,
    description: String,
    level: RuleLevel,
}

impl Rule for GitNameEmailSetup {
    fn new(workflow_rule_config: Option<&WorkflowRules>) -> Self {
        let default_rule_level = RuleLevel::Error;
        let name = "GitNameEmailSetup";
        let rule_level = workflow_rule_config
            .and_then(|config| config.get_rule_level(name))
            .cloned()
            .unwrap_or(default_rule_level);

        Self {
            name: name.to_string(),
            description: "Ensure Git user.name and user.email are configured".to_string(),
            level: rule_level,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_description(&self) -> &str {
        &self.description
    }

    fn get_level(&self) -> RuleLevel {
        self.level.clone()
    }

    fn check(&self) -> Result<RuleOutput, Box<BGitError>> {
        let config = match Config::open_default() {
            Ok(config) => config,
            Err(e) => {
                return Ok(RuleOutput::Exception(format!(
                    "Failed to open Git config: {e}"
                )));
            }
        };

        let user_name = config.get_string("user.name").ok();
        let user_email = config.get_string("user.email").ok();

        match (user_name, user_email) {
            (Some(name), Some(email)) if !name.trim().is_empty() && !email.trim().is_empty() => {
                Ok(RuleOutput::Success)
            }
            _ => Ok(RuleOutput::Exception(
                "Git user.name and/or user.email is not configured".to_string(),
            )),
        }
    }

    fn try_fix(&self) -> Result<bool, Box<BGitError>> {
        println!("Git user configuration is missing. Please run the following commands:");
        println!("  git config --global user.name \"Your Name\"");
        println!("  git config --global user.email \"your.email@example.com\"");

        Ok(false)
    }
}
