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
  - `claude-e`
  - `claude-i`
  - `jaw-claude-i`
- Add default `claude -p`-style PTY mode so `claude-exec "prompt"` and
  `claude-e "prompt"` can be used like a print-mode Claude command without
  leaving the interactive wrapper path.
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

## Print-Compatible PTY Follow-up

- Top-level `claude-exec ...` and `claude-e ...` parse `claude -p`-style args
  and then call the same PTY-backed runtime used by `run`.
- `claude-e p ...` and `claude-e print ...` are accepted as explicit aliases
  for the same top-level print-compatible mode.
- `-p` and `--print` are accepted as compatibility flags, while `claude-e`
  remains PTY-backed internally.
- The print-compatible parser covers the current non-interactive help surface:
  stdin/output formats, JSON schema instruction, session id controls, partial
  and hook stream flags, model/effort/permission flags, tools, MCP/settings,
  system prompt, append-system-prompt, plugin, browser, debug, and worktree
  controls.
- Print-only behaviors that cannot be enforced by the PTY path are consumed for
  command-shape compatibility instead of being forwarded into interactive
  Claude.
- `run` and `exec` subcommands remain reserved for the PTY wrapper path.
- The Claude binary can be overridden with `CLAUDE_EXEC_CLAUDE_BIN`, then
  `CLAUDE_BIN`; otherwise it resolves `claude` from PATH.
- Internal `jaw_runtime` lifecycle events are suppressed from stdout in this
  top-level print-compatible mode.
- npm one-shot behavior differs by package name: `npx claude-exec "prompt"`
  works for the `claude-exec` package, while `claude-e` as a one-shot package
  name needs a separate alias package or `npx -p claude-exec claude-e "prompt"`.
- npm packaging now has a Cargo-backed `postinstall`, local release scripts, and
  GitHub workflows for Rust verification, npm package dry-runs, and npm publish
  from GitHub Releases.

## Runtime Follow-up

- `--auto-accept-workspace-trust` is an active wrapper behavior, not a reserved flag.
- The wrapper samples the PTY screen before `SessionStart` and submits the affirmative trust choice when Claude asks whether the workspace files are trusted.
- `SessionStart` timeout errors include a compact screen snapshot so cwd-specific startup prompts are visible in JSONL diagnostics.
- Prompt-injection verification accepts either a new `user` transcript record or
  a new `assistant` record after the prompt offset. This prevents false failures
  when Claude starts answering before the user record is flushed.

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
