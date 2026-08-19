#!/usr/bin/env node

const assert = require("assert/strict");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { execFileSync } = require("child_process");
const { compile, DiagnosticError } = require("../relay-generic/compiler");
const RelayDB = require("../relay-generic/v6-reader");

const root = path.resolve(__dirname, "../..");
const fixtures = path.join(__dirname, "fixtures");
const temporaryDirectory = fs.mkdtempSync(path.join(os.tmpdir(), "relaydb-reference-v1-"));

try {
  const artifact = path.join(temporaryDirectory, "valid.relay");
  execFileSync(
    process.execPath,
    [path.join(root, "scripts", "relay-master", "compile.js"), path.join(fixtures, "reference-v1.jsonl"), artifact],
    { cwd: root, stdio: "pipe" },
  );
  const db = RelayDB.open(artifact);
  const actor = db.get("actor-1", { collection: "actors", depth: 0, cache: false });
  const film = db.get("film-2", { collection: "films", depth: 0, cache: false });

  assert.equal(actor["~active"], true);
  assert.equal(actor["~inactive"], false);
  assert.equal(actor["~score"], 0);
  assert.equal(actor["~empty"], "");
  assert.equal("~missing" in actor, false);
  assert.equal(film["~large"], 2147483648);
  assert.equal(film["~rating"], 8.5);
  assert.equal(film["~optional"], null);
  assert.equal(film["@director"], "actor-1");
  assert.deepEqual(film["@cast"], []);

  assert.throws(
    () => compile(
      path.join(fixtures, "invalid-identity.jsonl"),
      path.join(temporaryDirectory, "invalid-identity.relay"),
    ),
    (error) => error instanceof DiagnosticError && error.diagnostics.some((item) => item.code === "RG010"),
  );

  const ambiguousArtifact = path.join(temporaryDirectory, "ambiguous.relay");
  execFileSync(
    process.execPath,
    [path.join(root, "scripts", "relay-master", "compile.js"), path.join(fixtures, "ambiguous-ids.jsonl"), ambiguousArtifact],
    { cwd: root, stdio: "pipe" },
  );
  const ambiguousDb = RelayDB.open(ambiguousArtifact);
  assert.throws(() => ambiguousDb.get("123"), /ambiguous ID/);
  assert.equal(ambiguousDb.get("123", { collection: "people" }).name, "Person 123");

  const cycleArtifact = path.join(temporaryDirectory, "cycle.relay");
  execFileSync(
    process.execPath,
    [path.join(root, "scripts", "relay-master", "compile.js"), path.join(fixtures, "cycle.jsonl"), cycleArtifact],
    { cwd: root, stdio: "pipe" },
  );
  const cycle = RelayDB.open(cycleArtifact).get("a", { collection: "nodes", depth: 1 });
  assert.equal(cycle.link["#id"], "b");

  const crime = db.query({ "^": "films", "~year": { gte: 2000, lt: 2020 } });
  assert.equal(crime.length, 1);
  assert.equal(db.query({ "^": "films", "~year": { gt: 2024 } }).length, 0);
  assert.equal(db.query({ "^": "films", "~year": { lte: 2000 } }).length, 1);
  assert.equal(db.query({ "^": "films", "~year": { ne: 2024 } }).length, 1);

  const page = db.queryPage({ "^": "films" }, { pageSize: 1, fields: ["name"] });
  assert.equal(page.results.length, 1);
  assert.equal(page.total, 2);
  assert.equal(page.next().results.length, 1);

  assert.throws(
    () => compile(
      path.join(fixtures, "invalid-mixed-cardinality.jsonl"),
      path.join(temporaryDirectory, "invalid-cardinality.relay"),
    ),
    (error) => error instanceof DiagnosticError && error.diagnostics.some((item) => item.code === "RG011"),
  );

  console.log("Reference V1 tests passed");
} finally {
  fs.rmSync(temporaryDirectory, { recursive: true, force: true });
}
