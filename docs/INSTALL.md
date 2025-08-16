# Installation Guide

This guide covers all installation methods for bgit, including one-liners, advanced options, supported targets, checksums/signatures, and troubleshooting.

## Quick install (recommended)

- Linux/macOS

```bash
curl -fsSL https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.sh | bash
# or
wget -qO- https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.sh | bash
```

- Windows (PowerShell)

```powershell
iwr -useb https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.ps1 | iex
```

## Advanced options

- Choose a specific version:

  - Linux/macOS

    ```bash
    curl -fsSL https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.sh | bash -s -- install --tag vX.Y.Z
    ```

  - Windows (PowerShell)

    ```powershell
    iwr -useb https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.ps1 | iex; Install-Bgit -Tag vX.Y.Z
    ```

- Custom install directory:

  - Linux/macOS: add `--to /path/to/bin` (will use sudo if needed)
  - Windows: add `-To 'C:\Path\To\bgit'`

- Prefer static musl on Linux x86_64: prefix `PREFER_MUSL=1` (or `MUSL=1`)

  ```bash
  PREFER_MUSL=1 curl -fsSL https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.sh | bash
  ```

## Lifecycle commands

The installers support the following commands:

- install (default)
- update
- uninstall
- purge — complete uninstall: also removes `~/.bgit` and `~/.ssh/bgit_ssh_agent.sock`

Examples (Linux/macOS):

```bash
curl -fsSL https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.sh | bash -s -- update
curl -fsSL https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.sh | bash -s -- uninstall
curl -fsSL https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.sh | bash -s -- purge
```

Examples (Windows):

```powershell
iwr -useb https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.ps1 | iex; Install-Bgit          # install/update
iwr -useb https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.ps1 | iex; Uninstall-Bgit        # uninstall
iwr -useb https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.ps1 | iex; Purge-Bgit            # purge
```

## Prebuilt binaries (GitHub Releases)

Typical release assets include:

- Linux x86_64 (glibc)
- Linux x86_64 (musl/static)
- macOS arm64 (Apple Silicon)
- Windows x86_64

Asset naming convention:

- `bgit-vX.Y.Z-<os>-<arch>.{tar.gz|zip}` with `-musl` suffix where applicable. Examples:

  - `bgit-v0.3.1-ubuntu-latest-x86_64.tar.gz`
  - `bgit-v0.3.1-ubuntu-latest-x86_64-musl.tar.gz`
  - `bgit-v0.3.1-macos-latest-arm64.tar.gz`
  - `bgit-v0.3.1-windows-latest-AMD64.zip`

Note: Linux aarch64 prebuilt binaries are not currently published. For ARM64 Linux, please build from source using Rust and Cargo.

## Checksums and signatures

- Each artifact has a `.sha256` file.
- `RELEASES.txt` aggregates all checksums in one file.
- If enabled in CI, `RELEASES.txt.asc` is a detached GPG signature of `RELEASES.txt`.

## Install via Cargo (crates.io)

```bash
cargo install bgit
```

## Troubleshooting

- PATH: If you installed to `~/.local/bin` or a custom dir, ensure it’s in your PATH.
- sudo: When installing to system locations (e.g., `/usr/local/bin`), the installer may use sudo.
- musl vs glibc: Prefer `PREFER_MUSL=1` on Linux x86_64 if you want static binaries.
- Windows PATH: If the installer’s directory isn’t on PATH, add it via System Properties or using `setx`.
