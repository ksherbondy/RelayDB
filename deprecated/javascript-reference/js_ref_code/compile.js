#!/usr/bin/env node

const path = require("path");
const { execFileSync } = require("child_process");
const input = process.argv[2];
const output = process.argv[3];

if (!input || !output) {
  console.error("usage: node scripts/relay-master/compile.js <input> <output>");
  process.exit(2);
}

const compilerPath = path.resolve(__dirname, "../relay-generic/v5-profiled-columnar.js");
execFileSync(process.execPath, [compilerPath, path.resolve(input), path.resolve(output)], {
  cwd: path.resolve(__dirname, "../.."),
  stdio: "inherit",
});

const readerPath = path.resolve(__dirname, "../relay-generic/v6-reader.js");
execFileSync(process.execPath, [
  "-e",
  `const RelayDB = require(${JSON.stringify(readerPath)}); RelayDB.open(${JSON.stringify(path.resolve(output))});`,
], {
  cwd: path.resolve(__dirname, "../.."),
  stdio: "inherit",
});

console.log(`verified ${path.resolve(output)}`);
