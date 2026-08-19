# RelayDB Rust Port Handoff

This directory is the JavaScript behavioral reference for the first Rust RelayDB implementation. Read this file and `REFERENCE_FORMAT_V1.md` before changing the Rust repository.

## Project Purpose

RelayDB compiles static relational JSON/JSONL into a portable read-only artifact. The goal is to move relationship resolution, validation, profiling, and indexing to build time so runtime readers can select and hydrate only the data an application requests.

```text
JSON/JSONL
  -> validate and profile
  -> compile once
  -> .relay artifact
  -> language-specific reader
  -> simple queries and hydrated results
```

The original developer problem is many related JSON files requiring repeated runtime loading, lookup maps, relationship assembly, and display transformations. RelayDB should reduce that work to a small reader API.

## Canonical Source Semantics

```text
# or #id  record identity; exactly one form
^          collection/type
@name      named relationship or foreign key
~name      searchable/indexed field
ordinary   payload field
```

Preserve developer-owned IDs publicly. Internal integer record indexes are allowed and preferred for compiled relationships.

Reference V1 rules:

- Non-empty identity is required.
- A record cannot contain both `#` and `#id`.
- `^` is required and is a non-empty collection string.
- IDs are unique within a collection and may repeat across collections.
- Unqualified ambiguous lookup is an error.
- `@` values are null, scalar string, or string array.
- Null does not establish relationship cardinality.
- Scalar/array cardinality must be consistent within a collection relationship field.
- Missing, null, false, zero, and empty string are distinct logical values.
- Integers must remain within the JavaScript safe integer range for Reference V1.

## Reader Behavior

The public behavior is:

```js
const db = await RelayDB.open("data.relay");
db.get(id, options?)
db.has(id, options?)
db.query(criteria, options?)
db.queryPage(criteria, options?)
db.selectIds(criteria?)
db.iterate(criteria, options?)
db.close()
```

- `get(id, { depth: 0 })` returns the record with relationship IDs.
- Deeper `get` calls hydrate using compiler-resolved integer relationship indexes.
- `queryPage` returns bounded results and pagination metadata.
- Projection always retains `#id` and `^` and returns requested fields only.
- Runtime should avoid materializing every match by default.
- Browser readers may run through a Web Worker with the same logical API.

Query criteria use object syntax:

```js
{
  "^": "movies",
  "~genres": "Crime",
  "~release_year": { gte: 2000, lt: 2010 }
}
```

Operators are `eq`, `ne`, `gt`, `gte`, `lt`, and `lte`. String equality is case-insensitive and `ne` is its logical inverse. Array equality matches when any element matches.

## What Has Been Proven

- Generic JSON/JSONL/directory ingestion works.
- Invalid relationships block artifact generation.
- Mixed relationship cardinality is rejected.
- Presence-aware lanes preserve null versus missing values.
- Exact source-to-reader round trip passes for the 151,676-record generated fixture.
- Relationship hydration works through cycles with depth limits.
- Projected pages, pagination, device-aware batches, and async iteration work.
- Browser worker reads work through the unified client smoke test.
- The master compile/verify/test workflow passes.

Run the JavaScript reference checks:

```bash
node scripts/relay-master/reference-v1-tests.js
node scripts/relay-master/test.js
node scripts/relay-master/compile.js misc/stuff builds/master/movies.relay
node scripts/relay-generic/v6-verify.js datasets/generated/merged/people-companies.10000x100000.4tag.merged.jsonl builds/generic-versions/reference-v1-large.relay
```

## What Is Not Frozen

Do not treat the current physical bytes as the final Rust artifact contract yet. The following remain provisional:

- binary record directory
- binary section directory
- dictionary encoding layout
- checksum sections
- release artifact without duplicated source payload
- independent cross-language parser
- exact on-disk version/magic assignment

The current JS files are laboratory equipment and executable behavioral reference. Do not port their accidental JSON metadata layout directly into Rust.

## Rust Port Order

Start with logical behavior, not bytes:

1. `relay-core`: source record model, identity, collections, relationships, diagnostics, validation.
2. `relay-profile`: field statistics, nullability, cardinality, numeric ranges, encoding plans.
3. `relay-compiler`: grouped records, remapped relationship indexes, compiled logical model.
4. Port the compatibility fixtures and diagnostic expectations.
5. Cross-check JS and Rust logical results.
6. Define the final physical artifact format independently.
7. Implement the Rust writer and reader.
8. Add JS/Rust artifact compatibility tests.

Do not reopen settled source semantics unless an executable correctness problem requires it. Put compression, SIMD, bitmap variants, cost-based planning, adaptive prefetch, and other performance ideas on a post-V1 optimization backlog.

## Repository Handoff

Copy the entire `scripts/relay-master/` directory into the Rust repository as a clearly labeled JavaScript reference, for example:

```text
reference/relay-master/
```

Keep it separate from Rust production code. The important files are:

- `REFERENCE_FORMAT_V1.md`
- `RUST_PORT_HANDOFF.md`
- `fixtures/`
- `reference-v1-tests.js`
- `README.md`

The compatibility corpus is the source of truth for cross-language behavior:

```text
JS compiler -> JS reader
JS compiler -> Rust reader
Rust compiler -> JS reader
Rust compiler -> Rust reader
```

The Rust implementation is successful when these paths agree on logical records, relationships, query results, diagnostics, and edge-case semantics.
