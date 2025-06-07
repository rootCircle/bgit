
use crate::bgit_error::{BGitError};
use crate::rules::{Rule, RuleLevel, RuleOutput};
use git2::{Config};

pub(crate) struct GitNameEmailSetup {
    name: String,
    description: String,
    level: RuleLevel,
}

impl Rule for GitNameEmailSetup {
    fn new() -> Self {
        GitNameEmailSetup {
            name: "GitNameEmailSetup".to_string(),
            description: "Ensure Git user.name and user.email are configured".to_string(),
            level: RuleLevel::Error,
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
                    "Failed to open Git config: {}",
                    e
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