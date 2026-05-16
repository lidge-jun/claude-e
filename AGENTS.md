# claude-exec

`claude-exec` is a standalone Rust runtime wrapper for Claude Code.

## Source Rules

- Keep generated build output out of git. `target/` must stay ignored.
- Prefer `cargo fmt`, `cargo test`, and `cargo build --release` before publishing runtime changes.
- Keep the primary binary name `claude-exec`. Compatibility aliases `claude-i` and `jaw-claude-i` are intentional until cli-jaw fully migrates.
- Preserve the stdout JSONL contract unless a migration document in `structure/` and `devlog/` describes the compatibility plan.

## Documentation

- `structure/` is the architecture and runtime surface reference.
- `devlog/_plan/` holds active planning work. New durable plan docs use numbered prefixes such as `00_overview.md`.
- Update `README.md`, `structure/INDEX.md`, and the relevant devlog plan when command flags, protocol events, exit codes, packaging, or cli-jaw integration behavior changes.
