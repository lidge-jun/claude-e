#!/usr/bin/env node

import fs from 'fs';
import path from 'path';

const version = process.argv[2];
if (!version) {
  console.error('usage: node scripts/sync-package-versions.mjs <version>');
  process.exit(2);
}

function readJson(filePath) {
  return JSON.parse(fs.readFileSync(filePath, 'utf8'));
}

function writeJson(filePath, data) {
  fs.writeFileSync(filePath, `${JSON.stringify(data, null, 2)}\n`);
}

const rootPackagePath = 'package.json';
const rootPackage = readJson(rootPackagePath);
rootPackage.version = version;
for (const name of Object.keys(rootPackage.optionalDependencies || {})) {
  rootPackage.optionalDependencies[name] = version;
}
writeJson(rootPackagePath, rootPackage);

for (const target of fs.readdirSync('platform-packages')) {
  const packagePath = path.join('platform-packages', target, 'package.json');
  if (!fs.existsSync(packagePath)) continue;
  const platformPackage = readJson(packagePath);
  platformPackage.version = version;
  writeJson(packagePath, platformPackage);
}
