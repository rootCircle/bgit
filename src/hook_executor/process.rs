use log::{error, info};

use crate::bgit_error::BGitError;
use std::io::{BufRead, BufReader};
use std::process;
use std::thread;

use super::error::create_hook_error;

/// Handles the stdout and stderr of a child process
pub fn handle_process_output(child: &mut process::Child) -> Result<(), Box<BGitError>> {
    // Take ownership of stdout and stderr
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| create_hook_error("Failed to capture stdout", "", "hook execution"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| create_hook_error("Failed to capture stderr", "", "hook execution"))?;

    // Stream stdout
    let stdout_thread = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            info!("[hook_executor] {}", line);
        }
    });

    // Stream stderr
    let stderr_thread = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            error!("[hook_executor] {}", line);
        }
    });

    // Wait for the streaming threads to finish
    stdout_thread
        .join()
        .map_err(|_| create_hook_error("Failed to join stdout thread", "", "hook execution"))?;
    stderr_thread
        .join()
        .map_err(|_| create_hook_error("Failed to join stderr thread", "", "hook execution"))?;

    Ok(())
}
