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
RELEASES_URL="https://github.com/${REPO}/releases"

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

    # GitHub returns the resolved latest-release URL; extract the tag from it.
    LATEST_URL=$(curl -sSIo /dev/null -w '%{url_effective}' "${RELEASES_URL}/latest" 2>/dev/null)
    TAG="${LATEST_URL##*/}"          # e.g. v1.2.0-cli
    VERSION="${TAG#v}"               # strip leading v
    VERSION="${VERSION%-cli}"        # strip trailing -cli

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
    DOWNLOAD_URL="${RELEASES_URL}/download/${TAG}/${BIN_NAME}-${TARGET}"

    say "Installing ${BIN_NAME} ${VERSION} (${TARGET})"
    say "  → ${INSTALL_DIR}/${BIN_NAME}"

    # Create install dir if necessary
    mkdir -p "$INSTALL_DIR"

    TMP="$(mktemp)"
    # shellcheck disable=SC2064
    trap "rm -f '$TMP'" EXIT

    say "Downloading…"
    curl -sSL --fail "$DOWNLOAD_URL" -o "$TMP" \
        || err "download failed — check that version ${VERSION} has a release for ${TARGET}:\n  ${DOWNLOAD_URL}"

    chmod +x "$TMP"
    mv "$TMP" "${INSTALL_DIR}/${BIN_NAME}"

    ok "${BIN_NAME} ${VERSION} installed to ${INSTALL_DIR}/${BIN_NAME}"

    # Warn if the install dir is not in PATH
    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*) ;;
        *) printf '\033[33mwarning: %s is not in your PATH.\033[0m\n  Add this to your shell profile:\n    export PATH="%s:$PATH"\n' "$INSTALL_DIR" "$INSTALL_DIR" ;;
    esac

    say "Run '${BIN_NAME} --help' to get started."
}

main "$@"
