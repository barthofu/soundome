#!/usr/bin/env sh
# install.sh — one-liner installer for the Soundome CLI
#
# Usage:
#   curl -sSL https://raw.githubusercontent.com/barthofu/soundome/main/helpers/scripts/install.sh | sh
#
# Options (env vars):
#   SOUNDOME_VERSION  — specific version to install (default: latest)
#   INSTALL_DIR       — directory to install into (default: ~/.local/bin, or /usr/local/bin if root)
#
set -eu

REPO="barthofu/soundome"
BIN_NAME="soundome"
RELEASE_BIN_NAME="soundome-cli"
RELEASES_URL="https://github.com/${REPO}/releases"
GITHUB_API_URL="https://api.github.com/repos/${REPO}"

# ── helpers ──────────────────────────────────────────────────────────────────

say()  { printf '\033[1m%s\033[0m\n' "$*"; }
ok()   { printf '\033[32m✔ %s\033[0m\n' "$*"; }
err()  { printf '\033[31merror: %s\033[0m\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || err "required tool not found: $1"; }

# ── detect OS and architecture ───────────────────────────────────────────────

detect_target() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)
            case "$ARCH" in
                x86_64)  echo "x86_64-unknown-linux-musl" ;;
                aarch64 | arm64) echo "aarch64-unknown-linux-musl" ;;
                *) err "unsupported Linux architecture: $ARCH" ;;
            esac
            ;;
        Darwin)
            case "$ARCH" in
                x86_64)  echo "x86_64-apple-darwin" ;;
                arm64)   echo "aarch64-apple-darwin" ;;
                *) err "unsupported macOS architecture: $ARCH" ;;
            esac
            ;;
        *)
            err "unsupported operating system: $OS (Windows is not supported by this script)"
            ;;
    esac
}

# ── resolve version ───────────────────────────────────────────────────────────

resolve_version() {
    if [ -n "${SOUNDOME_VERSION:-}" ]; then
        echo "$SOUNDOME_VERSION"
        return
    fi

    need curl
    need python3

    RELEASES_JSON=$(curl -fsSL \
        -H 'Accept: application/vnd.github+json' \
        -H 'User-Agent: soundome-installer' \
        "${GITHUB_API_URL}/releases?per_page=100") \
        || err "could not retrieve releases from GitHub"

    # Releases are returned newest-first; ignore server releases and prereleases.
    VERSION=$(printf '%s' "$RELEASES_JSON" | python3 -c '
import json
import re
import sys

releases = json.load(sys.stdin)
for release in releases:
    match = re.fullmatch(r"v(.+)-cli", release.get("tag_name", ""))
    if match and not release.get("draft") and not release.get("prerelease"):
        print(match.group(1))
        break
' 2>/dev/null) || err "could not parse CLI releases from GitHub"

    [ -n "$VERSION" ] || err "could not resolve latest version from GitHub"
    echo "$VERSION"
}

# ── determine install directory ───────────────────────────────────────────────

resolve_install_dir() {
    if [ -n "${INSTALL_DIR:-}" ]; then
        echo "$INSTALL_DIR"
    elif [ "$(id -u)" -eq 0 ]; then
        echo "/usr/local/bin"
    else
        echo "${HOME}/.local/bin"
    fi
}

# ── download and install ──────────────────────────────────────────────────────

main() {
    need curl

    TARGET="$(detect_target)"
    VERSION="$(resolve_version)"
    INSTALL_DIR="$(resolve_install_dir)"

    TAG="v${VERSION}-cli"
    ASSET="${RELEASE_BIN_NAME}-${TARGET}"
    DOWNLOAD_URL="${RELEASES_URL}/download/${TAG}/${ASSET}"
    CHECKSUM_URL="${RELEASES_URL}/download/${TAG}/checksums.txt"

    say "Installing ${BIN_NAME} ${VERSION} (${TARGET})"
    say "  → ${INSTALL_DIR}/${BIN_NAME}"

    # Create install dir if necessary
    mkdir -p "$INSTALL_DIR"

    TMP_DIR="$(mktemp -d)" || err "could not create a temporary directory"
    trap 'rm -rf "$TMP_DIR"' EXIT
    trap 'exit 1' HUP INT TERM
    BINARY_TMP="${TMP_DIR}/${ASSET}"
    CHECKSUMS_TMP="${TMP_DIR}/checksums.txt"
    EXPECTED_CHECKSUM="${TMP_DIR}/expected-checksum.txt"

    say "Downloading binary and checksums…"
    curl -fsSL "$DOWNLOAD_URL" -o "$BINARY_TMP" \
        || err "download failed for ${TAG} (${TARGET}): ${DOWNLOAD_URL}"
    curl -fsSL "$CHECKSUM_URL" -o "$CHECKSUMS_TMP" \
        || err "could not download release checksums: ${CHECKSUM_URL}"

    awk -v asset="$ASSET" '$2 == asset { print; count++ } END { if (count != 1) exit 1 }' \
        "$CHECKSUMS_TMP" > "$EXPECTED_CHECKSUM" \
        || err "checksums do not contain exactly one entry for ${ASSET}"

    if command -v sha256sum >/dev/null 2>&1; then
        (cd "$TMP_DIR" && sha256sum --check "$EXPECTED_CHECKSUM" >/dev/null) \
            || err "SHA-256 checksum verification failed for ${ASSET}"
    elif command -v shasum >/dev/null 2>&1; then
        (cd "$TMP_DIR" && shasum -a 256 --check "$EXPECTED_CHECKSUM" >/dev/null) \
            || err "SHA-256 checksum verification failed for ${ASSET}"
    else
        err "required checksum tool not found (need sha256sum or shasum)"
    fi
    ok "SHA-256 checksum verified"

    chmod +x "$BINARY_TMP"
    mv "$BINARY_TMP" "${INSTALL_DIR}/${BIN_NAME}"

    ok "${BIN_NAME} ${VERSION} installed to ${INSTALL_DIR}/${BIN_NAME}"

    # Warn if the install dir is not in PATH
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *) printf '\033[33mwarning: %s is not in your PATH.\033[0m\n  Add this to your shell profile:\n    export PATH="%s:$PATH"\n' "$INSTALL_DIR" "$INSTALL_DIR" ;;
    esac

    say "Run '${BIN_NAME} --help' to get started."
}

main "$@"
