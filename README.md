# claude-exec

`claude-exec` is a non-interactive execution wrapper for Claude Code.

It has two surfaces:

- `claude-exec ...` / `claude-e ...`: `claude -p`-style command surface backed by the interactive PTY runtime. Prompt arguments, piped stdin, `p` / `print` aliases, print-only formatting flags, session controls, and common Claude runtime flags are accepted without requiring the explicit `run` subcommand.
- `claude-exec run ...`: interactive PTY runtime for agent systems. It provides stdin prompt input, JSONL runtime events, transcript replay, timeout handling, resume support, and explicit Claude binary resolution.

The code was extracted from cli-jaw's native `jaw-claude-i` helper. The primary public name is now `claude-exec`; `claude-i` and `jaw-claude-i` remain compatibility binary aliases.

## Build

```bash
cargo build --release
```

This builds four binaries from the same source:

```text
target/release/claude-exec
target/release/claude-e
target/release/claude-i
target/release/jaw-claude-i
```

## Run

Default mode mirrors the `claude -p` command shape while still using the PTY wrapper:

```bash
claude-exec "your prompt here"
claude-e "your prompt here"
claude-e p "your prompt here"

claude-exec --output-format json "summarize this commit" < commit.diff
claude-e --output-format stream-json "audit src/" --verbose | jq .
claude-exec --model opus "explain quicksort to a 10-year-old"
claude-e p --input-format stream-json --output-format json < messages.jsonl
```

The PTY-backed print-compatible Claude binary defaults to `claude`. Override it with
`CLAUDE_EXEC_CLAUDE_BIN=/path/to/claude` or `CLAUDE_BIN=/path/to/claude`.

The PTY wrapper mode remains explicit:

```bash
printf 'Say hello in one short sentence.\n' \
  | cargo run --quiet --bin claude-exec -- run \
      --jsonl \
      --output-format stream-json \
      --timeout-ms 600000 \
      --auto-accept-workspace-trust \
      --claude-bin "$(command -v claude)" \
      -- \
      --model claude-opus-4-6 \
      --dangerously-skip-permissions
```

`run` also has a visible `exec` alias:

```bash
printf 'Say hello.\n' | claude-exec exec --claude-bin "$(command -v claude)" -- --model claude-opus-4-6
```

## Install Locally

```bash
cargo install --path . --locked
```

For development without installing, the `bin/` wrappers run `target/release/claude-exec` when present and otherwise fall back to `cargo run`.

The package also supports npm installation from source. The npm `postinstall`
script builds the Rust release binary with Cargo, and the installed bin wrappers
execute that package-local binary:

```bash
npm install -g claude-exec
npx claude-exec "your prompt here"
npx -p claude-exec claude-e p --model opus "explain quicksort"
```

Set `CLAUDE_EXEC_SKIP_BUILD=1` only when you intentionally provide a built
binary through `CLAUDE_EXEC_BIN` or a separate packaging layer. Without a built
binary, the wrapper requires Cargo at runtime and prints an explicit error if
Cargo is missing.

Packaging scripts:

```bash
npm run verify        # cargo fmt --check + cargo test + cargo build --release
npm run pack:dry      # inspect the npm package contents
npm run release:check # full local release dry run
npm run release:npm   # publish to npm with --access public
```

The GitHub workflows run Rust tests/builds plus npm package dry-runs on push/PR,
and publish to npm from GitHub Releases when `NPM_TOKEN` is configured.

## Contract

Default print-compatible contract:

- Parses `claude -p`-style arguments.
- Builds a prompt from positional prompt text plus piped stdin.
- Runs the existing interactive Claude PTY wrapper with runtime diagnostics suppressed from stdout.
- Accepts `claude-e p ...`, `claude-e print ...`, `claude-e -p ...`, and `claude-e --print ...` as aliases for the same PTY-backed print-compatible mode.
- Intercepts `--input-format text|stream-json`, `--output-format text|json|stream-json`, `--session-id`, `--no-session-persistence`, `--json-schema`, and wrapper-local controls for output and session normalization.
- Accepts print-only flags such as `--verbose`, `--include-partial-messages`, `--include-hook-events`, `--replay-user-messages`, `--fallback-model`, and `--max-budget-usd` for command-shape compatibility. The PTY path consumes flags it cannot faithfully enforce.
- Forwards common Claude runtime flags such as `--model`, `--effort`, `--permission-mode`, `--add-dir`, `--allowed-tools`, `--tools`, `--mcp-config`, `--settings`, `--system-prompt`, `--append-system-prompt`, `--plugin-dir`, and `--plugin-url`.
- Supports `--flag=value` spelling for recognized value flags. Variadic Claude flags consume one value per occurrence; repeat the flag or use `--` before prompt text when the boundary is ambiguous.

PTY `run` input:

- Prompt is read from stdin.
- Empty stdin is rejected.
- Prompt input is capped at 10 MB.

PTY `run` output:

- JSONL is written to stdout.
- Runtime lifecycle records use `{"type":"jaw_runtime", ...}` for cli-jaw compatibility.
- Claude transcript records are normalized into Claude-like stream JSON.
- `--auto-accept-workspace-trust` watches the interactive PTY screen before `SessionStart` and accepts Claude's workspace trust prompt when it appears.

Exit codes:

- `0`: normal completion.
- `2`: graceful interrupt; session can be resumable.
- `4`: Claude spawn or PTY write failure.
- `5`: SessionStart failure or timeout. Timeout errors include a compact PTY screen snapshot when available.
- `6`: run timeout.
- `7`: prompt injection verification failure. The verification accepts a new `user` transcript record or an `assistant` transcript record after the prompt offset, because some Claude builds begin responding before the user record is flushed.
- `11`: Claude StopFailure hook.
- `13`: hook setup failure.
- `16`: prompt read or validation failure.

## Docs

- [structure/INDEX.md](structure/INDEX.md)
- [structure/cli_surface.md](structure/cli_surface.md)
- [structure/runtime_contract.md](structure/runtime_contract.md)
- [structure/cli_jaw_migration.md](structure/cli_jaw_migration.md)
- [devlog/_plan/260516_claude_exec_extraction/00_overview.md](devlog/_plan/260516_claude_exec_extraction/00_overview.md)
