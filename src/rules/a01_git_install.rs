use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_STEP};
use crate::rules::{Rule, RuleLevel, RuleOutput};
use std::process::Command;

pub(crate) struct IsGitInstalledLocally {
    name: String,
    description: String,
    level: RuleLevel,
}

impl Rule for IsGitInstalledLocally {
    fn new() -> Self {
        IsGitInstalledLocally {
            name: "IsGitInstalledLocally".to_string(),
            description: "Check if Git is installed".to_string(),
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
        let output = Command::new("git").arg("--version").output();
        match output {
            Err(e) => Ok(RuleOutput::Exception(format!(
                "Failed to execute command: {}",
                e
            ))),
            Ok(output_response) => {
                if output_response.status.success() {
                    Ok(RuleOutput::Success)
                } else {
                    Ok(RuleOutput::Exception("Git is not installed".to_string()))
                }
            }
        }
    }

    fn try_fix(&self) -> Result<bool, Box<BGitError>> {
        println!("Executing sudo apt-get install git");

        #[cfg(target_os = "linux")]
        let output = Command::new("sudo")
            .arg("apt-get")
            .arg("install")
            .arg("git")
            .output();

        #[cfg(target_os = "windows")]
        let output = Command::new("winget")
            .arg("install")
            .arg("git")
            .output();

        #[cfg(target_os = "macos")]
        let output = Command::new("brew")
            .arg("install")
            .arg("git")
            .output();

        match output {
            Err(e) => Err(Box::new(BGitError::new(
                "Failed to execute command",
                &e.to_string(),
                BGitErrorWorkflowType::Rules,
                NO_STEP,
                NO_EVENT,
                self.get_name(),
            ))),
            Ok(output_response) => {
                if output_response.status.success() {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
        }
    }
}
