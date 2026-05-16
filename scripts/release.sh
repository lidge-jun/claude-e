#!/usr/bin/env bash
# release.sh - verify, version, publish to npm, and create a GitHub release.
# Usage:
#   ./scripts/release.sh          # patch bump
#   ./scripts/release.sh minor    # minor bump
#   ./scripts/release.sh major    # major bump
#   ./scripts/release.sh 1.2.0    # explicit version
set -euo pipefail

PKG_NAME="claude-e"

cd "$(dirname "$0")/.."

if ! git diff --cached --quiet; then
  echo "Refusing release: staged changes exist" >&2
  exit 1
fi
if ! git diff --quiet; then
  echo "Refusing release: worktree has uncommitted changes" >&2
  exit 1
fi

NPM_LATEST="$(npm view "$PKG_NAME" dist-tags.latest 2>/dev/null || true)"
PKG_VERSION="$(node -p "require('./package.json').version")"

echo "$PKG_NAME release"
echo "npm latest:   ${NPM_LATEST:-'(not found)'}"
echo "package.json: $PKG_VERSION"

if [ -n "$NPM_LATEST" ]; then
  CLEAN_NPM="${NPM_LATEST%%-*}"
  CLEAN_PKG="${PKG_VERSION%%-*}"
  if [ "$CLEAN_PKG" != "$CLEAN_NPM" ]; then
    echo "Syncing package.json from $CLEAN_PKG to npm latest $CLEAN_NPM before bump."
    npm version "$CLEAN_NPM" --no-git-tag-version --allow-same-version
  fi
fi

BUMP_ARG="${1:-patch}"
if [[ "$BUMP_ARG" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  npm version "$BUMP_ARG" --no-git-tag-version
else
  npm version "$BUMP_ARG" --no-git-tag-version
fi

VERSION="$(node -p "require('./package.json').version")"
node scripts/sync-cargo-version.mjs "$VERSION"
echo "New version: $VERSION"

PREV_TAG="$(git tag --sort=-v:refname | grep -E '^v[0-9]' | head -1 || true)"
if [ -n "$PREV_TAG" ]; then
  CHANGELOG="$(git log "$PREV_TAG"..HEAD --pretty=format:"- %s" --no-merges | head -50)"
  COMMIT_COUNT="$(git rev-list "$PREV_TAG"..HEAD --count)"
else
  CHANGELOG="$(git log --oneline -20 --pretty=format:"- %s" --no-merges)"
  COMMIT_COUNT="?"
fi

echo
echo "Changes since ${PREV_TAG:-'(none)'} ($COMMIT_COUNT commits):"
echo "$CHANGELOG" | head -15
echo

npm run verify
npm run publish:dry-run

git add Cargo.toml package.json
[ -f package-lock.json ] && git add package-lock.json
[ -f npm-shrinkwrap.json ] && git add npm-shrinkwrap.json
git commit -m "[agent] chore: release v$VERSION" --allow-empty
git tag "v$VERSION"

TARBALL="$(npm pack --silent | tail -n 1)"
trap 'rm -f "$TARBALL"' EXIT

npm publish "$TARBALL" --access public

git push origin HEAD
git push origin "v$VERSION"

RELEASE_BODY="## Release v$VERSION

Previous: ${PREV_TAG:-'(first release)'}
Commits: $COMMIT_COUNT

### Changes
$CHANGELOG"

if command -v gh >/dev/null 2>&1; then
  gh release create "v$VERSION" \
    --title "v$VERSION" \
    --notes "$RELEASE_BODY" \
    --latest
else
  echo "Skipped GitHub Release: gh CLI not found."
fi

echo "$PKG_NAME@$VERSION published."
echo "Install: npm install -g $PKG_NAME"
