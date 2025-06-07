use crate::events::git_commit::GitCommit;
use crate::step::Task::ActionStepTask;
use crate::workflows::default::action::ta08_is_pulled_pushed::IsPushedPulled;
use crate::{
    bgit_error::BGitError,
    step::{ActionStep, Step},
};
use git2::{DiffOptions, Repository};
use google_generative_ai_rs::v1::{
    api::{Client, PostResult},
    gemini::{
        request::{Request, SystemInstructionContent, SystemInstructionPart},
        Content, Model, Part, Role,
    },
};
use log::debug;
use std::path::Path;

use crate::events::AtomicEvent;

pub(crate) struct AICommit {
    name: String,
    api_key: Option<String>,
}

impl ActionStep for AICommit {
    fn new() -> Self
    where
        Self: Sized,
    {
        AICommit {
            name: "ai_commit".to_owned(),
            api_key: None,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn execute(&self) -> Result<Step, Box<BGitError>> {
        // Get API key from environment or provided value
        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => std::env::var("GEMINI_API_KEY").map_err(|_| {
                Box::new(BGitError::new(
                    "BGitError",
                    "GEMINI_API_KEY environment variable not set and no API key provided",
                    crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                    crate::bgit_error::NO_EVENT,
                    &self.name,
                    crate::bgit_error::NO_RULE,
                ))
            })?,
        };

        // Get git diff
        let diff_content = self.get_git_diff()?;

        debug!("{diff_content}");

        if diff_content.trim().is_empty() {
            return Err(Box::new(BGitError::new(
                "BGitError",
                "No changes detected in the repository",
                crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                crate::bgit_error::NO_EVENT,
                &self.name,
                crate::bgit_error::NO_RULE,
            )));
        }

        // Generate commit message using AI
        let commit_message = self.generate_commit_message(&api_key, &diff_content)?;

        debug!("Generated commit message: {}", commit_message);

        // Execute GitCommit with the generated message
        let git_commit = GitCommit::new().with_commit_message(commit_message);
        git_commit.execute()?;

        // Return to ask commit step with generated message
        Ok(Step::Task(ActionStepTask(Box::new(IsPushedPulled::new()))))
    }
}

impl AICommit {
    /// Get git diff content as string (staged changes)
    fn get_git_diff(&self) -> Result<String, Box<BGitError>> {
        let repo = Repository::discover(Path::new(".")).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to open repository: {}", e),
                crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                crate::bgit_error::NO_EVENT,
                &self.name,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        let mut diff_opts = DiffOptions::new();
        diff_opts.include_untracked(false);

        // Get diff between HEAD and index (staged changes)
        let head_tree = repo
            .head()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get HEAD: {}", e),
                    crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                    crate::bgit_error::NO_EVENT,
                    &self.name,
                    crate::bgit_error::NO_RULE,
                ))
            })?
            .peel_to_tree()
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to peel HEAD to tree: {}", e),
                    crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                    crate::bgit_error::NO_EVENT,
                    &self.name,
                    crate::bgit_error::NO_RULE,
                ))
            })?;

        let index = repo.index().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get repository index: {}", e),
                crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                crate::bgit_error::NO_EVENT,
                &self.name,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        let diff = repo
            .diff_tree_to_index(Some(&head_tree), Some(&index), Some(&mut diff_opts))
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to create staged diff: {}", e),
                    crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                    crate::bgit_error::NO_EVENT,
                    &self.name,
                    crate::bgit_error::NO_RULE,
                ))
            })?;

        let mut diff_content = String::new();

        diff.print(git2::DiffFormat::Patch, |_delta, _hunk, line| {
            let line_str = std::str::from_utf8(line.content()).unwrap_or("");
            match line.origin() {
                '+' => diff_content.push_str(&format!("+{}", line_str)),
                '-' => diff_content.push_str(&format!("-{}", line_str)),
                ' ' => diff_content.push_str(&format!(" {}", line_str)),
                _ => diff_content.push_str(line_str),
            }
            true
        })
        .map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to process diff: {}", e),
                crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                crate::bgit_error::NO_EVENT,
                &self.name,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        Ok(diff_content)
    }

    /// Generate commit message using Google Gemini AI
    fn generate_commit_message(
        &self,
        api_key: &str,
        diff_content: &str,
    ) -> Result<String, Box<BGitError>> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to create async runtime: {}", e),
                crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                crate::bgit_error::NO_EVENT,
                &self.name,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        rt.block_on(async {
            self.generate_commit_message_async(api_key, diff_content)
                .await
        })
    }

    async fn generate_commit_message_async(
        &self,
        api_key: &str,
        diff_content: &str,
    ) -> Result<String, Box<BGitError>> {
        // Use the same approach as CodeSolutionGenerator - specify the model explicitly
        let client = Client::new_from_model(Model::Gemini2_0Flash, api_key.to_string());

        let system_prompt = "You are a git commit message generator. Generate concise, conventional commit messages based on git diffs. Follow conventional commit format (type: description). Keep the summary line under 50 characters. Focus on what changed and why.";

        let user_prompt = format!(
            "Generate a conventional commit message for the following git diff:\n\n{}",
            diff_content
        );

        // Create request similar to CodeSolutionGenerator
        let request = Request {
            contents: vec![Content {
                role: Role::User,
                parts: vec![Part {
                    text: Some(user_prompt),
                    inline_data: None,
                    file_data: None,
                    video_metadata: None,
                }],
            }],
            tools: vec![],
            safety_settings: vec![],
            generation_config: None,
            system_instruction: Some(SystemInstructionContent {
                parts: vec![SystemInstructionPart {
                    text: Some(system_prompt.to_string()),
                }],
            }),
        };

        // Use the same pattern as CodeSolutionGenerator for handling the response
        let result = client.post(30, &request).await.map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to generate commit message: {}", e.message),
                crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                crate::bgit_error::NO_EVENT,
                &self.name,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        // Handle the PostResult enum properly
        match result {
            PostResult::Rest(response) => {
                let commit_message = response
                    .candidates
                    .first()
                    .map(|candidate| candidate.content.clone())
                    .and_then(|content| content.parts.first().cloned())
                    .and_then(|part| part.text.clone())
                    .map(|text| text.trim().to_string())
                    .ok_or_else(|| {
                        Box::new(BGitError::new(
                            "BGitError",
                            "No generated text found in response",
                            crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                            crate::bgit_error::NO_EVENT,
                            &self.name,
                            crate::bgit_error::NO_RULE,
                        ))
                    })?;

                Ok(commit_message)
            }
            _ => Err(Box::new(BGitError::new(
                "BGitError",
                "Unexpected response type",
                crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                crate::bgit_error::NO_EVENT,
                &self.name,
                crate::bgit_error::NO_RULE,
            ))),
        }
    }
}
