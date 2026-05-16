# claude-e

`claude-e` is a standalone npm-distributed Rust runtime wrapper for Claude Code.

## Source Rules

- Keep generated build output out of git. `target/` must stay ignored.
- Prefer `cargo fmt`, `cargo test`, and `cargo build --release` before publishing runtime changes.
- Keep the public npm package and short command name `claude-e`. The long `claude-exec` binary remains an intentional compatibility alias, alongside `claude-i` and `jaw-claude-i`, until cli-jaw fully migrates.
- Preserve the stdout JSONL contract unless a migration document in `structure/` and `devlog/` describes the compatibility plan.

## Documentation

- `structure/` is the architecture and runtime surface reference.
- `devlog/_plan/` holds active planning work. New durable plan docs use numbered prefixes such as `00_overview.md`.
- Update `README.md`, `structure/INDEX.md`, and the relevant devlog plan when command flags, protocol events, exit codes, packaging, or cli-jaw integration behavior changes.
