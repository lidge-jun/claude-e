#!/usr/bin/env node
'use strict';

const { spawnSync } = require('node:child_process');
const { existsSync } = require('node:fs');
const { join } = require('node:path');

const root = join(__dirname, '..');
const built = join(root, 'target', 'release', process.platform === 'win32' ? 'claude-exec.exe' : 'claude-exec');

function log(message) {
  console.log(`[claude-exec:postinstall] ${message}`);
}

if (process.env.CLAUDE_EXEC_SKIP_POSTINSTALL === '1' || process.env.CLAUDE_EXEC_SKIP_BUILD === '1') {
  log('skipping native build because CLAUDE_EXEC_SKIP_POSTINSTALL/CLAUDE_EXEC_SKIP_BUILD is set');
  process.exit(0);
}

if (existsSync(built)) {
  log(`native binary already exists: ${built}`);
  process.exit(0);
}

const cargo = spawnSync('cargo', ['--version'], { encoding: 'utf8', stdio: 'pipe' });
if (cargo.status !== 0) {
  console.error('[claude-exec:postinstall] cargo is required to build the npm package from source.');
  console.error('[claude-exec:postinstall] Install Rust from https://rustup.rs, or set CLAUDE_EXEC_SKIP_BUILD=1 and provide a built binary separately.');
  process.exit(1);
}

log(`using ${cargo.stdout.trim()}`);
const build = spawnSync('cargo', ['build', '--release', '--locked'], {
  cwd: root,
  stdio: 'inherit',
  env: process.env,
});

if (build.status !== 0) {
  console.error(`[claude-exec:postinstall] cargo build failed with status ${build.status}`);
  process.exit(build.status || 1);
}

log(`native binary built: ${built}`);
