use std::{
    collections::HashMap,
    path::Path,
    process::{Command, Stdio},
};

use dialoguer::{Confirm, theme::ColorfulTheme};
use git2::{Error, ErrorClass, ErrorCode};
use log::debug;

pub fn parse_ssh_agent_output(output: &str) -> HashMap<String, String> {
    let mut env_vars = HashMap::new();

    for line in output.lines() {
        if line.contains('=') && (line.contains("SSH_AUTH_SOCK") || line.contains("SSH_AGENT_PID"))
        {
            if let Some(var_part) = line.split(';').next() {
                if let Some((key, value)) = var_part.split_once('=') {
                    env_vars.insert(key.to_string(), value.to_string());
                }
            }
        }
    }

    env_vars
}

pub fn add_key_interactive(key_path: &Path, key_name: &str) -> Result<bool, Error> {
    debug!("Trying interactive ssh-add for key: {}", key_name);

    // Ask user if they want to add this key interactively
    let should_add = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(format!(
            "Add SSH key '{}' to ssh-agent? (you may be prompted for passphrase)",
            key_name
        ))
        .default(true)
        .interact()
        .map_err(|e| {
            Error::new(
                ErrorCode::Auth,
                ErrorClass::Net,
                format!("Failed to get user confirmation: {}", e),
            )
        })?;

    if !should_add {
        debug!("User chose not to add key: {}", key_name);
        return Ok(false);
    }

    println!("Adding SSH key: {}", key_name);
    println!("If the key is passphrase-protected, you will be prompted to enter it.");

    // Use interactive ssh-add - this will prompt the user directly in the terminal
    let status = Command::new("ssh-add")
        .arg(key_path)
        .env(
            "SSH_AUTH_SOCK",
            std::env::var("SSH_AUTH_SOCK").unwrap_or_default(),
        )
        .stdin(Stdio::inherit()) // Allow user to input passphrase directly
        .stdout(Stdio::inherit()) // Show ssh-add output to user
        .stderr(Stdio::inherit()) // Show ssh-add errors to user
        .status() // Use status() instead of output() to allow real-time interaction
        .map_err(|e| {
            Error::new(
                ErrorCode::Auth,
                ErrorClass::Net,
                format!("Failed to spawn ssh-add: {}", e),
            )
        })?;

    if status.success() {
        debug!("Successfully added key: {}", key_name);
        println!("✓ SSH key '{}' added successfully!", key_name);
        Ok(true)
    } else {
        debug!("Interactive ssh-add failed for key: {}", key_name);
        println!("✗ Failed to add SSH key '{}'", key_name);
        Ok(false)
    }
}
