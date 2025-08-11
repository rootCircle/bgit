use crate::config::{StepFlags, WorkflowRules};
use crate::events::git_commit::GitCommit;
use crate::llm_tools::conventional_commit_tool::ValidateConventionalCommit;
use crate::rules::Rule;
use crate::rules::a02_git_name_email_setup::GitNameEmailSetup;
use crate::rules::a12_no_secrets_staged::NoSecretsStaged;
use crate::rules::a12b_no_secret_files_staged::NoSecretFilesStaged;
use crate::rules::a16_no_large_file::NoLargeFile;
use crate::rules::a17_conventional_commit_message::ConventionalCommitMessage;
use crate::step::Task::ActionStepTask;
use crate::workflows::default::action::ta08_is_pulled_pushed::IsPushedPulled;
use crate::{
    bgit_error::BGitError,
    step::{ActionStep, Step},
};
use git2::{DiffOptions, Repository};
use log::debug;
use rig::{completion::Prompt, providers::gemini};
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

    fn execute(
        &self,
        _step_config_flags: Option<&StepFlags>,
        workflow_rules_config: Option<&WorkflowRules>,
    ) -> Result<Step, Box<BGitError>> {
        // Get API key from environment or provided value
        let api_key = match &self.api_key {
            Some(key) => key.clone(),
            None => std::env::var("GOOGLE_API_KEY").map_err(|_| {
                Box::new(BGitError::new(
                    "BGitError",
                    "GOOGLE_API_KEY environment variable not set and no API key provided",
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

        debug!("Generated commit message: {commit_message}");

        // Execute GitCommit with the generated message
        let mut git_commit = GitCommit::new().with_commit_message(commit_message.clone());
        git_commit.add_pre_check_rule(Box::new(
            ConventionalCommitMessage::new(workflow_rules_config)
                .with_message(commit_message.clone()),
        ));

        git_commit.add_pre_check_rule(Box::new(NoSecretsStaged::new(workflow_rules_config)));
        git_commit.add_pre_check_rule(Box::new(NoSecretFilesStaged::new(workflow_rules_config)));
        git_commit.add_pre_check_rule(Box::new(NoLargeFile::new(workflow_rules_config)));
        git_commit.add_pre_check_rule(Box::new(GitNameEmailSetup::new(workflow_rules_config)));

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
                &format!("Failed to open repository: {e}"),
                crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                crate::bgit_error::NO_EVENT,
                &self.name,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        let mut diff_opts = DiffOptions::new();
        diff_opts.include_untracked(false);

        // Get diff between HEAD and index (staged changes) - handle unborn branch case
        let head_tree = match repo.head() {
            Ok(head) => Some(head.peel_to_tree().map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to peel HEAD to tree: {e}"),
                    crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                    crate::bgit_error::NO_EVENT,
                    &self.name,
                    crate::bgit_error::NO_RULE,
                ))
            })?),
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => {
                // No HEAD tree in unborn branch - use None to compare against empty tree
                None
            }
            Err(e) => {
                return Err(Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get HEAD: {e}"),
                    crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                    crate::bgit_error::NO_EVENT,
                    &self.name,
                    crate::bgit_error::NO_RULE,
                )));
            }
        };

        let index = repo.index().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get repository index: {e}"),
                crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                crate::bgit_error::NO_EVENT,
                &self.name,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        let diff = repo
            .diff_tree_to_index(head_tree.as_ref(), Some(&index), Some(&mut diff_opts))
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to create staged diff: {e}"),
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
                '+' => diff_content.push_str(&format!("+{line_str}")),
                '-' => diff_content.push_str(&format!("-{line_str}")),
                ' ' => diff_content.push_str(&format!(" {line_str}")),
                _ => diff_content.push_str(line_str),
            }
            true
        })
        .map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to process diff: {e}"),
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
                &format!("Failed to create async runtime: {e}"),
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
        let client = gemini::Client::new(api_key);

        let system_prompt = r#"You are an expert Git commit assistant.
Generate Conventional Commit messages strictly following these rules:

Constraints:
1) First line MUST be a Conventional Commit header:
    <type>[optional scope]: <short imperative summary>
    - Allowed types: feat, fix, docs, style, refactor, test, chore, build, ci, perf, revert
    - Summary ≤ 50 characters, no trailing period
2) If needed, include a body after a blank line:
    - Wrap lines at ~72 characters
    - Bullet key changes with concise points
    - Optionally add: BREAKING CHANGE: <details>

Type selection guidance:
- feat: new capability visible to users or API
- fix: bug fix or correct behavior
- docs: documentation-only changes
- style: formatting, linting, no logic change
- refactor: code restructure without behavior change
- test: add/modify tests only
- chore: maintenance tasks (deps, config, housekeeping)
- build: build system, dependencies, packaging
- ci: continuous integration/configuration
- perf: performance improvements
- revert: reverts a previous commit

Style:
- Use present tense, active voice, and concise language
- Avoid file paths unless essential to clarity
- No code blocks, quotes, backticks, or markdown decorations
- Output ONLY the commit message content (header and optional body)"#;

        let agent = client
            .agent("gemini-2.5-flash-lite")
            .preamble(system_prompt)
            .temperature(0.2)
            .tool(ValidateConventionalCommit)
            .build();

        let user_prompt = format!(
            r#"Generate a Conventional Commit message that meets the constraints above for the following staged git diff.

Diff:
```diff
{diff_content}
```

Remember:
- The first line must be the Conventional Commit header ONLY.
- If you include a body, put a blank line before it and wrap lines to ~72 chars.
- Do not include any extra commentary, explanations, or markdown—only the commit message."#
        );

        let response = agent.prompt(user_prompt).multi_turn(3).await.map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to generate commit message: {e}"),
                crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                crate::bgit_error::NO_EVENT,
                &self.name,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        let commit_message = response.trim().to_string();
        if commit_message.is_empty() {
            return Err(Box::new(BGitError::new(
                "BGitError",
                "No generated text found in response",
                crate::bgit_error::BGitErrorWorkflowType::ActionStep,
                crate::bgit_error::NO_EVENT,
                &self.name,
                crate::bgit_error::NO_RULE,
            )));
        }

        Ok(commit_message)
    }
}
