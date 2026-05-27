---
created: 2026-05-27
status: active
tags: [claude-e, npm, prebuilt, release]
---
# Prebuilt npm Release Contract

## Problem

Rust-less installs must resolve a prebuilt `claude-e` binary from an optional
platform package. The main npm package must never point at platform package
versions that were not built or published.

## Decision

- Keep npm publish out of this patch.
- Treat GitHub and local verification as the safe handoff boundary.
- Require the main package version, `optionalDependencies`, and
  `platform-packages/*/package.json` versions to match.
- Verify the package contract in `test:postinstall` and in the release workflow
  before the main package publish step.
- Sync Cargo metadata to the workflow input version in both platform and main
  release jobs.

## Scope

This plan covers `claude-e` packaging safety only. It does not publish npm,
change the public command surface, or remove compatibility bins.

## Verification

- `npm run test:postinstall`
- `npm run release:dry-run`
- `git diff --check`
