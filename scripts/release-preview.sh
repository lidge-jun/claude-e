#!/usr/bin/env bash
# release-preview.sh - publish a preview build to npm with the preview dist-tag.
# Usage:
#   ./scripts/release-preview.sh          # next patch preview
#   ./scripts/release-preview.sh 1.2.0    # explicit preview base version
set -euo pipefail

PKG_NAME="claude-e"

cd "$(dirname "$0")/.."

if ! git diff --cached --quiet; then
  echo "Refusing preview release: staged changes exist" >&2
  exit 1
fi
if ! git diff --quiet; then
  echo "Refusing preview release: worktree has uncommitted changes" >&2
  exit 1
fi

NPM_LATEST="$(npm view "$PKG_NAME" dist-tags.latest 2>/dev/null || true)"
PKG_VERSION="$(node -p "require('./package.json').version")"
RAW_VERSION="${NPM_LATEST:-$PKG_VERSION}"
RAW_VERSION="${RAW_VERSION%%-*}"

IFS='.' read -r MAJOR MINOR PATCH <<< "$RAW_VERSION"
BASE_VERSION="${MAJOR}.${MINOR}.$((PATCH + 1))"

if [ "${1:-}" != "" ]; then
  BASE_VERSION="$1"
fi

if [[ ! "$BASE_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "BASE_VERSION must look like 1.2.3; got: $BASE_VERSION" >&2
  exit 2
fi

PREID="${PREID:-preview}"
STAMP="${STAMP:-$(date +%Y%m%d%H%M%S)}"
PREVIEW_VERSION="${BASE_VERSION}-${PREID}.${STAMP}"

echo "$PKG_NAME preview release"
echo "npm latest:      ${NPM_LATEST:-'(not found)'}"
echo "package.json:    $PKG_VERSION"
echo "Preview version: $PREVIEW_VERSION"
echo "Dist-tag:        preview"

PREV_TAG="$(git tag --sort=-v:refname | grep -E '^v[0-9]' | head -1 || true)"
if [ -n "$PREV_TAG" ]; then
  CHANGELOG="$(git log "$PREV_TAG"..HEAD --pretty=format:"- %s" --no-merges | head -30)"
  COMMIT_COUNT="$(git rev-list "$PREV_TAG"..HEAD --count)"
else
  CHANGELOG="$(git log --oneline -10 --pretty=format:"- %s" --no-merges)"
  COMMIT_COUNT="?"
fi

echo
echo "Changes since ${PREV_TAG:-'(none)'} ($COMMIT_COUNT commits):"
echo "$CHANGELOG" | head -10
echo

npm version "$PREVIEW_VERSION" --no-git-tag-version
VERSION="$(node -p "require('./package.json').version")"
node scripts/sync-cargo-version.mjs "$VERSION"
npm run verify
npm run publish:dry-run

git add Cargo.toml package.json
[ -f package-lock.json ] && git add package-lock.json
[ -f npm-shrinkwrap.json ] && git add npm-shrinkwrap.json
git commit -m "[agent] chore: preview v$VERSION" --allow-empty
git tag "v$VERSION"

TARBALL="$(npm pack --silent | tail -n 1)"
trap 'rm -f "$TARBALL"' EXIT

npm publish "$TARBALL" --tag preview --access public

git push origin HEAD
git push origin "v$VERSION"

RELEASE_BODY="## Preview Release v$VERSION

Base: $RAW_VERSION -> $BASE_VERSION
Commits since ${PREV_TAG:-'(none)'}: $COMMIT_COUNT

### Changes
$CHANGELOG"

if command -v gh >/dev/null 2>&1; then
  gh release create "v$VERSION" \
    --title "v$VERSION (preview)" \
    --notes "$RELEASE_BODY" \
    --prerelease
else
  echo "Skipped GitHub prerelease: gh CLI not found."
fi

echo "Preview published: $PKG_NAME@$VERSION"
echo "Install: npm install -g $PKG_NAME@preview"
