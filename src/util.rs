use std::path::{Path, PathBuf};

/// # Extension Precedence (for Windows)
/// The module checks for hook files in the following order:
/// 1. Shell scripts with no extension (direct executable)
/// 2. .bat (Batch files)
/// 3. .cmd (Command files)
/// 4. .ps1 (PowerShell scripts)
/// 5. .exe (Executable files)
#[cfg(windows)]
pub(crate) fn find_hook_with_extension(hook_path: &Path) -> Option<PathBuf> {
    let hook_dir = hook_path.parent()?;
    let hook_name = hook_path.file_name()?.to_str()?;

    // If hook exists with no extension
    if hook_path.exists() {
        return Some(hook_path.to_path_buf());
    }

    // Try different extensions
    for ext in &[".bat", ".cmd", ".ps1", ".exe"] {
        let path = hook_dir.join(format!("{hook_name}{ext}"));
        if path.exists() {
            return Some(path);
        }
    }

    None
}

#[cfg(not(windows))]
pub(crate) fn find_hook_with_extension(hook_path: &Path) -> Option<PathBuf> {
    if hook_path.exists() {
        return Some(hook_path.to_path_buf());
    }

    None
}
