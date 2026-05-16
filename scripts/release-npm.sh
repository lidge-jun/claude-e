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

cargo fmt --check
cargo test --locked
cargo build --release --locked
npm pack --dry-run

if [ "$MODE" = "--publish" ]; then
  npm publish --access public
else
  echo "[claude-exec:release] dry-run complete; use npm run release:npm to publish."
fi
