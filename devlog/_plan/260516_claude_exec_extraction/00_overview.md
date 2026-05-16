---
created: 2026-05-16
status: planning
tags: [claude-exec, cli-jaw, rust, runtime]
---
# claude-exec Extraction

## Decision

Use `claude-exec` as the standalone repository and binary name.

Reason:

- It mirrors the mental model of `codex exec`.
- It describes a non-interactive model execution surface instead of an internal Jaw helper.
- It leaves room for reuse by runtimes other than cli-jaw.

## Initial Scope

- Extract the Rust PTY helper from `cli-jaw/native/jaw-claude-i`.
- Rename the package and primary binary to `claude-exec`.
- Keep compatibility binary aliases:
  - `claude-i`
  - `jaw-claude-i`
- Keep the stdout protocol stable for cli-jaw:
  - `jaw_runtime` lifecycle events.
  - Claude-like stream-json transcript replay.
- Add repo-local docs:
  - `README.md`
  - `structure/`
  - `devlog/`

## Non-goals

- Do not immediately remove `claude-i` from cli-jaw.
- Do not immediately rename the runtime JSON envelope.
- Do not require npm distribution before the local Rust runtime works.
- Do not change Claude tool permission policy in the extracted repo; forwarded Claude args own that.
- Workspace trust is wrapper-owned because it appears before `SessionStart` and blocks PTY prompt injection in fresh cwd homes.

## Runtime Follow-up

- `--auto-accept-workspace-trust` is an active wrapper behavior, not a reserved flag.
- The wrapper samples the PTY screen before `SessionStart` and submits the affirmative trust choice when Claude asks whether the workspace files are trusted.
- `SessionStart` timeout errors include a compact screen snapshot so cwd-specific startup prompts are visible in JSONL diagnostics.

## Verification

Required before considering the extraction usable:

```bash
cargo fmt --check
cargo test --locked
cargo build --release --locked
target/release/claude-exec --help
target/release/jaw-claude-i --help
```

Manual smoke when Claude auth is available:

```bash
bash scripts/smoke.sh
```
