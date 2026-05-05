#!/usr/bin/env bash
set -euo pipefail

# Usage: ./helpers/scripts/publish.sh <version>
# Example: ./helpers/scripts/publish.sh 1.2.0

VERSION="${1:-}"

if [[ -z "$VERSION" ]]; then
  echo "Usage: $0 <version>  (e.g. 1.2.0)"
  exit 1
fi

TAG="v${VERSION}"

# Ensure working tree is clean
if [[ -n "$(git status --porcelain)" ]]; then
  echo "Error: working tree is dirty. Commit or stash your changes first."
  exit 1
fi

# Ensure we are on main
BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "$BRANCH" != "main" ]]; then
  echo "Error: must be on the 'main' branch (currently on '$BRANCH')."
  exit 1
fi

# Check git-cliff is available
if ! command -v git-cliff &>/dev/null; then
  echo "Error: git-cliff is not installed. Install it with: cargo install git-cliff"
  exit 1
fi

echo "==> Generating CHANGELOG.md up to $TAG..."
git-cliff --tag "$TAG" --output CHANGELOG.md

echo "==> Staging CHANGELOG.md..."
git add CHANGELOG.md
git commit -m "chore(release): $TAG"

echo "==> Creating annotated tag $TAG..."
git tag -a "$TAG" -m "Release $TAG"

echo "==> Pushing commits and tag..."
git push origin main
git push origin "$TAG"

echo ""
echo "Released $TAG. The Docker image will be built and pushed by CI."
