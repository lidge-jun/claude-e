# CLI Surface

## Binary Names

`claude-exec` is the primary binary.

Two compatibility names are intentionally built from the same library entrypoint:

- `claude-i`: short transitional alias used by cli-jaw's provider id today.
- `jaw-claude-i`: legacy helper name used by existing cli-jaw detection and scripts.

## Command Form

Primary form:

```bash
claude-exec run [wrapper flags] -- [claude args]
```

Semantic alias:

```bash
claude-exec exec [wrapper flags] -- [claude args]
```

`run` remains the stable compatibility form because cli-jaw currently emits `jaw-claude-i run ...`.

## Wrapper Flags

| Flag | Default | Meaning |
|---|---:|---|
| `--jsonl` | `true` | Emit JSONL to stdout. Kept for compatibility. |
| `--output-format` | `stream-json` | Output normalization mode. |
| `--timeout-ms` | `600000` | Max runtime before timeout exit. |
| `--claude-bin` | `claude` | Claude CLI binary or absolute path. cli-jaw should pass an explicit resolved path. |
| `--cwd` | current dir | Working directory for Claude. |
| `--cols` | `120` | PTY columns. |
| `--rows` | `40` | PTY rows. |
| `--resume` | unset | Resume persisted Claude session. |
| `--auto-accept-workspace-trust` | `false` | Watch the interactive PTY before `SessionStart` and accept Claude's workspace trust prompt when detected. |

## Forwarded Claude Args

Everything after `--` is passed directly to Claude after wrapper-managed session/settings args.

Example:

```bash
printf 'Summarize this repo in two bullets.\n' \
  | claude-exec run \
      --claude-bin /Users/jun/.local/bin/claude \
      -- \
      --model claude-opus-4-6 \
      --dangerously-skip-permissions
```

## Naming Position

The intended mental model is:

```text
codex exec   -> non-interactive Codex execution surface
claude-exec  -> non-interactive Claude execution surface backed by interactive Claude Code
```
