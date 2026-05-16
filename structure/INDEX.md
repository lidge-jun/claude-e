# claude-e Structure

`claude-e` is the standalone npm package for the Rust extraction of cli-jaw's native Claude interactive helper.

## Map

| Area | Path | Notes |
|---|---|---|
| CLI entrypoint | `src/lib.rs`, `src/args.rs`, `src/bin/*`, `src/print_mode.rs` | default `claude -p`-style PTY mode plus `claude-e run` / `claude-e exec` command parsing and single-turn execution loop |
| PTY child | `src/child.rs`, `src/terminal.rs`, `src/cleanup.rs` | Claude process spawning, terminal query responses, signal/cleanup handling |
| Prompt safety | `src/sanitize.rs`, `src/lib.rs` | stdin read cap, prompt sanitization, bracketed paste injection |
| Hooks | `src/hook.rs` | temporary Claude settings and hook relay script |
| Transcript replay | `src/transcript.rs`, `src/normalize.rs` | transcript tailing and Claude-like stream-json normalization |
| Runtime protocol | `src/protocol.rs` | `jaw_runtime` JSONL lifecycle envelope |
| Packaging | `Cargo.toml`, `package.json`, `bin/`, `scripts/`, `.github/workflows/` | Rust build, `claude-e` alias, local npm-style wrappers, dry-run/publish/release scripts, npm publish workflow |

## Documents

- [cli_surface.md](cli_surface.md): command names, aliases, flags, and examples.
- [runtime_contract.md](runtime_contract.md): stdin, stdout, event, resume, timeout, and exit-code contract.
- [cli_jaw_migration.md](cli_jaw_migration.md): planned cli-jaw rename and compatibility path.

## Current Status

- Primary public package and command: `claude-e`
- Long compatibility alias: `claude-exec`
- Compatibility binaries: `claude-i`, `jaw-claude-i`
- Default command mode: `claude -p`-style PTY wrapper
- Primary subcommand: `run`
- Compatibility/semantic alias: `exec`
- Protocol envelope: `jaw_runtime` for cli-jaw compatibility
- npm release surface: current-version publish, semver release, preview release, and GitHub Release workflow
