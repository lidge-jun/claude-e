# claude-exec

`claude-exec` is a non-interactive execution wrapper for Claude Code. It gives Claude a Codex `exec`-style surface for agent runtimes: stdin prompt input, JSONL runtime events, transcript replay, timeout handling, resume support, and explicit Claude binary resolution.

The code was extracted from cli-jaw's native `jaw-claude-i` helper. The primary public name is now `claude-exec`; `claude-i` and `jaw-claude-i` remain compatibility binary aliases.

## Build

```bash
cargo build --release
```

This builds three binaries from the same source:

```text
target/release/claude-exec
target/release/claude-i
target/release/jaw-claude-i
```

## Run

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

The scaffold also includes `package.json` with npm-style binary wrappers. That makes local linking possible now and leaves room for an npm release flow later. The Rust crate still has `publish = false` until the final registry target and release workflow are chosen.

## Contract

Input:

- Prompt is read from stdin.
- Empty stdin is rejected.
- Prompt input is capped at 10 MB.

Output:

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
- `7`: prompt injection verification failure.
- `11`: Claude StopFailure hook.
- `13`: hook setup failure.
- `16`: prompt read or validation failure.

## Docs

- [structure/INDEX.md](structure/INDEX.md)
- [structure/cli_surface.md](structure/cli_surface.md)
- [structure/runtime_contract.md](structure/runtime_contract.md)
- [structure/cli_jaw_migration.md](structure/cli_jaw_migration.md)
- [devlog/_plan/260516_claude_exec_extraction/00_overview.md](devlog/_plan/260516_claude_exec_extraction/00_overview.md)
