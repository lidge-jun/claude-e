#!/usr/bin/env node
'use strict';

const { spawnSync } = require('node:child_process');
const { existsSync, mkdirSync, copyFileSync, chmodSync } = require('node:fs');
const { join, dirname } = require('node:path');
const { maybePromptGithubStar } = require('./star-prompt.cjs');

const root = join(__dirname, '..');
const ext = process.platform === 'win32' ? '.exe' : '';
const built = join(root, 'target', 'release', `claude-exec${ext}`);
const BINARIES = ['claude-exec', 'claude-e', 'claude-i', 'jaw-claude-i'];

function log(message) {
  console.log(`[claude-e:postinstall] ${message}`);
}

function platformPackageName() {
  const platform = process.platform;
  const arch = process.arch;
  const key = `${platform}-${arch}`;
  const supported = ['darwin-arm64', 'darwin-x64', 'linux-x64', 'linux-arm64'];
  if (!supported.includes(key)) return null;
  return `@bitkyc08/claude-e-${key}`;
}

function tryResolvePrebuilt() {
  const pkg = platformPackageName();
  if (!pkg) return false;

  let pkgDir;
  try {
    const pkgJson = require.resolve(`${pkg}/package.json`);
    pkgDir = dirname(pkgJson);
  } catch {
    return false;
  }

  const srcBin = join(pkgDir, 'bin', `claude-exec${ext}`);
  if (!existsSync(srcBin)) return false;

  const targetDir = join(root, 'target', 'release');
  mkdirSync(targetDir, { recursive: true });

  let copied = 0;
  for (const name of BINARIES) {
    const src = join(pkgDir, 'bin', `${name}${ext}`);
    const dst = join(targetDir, `${name}${ext}`);
    if (existsSync(src)) {
      copyFileSync(src, dst);
      chmodSync(dst, 0o755);
      copied++;
    }
  }

  if (copied > 0) {
    log(`resolved ${copied} prebuilt binaries from ${pkg}`);
    return true;
  }
  return false;
}

async function main() {
  if (process.env.CLAUDE_E_SKIP_POSTINSTALL === '1' || process.env.CLAUDE_EXEC_SKIP_POSTINSTALL === '1') {
    log('skipping postinstall because CLAUDE_E_SKIP_POSTINSTALL/CLAUDE_EXEC_SKIP_POSTINSTALL is set');
    return 0;
  }

  if (process.env.CLAUDE_E_SKIP_BUILD === '1' || process.env.CLAUDE_EXEC_SKIP_BUILD === '1') {
    log('skipping native build because CLAUDE_E_SKIP_BUILD/CLAUDE_EXEC_SKIP_BUILD is set');
    return promptForStar(0);
  }

  if (existsSync(built)) {
    log(`native binary already exists: ${built}`);
    return promptForStar(0);
  }

  try {
    if (tryResolvePrebuilt()) {
      return promptForStar(0);
    }
  } catch (err) {
    log(`prebuilt resolution failed: ${err instanceof Error ? err.message : err}`);
  }

  const cargo = spawnSync('cargo', ['--version'], { encoding: 'utf8', stdio: 'pipe' });
  if (cargo.status !== 0) {
    log('cargo not found — skipping native build.');
    log('  The claude-e binary will not be available until Rust is installed.');
    log('  Install Rust: https://rustup.rs');
    log('  Or set CLAUDE_EXEC_BIN to a prebuilt binary path.');
    return promptForStar(0);
  }

  log(`using ${cargo.stdout.trim()}`);
  const build = spawnSync('cargo', ['build', '--release', '--locked'], {
    cwd: root,
    stdio: 'inherit',
    env: process.env,
  });

  if (build.status !== 0) {
    log(`cargo build failed (status ${build.status}) — skipping native build.`);
    log('  The claude-e binary will not be available. Re-run: npm rebuild claude-e');
    return promptForStar(0);
  }

  log(`native binary built: ${built}`);
  return promptForStar(0);
}

async function promptForStar(code) {
  try {
    await maybePromptGithubStar();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    console.warn(`[claude-e:postinstall] GitHub star prompt skipped: ${message}`);
  }
  return code;
}

main()
  .then((code) => process.exit(code))
  .catch((error) => {
    const message = error instanceof Error ? error.stack || error.message : String(error);
    console.error(`[claude-e:postinstall] ${message}`);
    process.exit(0);
  });
