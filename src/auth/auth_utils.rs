use dialoguer::{Confirm, theme::ColorfulTheme};
use log::debug;

use crate::config::global::{BGitGlobalConfig, PreferredAuth};

/// Prompt the user to persist the preferred authentication method and save to global config.
/// No-op if the preferred method already matches.
pub fn prompt_persist_preferred_auth(cfg: &BGitGlobalConfig, method: PreferredAuth) {
    if cfg.auth.preferred == method {
        return;
    }
    let label = match method {
        PreferredAuth::Ssh => "SSH",
        PreferredAuth::Https => "HTTPS",
        PreferredAuth::RepositoryURLBased => "Repository URL based",
    };
    let question = format!("Set preferred auth to {} for future operations?", label);
    let confirm = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(question)
        .default(true)
        .interact()
        .unwrap_or(false);
    if !confirm {
        debug!("User declined setting preferred auth to {:?}", method);
        return;
    }
    let mut cfg_owned = cfg.clone();
    cfg_owned.auth.preferred = method;
    if let Err(e) = cfg_owned.save_global() {
        debug!("Failed to persist preferred auth: {:?}", e);
    } else {
        println!("Saved preferred auth to {}.", label);
        debug!("Persisted preferred auth to {:?}", method);
    }
}

/// Transform a repository URL to match the preferred auth method for known hosts.
/// Returns Some(new_url) when a transformation was applied, or None if unknown/unchanged.
pub fn transform_url_for_preference(url: &str, preferred: PreferredAuth) -> Option<String> {
    // Recognize current scheme/type
    let is_https = url.starts_with("https://");
    let is_ssh_scp = url.starts_with("git@") || url.starts_with("ssh://");

    match preferred {
        PreferredAuth::RepositoryURLBased => None, // keep as-is
        PreferredAuth::Https => {
            if is_https {
                None
            } else {
                to_https(url) // convert http/ssh->https for known hosts
            }
        }
        PreferredAuth::Ssh => {
            if is_ssh_scp {
                None
            } else {
                to_ssh(url) // convert https->ssh for known hosts
            }
        }
    }
}

fn to_https(url: &str) -> Option<String> {
    // SSH forms to convert:
    // - git@host:owner/repo(.git)
    // - ssh://git@host/owner/repo(.git)
    if let Some((host, path)) = parse_ssh_like(url) {
        if !is_known_host(&host) {
            return None;
        }
        let path_no_slash = path.strip_prefix('/').unwrap_or(&path);
        Some(format!("https://{}/{}", host, path_no_slash))
    } else if url.starts_with("http://") {
        // Upgrade http->https for known hosts
        if let Some(host) = host_from_http(url)
            && is_known_host(host)
        {
            return Some(url.replacen("http://", "https://", 1));
        }
        None
    } else {
        None
    }
}

fn to_ssh(url: &str) -> Option<String> {
    // HTTP form to SSH scp-like: https://host/owner/repo(.git) -> git@host:owner/repo(.git)
    if let Some((host, path)) = parse_http(url) {
        if !is_known_host(&host) {
            return None;
        }
        let path_no_slash = path.strip_prefix('/').unwrap_or(&path);
        Some(format!("git@{}:{}", host, path_no_slash))
    } else {
        None
    }
}

fn parse_http(url: &str) -> Option<(String, String)> {
    // naive parse: scheme://host/path
    let scheme_split = url.splitn(2, "://").collect::<Vec<_>>();
    if scheme_split.len() != 2 {
        return None;
    }
    let rest = scheme_split[1];
    let mut parts = rest.splitn(2, '/');
    let host = parts.next()?.to_string();
    let path = parts.next().unwrap_or("").to_string();
    Some((host, path))
}

fn host_from_http(url: &str) -> Option<&str> {
    let scheme_split = url.splitn(2, "://").collect::<Vec<_>>();
    if scheme_split.len() != 2 {
        return None;
    }
    let rest = scheme_split[1];
    let mut parts = rest.splitn(2, '/');
    parts.next()
}

fn parse_ssh_like(url: &str) -> Option<(String, String)> {
    // git@host:owner/repo(.git)
    if url.starts_with("git@") {
        let after_at = url.split_once('@')?.1;
        let mut host_path = after_at.splitn(2, ':');
        let host = host_path.next()?.to_string();
        let path = host_path.next()?.to_string();
        return Some((host, path));
    }
    // ssh://git@host/owner/repo(.git)
    if let Some(without_scheme) = url.strip_prefix("ssh://") {
        let after_at = without_scheme.split_once('@')?.1;
        let mut host_path = after_at.splitn(2, '/');
        let host = host_path.next()?.to_string();
        let path = host_path.next().unwrap_or("").to_string();
        return Some((host, path));
    }
    None
}

fn is_known_host(host: &str) -> bool {
    matches!(host, "github.com" | "gitlab.com" | "bitbucket.org")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssh_scp_to_https_known_hosts() {
        assert_eq!(
            transform_url_for_preference("git@github.com:owner/repo.git", PreferredAuth::Https)
                .as_deref(),
            Some("https://github.com/owner/repo.git")
        );
        assert_eq!(
            transform_url_for_preference("ssh://git@gitlab.com/owner/repo", PreferredAuth::Https)
                .as_deref(),
            Some("https://gitlab.com/owner/repo")
        );
    }

    #[test]
    fn https_to_ssh_known_hosts() {
        assert_eq!(
            transform_url_for_preference("https://github.com/owner/repo.git", PreferredAuth::Ssh)
                .as_deref(),
            Some("git@github.com:owner/repo.git")
        );
        assert_eq!(
            transform_url_for_preference("https://bitbucket.org/owner/repo", PreferredAuth::Ssh)
                .as_deref(),
            Some("git@bitbucket.org:owner/repo")
        );
    }

    #[test]
    fn http_upgrade_to_https_for_known_hosts() {
        assert_eq!(
            transform_url_for_preference("http://github.com/owner/repo", PreferredAuth::Https)
                .as_deref(),
            Some("https://github.com/owner/repo")
        );
    }

    #[test]
    fn unknown_hosts_do_not_transform() {
        assert!(
            transform_url_for_preference("git@example.com:owner/repo", PreferredAuth::Https)
                .is_none()
        );
        assert!(
            transform_url_for_preference("https://example.com/owner/repo", PreferredAuth::Ssh)
                .is_none()
        );
        assert!(
            transform_url_for_preference("http://example.com/owner/repo", PreferredAuth::Https)
                .is_none()
        );
    }

    #[test]
    fn no_op_when_already_matching_preference() {
        assert!(
            transform_url_for_preference("https://github.com/owner/repo", PreferredAuth::Https)
                .is_none()
        );
        assert!(
            transform_url_for_preference("git@github.com:owner/repo", PreferredAuth::Ssh).is_none()
        );
    }
}
