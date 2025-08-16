use dialoguer::{Confirm, Input, Password, theme::ColorfulTheme};
use git2::{Cred, Error, ErrorClass, ErrorCode};
use log::debug;

use crate::auth::auth_utils::prompt_persist_preferred_auth;
use crate::config::global::{BGitGlobalConfig, PreferredAuth};

pub fn try_userpass_authentication(
    username_from_url: Option<&str>,
    cfg: &BGitGlobalConfig,
) -> Result<Cred, Error> {
    debug!("USER_PASS_PLAINTEXT authentication allowed; trying global config first");
    // Try global config first; fall back to prompt if it fails
    if let Some((u, t)) = cfg.get_https_credentials() {
        match Cred::userpass_plaintext(u, t) {
            Ok(cred) => {
                debug!("Using HTTPS credentials from global config");
                return Ok(cred);
            }
            Err(e) => {
                debug!("Global HTTPS credentials failed: {e}; falling back to prompt");
            }
        }
    }

    let username = if let Some(user) = username_from_url {
        user.to_string()
    } else {
        Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter your username")
            .interact()
            .map_err(|e| {
                Error::new(
                    ErrorCode::Auth,
                    ErrorClass::Net,
                    format!("Failed to read username: {e}"),
                )
            })?
    };

    let token = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter your personal access token")
        .interact()
        .map_err(|e| {
            Error::new(
                ErrorCode::Auth,
                ErrorClass::Net,
                format!("Failed to read token: {e}"),
            )
        })?;

    if !username.is_empty() && !token.is_empty() {
        debug!("Creating credentials with username and token");
        match Cred::userpass_plaintext(&username, &token) {
            Ok(cred) => {
                debug!("Username/token authentication succeeded");
                // Offer to save to global config
                prompt_persist_https_credentials(cfg, &username, &token);
                // Offer to set preferred auth to HTTPS
                prompt_persist_preferred_auth(cfg, PreferredAuth::Https);
                Ok(cred)
            }
            Err(e) => {
                debug!("Username/token authentication failed: {e}");
                Err(e)
            }
        }
    } else {
        debug!("Username or token is empty, skipping userpass authentication");
        Err(Error::new(
            ErrorCode::Auth,
            ErrorClass::Net,
            "Username or token cannot be empty",
        ))
    }
}

fn prompt_persist_https_credentials(cfg: &BGitGlobalConfig, username: &str, token: &str) {
    // Skip if already configured with identical values
    if cfg.auth.https.username.as_deref() == Some(username)
        && cfg.auth.https.pat.as_deref() == Some(token)
    {
        return;
    }

    let question = format!(
        "Save HTTPS credentials for '{}' to global config? (token stored base64-encoded)",
        username
    );
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .default(false)
        .interact()
        .unwrap_or(false);
    if !confirm {
        debug!("User declined persisting HTTPS credentials");
        return;
    }

    let mut cfg_owned = cfg.clone();
    cfg_owned.auth.https.username = Some(username.to_string());
    cfg_owned.auth.https.pat = Some(token.to_string());
    if let Err(e) = cfg_owned.save_global() {
        debug!("Failed to persist HTTPS credentials: {:?}", e);
    } else {
        println!(
            "Saved HTTPS username + token to global config for user '{}'.",
            username
        );
        debug!("Persisted HTTPS credentials for user '{}'.", username);
    }
}
