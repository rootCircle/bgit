use super::AtomicEvent;
use crate::{bgit_error::BGitError, config::global::BGitGlobalConfig, rules::Rule};
use git2::{Commit, Repository};
use std::path::Path;

pub(crate) struct GitCommit<'a> {
    name: String,
    commit_message: Option<String>,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    _global_config: &'a BGitGlobalConfig,
}

impl<'a> AtomicEvent<'a> for GitCommit<'a> {
    fn new(_global_config: &'a BGitGlobalConfig) -> Self
    where
        Self: Sized,
    {
        GitCommit {
            name: "git_commit".to_owned(),
            commit_message: None,
            pre_check_rules: vec![],
            _global_config,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_action_description(&self) -> &str {
        "Commit staged files with auto-generated message"
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        let message = match &self.commit_message {
            Some(msg) => {
                if msg.trim().is_empty() {
                    return Err(self.to_bgit_error("Commit message cannot be empty."));
                }
                msg.clone()
            }
            None => {
                return Err(self.to_bgit_error(
                    "No commit message provided. Use with_message() to set a commit message.",
                ));
            }
        };

        self.commit_changes(&message)
    }
}

impl<'a> GitCommit<'a> {
    pub fn with_commit_message(mut self, commit_message: String) -> Self {
        self.commit_message = Some(commit_message);
        self
    }

    fn commit_changes(&self, message: &str) -> Result<bool, Box<BGitError>> {
        let repo = Repository::discover(Path::new("."))
            .map_err(|e| self.to_bgit_error(&format!("Failed to open repository: {e}")))?;

        let signature = repo
            .signature()
            .map_err(|e| self.to_bgit_error(&format!("Failed to get signature: {e}")))?;

        let mut index = repo
            .index()
            .map_err(|e| self.to_bgit_error(&format!("Failed to get repository index: {e}")))?;

        if index.has_conflicts() {
            return Err(self.to_bgit_error(
                "Merge conflicts found in index. Please resolve them before committing.",
            ));
        }

        let tree_id = index
            .write_tree()
            .map_err(|e| self.to_bgit_error(&format!("Failed to write tree: {e}")))?;

        let tree = repo
            .find_tree(tree_id)
            .map_err(|e| self.to_bgit_error(&format!("Failed to find tree: {e}")))?;

        let parent_commit: Option<Commit> = match repo.head() {
            Ok(head) => Some(
                head.peel_to_commit()
                    .map_err(|e| self.to_bgit_error(&format!("Failed to get HEAD commit: {e}")))?,
            ),
            Err(e) if e.code() == git2::ErrorCode::UnbornBranch => None,
            Err(e) => {
                return Err(self.to_bgit_error(&format!("Failed to get HEAD reference: {e}")));
            }
        };

        if let Some(parent) = &parent_commit
            && parent.tree_id() == tree.id()
        {
            return Ok(false);
        }

        let parents: Vec<&Commit> = parent_commit.iter().collect();

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &parents,
        )
        .map_err(|e| self.to_bgit_error(&format!("Failed to create commit: {e}")))?;

        Ok(true)
    }
}
