use git2::{CertificateCheckStatus, CredentialType, RemoteCallbacks};
use log::debug;
use std::sync::{Arc, Mutex};

use crate::auth::{git_http::try_userpass_authentication, git_ssh::ssh_authenticate_git};

pub fn setup_auth_callbacks() -> RemoteCallbacks<'static> {
    let mut callbacks = RemoteCallbacks::new();

    // Track attempt count across callback invocations
    let attempt_count: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    callbacks.credentials(move |url, username_from_url, allowed_types| {
        let mut count = attempt_count.lock().unwrap();
        *count += 1;
        let current_attempt = *count;
        drop(count);

        if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
            try_userpass_authentication(username_from_url)
        } else {
            ssh_authenticate_git(url, username_from_url, allowed_types, current_attempt)
        }
    });

    // Set up certificate check callback for HTTPS
    callbacks.certificate_check(|_cert, _host| {
        // TODO(rootCircle): make this configurable and secure. For now we accept all certs.
        debug!("Skipping certificate verification (INSECURE)");
        Ok(CertificateCheckStatus::CertificateOk)
    });

    callbacks
}
