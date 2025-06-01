use super::AtomicEvent;
use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_RULE, NO_STEP};
use crate::rules::Rule;
use git2::Repository;

pub struct GitPush {
    pub pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    pub force: bool,
    pub set_upstream: bool,
}

impl AtomicEvent for GitPush {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitPush {
            pre_check_rules: vec![],
            force: false,
            set_upstream: false,
        }
    }

    fn get_name(&self) -> &str {
        "git_push"
    }

    fn get_action_description(&self) -> &str {
        "Push changes to remote repository"
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        // Open the repository in the current directory
        let repo = Repository::open(".").map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to open repository: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        // Get the current branch
        let head = repo.head().map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to get HEAD reference: {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        let branch_name = head.shorthand().ok_or_else(|| {
            Box::new(BGitError::new(
                "BGitError",
                "Failed to get branch name",
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        // Get remote
        let mut remote = repo.find_remote("origin").map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to find remote 'origin': {}", e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        // Prepare push options
        let mut push_options = git2::PushOptions::new();

        // Set up callbacks for authentication if needed
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.push_update_reference(|refname, status| match status {
            Some(msg) => {
                println!("Push failed for {}: {}", refname, msg);
                Err(git2::Error::from_str(msg))
            }
            None => {
                println!("Push successful for {}", refname);
                Ok(())
            }
        });

        push_options.remote_callbacks(callbacks);

        // Determine refspec
        let refspec = if self.set_upstream {
            format!("refs/heads/{}:refs/heads/{}", branch_name, branch_name)
        } else {
            format!("refs/heads/{}", branch_name)
        };

        // Check if we need to force push
        if !self.force {
            // Check if we're up to date with remote
            if let Ok(remote_ref) =
                repo.find_reference(&format!("refs/remotes/origin/{}", branch_name))
            {
                let local_commit = head.peel_to_commit().map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!("Failed to get local commit: {}", e),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_STEP,
                        self.get_name(),
                        NO_RULE,
                    ))
                })?;

                let remote_commit = remote_ref.peel_to_commit().map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!("Failed to get remote commit: {}", e),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_STEP,
                        self.get_name(),
                        NO_RULE,
                    ))
                })?;

                // Check if local is behind remote
                let merge_base = repo
                    .merge_base(local_commit.id(), remote_commit.id())
                    .map_err(|e| {
                        Box::new(BGitError::new(
                            "BGitError",
                            &format!("Failed to find merge base: {}", e),
                            BGitErrorWorkflowType::AtomicEvent,
                            NO_STEP,
                            self.get_name(),
                            NO_RULE,
                        ))
                    })?;

                if merge_base == local_commit.id() && local_commit.id() != remote_commit.id() {
                    return Err(Box::new(BGitError::new(
                        "BGitError",
                        "Local branch is behind remote. Pull changes first or use --force",
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_STEP,
                        self.get_name(),
                        NO_RULE,
                    )));
                }
            }
        }

        // Perform the push
        let refspecs = if self.force {
            vec![format!("+{}", refspec)]
        } else {
            vec![refspec]
        };

        remote
            .push(&refspecs, Some(&mut push_options))
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to push to remote: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

        // Set upstream if requested and push was successful
        if self.set_upstream {
            self.set_upstream_branch(&repo, branch_name)?;
        }

        Ok(true)
    }
}

impl GitPush {
    pub fn set_force(&mut self, force: bool) -> &mut Self {
        self.force = force;
        self
    }

    pub fn set_upstream_flag(&mut self, set_upstream: bool) -> &mut Self {
        self.set_upstream = set_upstream;
        self
    }

    fn set_upstream_branch(
        &self,
        repo: &Repository,
        branch_name: &str,
    ) -> Result<(), Box<BGitError>> {
        let mut branch = repo
            .find_branch(branch_name, git2::BranchType::Local)
            .map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to find local branch {}: {}", branch_name, e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_STEP,
                    self.get_name(),
                    NO_RULE,
                ))
            })?;

        let upstream_name = format!("origin/{}", branch_name);
        branch.set_upstream(Some(&upstream_name)).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Failed to set upstream to {}: {}", upstream_name, e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_STEP,
                self.get_name(),
                NO_RULE,
            ))
        })?;

        Ok(())
    }
}
