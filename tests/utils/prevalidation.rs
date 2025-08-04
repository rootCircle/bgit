use std::process::Command;

pub struct PreValidation;

impl PreValidation {
    pub fn validate_all() -> Result<(), Box<dyn std::error::Error>> {
        Self::validate_git()?;
        Ok(())
    }

    fn validate_git() -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git").args(["--version"]).output()?;

        if !output.status.success() {
            return Err("Git is not installed or not accessible".into());
        }

        Ok(())
    }
}
