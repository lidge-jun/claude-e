#!/usr/bin/env node
'use strict';

const { spawnSync } = require('node:child_process');
const { existsSync, mkdirSync, readFileSync, writeFileSync } = require('node:fs');
const { homedir } = require('node:os');
const { dirname, join } = require('node:path');
const { createInterface } = require('node:readline/promises');

const REPO = 'lidge-jun/claude-e';
const REPO_URL = `https://github.com/${REPO}`;

function log(message) {
  console.log(`[claude-e:postinstall] ${message}`);
}

function resolveHomePath(value) {
  if (!value) return join(homedir(), '.claude-e');
  if (value === '~') return homedir();
  if (value.startsWith('~/')) return join(homedir(), value.slice(2));
  return value;
}

function starPromptStatePath(env = process.env) {
  const baseDir = resolveHomePath(env.CLAUDE_E_HOME || env.CLAUDE_EXEC_HOME);
  return join(baseDir, 'state', 'star-prompt.json');
}

function truthy(value) {
  if (typeof value !== 'string') return false;
  const normalized = value.trim().toLowerCase();
  return normalized === '1' || normalized === 'true' || normalized === 'yes';
}

function shouldSkipStarPrompt(env = process.env) {
  return truthy(env.CI)
    || truthy(env.CLAUDE_E_SKIP_STAR_PROMPT)
    || truthy(env.CLAUDE_EXEC_SKIP_STAR_PROMPT)
    || truthy(env.npm_config_claude_e_skip_star_prompt)
    || truthy(env.npm_config_claude_exec_skip_star_prompt);
}

function hasBeenPrompted(env = process.env) {
  const path = starPromptStatePath(env);
  if (!existsSync(path)) return false;
  try {
    const state = JSON.parse(readFileSync(path, 'utf8'));
    return typeof state.prompted_at === 'string';
  } catch {
    return false;
  }
}

function markPrompted(env = process.env) {
  const path = starPromptStatePath(env);
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, JSON.stringify({ prompted_at: new Date().toISOString() }, null, 2));
}

function isGhInstalled(spawnSyncFn = spawnSync) {
  const result = spawnSyncFn('gh', ['--version'], {
    encoding: 'utf8',
    stdio: ['ignore', 'ignore', 'ignore'],
    timeout: 3000,
    windowsHide: true,
  });
  return !result.error && result.status === 0;
}

function starRepo(spawnSyncFn = spawnSync) {
  const result = spawnSyncFn('gh', ['api', '-X', 'PUT', `/user/starred/${REPO}`], {
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
    timeout: 10000,
    windowsHide: true,
  });

  if (result.error) return { ok: false, error: result.error.message };
  if (result.status !== 0) {
    const stderr = (result.stderr || '').trim();
    const stdout = (result.stdout || '').trim();
    return { ok: false, error: stderr || stdout || `gh exited ${result.status}` };
  }
  return { ok: true };
}

async function askYesNo(question) {
  const rl = createInterface({ input: process.stdin, output: process.stdout });
  try {
    const answer = (await rl.question(question)).trim().toLowerCase();
    return answer === '' || answer === 'y' || answer === 'yes';
  } finally {
    rl.close();
  }
}

async function maybePromptGithubStar(deps = {}) {
  const env = deps.env || process.env;
  if (shouldSkipStarPrompt(env)) return;

  const hasBeenPromptedImpl = deps.hasBeenPromptedFn || (() => hasBeenPrompted(env));
  if (hasBeenPromptedImpl()) return;

  const stdinIsTTY = deps.stdinIsTTY ?? process.stdin.isTTY;
  const stdoutIsTTY = deps.stdoutIsTTY ?? process.stdout.isTTY;
  const isGhInstalledImpl = deps.isGhInstalledFn || isGhInstalled;
  const ghInstalled = isGhInstalledImpl();

  if (!stdinIsTTY || !stdoutIsTTY || !ghInstalled) {
    const markPromptedImpl = deps.markPromptedFn || (() => markPrompted(env));
    markPromptedImpl();
    const logFn = deps.logFn || log;
    logFn(`If claude-e helps, please star it: ${REPO_URL}`);
    if (!stdinIsTTY || !stdoutIsTTY) {
      logFn('GitHub star prompt skipped because npm postinstall is non-interactive.');
    } else {
      logFn('Install and authenticate GitHub CLI to star automatically: gh auth login');
    }
    return;
  }

  const markPromptedImpl = deps.markPromptedFn || (() => markPrompted(env));
  markPromptedImpl();

  const askYesNoImpl = deps.askYesNoFn || askYesNo;
  const approved = await askYesNoImpl('[claude-e] Enjoying claude-e? Star it on GitHub? [Y/n] ');
  if (!approved) return;

  const starRepoImpl = deps.starRepoFn || starRepo;
  const result = starRepoImpl();
  if (result.ok) {
    const logFn = deps.logFn || log;
    logFn('Thanks for the star!');
    return;
  }

  const warnFn = deps.warnFn || console.warn;
  warnFn(`[claude-e:postinstall] Could not star repository automatically: ${result.error}`);
}

module.exports = {
  maybePromptGithubStar,
  starPromptStatePath,
  hasBeenPrompted,
  markPrompted,
  isGhInstalled,
  starRepo,
  shouldSkipStarPrompt,
};
