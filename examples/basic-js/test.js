import fs from "node:fs";
import RelayDB from "../../packages/relaydb-js/src/index.js";

const relayPath =
  process.argv[2] ?? "../../relay-compiler/builds/relaydb-v1-self-docs.relay";

const buffer = fs.readFileSync(relayPath);
const arrayBuffer = buffer.buffer.slice(
  buffer.byteOffset,
  buffer.byteOffset + buffer.byteLength
);

const db = RelayDB.fromBytes(arrayBuffer);

console.log("Relay file:", relayPath);
console.log("Anchor count:", db.size());
console.log("First 10 anchors:", db.anchors().slice(0, 10));

const project = db.get("project:relaydb");
console.log("Project:", project);

const jumpNode = db.get("function:relay_jump_from");
console.log("relay_jump_from:", jumpNode);

const traversal = db.jump("project:relaydb");
console.log("Traversal visited:", traversal.visited);
console.log("Traversal nodes returned:", traversal.nodes.length);
console.log("Traversal missing anchors:", traversal.missing);

const integrity = db.integrityReport();
console.log("Integrity:", integrity);

if (!integrity.ok) {
  process.exitCode = 1;
}
