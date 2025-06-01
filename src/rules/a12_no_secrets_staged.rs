use crate::bgit_error::BGitError;
use crate::rules::{Rule, RuleLevel, RuleOutput};
use regex::Regex;
use std::process::Command;

pub(crate) struct NoSecretsStaged {
    name: String,
    description: String,
    level: RuleLevel,
}

impl Rule for NoSecretsStaged {
    fn new() -> Self {
        NoSecretsStaged {
            name: "NoSecretsStaged".to_string(),
            description: "Check that no secrets are staged for commit".to_string(),
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
        // Get staged files content
        let output = Command::new("git").arg("diff").arg("--staged").output();

        match output {
            Err(e) => Ok(RuleOutput::Exception(format!(
                "Failed to execute git diff --staged: {}",
                e
            ))),
            Ok(output_response) => {
                if !output_response.status.success() {
                    return Ok(RuleOutput::Exception(
                        "Git command failed - ensure you're in a git repository".to_string(),
                    ));
                }

                let diff_content = String::from_utf8_lossy(&output_response.stdout);

                if let Some(secrets) = self.detect_secrets(&diff_content) {
                    Ok(RuleOutput::Exception(format!(
                        "Potential secrets detected in staged files: {}",
                        secrets.join(", ")
                    )))
                } else {
                    Ok(RuleOutput::Success)
                }
            }
        }
    }

    fn try_fix(&self) -> Result<bool, Box<BGitError>> {
        println!("Cannot automatically fix secrets in staged files.");
        println!("Please review and remove any sensitive information manually.");
        println!("You can unstage files using: git reset HEAD <file>");

        // Cannot automatically fix secrets - requires manual intervention
        Ok(false)
    }
}

impl NoSecretsStaged {
    fn detect_secrets(&self, content: &str) -> Option<Vec<String>> {
        let mut found_secrets = Vec::new();

        // Common secret patterns
        let patterns = vec![
            (
                r"(?i)api[_-]?key['\s]*[:=]['\s]*[a-zA-Z0-9]{20,}",
                "API Key",
            ),
            (
                r"(?i)secret[_-]?key['\s]*[:=]['\s]*[a-zA-Z0-9]{20,}",
                "Secret Key",
            ),
            (
                r"(?i)access[_-]?token['\s]*[:=]['\s]*[a-zA-Z0-9]{20,}",
                "Access Token",
            ),
            (r"(?i)password['\s]*[:=]['\s]*[^\s]{8,}", "Password"),
            (r"(?i)pwd['\s]*[:=]['\s]*[^\s]{8,}", "Password"),
            (r"(?i)private[_-]?key['\s]*[:=]", "Private Key"),
            (
                r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----",
                "Private Key Block",
            ),
            (r"(?i)bearer\s+[a-zA-Z0-9\-_]{20,}", "Bearer Token"),
            (
                r"(?i)aws[_-]?access[_-]?key[_-]?id['\s]*[:=]['\s]*[A-Z0-9]{20}",
                "AWS Access Key",
            ),
            (
                r"(?i)aws[_-]?secret[_-]?access[_-]?key['\s]*[:=]['\s]*[a-zA-Z0-9/+=]{40}",
                "AWS Secret Key",
            ),
            (r"ghp_[a-zA-Z0-9]{36}", "GitHub Personal Access Token"),
            (r"gho_[a-zA-Z0-9]{36}", "GitHub OAuth Token"),
            (r"ghu_[a-zA-Z0-9]{36}", "GitHub User Token"),
            (r"ghs_[a-zA-Z0-9]{36}", "GitHub Server Token"),
            (r"ghr_[a-zA-Z0-9]{36}", "GitHub Refresh Token"),
        ];

        for (pattern, secret_type) in patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if regex.is_match(content) {
                    found_secrets.push(secret_type.to_string());
                }
            }
        }

        // Check for common configuration files that might contain secrets
        let lines: Vec<&str> = content.lines().collect();
        for line in &lines {
            if (line.starts_with("+++") || line.starts_with("---")) && (line.contains(".env")
                    || line.contains("config.json")
                    || line.contains("secrets.") || line.contains("credentials")) {
                found_secrets.push("Sensitive configuration file".to_string());
                break;
            }
        }

        if found_secrets.is_empty() {
            None
        } else {
            found_secrets.dedup();
            Some(found_secrets)
        }
    }
}
