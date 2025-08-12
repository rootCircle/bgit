#!/usr/bin/env bash
set -euo pipefail

# Installer for bgit: curl -fsSL https://raw.githubusercontent.com/rootCircle/bgit/main/scripts/install.sh | bash

REPO="rootCircle/bgit"
BIN_NAME="bgit"
INSTALL_DIR_DEFAULT="$HOME/.local/bin"
SUDO_BIN="sudo"

command_exists() { command -v "$1" >/dev/null 2>&1; }

detect_os() {
  case "$(uname -s)" in
    Linux) echo linux;;
    Darwin) echo macos;;
    *) echo unsupported;;
  esac
}

detect_arch() {
  local arch
  arch=$(uname -m)
  case "$arch" in
    x86_64|amd64) echo x86_64;;
  arm64) echo arm64;;
  aarch64) echo aarch64;;
    *) echo "$arch";;
  esac
}

usage() {
  cat <<EOF
Usage: install.sh [install|update|uninstall|purge] [--tag vX.Y.Z] [--to DIR] [--no-sudo]

Options:
  --tag       Install specific tag (default: latest)
  --to DIR    Install directory (default: $INSTALL_DIR_DEFAULT or /usr/local/bin)
  --no-sudo   Do not use sudo when writing to system directories
Commands:
  install     Install bgit (default)
  update      Update bgit to latest or specified --tag
  uninstall   Remove the bgit binary from common install locations
  purge       Complete uninstall: uninstall + remove ~/.bgit and ~/.ssh/bgit_ssh_agent.sock
EOF
}

CMD="install"
TAG=""
INSTALL_DIR=""
USE_SUDO=1
while [ "$#" -gt 0 ]; do
  case "$1" in
    install|update|uninstall|purge)
      CMD="$1"; shift;;
    --tag) TAG="$2"; shift 2;;
    --to) INSTALL_DIR="$2"; shift 2;;
    --no-sudo) USE_SUDO=0; shift;;
    -h|--help) usage; exit 0;;
    *) echo "Unknown arg: $1" >&2; usage; exit 2;;
  esac
done

OS=$(detect_os)
ARCH=$(detect_arch)
if [ "$OS" = unsupported ]; then
  echo "Unsupported OS: $(uname -s)" >&2
  exit 1
fi

if [ -z "$INSTALL_DIR" ]; then
  if [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
  else
    INSTALL_DIR="$INSTALL_DIR_DEFAULT"
  fi
fi

[ "$CMD" != "uninstall" ] && [ "$CMD" != "purge" ] && mkdir -p "$INSTALL_DIR"

if ! command_exists curl && ! command_exists wget; then
  echo "Need curl or wget to download assets" >&2
  exit 1
fi

gh_api() {
  local url=$1
  if command_exists curl; then
    curl -fsSL "$url"
  else
    wget -qO- "$url"
  fi
}

get_latest_tag() {
  gh_api "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/^[[:space:]]*"tag_name":[[:space:]]*"\(v[^"\n]*\)".*/\1/p'
}

do_resolve_tag() {
  TAG=${TAG:-$(get_latest_tag)}
  if [ -z "$TAG" ]; then
    echo "Could not determine latest release tag" >&2
    exit 1
  fi
}

# Map matrix.os to artifact OS label
case "$OS" in
  linux) ART_OS="ubuntu-latest";;
  macos) ART_OS="macos-latest";;
esac

# Try common arch labels + prefer musl on Linux if available via --prefer-musl or MUSL=1
PREFER_MUSL=${PREFER_MUSL:-${MUSL:-0}}
case "$ARCH" in
  x86_64)
    if [ "$OS" = linux ] && [ "$PREFER_MUSL" != 0 ]; then ART_ARCH="x86_64-musl"; else ART_ARCH="x86_64"; fi;;
  aarch64) ART_ARCH="aarch64";;
  arm64)
    if [ "$OS" = macos ]; then ART_ARCH="aarch64"; else ART_ARCH="arm64"; fi;;
  *) ART_ARCH="$ARCH";;
esac

do_install() {
  do_resolve_tag
  ASSET_TAR="${BIN_NAME}-${TAG}-${ART_OS}-${ART_ARCH}.tar.gz"
  ASSET_SHA="${ASSET_TAR}.sha256"

  REL_JSON=$(gh_api "https://api.github.com/repos/$REPO/releases/tags/$TAG")
  URL_TAR=$(printf "%s" "$REL_JSON" | sed -n "s@^[[:space:]]*\"browser_download_url\":[[:space:]]*\"\(.*${ASSET_TAR}\)\".*@\1@p")
  URL_SHA=$(printf "%s" "$REL_JSON" | sed -n "s@^[[:space:]]*\"browser_download_url\":[[:space:]]*\"\(.*${ASSET_SHA}\)\".*@\1@p")

  if [ -z "$URL_TAR" ]; then
    echo "Could not find asset: $ASSET_TAR" >&2
    echo "Release may not have an artifact for ${OS}/${ARCH}" >&2
    exit 1
  fi

  TMP_DIR=$(mktemp -d)
  cleanup() { rm -rf "$TMP_DIR"; }
  trap cleanup EXIT

  DL() {
    if command_exists curl; then
      curl -fL --retry 3 -o "$1" "$2"
    else
      wget -O "$1" "$2"
    fi
  }

  echo "Downloading: $ASSET_TAR"
  DL "$TMP_DIR/$ASSET_TAR" "$URL_TAR"
  if [ -n "$URL_SHA" ]; then
    echo "Downloading checksum: $ASSET_SHA"
    DL "$TMP_DIR/$ASSET_SHA" "$URL_SHA" || true
  fi

  echo "Verifying checksum (if available)"
  if [ -f "$TMP_DIR/$ASSET_SHA" ]; then
    (cd "$TMP_DIR" && sha256sum -c "$ASSET_SHA" 2>/dev/null) || (cd "$TMP_DIR" && shasum -a 256 -c "$ASSET_SHA")
  fi

  echo "Extracting"
  tar -C "$TMP_DIR" -xzf "$TMP_DIR/$ASSET_TAR"
  if [ ! -x "$TMP_DIR/$BIN_NAME" ]; then
    echo "Binary not found in archive" >&2
    exit 1
  fi

  TARGET="$INSTALL_DIR/$BIN_NAME"
  if [ "$USE_SUDO" = 1 ] && [ ! -w "$INSTALL_DIR" ]; then
    if command_exists sudo; then SUDO=$SUDO_BIN; else SUDO=""; fi
  else
    SUDO=""
  fi

  echo "Installing to $TARGET"
  ${SUDO:+$SUDO }install -m 0755 "$TMP_DIR/$BIN_NAME" "$TARGET"

  case :$PATH: in
    *:"$INSTALL_DIR":*) ;;
    *) echo "Note: $INSTALL_DIR is not on PATH. Consider adding it." ;;
  esac

  echo "Installed $BIN_NAME $TAG to $TARGET"
}

do_uninstall() {
  # Try to find installed binary
  TARGET="${INSTALL_DIR%/}/$BIN_NAME"
  FOUND=""
  if [ -x "$TARGET" ]; then
    FOUND="$TARGET"
  else
    FOUND=$(command -v "$BIN_NAME" || true)
  fi

  if [ -z "$FOUND" ]; then
    # Try common locations
    for d in "/usr/local/bin" "$HOME/.local/bin"; do
      if [ -x "$d/$BIN_NAME" ]; then FOUND="$d/$BIN_NAME"; break; fi
    done
  fi

  if [ -z "$FOUND" ]; then
    echo "$BIN_NAME not found; nothing to uninstall"; return 0
  fi

  DIR=$(dirname "$FOUND")
  if [ -w "$DIR" ]; then
    rm -f "$FOUND"
  else
    if command_exists sudo; then
      $SUDO_BIN rm -f "$FOUND"
    else
      echo "Insufficient permissions to remove $FOUND and sudo not available" >&2
      exit 1
    fi
  fi
  echo "Removed $FOUND"
}

do_purge() {
  do_uninstall || true
  # Remove data dirs
  BGIT_DIR="$HOME/.bgit"
  AGENT_SOCK="$HOME/.ssh/bgit_ssh_agent.sock"
  [ -d "$BGIT_DIR" ] && rm -rf "$BGIT_DIR" && echo "Removed $BGIT_DIR" || true
  [ -S "$AGENT_SOCK" ] || [ -f "$AGENT_SOCK" ] && rm -f "$AGENT_SOCK" && echo "Removed $AGENT_SOCK" || true
}

case "$CMD" in
  install|update) do_install ;;
  uninstall) do_uninstall ;;
  purge) do_purge ;;
esac
