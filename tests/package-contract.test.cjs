const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const test = require('node:test');

const root = path.resolve(__dirname, '..');

function readJson(relativePath) {
  return JSON.parse(fs.readFileSync(path.join(root, relativePath), 'utf8'));
}

test('platform package versions match the main claude-e package', () => {
  const pkg = readJson('package.json');
  const expected = {
    '@bitkyc08/claude-e-darwin-arm64': 'platform-packages/darwin-arm64/package.json',
    '@bitkyc08/claude-e-darwin-x64': 'platform-packages/darwin-x64/package.json',
    '@bitkyc08/claude-e-linux-arm64': 'platform-packages/linux-arm64/package.json',
    '@bitkyc08/claude-e-linux-x64': 'platform-packages/linux-x64/package.json',
  };

  assert.deepEqual(Object.keys(pkg.optionalDependencies).sort(), Object.keys(expected).sort());

  for (const [name, packagePath] of Object.entries(expected)) {
    assert.equal(pkg.optionalDependencies[name], pkg.version, `${name} optional dependency version`);
    const platformPkg = readJson(packagePath);
    assert.equal(platformPkg.name, name, `${packagePath} package name`);
    assert.equal(platformPkg.version, pkg.version, `${packagePath} package version`);
    assert.deepEqual(platformPkg.files, ['bin/'], `${packagePath} published files`);
  }
});

test('postinstall knows every declared prebuilt platform package', () => {
  const pkg = readJson('package.json');
  const postinstall = fs.readFileSync(path.join(root, 'scripts/postinstall.cjs'), 'utf8');

  for (const name of Object.keys(pkg.optionalDependencies)) {
    const platform = name.replace('@bitkyc08/claude-e-', '');
    assert.ok(postinstall.includes(`'${platform}'`), `postinstall should support ${platform}`);
  }
  assert.ok(postinstall.includes('return `@bitkyc08/claude-e-${key}`;'));
});
