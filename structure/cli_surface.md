# CLI Surface

## Binary Names

`claude-exec` is the primary binary.

`claude-e` is a short public alias for the same binary behavior.

Two compatibility names are intentionally built from the same library entrypoint:

- `claude-i`: short transitional alias used by cli-jaw's provider id today.
- `jaw-claude-i`: legacy helper name used by existing cli-jaw detection and scripts.

## Default `claude -p`-Style PTY Mode

Without a `run` or `exec` subcommand, `claude-exec` and `claude-e` expose a
`claude -p`-style surface while still running the interactive PTY wrapper:

```text
claude-e [claude -p style args] <prompt>
```

The wrapper parses prompt arguments and piped stdin, suppresses internal
`jaw_runtime` events from stdout, and maps `--output-format` to transcript
normalization. Claude runtime flags such as `--model` are forwarded into the PTY
Claude process.

Examples:

```bash
claude-exec "your prompt here"
claude-e "your prompt here"
claude-e p "your prompt here"

claude-exec --output-format json "summarize this commit" < commit.diff
claude-e --output-format stream-json "audit src/" --verbose | jq .
claude-exec --model opus "explain quicksort to a 10-year-old"
```

Claude binary resolution for print-compatible PTY mode:

1. `CLAUDE_EXEC_CLAUDE_BIN`
2. `CLAUDE_BIN`
3. `claude`

`p` and `print` are accepted as optional leading aliases for this same default
mode, so `claude-e p "prompt"` is equivalent to `claude-e "prompt"`.

## PTY Runtime Command Form

Primary form:

```bash
claude-exec run [wrapper flags] -- [claude args]
```

Semantic alias:

```bash
claude-exec exec [wrapper flags] -- [claude args]
```

`run` remains the stable compatibility form because cli-jaw currently emits `jaw-claude-i run ...`.

## PTY Wrapper Flags

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
