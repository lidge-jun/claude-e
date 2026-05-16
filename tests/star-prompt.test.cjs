'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const { mkdtemp, rm } = require('node:fs/promises');
const { tmpdir } = require('node:os');
const { join } = require('node:path');

const {
  maybePromptGithubStar,
  shouldSkipStarPrompt,
  starPromptStatePath,
  starRepo,
} = require('../scripts/star-prompt.cjs');

test('starPromptStatePath honors CLAUDE_E_HOME', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'claude-e-star-home-'));
  try {
    assert.equal(starPromptStatePath({ CLAUDE_E_HOME: dir }), join(dir, 'state', 'star-prompt.json'));
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

test('shouldSkipStarPrompt skips CI and explicit postinstall opt-out', () => {
  assert.equal(shouldSkipStarPrompt({ CI: 'true' }), true);
  assert.equal(shouldSkipStarPrompt({ CLAUDE_E_SKIP_STAR_PROMPT: '1' }), true);
  assert.equal(shouldSkipStarPrompt({ CLAUDE_EXEC_SKIP_STAR_PROMPT: 'true' }), true);
  assert.equal(shouldSkipStarPrompt({}), false);
});

test('starRepo calls gh starred API with hidden Windows console', () => {
  let seenCommand = '';
  let seenArgs = [];
  let seenOptions;
  const result = starRepo((command, args, options) => {
    seenCommand = command;
    seenArgs = args;
    seenOptions = options;
    return {
      status: 0,
      signal: null,
      error: undefined,
      stdout: '',
      stderr: '',
      output: [],
      pid: 1,
    };
  });

  assert.deepEqual(result, { ok: true });
  assert.equal(seenCommand, 'gh');
  assert.deepEqual(seenArgs, ['api', '-X', 'PUT', '/user/starred/lidge-jun/claude-e']);
  assert.equal(seenOptions.windowsHide, true);
});

test('maybePromptGithubStar prints install-time URL for non-TTY sessions', async () => {
  const logs = [];
  let marked = false;

  await maybePromptGithubStar({
    env: {},
    stdinIsTTY: false,
    stdoutIsTTY: false,
    hasBeenPromptedFn: () => false,
    isGhInstalledFn: () => false,
    markPromptedFn: () => { marked = true; },
    logFn: (message) => logs.push(message),
  });

  assert.equal(marked, true);
  assert.ok(logs.some((line) => line.includes('https://github.com/lidge-jun/claude-e')));
  assert.ok(logs.some((line) => line.includes('non-interactive')));
});

test('maybePromptGithubStar asks and stars in interactive gh sessions', async () => {
  const logs = [];
  let marked = false;
  let starred = false;

  await maybePromptGithubStar({
    env: {},
    stdinIsTTY: true,
    stdoutIsTTY: true,
    hasBeenPromptedFn: () => false,
    isGhInstalledFn: () => true,
    markPromptedFn: () => { marked = true; },
    askYesNoFn: async () => true,
    starRepoFn: () => {
      starred = true;
      return { ok: true };
    },
    logFn: (message) => logs.push(message),
  });

  assert.equal(marked, true);
  assert.equal(starred, true);
  assert.deepEqual(logs, ['Thanks for the star!']);
});
