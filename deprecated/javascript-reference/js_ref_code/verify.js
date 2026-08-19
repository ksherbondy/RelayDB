#!/usr/bin/env node

const path = require("path");
const RelayDB = require("../relay-generic/v6-reader");

const artifact = process.argv[2];

if (!artifact) {
  console.error("usage: node scripts/relay-master/verify.js <file.relay>");
  process.exit(2);
}

RelayDB.open(path.resolve(artifact));
console.log(`verified ${path.resolve(artifact)}`);
