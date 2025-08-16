#![allow(unused)]
use crate::bgit_error::BGitError;
use base64::Engine;
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Global, per-user configuration stored under the user's config directory
/// (e.g. Linux/macOS: ~/.config/bgit/config.toml, Windows: %APPDATA%/bgit/config.toml).
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BGitGlobalConfig {
    #[serde(default)]
    pub auth: GlobalAuth,
    /// Third-party integrations and API keys
    #[serde(default)]
    pub integrations: GlobalIntegrations,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum PreferredAuth {
    #[serde(rename = "repositoryURLBased")]
    #[default]
    RepositoryURLBased,
    #[serde(rename = "ssh")]
    Ssh,
    #[serde(rename = "https")]
    Https,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlobalAuth {
    /// Preferred authentication method when both are possible.
    /// Values: "repositoryURLBased" | "ssh" | "https"
    #[serde(default)]
    pub preferred: PreferredAuth,
    /// HTTPS credentials (optional)
    #[serde(default)]
    pub https: HttpsAuth,
    /// SSH settings (optional)
    #[serde(default)]
    pub ssh: SshAuth,
}

impl Default for GlobalAuth {
    fn default() -> Self {
        Self {
            preferred: PreferredAuth::RepositoryURLBased,
            https: HttpsAuth::default(),
            ssh: SshAuth::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GlobalIntegrations {
    /// Optional Google API key stored as base64 in config and decoded on load.
    /// TOML path: [integrations] google_api_key = "...base64..."
    #[serde(default, deserialize_with = "deserialize_b64_opt")]
    pub google_api_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct HttpsAuth {
    /// Username for HTTPS auth
    pub username: Option<String>,
    /// Personal Access Token (base64-encoded in config, decoded on load)
    #[serde(default, deserialize_with = "deserialize_b64_opt")]
    pub pat: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SshAuth {
    /// Path to private key file to use for SSH auth (optional)
    pub key_file: Option<std::path::PathBuf>,
}

// Custom deserializer to decode optional base64 strings (generic messages)
fn deserialize_b64_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let opt = Option::<String>::deserialize(deserializer)?;
    if let Some(s) = opt {
        if s.is_empty() {
            return Ok(None);
        }
        match base64::engine::general_purpose::STANDARD.decode(s.as_bytes()) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(decoded) => Ok(Some(decoded)),
                Err(e) => Err(serde::de::Error::custom(format!(
                    "base64 decodes to non-UTF8: {e}"
                ))),
            },
            Err(e) => Err(serde::de::Error::custom(format!("Invalid base64: {e}"))),
        }
    } else {
        Ok(None)
    }
}

impl BGitGlobalConfig {
    /// Load global per-user config from the platform's config directory.
    /// If file is missing or invalid, returns defaults.
    pub fn load_global() -> Result<BGitGlobalConfig, Box<BGitError>> {
        let path = BGitGlobalConfig::find_global_config_path();
        debug!("Global config - resolved path: {}", path.display());

        if !path.exists() {
            debug!(
                "Global config file not found at {}, using defaults",
                path.display()
            );
            return Ok(BGitGlobalConfig::default());
        }

        let config_content = fs::read_to_string(&path).map_err(|e| {
            Box::new(BGitError::new(
                "Failed to read global config file",
                &format!("Could not read {}: {}", path.display(), e),
                crate::bgit_error::BGitErrorWorkflowType::Config,
                crate::bgit_error::NO_STEP,
                crate::bgit_error::NO_EVENT,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        let config: BGitGlobalConfig = toml::from_str(&config_content).map_err(|e| {
            Box::new(BGitError::new(
                "Failed to parse global config file",
                &format!("Invalid TOML in {}: {}", path.display(), e),
                crate::bgit_error::BGitErrorWorkflowType::Config,
                crate::bgit_error::NO_STEP,
                crate::bgit_error::NO_EVENT,
                crate::bgit_error::NO_RULE,
            ))
        })?;

        debug!(
            "Global config loaded: auth.preferred={:?}",
            config.auth.preferred
        );

        Ok(config)
    }

    /// Platform-appropriate path to the per-user bgit config file
    /// Linux/macOS: $XDG_CONFIG_HOME/bgit/config.toml or ~/.config/bgit/config.toml
    /// Windows: %APPDATA%/bgit/config.toml
    pub fn find_global_config_path() -> PathBuf {
        // Windows first
        #[cfg(windows)]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                let mut p = PathBuf::from(appdata);
                p.push("bgit");
                p.push("config.toml");
                debug!("Using Windows APPDATA for global config: {}", p.display());
                return p;
            }
        }

        // XDG on Unix (and also as a fallback on Windows if APPDATA not set)
        if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
            let mut p = PathBuf::from(xdg);
            p.push("bgit");
            p.push("config.toml");
            debug!("Using XDG_CONFIG_HOME for global config: {}", p.display());
            return p;
        }

        // Default to ~/.config/bgit/config.toml
        let mut p = home::home_dir().unwrap_or_else(|| PathBuf::from("."));
        p.push(".config");
        p.push("bgit");
        p.push("config.toml");
        debug!(
            "Using default ~/.config path for global config: {}",
            p.display()
        );
        p
    }
    /// Helper to fetch Google API key from new location.
    pub fn get_google_api_key(&self) -> Option<&str> {
        self.integrations.google_api_key.as_deref()
    }

    /// Helper to fetch HTTPS credentials if configured (username, pat)
    pub fn get_https_credentials(&self) -> Option<(&str, &str)> {
        match (&self.auth.https.username, &self.auth.https.pat) {
            (Some(u), Some(t)) if !u.is_empty() && !t.is_empty() => Some((u.as_str(), t.as_str())),
            _ => None,
        }
    }

    /// Helper to fetch preferred SSH key file path if configured, expanding ~ if present
    pub fn get_ssh_key_file(&self) -> Option<std::path::PathBuf> {
        let p = self.auth.ssh.key_file.as_ref()?;
        let s = p.to_string_lossy();
        if let Some(rest) = s.strip_prefix("~/")
            && let Some(home) = home::home_dir()
        {
            return Some(home.join(rest));
        }
        Some(p.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_global_google_api_key_decoding() {
        let key_plain = "my-google-api-key-123";
        let key_b64 = base64::engine::general_purpose::STANDARD.encode(key_plain.as_bytes());

        let content = format!(
            "[auth]\npreferred = \"repositoryURLBased\"\n\n[integrations]\ngoogle_api_key = \"{}\"\n",
            key_b64
        );
        let cfg: BGitGlobalConfig = toml::from_str(&content).unwrap();
        assert_eq!(cfg.integrations.google_api_key.as_deref(), Some(key_plain));
        // Accessor fallback should also return it
        assert_eq!(cfg.get_google_api_key(), Some(key_plain));
    }

    #[test]
    fn test_global_defaults_from_empty() {
        // Empty TOML should yield defaults
        let cfg: BGitGlobalConfig = toml::from_str("").unwrap();
        assert_eq!(cfg.auth.preferred, PreferredAuth::RepositoryURLBased);
        assert!(cfg.integrations.google_api_key.is_none());
        assert!(cfg.get_google_api_key().is_none());
    }

    #[test]
    fn test_global_preferred_variants() {
        let toml_repouri = r#"[auth]
preferred = "repositoryURLBased"
"#;
        let cfg: BGitGlobalConfig = toml::from_str(toml_repouri).unwrap();
        assert_eq!(cfg.auth.preferred, PreferredAuth::RepositoryURLBased);

        let toml_ssh = r#"[auth]
preferred = "ssh"
"#;
        let cfg: BGitGlobalConfig = toml::from_str(toml_ssh).unwrap();
        assert_eq!(cfg.auth.preferred, PreferredAuth::Ssh);

        let toml_https = r#"[auth]
preferred = "https"
"#;
        let cfg: BGitGlobalConfig = toml::from_str(toml_https).unwrap();
        assert_eq!(cfg.auth.preferred, PreferredAuth::Https);
    }

    #[test]
    fn test_global_google_api_key_invalid_base64() {
        let content = "[auth]\npreferred = \"repositoryURLBased\"\n[integrations]\ngoogle_api_key = \"not_base64!\"\n";
        let err = toml::from_str::<BGitGlobalConfig>(content).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("Invalid base64"));
    }

    #[test]
    fn test_global_https_pat_decoding() {
        let user = "alice";
        let pat_plain = "tok_123";
        let pat_b64 = base64::engine::general_purpose::STANDARD.encode(pat_plain.as_bytes());
        let content = format!(
            "[auth]\npreferred=\"https\"\n[auth.https]\nusername=\"{user}\"\npat=\"{pat}\"\n",
            user = user,
            pat = pat_b64
        );
        let cfg: BGitGlobalConfig = toml::from_str(&content).unwrap();
        let creds = cfg.get_https_credentials().unwrap();
        assert_eq!(creds.0, user);
        assert_eq!(creds.1, pat_plain);
    }
}
