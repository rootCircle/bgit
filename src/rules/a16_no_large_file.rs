use crate::bgit_error::{BGitError, BGitErrorWorkflowType, NO_EVENT, NO_STEP};
use crate::config::WorkflowRules;
use crate::constants::{
    DEFAULT_MAX_CUMMULATIVE_STAGED_FILE_SIZE_IN_BYTES, DEFAULT_MAX_LARGE_FILE_SIZE_IN_BYTES,
};
use crate::rules::{Rule, RuleLevel, RuleOutput};
use git2::{Repository, Status, StatusOptions};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub(crate) struct NoLargeFile {
    name: String,
    description: String,
    level: RuleLevel,
    threshold_bytes: u64,
    /// Total size threshold for cummulative large files in bytes
    total_threshold_bytes: u64,
}

impl Rule for NoLargeFile {
    fn new(workflow_rule_config: Option<&WorkflowRules>) -> Self {
        let default_rule_level = RuleLevel::Error;
        let name = "NoLargeFile";
        let rule_level = workflow_rule_config
            .and_then(|config| config.get_rule_level(name))
            .cloned()
            .unwrap_or(default_rule_level);

        Self {
            name: name.to_string(),
            description: "Ensure large files are tracked with Git LFS".to_string(),
            level: rule_level,
            threshold_bytes: DEFAULT_MAX_LARGE_FILE_SIZE_IN_BYTES,
            total_threshold_bytes: DEFAULT_MAX_CUMMULATIVE_STAGED_FILE_SIZE_IN_BYTES,
        }
    }

    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_description(&self) -> &str {
        &self.description
    }

    fn get_level(&self) -> RuleLevel {
        self.level.clone()
    }

    fn check(&self) -> Result<RuleOutput, Box<BGitError>> {
        let repo = match Repository::open(".") {
            Ok(repo) => repo,
            Err(e) => {
                return Ok(RuleOutput::Exception(format!(
                    "Failed to open repository: {}",
                    e
                )));
            }
        };

        let mut status_options = StatusOptions::new();
        status_options.include_untracked(true);
        status_options.include_ignored(false);

        let statuses = match repo.statuses(Some(&mut status_options)) {
            Ok(statuses) => statuses,
            Err(e) => {
                return Ok(RuleOutput::Exception(format!(
                    "Failed to get repository status: {}",
                    e
                )));
            }
        };

        let mut total_size = 0u64;
        let mut file_count = 0;
        let mut large_files = Vec::new();

        for entry in statuses.iter() {
            let file_path = match entry.path() {
                Some(path) => path,
                None => continue,
            };

            let status = entry.status();

            // Check if file is staged or modified (but not ignored)
            if status.contains(Status::INDEX_NEW)
                || status.contains(Status::INDEX_MODIFIED)
                || status.contains(Status::WT_NEW)
                || status.contains(Status::WT_MODIFIED)
            {
                if let Ok(file_size) = Self::get_path_size(file_path) {
                    total_size += file_size;
                    file_count += 1;
                    if file_size > self.threshold_bytes && !self.is_lfs_tracked(file_path)? {
                        large_files.push(format!(
                            "{} ({:.1} MB)",
                            file_path,
                            file_size as f64 / (1024.0 * 1024.0)
                        ));
                    }
                }
            }
        }

        if total_size > self.total_threshold_bytes {
            Ok(RuleOutput::Exception(format!(
                "Total size of staged/modified files ({:.1} MB across {} files) exceeds threshold ({:.1} MB). Consider using Git LFS or .gitignore for large files.",
                total_size as f64 / (1024.0 * 1024.0),
                file_count,
                self.total_threshold_bytes as f64 / (1024.0 * 1024.0)
            )))
        } else if !large_files.is_empty() {
            Ok(RuleOutput::Exception(format!(
                "Large files detected that should use Git LFS: {}",
                large_files.join(", ")
            )))
        } else {
            Ok(RuleOutput::Success)
        }
    }

    fn try_fix(&self) -> Result<bool, Box<BGitError>> {
        let repo = match Repository::open(".") {
            Ok(repo) => repo,
            Err(e) => {
                return Err(Box::new(BGitError::new(
                    "Failed to open repository",
                    &e.to_string(),
                    BGitErrorWorkflowType::Rules,
                    NO_STEP,
                    NO_EVENT,
                    self.get_name(),
                )));
            }
        };

        let mut status_options = StatusOptions::new();
        status_options.include_untracked(true);
        status_options.include_ignored(false);

        let statuses = match repo.statuses(Some(&mut status_options)) {
            Ok(statuses) => statuses,
            Err(e) => {
                return Err(Box::new(BGitError::new(
                    "Failed to get repository status",
                    &e.to_string(),
                    BGitErrorWorkflowType::Rules,
                    NO_STEP,
                    NO_EVENT,
                    self.get_name(),
                )));
            }
        };

        let mut total_size = 0u64;
        let mut file_count = 0;
        let mut large_files = Vec::new();

        for entry in statuses.iter() {
            let file_path = match entry.path() {
                Some(path) => path,
                None => continue,
            };

            let status = entry.status();

            if status.contains(Status::INDEX_NEW)
                || status.contains(Status::INDEX_MODIFIED)
                || status.contains(Status::WT_NEW)
                || status.contains(Status::WT_MODIFIED)
            {
                if let Ok(file_size) = Self::get_path_size(file_path) {
                    total_size += file_size;
                    file_count += 1;
                    if file_size > self.threshold_bytes && !self.is_lfs_tracked(file_path)? {
                        large_files.push(file_path.to_string());
                    }
                }
            }
        }

        // Check if total size threshold is exceeded
        let total_threshold_exceeded = total_size > self.total_threshold_bytes;

        if !total_threshold_exceeded && large_files.is_empty() {
            return Ok(true);
        }

        if total_threshold_exceeded {
            println!(
                "Total size of staged/modified files ({:.1} MB across {} files) exceeds threshold ({:.1} MB).",
                total_size as f64 / (1024.0 * 1024.0),
                file_count,
                self.total_threshold_bytes as f64 / (1024.0 * 1024.0)
            );
            println!("Consider using Git LFS for large files or adding them to .gitignore.\n");
        }

        if !large_files.is_empty() {
            println!("Large files detected that should use Git LFS:");
            for file in &large_files {
                let size = Self::get_path_size(file).unwrap_or(0);
                println!("  {} ({:.1} MB)", file, size as f64 / (1024.0 * 1024.0));
            }
        }

        println!("\nTo fix this issue:");
        println!("1. Install Git LFS if not already installed:");
        println!("   git lfs install");

        if !large_files.is_empty() {
            println!("\n2. Track large files by extension or specific files:");

            // Suggest tracking by extension
            let mut extensions = std::collections::HashSet::new();
            for file in &large_files {
                if let Some(ext) = Path::new(file).extension().and_then(|s| s.to_str()) {
                    extensions.insert(ext);
                }
            }

            for ext in &extensions {
                println!("   git lfs track \"*.{}\"", ext);
            }

            println!("\n3. Add .gitattributes and re-add the files:");
            println!("   git add .gitattributes");
            println!("   git add {}", large_files.join(" "));

            // For automatic fix, we'll add the extensions to .gitattributes
            match self.add_lfs_tracking(&extensions.into_iter().collect::<Vec<_>>()) {
                Ok(_) => {
                    println!("\nAutomatically added LFS tracking to .gitattributes");
                    Ok(true)
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to automatically update .gitattributes: {}",
                        e
                    );
                    Ok(false)
                }
            }
        } else {
            println!("\nConsider reviewing your files and either:");
            println!("- Adding large files to .gitignore if they shouldn't be tracked");
            println!("- Setting up Git LFS for files that should be version controlled");
            Ok(false)
        }
    }
}

impl NoLargeFile {
    fn get_path_size(path: &str) -> Result<u64, std::io::Error> {
        let metadata = fs::metadata(path)?;
        if metadata.is_file() {
            Ok(metadata.len())
        } else if metadata.is_dir() {
            let mut total_size = 0u64;
            let entries = fs::read_dir(path)?;
            for entry in entries {
                let entry = entry?;
                let entry_path = entry.path();
                if let Some(path_str) = entry_path.to_str() {
                    total_size += Self::get_path_size(path_str)?;
                }
            }
            Ok(total_size)
        } else {
            Ok(0)
        }
    }

    fn is_lfs_tracked(&self, file_path: &str) -> Result<bool, Box<BGitError>> {
        let repo = match Repository::open(".") {
            Ok(repo) => repo,
            Err(_) => return Ok(false),
        };

        let repo_path = match repo.workdir() {
            Some(path) => path,
            None => return Ok(false),
        };

        let gitattributes_path = repo_path.join(".gitattributes");

        if !gitattributes_path.exists() {
            return Ok(false);
        }

        let file = match fs::File::open(&gitattributes_path) {
            Ok(file) => file,
            Err(_) => return Ok(false),
        };

        let reader = BufReader::new(file);
        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(file_path);

        let file_ext = Path::new(file_path)
            .extension()
            .and_then(|ext| ext.to_str());

        for line in reader.lines() {
            let line = match line {
                Ok(line) => line.trim().to_string(),
                Err(_) => continue,
            };

            if line.contains("filter=lfs") {
                let pattern = line.split_whitespace().next().unwrap_or("");

                // Check if the pattern matches the file
                if pattern == file_path || pattern == file_name {
                    return Ok(true);
                }

                // Check wildcard patterns like *.mp4
                if let Some(ext) = file_ext {
                    if pattern == format!("*.{}", ext) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }

    fn add_lfs_tracking(&self, extensions: &[&str]) -> Result<(), std::io::Error> {
        let repo = match Repository::open(".") {
            Ok(repo) => repo,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not open git repository",
                ));
            }
        };

        let repo_path = match repo.workdir() {
            Some(path) => path,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Repository has no working directory",
                ));
            }
        };

        let gitattributes_path = repo_path.join(".gitattributes");

        // Read existing content to avoid duplicates
        let existing_content = if gitattributes_path.exists() {
            fs::read_to_string(&gitattributes_path)?
        } else {
            String::new()
        };

        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&gitattributes_path)?;

        for ext in extensions {
            let pattern = format!("*.{}", ext);
            let lfs_line = format!("{} filter=lfs diff=lfs merge=lfs -text", pattern);

            // Only add if not already present
            if !existing_content.contains(&lfs_line) {
                writeln!(file, "{}", lfs_line)?;
            }
        }

        Ok(())
    }
}
