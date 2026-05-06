#!/usr/bin/env bash
# Update all git submodules to the latest upstream commit on their tracked branch.
set -euo pipefail

REPO_ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$REPO_ROOT"

echo "Fetching submodule remotes..."
git submodule foreach --recursive 'git fetch origin'

echo "Updating submodules to latest upstream..."
git submodule update --remote --merge

echo "Submodule status after update:"
git submodule status

echo ""
echo "Staging updated submodule references..."
git add .gitmodules
git submodule foreach --quiet 'echo "$displaypath"' | xargs git add 2>/dev/null || true

echo ""
echo "Done. Review the changes and commit:"
echo "  git commit -m 'chore: update submodules to latest upstream'"
