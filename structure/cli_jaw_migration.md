# cli-jaw Migration Plan

## Goal

Move cli-jaw from the internal `claude-i` / `jaw-claude-i` naming to an external `claude-exec` runtime without breaking saved settings, session buckets, doctor output, tests, or installed helper paths.

## Current Inventory

The current cli-jaw references are concentrated in these surfaces:

- Provider/type key: `src/types/cli-engine.ts`, `src/cli/registry.ts`, tests.
- Binary detection: `src/core/config.ts`, `bin/commands/doctor.ts`, `tests/unit/claude-i-detection.test.ts`.
- Spawn/args: `src/agent/args.ts`, `src/agent/spawn.ts`.
- Events/runtime: `src/agent/claude-i-runtime.ts`, `src/agent/events.ts`, `structure/stream-events.md`.
- Session and model helpers: `src/agent/session-persistence.ts`, `src/agent/resume-classifier.ts`, `src/agent/cli-helpers.ts`, `src/cli/claude-models.ts`.
- UI/settings metadata: `public/manager/src/settings/pages/components/agent/agent-meta.ts`.
- Build scripts and docs: `package.json`, `structure/*.md`, `devlog/_plan/260516_claude_i_interactive_wrapper/`.

This is not a huge runtime surface, but the string appears in many docs and tests. Rename it in phases.

## Phase 0: Extraction

- Create standalone `700_projects/claude-exec`.
- Build `claude-exec`, `claude-i`, and `jaw-claude-i` from the same Rust source.
- Keep `jaw_runtime` output stable.
- Keep cli-jaw source behavior unchanged except for explicit `--claude-bin` passing.

## Phase 1: Binary Detection Alias

In cli-jaw:

- Add `CLAUDE_EXEC_BIN` as the preferred explicit env var.
- Prefer embedded npm `claude-exec`, then `claude-exec` on PATH.
- Fall back to `JAW_CLAUDE_I_BIN`, `jaw-claude-i`, `claude-i`, and legacy `native/jaw-claude-i/target/...`.
- Rename helper candidate function from `getClaudeIHelperCandidates` to `getClaudeExecCandidates`, leaving an exported deprecated alias for tests and older callers.

## Phase 2: Provider Alias

Add a new provider key:

```ts
'claude-exec': {
  label: 'Claude Exec',
  binary: 'claude-exec',
  experimental: true,
  ...
}
```

Keep `claude-i` as a hidden/deprecated alias for one release line.

Settings migration:

- If `settings.cli === 'claude-i'`, migrate to `claude-exec`.
- If `perCli['claude-i']` exists and `perCli['claude-exec']` is absent, copy the config.
- Keep session buckets separate at first. Add an explicit bucket migration only after resume smoke tests pass.

## Phase 3: Runtime/Event Rename

Additive first:

- Rename `src/agent/claude-i-runtime.ts` to `claude-exec-runtime.ts`.
- Broadcast both `agent:claude-exec:*` and deprecated `agent:claude-i:*` for one compatibility window.
- Accept both `jaw_runtime` and future `claude_exec_runtime` if the standalone wrapper adds it.

Removal later:

- Remove deprecated `agent:claude-i:*` broadcasts only after UI/docs/tests no longer consume them.

## Phase 4: Build and Docs

- Replace `build:claude-i` with `build:claude-exec`.
- Replace `test:claude-i` with `test:claude-exec`.
- Keep deprecated npm script aliases for at least one release:
  - `build:claude-i -> npm run build:claude-exec`
  - `test:claude-i -> npm run test:claude-exec`
- Update `structure/INDEX.md`, `structure/agent_spawn.md`, `structure/commands.md`, `structure/prompt_flow.md`, `structure/stream-events.md`, and `structure/str_func.md`.

## Phase 5: Remove Embedded Rust

Only after `claude-exec` is published or installed locally:

- Stop building `native/jaw-claude-i` in cli-jaw release gates.
- Keep the folder for one release as a vendored fallback or remove it behind a clear changelog entry.
- Prefer an installed `claude-exec` binary and explicit `CLAUDE_EXEC_BIN`.

## Recommended First cli-jaw Diff

The first safe cli-jaw code diff should be narrow:

1. Add `CLAUDE_EXEC_BIN`, embedded npm `claude-exec`, and PATH `claude-exec` to detection before legacy helper names.
2. Pass explicit `--claude-bin <resolved claude path>` in `spawn.ts`.
3. Keep provider id `claude-i` unchanged.
4. Add tests proving detection priority and `--claude-bin` args.

That fixes the live PATH failure while avoiding saved-settings churn.
