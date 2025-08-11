use std::convert::Infallible;

use crate::rules::{Rule, RuleOutput, a17_conventional_commit_message::ConventionalCommitMessage};
use rig::{completion::ToolDefinition, tool::Tool};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Tool arguments for validating a Conventional Commit message.
#[derive(Debug, Deserialize)]
pub struct ValidateConventionalCommitArgs {
    /// The commit message to validate.
    pub message: String,
}

/// Tool output for Conventional Commit validation.
#[derive(Debug, Serialize)]
pub struct ValidateConventionalCommitResult {
    /// Whether the message is valid.
    pub valid: bool,
    /// If invalid, a human-readable error message.
    pub error: Option<String>,
}

/// A rig tool that validates Conventional Commit messages using the project's rule logic.
#[derive(Default)]
pub struct ValidateConventionalCommit;

impl Tool for ValidateConventionalCommit {
    const NAME: &'static str = "validate_conventional_commit";

    type Error = Infallible;
    type Args = ValidateConventionalCommitArgs;
    type Output = ValidateConventionalCommitResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        serde_json::from_value(json!({
            "name": Self::NAME,
            "description": "Validate a Conventional Commit message header and basic style.",
            "parameters": {
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "The commit message to validate (header and optional body)."
                    }
                },
                "required": ["message"]
            }
        }))
        .expect("valid tool definition")
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let rule = ConventionalCommitMessage::new(None).with_message(args.message);
        let result = match rule.check() {
            Ok(RuleOutput::Success) => ValidateConventionalCommitResult {
                valid: true,
                error: None,
            },
            Ok(RuleOutput::Exception(msg)) => ValidateConventionalCommitResult {
                valid: false,
                error: Some(msg),
            },
            Err(err) => ValidateConventionalCommitResult {
                valid: false,
                error: Some(format!("Internal error: {err:?}")),
            },
        };

        Ok(result)
    }
}
