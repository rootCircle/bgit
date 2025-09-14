// Platform-specific SSH implementations
#[cfg(unix)]
mod unix;
#[cfg(not(any(windows, unix)))]
mod unsupported;
#[cfg(windows)]
mod windows;

// Re-export platform-specific functions with a unified interface
// This follows the same pattern as hook_executor
#[cfg(unix)]
pub use self::unix::{
    add_all_ssh_keys, agent_identities_count, ensure_agent_ready, try_ssh_key_files_directly,
};

#[cfg(windows)]
pub use self::windows::{
    add_all_ssh_keys, agent_identities_count, ensure_agent_ready, try_ssh_key_files_directly,
};

#[cfg(not(any(windows, unix)))]
pub use self::unsupported::{
    add_all_ssh_keys, agent_identities_count, ensure_agent_ready, try_ssh_key_files_directly,
};
