#!/usr/bin/env bash
set -euo pipefail

MODE="${1:---dry-run}"
case "$MODE" in
  --dry-run|--publish) ;;
  *)
    echo "usage: $0 [--dry-run|--publish]" >&2
    exit 2
    ;;
esac

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"

if [ "$MODE" = "--publish" ]; then
  bash scripts/ensure-npm-auth.sh claude-e
fi

VERSION="$(node -p "require('./package.json').version")"
node scripts/sync-cargo-version.mjs "$VERSION"
cargo update -p claude-exec --precise "$VERSION"
cargo fmt --check
cargo test --locked
cargo build --release --locked
npm pack --dry-run
npm run publish:dry-run

if [ "$MODE" = "--publish" ]; then
  npm publish --access public --ignore-scripts
else
  echo "[claude-e:release] dry-run complete; use npm run release:patch for a new npm release, or release:npm to publish the current package version."
fi
