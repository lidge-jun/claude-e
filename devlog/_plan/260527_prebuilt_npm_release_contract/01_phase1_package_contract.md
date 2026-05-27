---
created: 2026-05-27
status: active
tags: [claude-e, npm, package-contract]
---
# Phase 1 Package Contract Patch

## Files

### package.json

- `test:postinstall` runs every `tests/*.test.cjs` contract test.
- Optional platform package versions match the main package version.

### platform-packages/*/package.json

- Platform package versions match the main package version.

### tests/package-contract.test.cjs

- Asserts optional dependency names are exactly the supported platform packages.
- Asserts every platform package version and published file list matches the
  main package contract.
- Asserts postinstall knows every declared platform package name.

### .github/workflows/release.yml

- Syncs Cargo metadata to the requested release version before platform builds.
- Installs Rust in the main publish job so Cargo metadata can be synced there
  too.
- Runs the package contract test after optional dependency rewriting and before
  main package publish.

### scripts/sync-package-versions.mjs

- Updates the main package version, every optional dependency version, and every
  `platform-packages/*/package.json` version for a workflow release version.

## Operator Notes

After this patch is pushed, the next safe release is a new semver version such
as `0.1.10`. Use the workflow dry-run first. Do not reuse an already-published
version for the final publish run.
