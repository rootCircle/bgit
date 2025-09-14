pub(crate) const DEFAULT_MAX_LARGE_FILE_SIZE_IN_BYTES: u64 = 2 * 1024 * 1024; // 2 MiB
pub(crate) const DEFAULT_MAX_REPO_SIZE_IN_MIB: u64 = 128; // 128 MiB
pub(crate) const DEFAULT_MAX_CUMMULATIVE_STAGED_FILE_SIZE_IN_BYTES: u64 = 32 * 1024 * 1024; // 32 MiB

// Authentication related defaults
pub(crate) const MAX_AUTH_ATTEMPTS: usize = 3;

// SSH agent socket basename.
// On Unix we bind ssh-agent to $HOME/.ssh/bgit_ssh_agent.sock.
// On non-Unix platforms this value is still defined for cross-platform builds,
// but may not be used (e.g., Windows typically relies on named pipes or env vars).
pub(crate) const SSH_AGENT_SOCKET_BASENAME: &str = "bgit_ssh_agent.sock";
