# RelayDB v1 JavaScript Reader

This is a JavaScript reader for the current RelayDB v1 `.relay` artifact format.

It allows browser and Node.js environments to load a compiled `.relay` file and perform direct lookups and graph traversal.

> This reader is part of the RelayDB v1 prototype and is not intended for production use.

---

## Supported v1 format

- 32-byte header  
- jump-table pointer stored as `u64 little-endian` at byte offset `16`  
- null-terminated UTF-8 JSON node payloads  
- text jump table at the end of the file using `anchor_id:byte_offset`  

---

## Installation (local usage)

This package is currently used directly from the repo:

```js
import RelayDB from "../../packages/relaydb-js/src/index.js";
```

Future versions may be published as an installable package.

---

## Browser usage

Serve the repo locally (for example with `npx serve .`), then in the browser console:

```js
const { default: RelayDB } = await import("/packages/relaydb-js/src/index.js");

const db = await RelayDB.open("/builds/bacon_standard.relay");

console.log(db.get("kevin_bacon"));
console.log(db.jump("the_terminal"));
console.log(db.verifyIntegrity());
```


<img src="Relay-Served.png" width="500" alt="RelayDB logo" />
<img src="Relay-Served-HTTP.png" width="500" alt="RelayDB logo" />



---

## Node.js usage

Make sure your project is ESM-enabled:

```json
{
  "type": "module"
}
```

Then:

```js
import RelayDB from "../../packages/relaydb-js/src/index.js";

const db = await RelayDB.open("./builds/bacon_standard.relay");

console.log(db.anchors());
console.log(db.get("kevin_bacon"));
```

---

## Core API

### Open a database

```js
const db = await RelayDB.open("/path/to/file.relay");
```

---

### Lookup by anchor

```js
const node = db.get("kevin_bacon");
```

---

### List anchors

```js
db.anchors();
```

---

### Traverse relationships

```js
const result = db.jump("the_terminal", { filter: "Drama" });

console.log(result.visited);
console.log(result.nodes);
```

---

### Integrity check

```js
const ok = db.verifyIntegrity();

if (!ok) {
  console.error("Integrity check failed");
}
```

---

## Notes

- This reader is **v1-specific**
- The current implementation uses a text-based jump table
- Future versions (v2) will introduce:
  - fixed-width binary index
  - structured node storage
  - improved lookup performance
