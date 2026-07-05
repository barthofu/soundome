#!/usr/bin/env bash
set -euo pipefail

# ─────────────────────────────────────────────────────────────────────────────
# Usage:
#   ./helpers/scripts/publish.sh server <version>   # release the server (Docker image)
#   ./helpers/scripts/publish.sh cli    <version>   # release the CLI (binary artefacts)
#
# What it does:
#   1. Validates inputs and environment.
#   2. Bumps the version in the relevant Cargo.toml (and package.json for server).
#   3. Updates Cargo.lock (cargo update -p <crate>).
#   4. Prints the changelog excerpt for the release (ready to copy-paste).
#   5. Commits the version bump, creates an annotated tag, and pushes.
#
# The CI then takes over:
#   • server tag (v*.*.*) → docker.yaml builds and pushes the Docker image.
#   • cli tag (v*.*.*-cli) → release-cli.yaml cross-compiles and creates a GitHub Release.
# ─────────────────────────────────────────────────────────────────────────────

# ── helpers ──────────────────────────────────────────────────────────────────

say()  { echo "==> $*"; }
ok()   { echo "    ✔  $*"; }
err()  { echo "error: $*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || err "'$1' is required but not found."; }

# In-place sed that works on both Linux (GNU) and macOS (BSD).
sed_inplace() {
    local pattern="$1" file="$2"
    if sed --version 2>/dev/null | grep -q GNU; then
        sed -i "$pattern" "$file"
    else
        sed -i '' "$pattern" "$file"
    fi
}

# ── argument parsing ──────────────────────────────────────────────────────────

TARGET="${1:-}"
VERSION="${2:-}"

if [[ -z "$TARGET" || -z "$VERSION" ]]; then
    echo "Usage: $0 <server|cli> <version>  (e.g. $0 server 1.2.0 or 1.2.0-rc1)"
    exit 1
fi

if [[ "$TARGET" != "server" && "$TARGET" != "cli" ]]; then
    err "first argument must be 'server' or 'cli', got: '$TARGET'"
fi

# Validate semver (x.y.z with optional prerelease/build metadata)
SEMVER_REGEX='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-([0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*))?(\+([0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*))?$'
if ! [[ "$VERSION" =~ $SEMVER_REGEX ]]; then
    err "version must be semver (e.g. 1.2.0 or 1.2.0-rc1), got: '$VERSION'"
fi

# ── environment checks ────────────────────────────────────────────────────────

need git
need git-cliff
need cargo

# Ensure working tree is clean
if [[ -n "$(git status --porcelain)" ]]; then
    err "working tree is dirty. Commit or stash your changes first."
fi

# Ensure we are on main
BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$BRANCH" != "main" ]]; then
    err "must be on the 'main' branch (currently on '$BRANCH')."
fi

# ── run tests ─────────────────────────────────────────────────────────────────

say "Running tests…"
if ! cargo test -q --workspace --exclude openrouter_api; then
    err "tests failed. Fix the failures and try again."
fi
ok "All tests passed"

# ── resolve paths and tag ─────────────────────────────────────────────────────

REPO_ROOT="$(git rev-parse --show-toplevel)"

case "$TARGET" in
    server)
        TAG="v${VERSION}"
        CARGO_TOML="${REPO_ROOT}/apps/server/Cargo.toml"
        ROOT_PKG_JSON="${REPO_ROOT}/package.json"
        WEB_PKG_JSON="${REPO_ROOT}/apps/web/package.json"
        CRATE_NAME="soundome-server"
        ;;
    cli)
        TAG="v${VERSION}-cli"
        CARGO_TOML="${REPO_ROOT}/apps/cli/Cargo.toml"
        CRATE_NAME="cli"
        ;;
esac

# ── bump versions ─────────────────────────────────────────────────────────────

say "Bumping ${TARGET} to ${VERSION}…"

# Cargo.toml: replace `version = "x.y.z"` on the first occurrence (the [package] block)
sed_inplace "0,/^version = \".*\"/{s/^version = \".*\"/version = \"${VERSION}\"/}" "$CARGO_TOML"
ok "Updated ${CARGO_TOML#"$REPO_ROOT/"}"

if [[ "$TARGET" == "server" ]]; then
    # Root package.json
    sed_inplace "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" "$ROOT_PKG_JSON"
    ok "Updated ${ROOT_PKG_JSON#"$REPO_ROOT/"}"

    # apps/web/package.json
    sed_inplace "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" "$WEB_PKG_JSON"
    ok "Updated ${WEB_PKG_JSON#"$REPO_ROOT/"}"
fi

# Refresh Cargo.lock for the bumped crate
say "Updating Cargo.lock…"
cargo update -p "$CRATE_NAME" --manifest-path "${REPO_ROOT}/Cargo.toml" 2>/dev/null || true
ok "Cargo.lock refreshed"

# ── generate and display changelog ───────────────────────────────────────────

say "Generating changelog excerpt for ${TAG}…"

# Find the previous tag of the same kind to scope the diff
if [[ "$TARGET" == "cli" ]]; then
    PREV_TAG=$(git tag --list "v*-cli" --sort=-version:refname | head -n1 || true)
else
    # server tags: plain semver tags (exclude -cli tags)
    PREV_TAG=$(git tag --list "v*.*.*" --sort=-version:refname | grep -v "\-cli" | head -n1 || true)
fi

echo ""
echo "────────────────────────────────────────────────────────────────────"
echo "  CHANGELOG — copy-paste this into the GitHub Release description"
echo "────────────────────────────────────────────────────────────────────"

if [[ -n "$PREV_TAG" ]]; then
    git-cliff "${PREV_TAG}..HEAD" --tag "$TAG" --strip all
else
    git-cliff --tag "$TAG" --strip all
fi

echo "────────────────────────────────────────────────────────────────────"
echo ""

# ── commit, tag, push ─────────────────────────────────────────────────────────

say "Formatting code…"
cargo fmt --all
ok "Code formatted"

say "Staging version bump files…"
git add "$CARGO_TOML" "${REPO_ROOT}/Cargo.lock"
if [[ "$TARGET" == "server" ]]; then
    git add "$ROOT_PKG_JSON" "$WEB_PKG_JSON"
fi

git commit -m "chore(release): ${TARGET} ${TAG}"
ok "Committed version bump"

say "Creating annotated tag ${TAG}…"
git tag -a "$TAG" -m "Release ${TAG}"
ok "Tag created"

say "Pushing to origin…"
git push origin main
git push origin "$TAG"
ok "Pushed"

echo ""
echo "Released ${TAG}."
case "$TARGET" in
    server) echo "The Docker image will be built and pushed by CI (docker.yaml)." ;;
    cli)    echo "The CLI binaries will be built and a GitHub Release created by CI (release-cli.yaml)." ;;
esac
