use super::AtomicEvent;
use crate::{bgit_error::BGitError, config::global::BGitGlobalConfig, rules::Rule};
use git2::{Repository, RepositoryInitOptions};
use std::{env, path::Path};

pub struct GitInit<'a> {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    path: String, // Add path field
    _global_config: &'a BGitGlobalConfig,
}

impl<'a> GitInit<'a> {
    // Add a method to set custom path
    pub fn with_path(mut self, path: &str) -> Self {
        self.path = path.to_owned();
        self
    }

    fn update_cwd_path(&self) -> Result<(), Box<BGitError>> {
        match env::set_current_dir(&self.path) {
            Ok(_) => Ok(()),
            Err(_) => Err(self.to_bgit_error("Failed to update current working directory path")),
        }
    }
}

impl<'a> AtomicEvent<'a> for GitInit<'a> {
    fn new(_global_config: &'a BGitGlobalConfig) -> Self
    where
        Self: Sized,
    {
        GitInit {
            name: "git_init".to_owned(),
            pre_check_rules: vec![],
            path: ".".to_owned(), // Default to current directory
            _global_config,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_action_description(&self) -> &str {
        "Initialize git repository"
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        let mut opts = RepositoryInitOptions::new();
        opts.initial_head("main");

        Repository::init_opts(Path::new(&self.path), &opts).map_err(|e| {
            self.to_bgit_error(&format!(
                "Failed to initialize repository at '{}': {}",
                self.path, e
            ))
        })?;

        self.update_cwd_path()?;

        Ok(true)
    }
}
