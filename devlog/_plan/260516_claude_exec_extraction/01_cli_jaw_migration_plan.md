---
created: 2026-05-16
status: planning
tags: [claude-exec, cli-jaw, migration]
---
# cli-jaw Naming Migration Plan

## Short Answer

The `claude-i` surface is not enormous, but it is cross-cutting. The safe path is alias-first:

```text
claude-i provider -> claude-exec provider
jaw-claude-i binary -> claude-exec binary
JAW_CLAUDE_I_BIN env -> CLAUDE_EXEC_BIN env
agent:claude-i:* events -> agent:claude-exec:* events
```

Each arrow should be additive before it is destructive.

## Phase Order

1. Keep cli-jaw provider id `claude-i` for now.
2. Teach cli-jaw detection to prefer `CLAUDE_EXEC_BIN`, embedded npm `claude-exec`, and PATH `claude-exec`.
3. Keep `JAW_CLAUDE_I_BIN`, `jaw-claude-i`, and `claude-i` as compatibility fallbacks.
4. Add provider id `claude-exec` only after detection and doctor are stable.
5. Migrate saved settings from `claude-i` to `claude-exec`.
6. Broadcast `agent:claude-exec:*` while still broadcasting deprecated `agent:claude-i:*`.
7. Update docs and tests.
8. Remove embedded `native/jaw-claude-i` after external install/publish is reliable.

## First cli-jaw Fix

Before broad naming work, land the small functional fix:

- Resolve the underlying `claude` binary in cli-jaw.
- Pass it to the wrapper as `--claude-bin <path>`.

That avoids PATH drift inside long-running servers and addresses the observed immediate helper failure.
