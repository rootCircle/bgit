# Global (User) Configuration

BGit supports a per-user global configuration in addition to per-project `.bgit/config.toml`.

Locations:

- Linux/macOS: `~/.config/bgit/config.toml` (or `$XDG_CONFIG_HOME/bgit/config.toml`)
- Windows: `%APPDATA%/bgit/config.toml`

Schema (TOML):

```toml
[auth]
# One of: "repositoryURLBased" | "ssh" | "https"
preferred = "repositoryURLBased"

[auth.https]
# Optional: username and base64-encoded PAT for HTTPS remotes
# username = "alice"
# pat = "dG9rXzEyMw==" # base64 for tok_123

[auth.ssh]
# Optional: path to private key file (e.g., ~/.ssh/id_ed25519)
# key_file = "~/.ssh/id_ed25519"

[integrations]
# Optional base64-encoded Google API key
# google_api_key = "bXktZ29vZ2xlLWFwaS1rZXk="
google_api_key = ""
```

Behavior:

- `repositoryURLBased` (default): Same as current bgit logic.
- `ssh`: Prefer SSH keys/agent authentication when supported.
- `https`: Prefer HTTPS username/token when supported.

Notes:

- HTTPS credentials from `[auth.https]` are used automatically when set, otherwise you’ll be prompted.
- SSH `key_file` from `[auth.ssh]` is tried first; if it fails, bgit falls back to ssh-agent and auto-discovery in `~/.ssh`.

This file is optional. See `docs/config/CONFIGURATION.md` for project-level config.
