---
created: 2026-05-16
status: planning
tags: [claude-exec, cli-jaw, rust, runtime]
---
# claude-e Extraction

## Decision

Use `claude-e` as the standalone GitHub repository and npm package name.

Reason:

- It gives the public command a short, memorable surface.
- It avoids the already-owned npm package name `claude-exec`.
- It still keeps `claude-exec` as a long compatibility alias for users who prefer the explicit `codex exec` mental model.

## Initial Scope

- Extract the Rust PTY helper from `cli-jaw/native/jaw-claude-i`.
- Rename the npm package and GitHub repository to `claude-e`.
- Keep `claude-exec` as a compatibility binary alias.
- Keep compatibility binary aliases:
  - `claude-e`
  - `claude-i`
  - `jaw-claude-i`
- Add default `claude -p`-style PTY mode so `claude-e "prompt"` and
  `claude-exec "prompt"` can be used like a print-mode Claude command without
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
- Default to unattended Claude automation: add permission bypass unless the
  caller supplied an explicit permission policy.
- Workspace/folder trust is wrapper-owned because it appears before
  `SessionStart` and blocks PTY prompt injection in fresh cwd homes.

## Print-Compatible PTY Follow-up

- Top-level `claude-e ...` and `claude-exec ...` parse `claude -p`-style args
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
- npm usage is documented global-install first:
  `npm install -g claude-e`, then `claude-e "prompt"`. One-shot `npx claude-e`
  remains supported.
- `--tool`, `--t`, and `-t` expose terminal-friendly tool progress on stderr
  while keeping stdout parseable.
- Print-compatible runs emit a stderr resume footer by default, including the
  current session id and a `claude-e --resume <session-id> ...` command.
- npm packaging now has a Cargo-backed `postinstall`, local release scripts,
  `npm publish --dry-run` validation, semver release helpers, preview release
  helpers, and GitHub workflows for Rust verification, npm package dry-runs, and
  npm publish from GitHub Releases.
- `postinstall` asks once for a GitHub star when npm is interactive and the
  authenticated `gh` CLI is available. Non-interactive installs print the
  repository URL instead. `CLAUDE_E_SKIP_STAR_PROMPT=1` suppresses only that
  request.

## Runtime Follow-up

- `--auto-accept-workspace-trust` is an active wrapper behavior, not a reserved flag, and is enabled by default.
- The wrapper samples the PTY screen before `SessionStart` and submits the affirmative trust choice when Claude asks whether the workspace or folder files are trusted.
- The wrapper appends `--dangerously-skip-permissions` to Claude args unless
  the caller already supplied `--permission-mode`,
  `--permission-mode=...`, `--dangerously-skip-permissions`, or
  `--allow-dangerously-skip-permissions`.
- `SessionStart` timeout errors include a compact screen snapshot so cwd-specific startup prompts are visible in JSONL diagnostics.
- Prompt-injection verification accepts either a new `user` transcript record or
  a new `assistant` record after the prompt offset. This prevents false failures
  when Claude starts answering before the user record is flushed.
- Runtime timeout is split into activity-aware idle timeout and hard cap:
  `--idle-timeout-ms` resets on transcript activity, active tool calls suppress
  idle timeout until tool results arrive, `--hard-timeout-ms` remains the
  absolute process cap, and legacy `--timeout-ms` is retained as an idle-timeout
  alias.
- Claude `rate_limit_event` transcript records are passed through unchanged so
  cli-jaw can treat 429 pacing as wait/retry state rather than fallback-worthy
  failure.

## Verification

Required before considering the extraction usable:

```bash
cargo fmt --check
cargo test --locked
cargo build --release --locked
target/release/claude-e --help
target/release/claude-exec --help
target/release/jaw-claude-i --help
```

Manual smoke when Claude auth is available:

```bash
bash scripts/smoke.sh
```
