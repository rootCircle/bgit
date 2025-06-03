use super::AtomicEvent;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    rules::Rule,
};
use git2::Repository;
use std::collections::HashSet;
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) enum LogOperation {
    CheckSoleContributor,
}

pub(crate) struct GitLog {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    operation: LogOperation,
}

impl GitLog {
    pub fn check_sole_contributor() -> Self {
        GitLog {
            name: "git_log".to_owned(),
            pre_check_rules: vec![],
            operation: LogOperation::CheckSoleContributor,
        }
    }

    // pub fn set_operation(&mut self, operation: LogOperation) {
    //     self.operation = operation;
    // }
}

impl AtomicEvent for GitLog {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitLog {
            name: "git_log".to_owned(),
            pre_check_rules: vec![],
            operation: LogOperation::CheckSoleContributor,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_action_description(&self) -> &str {
        match self.operation {
            LogOperation::CheckSoleContributor => "Check if current author is the sole contributor",
        }
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        // Open the repository at the current directory
        let repo = Repository::discover(Path::new(".")).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to open repository: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        match self.operation {
            LogOperation::CheckSoleContributor => self.check_sole_contributor_impl(&repo),
        }
    }
}

impl GitLog {
    fn check_sole_contributor_impl(&self, repo: &Repository) -> Result<bool, Box<BGitError>> {
        // Get current user's configuration
        let config = repo.config().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get repository config: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        let current_user_name = config.get_string("user.name").map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get current user name: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        let current_user_email = config.get_string("user.email").map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get current user email: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        // Collect all unique authors and committers
        let mut authors = HashSet::new();
        let mut committers = HashSet::new();

        let mut revwalk = repo.revwalk().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to create revwalk: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        revwalk.push_head().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to push HEAD to revwalk: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        for oid_result in revwalk {
            let oid = oid_result.map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get commit OID: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

            let commit = repo.find_commit(oid).map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to find commit: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

            // Get author information
            let author = commit.author();
            if let (Some(author_name), Some(author_email)) = (author.name(), author.email()) {
                authors.insert((author_name.to_string(), author_email.to_string()));
            }

            // Get committer information
            let committer = commit.committer();
            if let (Some(committer_name), Some(committer_email)) =
                (committer.name(), committer.email())
            {
                committers.insert((committer_name.to_string(), committer_email.to_string()));
            }
        }

        // Check if current user is the sole contributor
        let is_sole_author = authors.len() == 1
            && authors.contains(&(current_user_name.clone(), current_user_email.clone()));

        Ok(is_sole_author)
    }
}
