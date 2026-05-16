#!/usr/bin/env sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
CLAUDE_BIN=${CLAUDE_BIN:-$(command -v claude)}
CLAUDE_EXEC_MODEL=${CLAUDE_EXEC_MODEL:-claude-opus-4-6}
CLAUDE_EXEC_PROMPT=${CLAUDE_EXEC_PROMPT:-Say hello in one short sentence.}

printf '%s\n' "$CLAUDE_EXEC_PROMPT" \
  | cargo run --quiet --manifest-path "$ROOT/Cargo.toml" --bin claude-exec -- run \
      --jsonl \
      --output-format stream-json \
      --timeout-ms 600000 \
      --claude-bin "$CLAUDE_BIN" \
      -- \
      --model "$CLAUDE_EXEC_MODEL" \
      --dangerously-skip-permissions
