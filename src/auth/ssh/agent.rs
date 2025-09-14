use git2::Error;
use std::path::Path;

/// Trait for platform-specific SSH agent management behavior
pub trait SshAgentManager {
    /// Ensure an SSH agent is ready and available
    /// This method handles platform-specific agent setup and socket management
    fn ensure_agent_ready() -> Result<(), Error>;

    /// Start SSH agent in a detached manner
    /// Unix: Can bind to specific socket path
    /// Windows: Simple detached spawn
    /// Unsupported: Returns error
    fn start_agent_detached(socket_path: Option<&Path>) -> Result<(), Error>;
}
