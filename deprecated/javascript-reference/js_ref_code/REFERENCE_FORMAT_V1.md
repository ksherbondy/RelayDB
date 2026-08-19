# RelayDB Reference V1

**Status:** Behavioral contract substantially frozen. Physical artifact requirements remain provisional until binary record directories, checksums, and an independent parser are complete.

This document is the implementation contract for the first Rust-portable RelayDB version. Experimental V1-V7 layouts remain research history; new readers and compilers target this contract.

## Scope

RelayDB V1 is a build-time compiler and read-only runtime for static relational JSON/JSONL. Compilation work is excluded from runtime benchmarks.

```text
JSON/JSONL -> validate/profile/compile -> .relay
.relay -> open/select/project/hydrate
```

SQL/NoSQL extraction is a separate layer whose output is JSON/JSONL.

## Source Contract

Records are JSON objects. Input may be a JSON object, JSON array, JSONL file, or directory of `.json`/`.jsonl` files.

```text
# or #id  identity; exactly one form per record
^          collection/type; non-empty string
@name      named relationship; string, array of strings, or null
~name      named searchable/indexed field
ordinary   payload field
```

The compiler preserves developer-owned IDs. It may assign internal integer record indexes, but those indexes never replace public IDs.

### Required rules

- Every record requires a non-empty string `#` or `#id`.
- A record must not contain both `#` and `#id`.
- Every record requires a non-empty string `^`.
- IDs must be unique within a collection.
- IDs may repeat across collections; unqualified lookup of an ambiguous ID is an error.
- Relationship values must be `null`, a string, or an array of strings.
- Relationship fields must have one cardinality per collection: scalar or array. `null` does not establish cardinality; non-null values do. Mixed cardinality is invalid in V1.
- Relationship targets must resolve to a record in the compiled input.
- Missing ordinary fields are valid.
- `null`, missing, `false`, `0`, and `""` are distinct logical values.
- JSON numbers must be finite and integers must be within the JavaScript safe integer range. V1 readers must not silently lose numeric precision.

## Target V1 Artifact Requirements (Physical Layout Not Yet Frozen)

The V1 artifact is immutable and little-endian. All multibyte integers and floats are explicitly encoded little-endian; host typed-array byte order must never be written directly.

The artifact must contain:

```text
fixed header
version and feature flags
validated section directory
collection/record directory
string dictionaries
relationship offset/value tables
typed scalar lanes and validity information
search indexes
optional cold payload blocks
checksums
```

Every section has an offset and byte length. Readers validate all bounds before reading section contents. Unsupported versions and malformed sections are fatal.

Debug artifacts may retain source payloads for semantic verification. Release artifacts may omit duplicated source JSON once lane reconstruction is proven equivalent.

## Reader Contract

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

`get()` returns a clean object with `#id` and `^`, preserving scalar/array relationship shape. `depth: 0` exposes relationship IDs; deeper reads hydrate using compiled integer relationship indexes. `fields` projects requested fields while always retaining identity and collection.

`queryPage()` returns:

```text
results, total, offset, limit, hasMore, nextOffset, next()
```

The default runtime should hydrate a bounded page, not every match. Device-aware batch sizing and worker execution are reader policies, not artifact semantics.

## Query Contract

Object criteria are authoritative:

```js
{
  "^": "movies",
  "~genres": "Crime",
  "~release_year": { gte: 2000, lt: 2010 }
}
```

Supported operators are `eq`, `ne`, `gt`, `gte`, `lt`, and `lte`. Equality is case-insensitive for strings; `ne` is its logical inverse. Array fields match equality when any element matches.

## Verification Levels

```text
relay verify artifact.relay
  structural: header, version, bounds, sections, indexes, checksums

relay verify artifact.relay --source data/
  semantic: reconstruct every record and compare with source
```

A V1 compiler must never publish an artifact after a fatal validation or verification error. Writes are temporary followed by verification and atomic rename.

## Portability Requirements

The JS reference and Rust implementation must agree on:

- IDs and collection scoping
- null/missing behavior
- scalar/array cardinality
- numeric widths and overflow behavior
- UTF-8 strings
- little-endian encoding
- relationship traversal and cycle handling
- query operator semantics
- malformed artifact errors

The compatibility corpus is the source of truth for cross-language behavior.
