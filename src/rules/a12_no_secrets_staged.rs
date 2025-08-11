use crate::bgit_error::BGitError;
use crate::config::WorkflowRules;
use crate::rules::{Rule, RuleLevel, RuleOutput};
use regex::Regex;
use std::collections::HashSet;
use std::process::Command;

pub(crate) struct NoSecretsStaged {
    name: String,
    description: String,
    level: RuleLevel,
    secret_patterns: Vec<SecretPattern>,
}

#[derive(Clone)]
struct SecretPattern {
    regex: Regex,
    name: String,
    entropy_threshold: Option<f64>,
    min_length: usize,
    validate_fn: Option<fn(&str) -> bool>,
}

impl Rule for NoSecretsStaged {
    fn new(workflow_rule_config: Option<&WorkflowRules>) -> Self {
        let default_rule_level = RuleLevel::Error;
        let name = "NoSecretsStaged";
        let rule_level = workflow_rule_config
            .and_then(|config| config.get_rule_level(name))
            .cloned()
            .unwrap_or(default_rule_level);

        Self {
            name: name.to_string(),
            description: "Check that no secrets are staged for commit".to_string(),
            level: rule_level,
            secret_patterns: Self::initialize_patterns(),
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
                "Failed to execute git diff --staged: {e}"
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
        Ok(false)
    }
}

impl NoSecretsStaged {
    fn initialize_patterns() -> Vec<SecretPattern> {
        let mut patterns = Vec::new();

        // Helper function to safely create regex patterns
        let create_pattern = |pattern: &str,
                              name: &str,
                              entropy: Option<f64>,
                              min_len: usize,
                              validate: Option<fn(&str) -> bool>|
         -> Option<SecretPattern> {
            match Regex::new(pattern) {
                Ok(regex) => Some(SecretPattern {
                    regex,
                    name: name.to_string(),
                    entropy_threshold: entropy,
                    min_length: min_len,
                    validate_fn: validate,
                }),
                Err(_) => None,
            }
        };

        // High-confidence patterns with specific formats
        let pattern_definitions = vec![
            // GitHub tokens (very specific format)
            (
                "ghp_[a-zA-Z0-9]{36}",
                "GitHub Personal Access Token",
                None,
                40,
                None,
            ),
            ("gho_[a-zA-Z0-9]{36}", "GitHub OAuth Token", None, 40, None),
            ("ghs_[a-zA-Z0-9]{36}", "GitHub Server Token", None, 40, None),
            ("ghu_[a-zA-Z0-9]{36}", "GitHub User Token", None, 40, None),
            (
                "ghr_[a-zA-Z0-9]{36}",
                "GitHub Refresh Token",
                None,
                40,
                None,
            ),
            // AWS Keys (standard format)
            (
                "AKIA[0-9A-Z]{16}",
                "AWS Access Key ID (Standard Format)",
                None,
                20,
                None,
            ),
            // JWT tokens
            (
                r"eyJ[a-zA-Z0-9_-]*\.eyJ[a-zA-Z0-9_-]*\.[a-zA-Z0-9_-]*",
                "JWT Token",
                Some(4.0),
                50,
                None,
            ),
            // Private keys
            (
                r"-----BEGIN\s+(RSA\s+)?PRIVATE\s+KEY-----",
                "Private Key Block",
                None,
                20,
                None,
            ),
            // Slack tokens
            (
                "xox[baprs]-[0-9a-zA-Z]{10,48}",
                "Slack Token",
                None,
                15,
                None,
            ),
        ];

        // Add simple patterns
        for (pattern, name, entropy, min_len, validate) in pattern_definitions {
            if let Some(secret_pattern) = create_pattern(pattern, name, entropy, min_len, validate)
            {
                patterns.push(secret_pattern);
            }
        }

        // Complex patterns that need careful escaping
        let complex_patterns = vec![
            // AWS patterns with variable names
            Self::build_aws_access_key_pattern(),
            Self::build_aws_secret_key_pattern(),
            // Add a more general AWS pattern to catch variations
            Self::build_general_aws_pattern(),
            Self::build_api_key_pattern(),
            Self::build_secret_key_pattern(),
            Self::build_access_token_pattern(),
            Self::build_bearer_token_pattern(),
        ];

        for pattern in complex_patterns.into_iter().flatten() {
            patterns.push(pattern);
        }

        patterns
    }

    fn build_aws_access_key_pattern() -> Option<SecretPattern> {
        // Match both AWS_ACCESS_KEY_ID and AWS_ACCESS_KEY patterns
        let pattern = "(?i)aws[_-]?access[_-]?key(?:[_-]?id)?[\\s]*[:=][\\s]*([\"']?)([A-Za-z0-9@#$%^&*!+=/._-]{16,})\\1".to_string();

        Regex::new(&pattern).ok().map(|regex| SecretPattern {
            regex,
            name: "AWS Access Key".to_string(),
            entropy_threshold: Some(3.5),
            min_length: 16,
            validate_fn: Some(Self::validate_not_common_word),
        })
    }

    fn build_aws_secret_key_pattern() -> Option<SecretPattern> {
        let pattern =
            "(?i)aws[_-]?secret[_-]?access[_-]?key[\\s]*[:=][\\s]*([\"']?)([a-zA-Z0-9/+=]{40})\\1"
                .to_string();

        Regex::new(&pattern).ok().map(|regex| SecretPattern {
            regex,
            name: "AWS Secret Access Key".to_string(),
            entropy_threshold: Some(4.5),
            min_length: 40,
            validate_fn: Some(Self::validate_base64),
        })
    }

    fn build_api_key_pattern() -> Option<SecretPattern> {
        let pattern = "(?i)api[_-]?key[\\s]*[:=][\\s]*([\"']?)([a-zA-Z0-9]{20,})\\1".to_string();

        Regex::new(&pattern).ok().map(|regex| SecretPattern {
            regex,
            name: "API Key".to_string(),
            entropy_threshold: Some(4.2),
            min_length: 20,
            validate_fn: Some(Self::validate_not_common_word),
        })
    }

    fn build_secret_key_pattern() -> Option<SecretPattern> {
        let pattern = "(?i)secret[_-]?key[\\s]*[:=][\\s]*([\"']?)([a-zA-Z0-9]{20,})\\1".to_string();

        Regex::new(&pattern).ok().map(|regex| SecretPattern {
            regex,
            name: "Secret Key".to_string(),
            entropy_threshold: Some(4.2),
            min_length: 20,
            validate_fn: Some(Self::validate_not_common_word),
        })
    }

    fn build_access_token_pattern() -> Option<SecretPattern> {
        let pattern =
            "(?i)access[_-]?token[\\s]*[:=][\\s]*([\"']?)([a-zA-Z0-9]{20,})\\1".to_string();

        Regex::new(&pattern).ok().map(|regex| SecretPattern {
            regex,
            name: "Access Token".to_string(),
            entropy_threshold: Some(4.0),
            min_length: 20,
            validate_fn: Some(Self::validate_not_common_word),
        })
    }

    fn build_bearer_token_pattern() -> Option<SecretPattern> {
        let pattern = r"(?i)bearer\s+([a-zA-Z0-9\-_.]{20,})".to_string();

        Regex::new(&pattern).ok().map(|regex| SecretPattern {
            regex,
            name: "Bearer Token".to_string(),
            entropy_threshold: Some(4.0),
            min_length: 20,
            validate_fn: Some(Self::validate_not_common_word),
        })
    }

    fn build_general_aws_pattern() -> Option<SecretPattern> {
        // Catch AWS_ACCESS_KEY, AWS_SECRET_KEY, AWS_SESSION_TOKEN, etc.
        let pattern = "(?i)aws[_-]?(?:access[_-]?key|secret[_-]?(?:access[_-]?)?key|session[_-]?token)[\\s]*[:=][\\s]*([\"']?)([A-Za-z0-9+/=_-]{16,})\\1".to_string();

        Regex::new(&pattern).ok().map(|regex| SecretPattern {
            regex,
            name: "AWS Credential".to_string(),
            entropy_threshold: Some(3.0),
            min_length: 16,
            validate_fn: Some(Self::validate_not_common_word),
        })
    }

    fn detect_secrets(&self, content: &str) -> Option<Vec<String>> {
        let mut found_secrets = Vec::new();
        let mut detected_types = HashSet::new();

        // Only check added lines (lines starting with +)
        let added_lines: Vec<&str> = content
            .lines()
            .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
            .collect();

        let added_content = added_lines.join("\n");

        // Check each pattern
        for pattern in &self.secret_patterns {
            for capture in pattern.regex.captures_iter(&added_content) {
                let full_match = capture.get(0).unwrap().as_str();
                // Extract the actual secret value (usually in capture group 2 for quoted patterns)
                let secret_value = if capture.len() > 2 && capture.get(2).is_some() {
                    capture.get(2).unwrap().as_str()
                } else if capture.len() > 1 && capture.get(1).is_some() {
                    capture.get(1).unwrap().as_str()
                } else {
                    full_match
                };

                // Apply length check
                if secret_value.len() < pattern.min_length {
                    continue;
                }

                // Apply entropy check if specified
                if let Some(threshold) = pattern.entropy_threshold
                    && Self::calculate_entropy(secret_value) < threshold
                {
                    continue;
                }

                // Apply custom validation if specified
                if let Some(validate_fn) = pattern.validate_fn
                    && !validate_fn(secret_value)
                {
                    continue;
                }

                // Avoid duplicate detections of the same type
                if !detected_types.contains(&pattern.name) {
                    found_secrets.push(format!(
                        "{} (line context: {})",
                        pattern.name,
                        Self::get_line_context(full_match, &added_content)
                    ));
                    detected_types.insert(pattern.name.clone());
                }
            }
        }

        // Check for sensitive files
        self.check_sensitive_files(content, &mut found_secrets);

        // Check for high-entropy strings in variable assignments
        self.check_high_entropy_assignments(&added_content, &mut found_secrets, &detected_types);

        if found_secrets.is_empty() {
            None
        } else {
            Some(found_secrets)
        }
    }

    fn check_sensitive_files(&self, content: &str, found_secrets: &mut Vec<String>) {
        let sensitive_files = vec![
            ".env",
            "config.json",
            "secrets.",
            "credentials",
            ".pem",
            ".key",
            ".p12",
            ".pfx",
            "id_rsa",
            "id_dsa",
        ];

        for line in content.lines() {
            if line.starts_with("+++") || line.starts_with("---") {
                for sensitive_pattern in &sensitive_files {
                    if line.contains(sensitive_pattern) {
                        found_secrets.push(format!(
                            "Sensitive file: {}",
                            line.trim_start_matches("+++")
                                .trim_start_matches("---")
                                .trim()
                        ));
                        break;
                    }
                }
            }
        }
    }

    fn check_high_entropy_assignments(
        &self,
        content: &str,
        found_secrets: &mut Vec<String>,
        detected_types: &HashSet<String>,
    ) {
        // Look for variable assignments with high-entropy values
        let pattern =
            r#"(?m)^\+.*?([a-zA-Z_][a-zA-Z0-9_]*)\s*[:=]\s*["']([a-zA-Z0-9+/=_-]{16,})["']"#;

        if let Ok(assignment_regex) = Regex::new(pattern) {
            for capture in assignment_regex.captures_iter(content) {
                let var_name = capture.get(1).unwrap().as_str().to_lowercase();
                let value = capture.get(2).unwrap().as_str();

                // Skip if we already detected this type of secret
                if !detected_types.is_empty() {
                    continue;
                }

                // Check if variable name suggests it might be a secret
                let suspicious_names = ["key", "secret", "token", "password", "pwd", "auth", "api"];
                let is_suspicious_name =
                    suspicious_names.iter().any(|&name| var_name.contains(name));

                if is_suspicious_name
                    && value.len() >= 16
                    && Self::calculate_entropy(value) > 4.0
                    && Self::validate_not_common_word(value)
                {
                    found_secrets.push(format!(
                        "High-entropy value in variable '{}' (entropy: {:.2})",
                        var_name,
                        Self::calculate_entropy(value)
                    ));
                }
            }
        }
    }

    fn calculate_entropy(s: &str) -> f64 {
        if s.is_empty() {
            return 0.0;
        }

        let mut char_counts = std::collections::HashMap::new();
        for c in s.chars() {
            *char_counts.entry(c).or_insert(0) += 1;
        }

        let len = s.len() as f64;
        let mut entropy = 0.0;

        for &count in char_counts.values() {
            let probability = count as f64 / len;
            entropy -= probability * probability.log2();
        }

        entropy
    }

    fn validate_base64(s: &str) -> bool {
        // Check if string looks like base64
        s.chars()
            .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '=')
            && s.len() % 4 == 0
    }

    fn validate_not_common_word(s: &str) -> bool {
        // List of common false positives
        let common_words = vec![
            "example",
            "placeholder",
            "dummy",
            "test",
            "sample",
            "default",
            "your_key_here",
            "insert_key_here",
            "replace_with_key",
            "changeme",
            "12345678901234567890",
            "abcdefghijklmnopqrstuvwxyz",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            "1234567890abcdef",
            "null",
            "none",
            "todo",
            "fixme",
            "temp",
            "temporary",
        ];

        let lower_s = s.to_lowercase();

        // Check for common false positives
        if common_words.iter().any(|&word| lower_s.contains(word)) {
            return false;
        }

        // Check if all same character
        if s.len() > 3 && s.chars().all(|c| c == s.chars().next().unwrap()) {
            return false;
        }

        // Check for sequential patterns
        if Self::is_sequential(s) {
            return false;
        }

        // Check for obvious test patterns
        if s.len() >= 8
            && (s.starts_with("test")
                || s.starts_with("fake")
                || s.starts_with("mock")
                || s.ends_with("test")
                || s.ends_with("example"))
        {
            return false;
        }

        true
    }

    fn is_sequential(s: &str) -> bool {
        if s.len() < 6 {
            return false;
        }

        let chars: Vec<char> = s.chars().collect();
        let mut sequential_count = 1;

        for i in 1..chars.len() {
            if chars[i] as u8 == chars[i - 1] as u8 + 1 {
                sequential_count += 1;
                if sequential_count >= 6 {
                    return true;
                }
            } else {
                sequential_count = 1;
            }
        }

        false
    }

    fn get_line_context(secret: &str, content: &str) -> String {
        for line in content.lines() {
            if line.contains(secret) {
                // Return a truncated version of the line for context (without the actual secret)
                let context = if line.len() > 50 {
                    format!("{}...", &line[..47])
                } else {
                    line.to_string()
                };
                return context.replace(secret, "[REDACTED]");
            }
        }
        "unknown context".to_string()
    }
}
