# RelayDB v1 JavaScript Reader

This is a JavaScript reader for the current RelayDB v1 / v1.2 `.relay` artifact format.

It allows browser and Node.js environments to load a compiled `.relay` file and perform anchor lookups, graph traversal, and integrity checks.

> This reader is part of the RelayDB v1 proof of concept and is not intended for production use yet.

---

## Supported v1 / v1.2 format

The reader supports the current universal RelayDB v1.2 format:

- 32-byte header
- jump-table pointer stored as `u64 little-endian` at byte offset `16`
- null-terminated UTF-8 JSON node payloads
- text jump table at the end of the file using:

```text
anchor<TAB>byte_offset
```

It also supports the older legacy v1 jump-table format:

```text
anchor_id:byte_offset
```

The tab-separated format is now preferred because universal RelayDB anchors may contain colons, such as:

```text
project:relaydb
function:relay_jump_from
concept:self-documentation-loop
```

The reader supports both anchor fields:

```json
{ "#": "project:relaydb" }
```

and legacy RelayDB data:

```json
{ "#id": "kevin_bacon" }
```

---

## Installation / local usage

This package is currently used directly from the repo:

```js
import RelayDB from "../../packages/relaydb-js/src/index.js";
```

Future versions may be published as an installable package.

---

## Browser usage

Serve the repo locally, for example:

```bash
npx serve .
```

Then in the browser console:

```js
const { default: RelayDB } = await import("/packages/relaydb-js/src/index.js");

const db = await RelayDB.open("/relay-compiler/builds/relaydb-v1-self-docs.relay");

console.log(db.anchors());

const project = db.get("project:relaydb");
console.log(project);

const traversal = db.jump("project:relaydb");
console.log(traversal);

console.log(db.verifyIntegrity());
```

Legacy Bacon Standard artifacts can still be tested if present:

```js
const db = await RelayDB.open("/builds/bacon_standard.relay");

console.log(db.get("kevin_bacon"));
console.log(db.jump("the_terminal", { filter: "Drama" }));
console.log(db.verifyIntegrity());
```

---

## Node.js usage

Make sure your project is ESM-enabled:

```json
{
  "type": "module"
}
```

For local file testing in Node, use `fs.readFileSync()` with `RelayDB.fromBytes()`:

```js
import fs from "node:fs";
import RelayDB from "../../packages/relaydb-js/src/index.js";

const buffer = fs.readFileSync("../../relay-compiler/builds/relaydb-v1-self-docs.relay");

const arrayBuffer = buffer.buffer.slice(
  buffer.byteOffset,
  buffer.byteOffset + buffer.byteLength
);

const db = RelayDB.fromBytes(arrayBuffer);

console.log(db.anchors());

const project = db.get("project:relaydb");
console.log(project);

const traversal = db.jump("project:relaydb");
console.log(traversal);

console.log(db.verifyIntegrity());
```

---

## Example test script

From the repo root, after building the self-documentation artifact:

```bash
make build \
  INPUT=atlas-memory/relaydb_v1_self_documentation.jsonl \
  OUTPUT=relay-compiler/builds/relaydb-v1-self-docs.relay
```

Run:

```bash
node examples/basic-js/test.js relay-compiler/builds/relaydb-v1-self-docs.relay
```

Expected result:

- anchor count prints
- `project:relaydb` resolves
- `function:relay_jump_from` resolves
- traversal from `project:relaydb` returns nodes
- integrity report returns `ok: true`

---

## Core API

### Open a database over HTTP / served path

```js
const db = await RelayDB.open("/path/to/file.relay");
```

---

### Open a database from bytes

```js
const db = RelayDB.fromBytes(arrayBuffer);
```

---

### Lookup by anchor

```js
const node = db.get("project:relaydb");
```

Legacy example:

```js
const node = db.get("kevin_bacon");
```

---

### Check whether an anchor exists

```js
db.has("function:relay_jump_from");
```

---

### List anchors

```js
db.anchors();
```

---

### Get jump-table entries

```js
db.entries();
```

---

### Get address for an anchor

```js
db.getAddress("project:relaydb");
```

---

### Fetch directly at a byte address

```js
db.fetchAt(32);
```

---

### Traverse relationships

```js
const result = db.jump("project:relaydb");

console.log(result.start);
console.log(result.visited);
console.log(result.nodes);
console.log(result.missing);
```

With a filter:

```js
const result = db.jump("project:relaydb", { filter: "compiler" });
```

---

### Integrity check

```js
const ok = db.verifyIntegrity();

if (!ok) {
  console.error("Integrity check failed");
}
```

Detailed report:

```js
const report = db.integrityReport();

console.log(report.ok);
console.log(report.entries);
console.log(report.failures);
```

---

## Notes

- This reader is v1 / v1.2 specific.
- The current implementation uses a text-based jump table.
- Anchor lookup uses an in-memory `Map` after the file is loaded.
- The Rust v1 compiler still writes the text jump table at the end of the `.relay` artifact.
- Future versions may introduce:
  - fixed-width binary index
  - length-prefixed node storage
  - structured result packets
  - stronger graph-health audit output
