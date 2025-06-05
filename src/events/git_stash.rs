use super::AtomicEvent;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    rules::Rule,
};
use git2::{Repository, StashApplyOptions};
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) enum StashOperation {
    Pop,
}

pub(crate) struct GitStash {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    operation: Option<StashOperation>,
    stash_index: Option<usize>,
}

impl GitStash {
    pub fn pop_stash(index: Option<usize>) -> Self {
        GitStash {
            name: "git_stash".to_owned(),
            pre_check_rules: vec![],
            operation: Some(StashOperation::Pop),
            stash_index: index,
        }
    }
}

impl AtomicEvent for GitStash {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitStash {
            name: "git_stash".to_owned(),
            pre_check_rules: vec![],
            operation: None,
            stash_index: None,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_action_description(&self) -> &str {
        match &self.operation {
            Some(StashOperation::Pop) => "Pop stash and apply changes",
            None => "No stash operation defined",
        }
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        let mut repo = Repository::discover(Path::new(".")).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to open repository: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        match &self.operation {
            Some(StashOperation::Pop) => self.pop_stash_impl(&mut repo),
            None => Err(Box::new(BGitError::new(
                "BGitError",
                "No stash operation defined",
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))),
        }
    }
}

impl GitStash {
    fn pop_stash_impl(&self, repo: &mut Repository) -> Result<bool, Box<BGitError>> {
        let index = self.stash_index.unwrap_or(0);

        // Check if stash exists
        self.check_stash_exists(repo, index)?;

        let mut apply_options = StashApplyOptions::default();

        repo.stash_pop(index, Some(&mut apply_options))
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to pop stash at index {}: {}", index, e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            })?;

        Ok(true)
    }

    fn check_stash_exists(
        &self,
        repo: &mut Repository,
        index: usize,
    ) -> Result<(), Box<BGitError>> {
        let mut stash_exists = false;
        let mut callback = |stash_index: usize, _message: &str, _oid: &git2::Oid| -> bool {
            if stash_index == index {
                stash_exists = true;
                false // Stop iteration
            } else {
                true // Continue
            }
        };

        repo.stash_foreach(&mut callback).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to check stash existence: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        if !stash_exists {
            return Err(Box::new(BGitError::new(
                "BGitError",
                &format!("Stash at index {} does not exist", index),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            )));
        }

        Ok(())
    }
}
