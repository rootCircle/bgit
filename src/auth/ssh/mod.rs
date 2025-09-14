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

// Re-export common SSH functions from ssh_utils (platform-agnostic)
pub use ssh_utils::{
    add_all_ssh_keys_with_auth, add_key_interactive_with_auth, agent_identities_count_with_auth,
    get_effective_ssh_auth, set_global_ssh_env_for_libgit2, try_ssh_key_files_directly,
};

// Re-export platform-specific functions
pub use platform::ensure_agent_ready;
