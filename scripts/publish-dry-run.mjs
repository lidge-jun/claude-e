#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

function npmCommand() {
  return process.platform === "win32" ? "npm.cmd" : "npm";
}

const pkg = JSON.parse(readFileSync(new URL("../package.json", import.meta.url), "utf8"));

const result = spawnSync(
  npmCommand(),
  ["publish", "--dry-run", "--ignore-scripts", "--access", "public"],
  {
    cwd: new URL("..", import.meta.url),
    encoding: "utf8",
    env: {
      ...process.env,
      npm_config_loglevel: process.env.npm_config_loglevel ?? "notice",
    },
  },
);

const stdout = result.stdout || "";
const stderr = result.stderr || "";
const combinedOutput = `${stdout}\n${stderr}`;

if (result.status === 0) {
  if (stdout) process.stdout.write(stdout);
  if (stderr) process.stderr.write(stderr);
  process.exit(0);
}

if (combinedOutput.includes("You cannot publish over the previously published versions")) {
  if (stdout) process.stdout.write(stdout);
  console.warn(
    `[${pkg.name}] npm publish dry-run reached registry validation; ${pkg.version} already exists, so package validation is complete.`,
  );
  process.exit(0);
}

if (stdout) process.stdout.write(stdout);
if (stderr) process.stderr.write(stderr);
process.exit(result.status ?? 1);
