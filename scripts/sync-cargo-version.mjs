#!/usr/bin/env node
import { readFileSync, writeFileSync } from "node:fs";

const version = process.argv[2];
if (!version || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(version)) {
  console.error("usage: sync-cargo-version.mjs <semver>");
  process.exit(2);
}

const cargoToml = new URL("../Cargo.toml", import.meta.url);
const current = readFileSync(cargoToml, "utf8");
const versionLine = current.match(/^version = "([^"]+)"$/m);

if (!versionLine) {
  console.error("Cargo.toml package version line was not found.");
  process.exit(1);
}

if (versionLine[1] !== version) {
  writeFileSync(cargoToml, current.replace(/^version = ".*"$/m, `version = "${version}"`));
}
