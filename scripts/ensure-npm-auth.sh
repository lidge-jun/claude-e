#!/usr/bin/env bash
set -euo pipefail

PKG_NAME="${1:-package}"

if npm whoami >/dev/null 2>&1; then
  exit 0
fi

echo "[$PKG_NAME:release] npm is not authenticated; starting browser login."
echo "[$PKG_NAME:release] Complete the npm browser flow, then return to this terminal."
npm login --auth-type=web

if ! npm whoami >/dev/null 2>&1; then
  echo "[$PKG_NAME:release] npm login did not complete successfully." >&2
  exit 4
fi
