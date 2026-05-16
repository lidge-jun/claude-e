# CLI Surface

## Binary Names

`claude-e` is the primary public npm command and package name.

`claude-exec` is the long descriptive compatibility alias for the same binary behavior.

Two compatibility names are intentionally built from the same library entrypoint:

- `claude-i`: short transitional alias used by cli-jaw's provider id today.
- `jaw-claude-i`: legacy helper name used by existing cli-jaw detection and scripts.

## Default `claude -p`-Style PTY Mode

Without a `run` or `exec` subcommand, `claude-e` and `claude-exec` expose a
`claude -p`-style surface while still running the interactive PTY wrapper:

```text
claude-e [claude -p style args] <prompt>
claude-e p [claude -p style args] <prompt>
claude-exec [claude -p style args] <prompt>
```

The wrapper parses prompt arguments and piped stdin, suppresses internal
`jaw_runtime` events from stdout, and maps `--output-format` to transcript
normalization. Claude runtime flags such as `--model` are forwarded into the PTY
Claude process. Permission bypass and workspace/folder trust acceptance are
enabled by default for unattended execution.

Examples:

```bash
claude-e "your prompt here"
claude-e p "your prompt here"
claude-exec "your prompt here"

claude-e --output-format json "summarize this commit" < commit.diff
claude-e --output-format stream-json "audit src/" --verbose | jq .
claude-e --model opus "explain quicksort to a 10-year-old"
claude-e --tool "use 10 tools and summarize the results"
claude-e --resume 6a304357-e92d-47e7-a56d-c54065e12be1 "continue"
claude-e p --input-format stream-json --output-format json < messages.jsonl
```

Claude binary resolution for print-compatible PTY mode:

1. `CLAUDE_EXEC_CLAUDE_BIN`
2. `CLAUDE_BIN`
3. `claude`

`p` and `print` are accepted as optional leading aliases for this same default
mode, so `claude-e p "prompt"` is equivalent to `claude-e "prompt"`.
`-p` and `--print` are also accepted as no-op compatibility flags.

### Print-Compatible Flag Coverage

Wrapper-owned flags:

| Flag | Behavior |
|---|---|
| `--input-format text|stream-json` | Reads plain stdin or extracts user text from JSONL messages. |
| `--output-format text|json|stream-json` | Normalizes transcript output to the requested print-style shape. |
| `--timeout-ms`, `--claude-bin`, `--cwd`, `--cols`, `--rows` | PTY wrapper controls. |
| `--session-id` | Uses the provided session id for the generated PTY session. |
| `--no-session-persistence` | Suppresses generated session id; consumed by the wrapper and not forwarded to interactive Claude. |
| `--resume` / `-r` | Resumes the provided session id in the PTY path. |
| `--json-schema` | Appends a JSON-only schema instruction to the prompt. |
| `--auto-accept-workspace-trust`, `--no-auto-accept-workspace-trust` | Controls pre-SessionStart workspace/folder trust prompt handling. Default is enabled. |
| `--tool`, `--t`, `-t` | Prints compact tool-use and tool-result progress to stderr. |
| `--no-session-footer` | Hides the final stderr resume footer in print-compatible mode. |

Accepted print-only compatibility flags:

| Flag | Behavior |
|---|---|
| `--verbose`, `--include-partial-messages`, `--include-hook-events`, `--replay-user-messages` | Accepted and consumed; transcript replay owns PTY output timing. |
| `--fallback-model`, `--max-budget-usd` | Accepted and consumed because the PTY path cannot enforce Claude print-mode fallback or budget policy. |

Forwarded Claude runtime flags include `--model`, `--effort`,
`--permission-mode`, `--add-dir`, `--allowed-tools`, `--tools`,
`--mcp-config`, `--settings`, `--system-prompt`, `--append-system-prompt`,
`--plugin-dir`, `--plugin-url`, browser flags, MCP debug flags, and related
Claude global controls from `claude --help`.

Recognized value flags support `--flag=value`. Variadic Claude flags consume one
value per occurrence; repeat the flag or insert `--` before prompt text when the
flag/prompt boundary is ambiguous.

If the caller does not supply `--permission-mode`,
`--permission-mode=...`, `--dangerously-skip-permissions`, or
`--allow-dangerously-skip-permissions`, the wrapper appends
`--dangerously-skip-permissions` before spawning Claude. This is the default
automation policy for both print-compatible and runtime modes.

Default text-mode runs print the Claude session id and a ready-to-copy
`claude-e --resume <session-id> ...` command to stderr. The footer is not part
of stdout, so programmatic output stays parseable.

## PTY Runtime Command Form

Primary form:

```bash
claude-exec run [wrapper flags] -- [claude args]
claude-e run [wrapper flags] -- [claude args]
```

Semantic alias:

```bash
claude-exec exec [wrapper flags] -- [claude args]
claude-e exec [wrapper flags] -- [claude args]
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
| `--auto-accept-workspace-trust` | `true` | Watch the interactive PTY before `SessionStart` and accept Claude's workspace/folder trust prompt when detected. |
| `--tool`, `--t`, `-t` | `false` | Show compact terminal tool progress on stderr. |

## Forwarded Claude Args

Everything after `--` is passed directly to Claude after wrapper-managed session/settings args.

Example:

```bash
printf 'Summarize this repo in two bullets.\n' \
  | claude-exec run \
      --claude-bin /Users/jun/.local/bin/claude \
      -- \
      --model claude-opus-4-6
```

## Naming Position

The intended mental model is:

```text
codex exec   -> non-interactive Codex execution surface
claude-e     -> short non-interactive Claude execution surface backed by interactive Claude Code
claude-exec  -> long compatibility alias for the same runtime
```

## npm Packaging

The `claude-e` npm package exposes these bins from the same Rust entrypoint:

- `claude-e`
- `claude-exec`
- `claude-i`
- `jaw-claude-i`

`npm install -g claude-e` runs `scripts/postinstall.cjs`, which builds the
release binary with Cargo and performs a one-time GitHub star prompt. When npm
is non-interactive, postinstall prints the repository URL instead of blocking
the install. Set `CLAUDE_E_SKIP_STAR_PROMPT=1` to suppress only that prompt, or
`CLAUDE_E_SKIP_POSTINSTALL=1` / `CLAUDE_EXEC_SKIP_POSTINSTALL=1` to skip all
postinstall work. The bin wrappers resolve npm/npx symlinks to the real package
root, execute `target/release/claude-exec` when it exists, and fall back to
`cargo run` only for source checkouts. Set `CLAUDE_E_SKIP_BUILD=1` or
`CLAUDE_EXEC_SKIP_BUILD=1` only when another packaging layer provides the binary.

Release helpers:

```bash
npm run verify
npm run pack:dry
npm run publish:dry-run
npm run release:check
npm run release:npm
npm run release:patch
npm run release:minor
npm run release:major
npm run release:preview
```

`release:npm` publishes the current package version. The semver release helpers
mirror the `ima2-gen` shape: they refuse dirty tracked worktrees, sync from npm
latest when needed, bump the requested version, run verification plus
`npm publish --dry-run`, commit `package.json`, create a `vX.Y.Z` tag, publish
the tarball to npm, push branch/tag, and create a GitHub release when `gh` is
available. `release:preview` publishes with the `preview` npm dist-tag and marks
the GitHub release as a prerelease.

GitHub Actions must not publish to npm. CI only verifies and dry-runs package
contents; the real npm publish step is intentionally local-only after interactive
npm authentication.
