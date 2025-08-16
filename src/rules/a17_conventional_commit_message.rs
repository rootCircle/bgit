use crate::bgit_error::BGitError;
use crate::config::local::WorkflowRules;
use crate::rules::{Rule, RuleLevel, RuleOutput};
use regex::Regex;

pub(crate) struct ConventionalCommitMessage {
    name: String,
    description: String,
    level: RuleLevel,
    message: Option<String>,
}

impl Rule for ConventionalCommitMessage {
    fn new(workflow_rule_config: Option<&WorkflowRules>) -> Self {
        let default_rule_level = RuleLevel::Warning;
        let name = "ConventionalCommitMessage";
        let rule_level = workflow_rule_config
            .and_then(|config| config.get_rule_level(name))
            .cloned()
            .unwrap_or(default_rule_level);

        Self {
            name: name.to_string(),
            description: "Ensure commit messages follow Conventional Commit specification"
                .to_string(),
            level: rule_level,
            message: None,
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
        let message = match &self.message {
            Some(msg) => msg,
            None => {
                return Ok(RuleOutput::Exception(
                    "No commit message provided for validation".to_string(),
                ));
            }
        };

        if self.is_conventional_commit(message) {
            Ok(RuleOutput::Success)
        } else {
            Ok(RuleOutput::Exception(format!(
                "Commit message does not follow Conventional Commit specification: '{}'",
                message.lines().next().unwrap_or(message)
            )))
        }
    }

    fn try_fix(&self) -> Result<bool, Box<BGitError>> {
        println!("Conventional Commit format violation detected.");
        println!("Please follow the Conventional Commit specification:");
        println!("  <type>[optional scope]: <description>");
        println!();
        println!("Examples:");
        println!("  feat: add user authentication");
        println!("  fix: resolve login issue");
        println!("  docs: update README");
        println!("  style: fix code formatting");
        println!("  refactor: simplify user service");
        println!("  test: add unit tests for auth");
        println!("  chore: update dependencies");
        println!();
        println!(
            "Valid types: feat, fix, docs, style, refactor, test, chore, build, ci, perf, revert"
        );

        Ok(false)
    }
}

impl ConventionalCommitMessage {
    pub fn with_message(mut self, message: String) -> Self {
        self.message = Some(message);
        self
    }

    fn is_conventional_commit(&self, message: &str) -> bool {
        let first_line = message.lines().next().unwrap_or("");

        // Conventional commit pattern: type(scope): description
        // type can be: feat, fix, docs, style, refactor, test, chore, build, ci, perf, revert
        // scope is optional
        let pattern =
            r"^(feat|fix|docs|style|refactor|test|chore|build|ci|perf|revert)(\(.+\))?: .+";

        match Regex::new(pattern) {
            Ok(regex) => regex.is_match(first_line),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_conventional_commits() {
        let rule = ConventionalCommitMessage::new(None);

        // Valid conventional commits
        assert!(rule.is_conventional_commit("feat: add user authentication"));
        assert!(rule.is_conventional_commit("fix: resolve login issue"));
        assert!(rule.is_conventional_commit("docs: update README"));
        assert!(rule.is_conventional_commit("style: fix code formatting"));
        assert!(rule.is_conventional_commit("refactor: simplify user service"));
        assert!(rule.is_conventional_commit("test: add unit tests for auth"));
        assert!(rule.is_conventional_commit("chore: update dependencies"));
        assert!(rule.is_conventional_commit("build: update webpack config"));
        assert!(rule.is_conventional_commit("ci: add GitHub Actions"));
        assert!(rule.is_conventional_commit("perf: optimize database queries"));
        assert!(rule.is_conventional_commit("revert: undo last commit"));

        // With scopes
        assert!(rule.is_conventional_commit("feat(auth): add user authentication"));
        assert!(rule.is_conventional_commit("fix(login): resolve login issue"));
        assert!(rule.is_conventional_commit("docs(readme): update installation guide"));

        // Multi-line commits (should check only first line)
        assert!(
            rule.is_conventional_commit("feat: add new feature\n\nThis is a detailed description")
        );
    }

    #[test]
    fn test_invalid_conventional_commits() {
        let rule = ConventionalCommitMessage::new(None);

        // Invalid conventional commits
        assert!(!rule.is_conventional_commit("Add user authentication"));
        assert!(!rule.is_conventional_commit("fix login issue"));
        assert!(!rule.is_conventional_commit("updated README"));
        assert!(!rule.is_conventional_commit("WIP: work in progress"));
        assert!(!rule.is_conventional_commit("hotfix: emergency fix"));
        assert!(!rule.is_conventional_commit("feature: new feature"));
        assert!(!rule.is_conventional_commit("bug: fix bug"));

        // Missing description
        assert!(!rule.is_conventional_commit("feat:"));
        assert!(!rule.is_conventional_commit("fix: "));

        // Wrong format
        assert!(!rule.is_conventional_commit("feat add authentication"));
        assert!(!rule.is_conventional_commit("feat(scope) add authentication"));
    }

    #[test]
    fn test_with_message_method() {
        let rule =
            ConventionalCommitMessage::new(None).with_message("feat: add new feature".to_string());

        let result = rule.check().unwrap();
        match result {
            RuleOutput::Success => (),
            _ => panic!("Expected success for valid conventional commit"),
        }
    }

    #[test]
    fn test_with_invalid_message() {
        let rule = ConventionalCommitMessage::new(None).with_message("Add new feature".to_string());

        let result = rule.check().unwrap();
        match result {
            RuleOutput::Exception(msg) => {
                assert!(msg.contains("does not follow Conventional Commit specification"));
            }
            _ => panic!("Expected exception for invalid conventional commit"),
        }
    }

    #[test]
    fn test_no_message_provided() {
        let rule = ConventionalCommitMessage::new(None);

        let result = rule.check().unwrap();
        match result {
            RuleOutput::Exception(msg) => {
                assert_eq!(msg, "No commit message provided for validation");
            }
            _ => panic!("Expected exception when no message is provided"),
        }
    }

    #[test]
    fn test_rule_properties() {
        let rule = ConventionalCommitMessage::new(None);

        assert_eq!(rule.get_name(), "ConventionalCommitMessage");
        assert_eq!(
            rule.get_description(),
            "Ensure commit messages follow Conventional Commit specification"
        );
        assert_eq!(rule.get_level(), RuleLevel::Warning);
    }

    #[test]
    fn test_try_fix_returns_false() {
        let rule = ConventionalCommitMessage::new(None);
        let result = rule.try_fix().unwrap();
        assert!(!result);
    }

    #[test]
    fn test_chaining_with_message() {
        let rule = ConventionalCommitMessage::new(None)
            .with_message("fix(auth): resolve token validation".to_string());

        let result = rule.check().unwrap();
        match result {
            RuleOutput::Success => (),
            _ => panic!("Expected success for valid scoped conventional commit"),
        }
    }
}
