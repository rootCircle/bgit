use dialoguer::{Input, Password, theme::ColorfulTheme};
use git2::{Cred, Error, ErrorClass, ErrorCode};
use log::debug;

pub fn try_userpass_authentication(username_from_url: Option<&str>) -> Result<Cred, Error> {
    debug!("USER_PASS_PLAINTEXT authentication is allowed, prompting for credentials");

    // Prompt for username if not provided in URL
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
                    format!("Failed to read username: {}", e),
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
                format!("Failed to read token: {}", e),
            )
        })?;

    if !username.is_empty() && !token.is_empty() {
        debug!("Creating credentials with username and token");
        match Cred::userpass_plaintext(&username, &token) {
            Ok(cred) => {
                debug!("Username/token authentication succeeded");
                Ok(cred)
            }
            Err(e) => {
                debug!("Username/token authentication failed: {}", e);
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
