// Shared utilities (platform-agnostic)
mod agent;
mod ssh_utils;

// Platform-specific SSH implementations
#[cfg(unix)]
mod unix;
#[cfg(not(any(windows, unix)))]
mod unsupported;
#[cfg(windows)]
mod windows;

// Platform aliases for easy access
#[cfg(unix)]
pub mod platform {
    pub use super::unix::*;
}

#[cfg(windows)]
pub mod platform {
    pub use super::windows::*;
}

#[cfg(not(any(windows, unix)))]
pub mod platform {
    pub use super::unsupported::*;
}

// Re-export functions based on platform
#[cfg(any(unix, windows))]
pub use ssh_utils::{add_all_ssh_keys, agent_identities_count, try_ssh_key_files_directly};

// On unsupported platforms, export functions from the unsupported module instead
#[cfg(not(any(windows, unix)))]
pub use platform::{add_all_ssh_keys, agent_identities_count, try_ssh_key_files_directly};

// Re-export platform-specific ensure_agent_ready function
pub use platform::ensure_agent_ready;
