# RelayDB Master Prototype

This directory is the consolidated prototype surface. Older experiments remain under `scripts/relay-generic/` and `scripts/relay-columnar/` as research history; new integration work should target this directory.

## Architecture

```text
JSON/JSONL
  -> compiler validation and relationship resolution
  -> profiled columnar .relay artifact
  -> Node reader or browser worker reader
  -> indexed selection, projected pages, lazy hydration
```

## Node usage

```js
const { RelayDB } = require("./scripts/relay-master");

const db = RelayDB.open("builds/generic-versions/v6-large.relay");
const page = db.queryPage(
  { "^": "person", "~status": "active" },
  { batch: "auto", fields: ["name", "~age", "~status"] },
);

const detail = db.get("person-id", {
  collection: "person",
  depth: 1,
});
```

## Browser usage

```js
import RelayDB from "./scripts/relay-master/browser-client.js";

const db = await RelayDB.open("/data/app.relay");
const page = await db.queryPage(
  { "^": "movies", "~genres": "Crime" },
  { pageSize: 20, fields: ["name", "release_year"] },
);
```

## Current contract

```text
# / #id = record identity
^         = collection/type
@name     = named relationship or foreign key
~name     = searchable/indexed field
ordinary  = payload field
```

The current master prototype uses the validated V6 reader and the profiler-driven columnar writer. The artifact version and binary directory still need to be formally frozen before release.

Reference status: source semantics, reader behavior, query behavior, presence/null behavior, and round-trip expectations are substantially frozen for the Rust port. The physical artifact remains provisional until the binary record directory, checksum sections, and independent format parser are complete.

## Reproducible checks

```bash
node scripts/relay-master/test.js
node scripts/relay-master/reference-v1-tests.js
node scripts/relay-master/verify.js builds/master/movies.relay
node scripts/relay-master/benchmark.js
```
