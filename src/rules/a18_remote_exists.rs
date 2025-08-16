use crate::bgit_error::BGitError;
use crate::config::local::WorkflowRules;
use crate::rules::{Rule, RuleLevel, RuleOutput};
use dialoguer::Input;
use dialoguer::theme::ColorfulTheme;
use git2::Repository;
use std::process::Command;

pub(crate) struct RemoteExists {
    name: String,
    description: String,
    level: RuleLevel,
    required_remote: String,
}

impl Rule for RemoteExists {
    fn new(workflow_rule_config: Option<&WorkflowRules>) -> Self {
        let default_rule_level = RuleLevel::Error;
        let name = "RemoteExists";
        let rule_level = workflow_rule_config
            .and_then(|config| config.get_rule_level(name))
            .cloned()
            .unwrap_or(default_rule_level);

        Self {
            name: name.to_string(),
            description: "Check that required Git remote exists before remote operations"
                .to_string(),
            level: rule_level,
            required_remote: "origin".to_string(),
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
        self.check_remote(&self.required_remote)
    }

    fn try_fix(&self) -> Result<bool, Box<BGitError>> {
        println!("Required remote '{}' does not exist.", self.required_remote);

        println!(
            r#"Helpful tips:
  - If you don't have a remote repository yet, create one: https://github.com/new
  - Prefer SSH URLs (recommended) for better auth: git@github.com:<user>/<repo>.git
  - To copy the SSH URL: on GitHub, open your repository, click "Code" → "SSH" → copy the URL.

You can paste the SSH URL below (HTTPS also works, but SSH is preferred)."#
        );

        let repo_url: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Enter the repository URL for remote '{}'",
                self.required_remote
            ))
            .interact_text()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "RemoteExists",
                    &format!("Failed to get user input: {e}"),
                    crate::bgit_error::BGitErrorWorkflowType::Rules,
                    "try_fix",
                    "remote_check",
                    "RemoteExists",
                ))
            })?;

        if repo_url.trim().is_empty() {
            println!("No URL provided. Remote not added.");
            return Ok(false);
        }

        let repo = Repository::discover(".").map_err(|e| {
            Box::new(BGitError::new(
                "RemoteExists",
                &format!("Failed to discover repository: {e}"),
                crate::bgit_error::BGitErrorWorkflowType::Rules,
                "try_fix",
                "repository_discovery",
                "RemoteExists",
            ))
        })?;

        repo.remote(&self.required_remote, repo_url.trim())
            .map_err(|e| {
                Box::new(BGitError::new(
                    "RemoteExists",
                    &format!("Failed to add remote: {e}"),
                    crate::bgit_error::BGitErrorWorkflowType::Rules,
                    "try_fix",
                    "add_remote",
                    "RemoteExists",
                ))
            })?;

        println!("Successfully added remote '{}'", self.required_remote);
        Ok(true)
    }
}

impl RemoteExists {
    #[allow(dead_code)]
    pub fn new_for_remote(remote_name: &str, workflow_rule_config: Option<&WorkflowRules>) -> Self {
        let default_rule_level = RuleLevel::Error;
        let name = "RemoteExists";
        let rule_level = workflow_rule_config
            .and_then(|config| config.get_rule_level(name))
            .cloned()
            .unwrap_or(default_rule_level);

        Self {
            name: name.to_string(),
            description: format!(
                "Check that '{remote_name}' remote exists before remote operations"
            ),
            level: rule_level,
            required_remote: remote_name.to_string(),
        }
    }

    /// Check if a specific remote exists
    pub fn check_remote(&self, remote_name: &str) -> Result<RuleOutput, Box<BGitError>> {
        let output = Command::new("git").arg("remote").output();

        match output {
            Err(e) => Ok(RuleOutput::Exception(format!(
                "Failed to execute 'git remote' command: {e}"
            ))),
            Ok(output_response) => {
                if !output_response.status.success() {
                    return Ok(RuleOutput::Exception(
                        "Git command failed - ensure you're in a git repository".to_string(),
                    ));
                }

                let remotes_output = String::from_utf8_lossy(&output_response.stdout);
                let remotes: Vec<&str> = remotes_output
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .collect();

                if remotes.contains(&remote_name) {
                    Ok(RuleOutput::Success)
                } else {
                    let available_remotes = if remotes.is_empty() {
                        "No remotes configured".to_string()
                    } else {
                        format!("Available remotes: {}", remotes.join(", "))
                    };

                    Ok(RuleOutput::Exception(format!(
                        "Required remote '{remote_name}' does not exist. {available_remotes}. Hint: create a repo at https://github.com/new and add it as '{remote_name}' (prefer SSH). In GitHub, click 'Code' → 'SSH' and copy the URL, then run: git remote add {remote_name} <ssh_url>"
                    )))
                }
            }
        }
    }
}
