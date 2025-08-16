use super::AtomicEvent;
use crate::auth::git_auth::setup_auth_callbacks;
use crate::bgit_error::BGitError;
use crate::config::global::BGitGlobalConfig;
use crate::rules::Rule;
use std::env;
use std::path::Path;

pub struct GitClone<'a> {
    pub pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    pub url: String,
    pub _global_config: &'a BGitGlobalConfig,
}

impl<'a> AtomicEvent<'a> for GitClone<'a> {
    fn new(_global_config: &'a BGitGlobalConfig) -> Self
    where
        Self: Sized,
    {
        GitClone {
            pre_check_rules: vec![],
            url: String::new(),
            _global_config,
        }
    }

    fn get_name(&self) -> &str {
        "git_clone"
    }

    fn get_action_description(&self) -> &str {
        "Clone a Git repository"
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        // Check if URL is set
        if self.url.is_empty() {
            return Err(self.to_bgit_error("Repository URL is not set"));
        }
        let url = &self.url;
        let repo_name = match url.split("/").last() {
            Some(repo_name) => repo_name.strip_suffix(".git").unwrap_or(repo_name),
            None => {
                return Err(self.to_bgit_error("Failed to get repository name from URL"));
            }
        };

        // Create fetch options with authentication
        let fetch_options = Self::create_fetch_options();

        // Clone repository with authentication options
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_options);

        builder.clone(&self.url, Path::new(repo_name)).map_err(|e| {
            self.to_bgit_error(&format!("Failed to clone repository: {e}. Please check your SSH keys or authentication setup."))
        })?;

        self.update_cwd_path()?;

        Ok(true)
    }
}

impl<'a> GitClone<'a> {
    pub fn set_url(&mut self, url: &str) -> &mut Self {
        self.url = url.to_owned();
        self
    }

    fn update_cwd_path(&self) -> Result<(), Box<BGitError>> {
        let repo_name = match self.url.split("/").last() {
            Some(repo_name) => repo_name.strip_suffix(".git").unwrap_or(repo_name),
            None => {
                return Err(self.to_bgit_error("Failed to get repository name from URL"));
            }
        };

        match env::set_current_dir(repo_name) {
            Ok(_) => Ok(()),
            Err(_) => Err(self.to_bgit_error("Failed to update current working directory path")),
        }
    }

    /// Create fetch options with authentication
    fn create_fetch_options() -> git2::FetchOptions<'static> {
        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(setup_auth_callbacks());
        fetch_options
    }
}
