use super::AtomicEvent;
use crate::{
    bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_RULE},
    rules::Rule,
};
use git2::{Config, Repository};
use std::path::Path;

#[derive(Debug, Clone)]
pub(crate) enum ConfigOperation {
    Get,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) enum ConfigScope {
    Local,
    Global,
    System,
}

pub(crate) struct GitConfig {
    name: String,
    pre_check_rules: Vec<Box<dyn Rule + Send + Sync>>,
    operation: Option<ConfigOperation>,
    scope: ConfigScope,
    key: Option<String>,
}

impl GitConfig {
    pub fn with_key(mut self, key: String) -> Self {
        self.key = Some(key);
        self
    }

    pub fn with_operation(mut self, operation: ConfigOperation) -> Self {
        self.operation = Some(operation);
        self
    }

    // Use this method to get the scope of the configuration
    pub fn get_value(&self) -> Result<String, Box<BGitError>> {
        let config = self.get_config_object()?;

        let key = self.key.as_ref().ok_or_else(|| {
            Box::new(BGitError::new(
                "BGitError",
                "Config key not provided for get operation",
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })?;

        config.get_string(key).map_err(|e| {
            Box::new(BGitError::new(
                "BGitError",
                &format!("Configuration key '{}' not found: {}", key, e),
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))
        })
    }
}

impl AtomicEvent for GitConfig {
    fn new() -> Self
    where
        Self: Sized,
    {
        GitConfig {
            name: "git_config".to_owned(),
            pre_check_rules: vec![],
            operation: None,
            scope: ConfigScope::Local,
            key: None,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_action_description(&self) -> &str {
        match &self.operation {
            Some(ConfigOperation::Get) => "Get git configuration value",
            None => "Git configuration operation (no operation specified)",
        }
    }

    fn add_pre_check_rule(&mut self, rule: Box<dyn Rule + Send + Sync>) {
        self.pre_check_rules.push(rule);
    }

    fn get_pre_check_rule(&self) -> &Vec<Box<dyn Rule + Send + Sync>> {
        &self.pre_check_rules
    }

    fn raw_execute(&self) -> Result<bool, Box<BGitError>> {
        match &self.operation {
            Some(ConfigOperation::Get) => Ok(self.get_value().is_ok()),
            None => Err(Box::new(BGitError::new(
                "BGitError",
                "No config operation specified",
                BGitErrorWorkflowType::AtomicEvent,
                NO_EVENT,
                &self.name,
                NO_RULE,
            ))),
        }
    }
}

impl GitConfig {
    fn get_config_object(&self) -> Result<Config, Box<BGitError>> {
        match self.scope {
            ConfigScope::Local => {
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

                repo.config().map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!("Failed to get local config: {}", e),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_EVENT,
                        &self.name,
                        NO_RULE,
                    ))
                })
            }
            ConfigScope::Global => Config::open_default().map_err(|e| {
                Box::new(BGitError::new(
                    "BGitError",
                    &format!("Failed to get global config: {}", e),
                    BGitErrorWorkflowType::AtomicEvent,
                    NO_EVENT,
                    &self.name,
                    NO_RULE,
                ))
            }),
            ConfigScope::System => {
                let mut config = Config::new().map_err(|e| {
                    Box::new(BGitError::new(
                        "BGitError",
                        &format!("Failed to create config object: {}", e),
                        BGitErrorWorkflowType::AtomicEvent,
                        NO_EVENT,
                        &self.name,
                        NO_RULE,
                    ))
                })?;

                if let Ok(system_path) = Config::find_system() {
                    config
                        .add_file(&system_path, git2::ConfigLevel::System, false)
                        .map_err(|e| {
                            Box::new(BGitError::new(
                                "BGitError",
                                &format!("Failed to add system config: {}", e),
                                BGitErrorWorkflowType::AtomicEvent,
                                NO_EVENT,
                                &self.name,
                                NO_RULE,
                            ))
                        })?;
                }

                Ok(config)
            }
        }
    }
}
