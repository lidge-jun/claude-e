# 2026-05-16 Hardening Audit

## Scope

- Review the PTY wrapper paths that can fail in unattended agent/runtime use.
- Prioritize security, release safety, and process-control failures over broad
  pedantic lint cleanup.
- Keep the public CLI contract unchanged: `claude-e` remains primary, and the
  compatibility bins stay available.

## Findings

1. Process-group cleanup accepted any `u32` child pid.
   - Risk: a zero pid would map to process group `0`, which targets the current
     process group on Unix.
   - Fix: reject pid `0` and values that cannot fit into `i32` before signaling.

2. PTY prompt writes used poisoned mutex unwraps.
   - Risk: a poisoned writer lock could panic the wrapper instead of producing a
     classified runtime error and cleanup event.
   - Fix: convert lock poisoning to an emitted error and process cleanup path.

3. Hook relay generation interpolated raw filesystem paths into shell and JSON.
   - Risk: unusual temp paths could break the generated command, and hook event
     names were not constrained.
   - Fix: shell-quote command paths, generate settings through `serde_json`, add
     `set -eu`, restrictive `umask`, atomic payload writes, and an event
     whitelist.

4. Transcript drain used `lines().flatten()`.
   - Risk: repeated read errors can be hidden in an unbounded iterator.
   - Fix: use `map_while(Result::ok)` for the final drain pass.

5. Local npm publishing could run with a dirty worktree.
   - Risk: publishing a package that does not match the committed release state.
   - Fix: `scripts/release-npm.sh --publish` now refuses unstaged or staged
     changes before npm auth and publish.

## Verification Plan

- `cargo fmt --check`
- `cargo test --locked`
- `cargo build --release --locked`
- `npm run verify`
- `npm pack --dry-run`
- push to `main` and watch GitHub Actions for the hardening commit.

## Remaining Audit Debt

- `cargo clippy --locked --all-targets -- -D warnings` still contains broad
  pedantic/style debt. Treat that as a separate cleanup phase so hardening fixes
  stay small and reviewable.
- `npm audit` is not meaningful in this package today because it has no lockfile
  and no runtime npm dependencies.
