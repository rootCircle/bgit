use crate::utils::prevalidation::PreValidation;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

pub struct TestEnv {
    pub temp_dir: TempDir,
    pub repo_path: PathBuf,
}

impl TestEnv {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        PreValidation::validate_all()?;

        let temp_dir = TempDir::with_prefix("bgit_test_")?;
        let repo_path = temp_dir.path().to_path_buf();

        let output = Command::new("git")
            .args(["init"])
            .current_dir(&repo_path)
            .output()?;

        if !output.status.success() {
            return Err("Failed to initialize git repository".into());
        }

        let test_env = TestEnv {
            temp_dir,
            repo_path,
        };

        Ok(test_env)
    }

    pub fn setup_git_user(
        &self,
        name: &str,
        email: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        std::process::Command::new("git")
            .args(["config", "user.name", name])
            .current_dir(&self.repo_path)
            .output()?;

        std::process::Command::new("git")
            .args(["config", "user.email", email])
            .current_dir(&self.repo_path)
            .output()?;

        Ok(())
    }

    pub fn git_init(&self) -> Result<(), Box<dyn std::error::Error>> {
        let output = std::process::Command::new("git")
            .args(["init"])
            .current_dir(&self.repo_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to initialize git: {stderr}").into());
        }

        // Setup basic git config
        self.setup_git_user("Test User", "test@example.com")?;

        Ok(())
    }
    pub fn create_file(
        &self,
        relative_path: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new(relative_path);
        if path.is_absolute()
            || path
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            return Err("Path must be relative and cannot contain '..' components".into());
        }

        let file_path = self.repo_path.join(relative_path);

        if !file_path.starts_with(self.temp_dir.path()) {
            return Err("File path must be within the repository directory".into());
        }

        let file_path = self.repo_path.join(relative_path);

        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(file_path, content)?;
        Ok(())
    }

    pub fn create_files(&self, files: &[(&str, &str)]) -> Result<(), Box<dyn std::error::Error>> {
        for (path, content) in files {
            self.create_file(path, content)?;
        }
        Ok(())
    }

    pub fn stage_files(&self, patterns: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        for pattern in patterns {
            let output = Command::new("git")
                .args(["add", pattern])
                .current_dir(&self.repo_path)
                .output()?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Failed to stage {pattern}: {stderr}").into());
            }
        }
        Ok(())
    }

    pub fn run_bgit(
        &self,
        args: &[&str],
    ) -> Result<std::process::Output, Box<dyn std::error::Error>> {
        let output = assert_cmd::Command::cargo_bin("bgit")?
            .args(args)
            .current_dir(&self.repo_path)
            .output()?;

        Ok(output)
    }

    pub fn path(&self) -> &Path {
        &self.repo_path
    }

    pub fn git_status(&self) -> Result<String, Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(&self.repo_path)
            .output()?;

        Ok(String::from_utf8(output.stdout)?)
    }

    pub fn commit(&self, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let output = Command::new("git")
            .args(["commit", "-m", message])
            .current_dir(&self.repo_path)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("Failed to commit: {stderr}").into());
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! test_env {
    ($name:ident) => {
        let $name = $crate::utils::test_env::TestEnv::new()?;
        $name.setup_git_user("Test User", "test@example.com")?;
    };
}
