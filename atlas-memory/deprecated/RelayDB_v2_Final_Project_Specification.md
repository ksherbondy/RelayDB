# RelayDB v2.0 Final Project Specification
## The Core Standard
### A Compiled Read Layer for Static Relational Knowledge

## 1. Mission

RelayDB is a **compiler-and-runtime system** for **static relational knowledge**.

Its purpose is to eliminate unnecessary runtime database overhead for data that is:
- known at build time,
- structurally related,
- primarily read-only in production,
- and better validated before shipping than rediscovered repeatedly at runtime.

RelayDB is an **optimization layer**, not a primary database. It compiles structured source data into a **portable, trusted, immutable `.relay` artifact** optimized for retrieval and traversal.

## 2. RelayDB Mandate

RelayDB v2.0 exists to answer this question:

**If data is static, relational, and known ahead of time, why should production pay the overhead of live-query database machinery to rediscover structure that could have been compiled once?**

RelayDB answers by shifting correctness, topology, and relationship resolution to build time.

## 3. Non-Goals

To preserve scope and maintain the “steady line” of deterministic behavior, RelayDB v2.0 explicitly excludes:

- **No runtime writes**
- **No transactions**
- **No migrations in production**
- **No query planner**
- **Not a system of record**
- **Not a full-text search engine**
- **Not a general-purpose mutable database**

The source truth remains the authored data. RelayDB is the **compiled truth**.

---

# Part I — Source Contract

## 4. Reference Authoring Format

JSON is the reference source format for RelayDB v2.0.

A Relay source node is a JSON object composed of:
- Relay-reserved fields,
- and ordinary payload fields.

The current project already demonstrates this through normalized `actors`, `directors`, and `movies` datasets using the 4-tag model.

## 5. Reserved Prefixes

RelayDB v2.0 formalizes four reserved source prefixes:

- `#` — **Anchor**
- `^` — **Topic**
- `@` — **Baton**
- `~` — **Metadata**

These are language features of Relay source authoring.

## 6. Anchor Rules

### `#id`
- MUST exist exactly once per node
- MUST be a string
- MUST be unique across the compiled dataset
- SHOULD remain stable across builds unless intentionally renamed

The anchor is the canonical identity of the node.

## 7. Topic Rules

### `^`
- MUST exist exactly once per node
- MUST be a string in v2.0
- identifies the node’s primary classification

Examples:
- `^: "actors"`
- `^: "movies"`
- `^: "directors"`

## 8. Baton Rules

Fields beginning with `@` define traversable graph relationships.

A baton field:
- MAY be a scalar string anchor
- MAY be an array of string anchors
- MUST resolve during compilation unless explicitly marked optional by future schema policy

Examples in the current dataset:
- `@director` as scalar
- `@cast` as array

Unresolved non-optional batons are **fatal build errors**.

## 9. Metadata Rules

Fields beginning with `~` define descriptive, filterable, or grouping metadata.

A metadata field:
- MAY be scalar or array
- MUST NOT be traversed as an edge
- MAY be used by filters, UI, context assembly, and diagnostics

Examples:
- `~genres`
- `~style`

## 10. Ordinary Payload Fields

Any non-reserved field is preserved as user payload.

Examples:
- `name`
- `bio`
- `release_year`

---

# Part II — Compiler and Build Contract

## 11. Compiler Philosophy

The compiler is not just a converter. It is the **trust gate**.

RelayDB v2.0 requires the compiler to behave more like a systems compiler than a data import script:
- precise failures
- exact blame
- linting
- topology validation
- artifact verification before acceptance

The current compiler already performs ingestion, graph analysis, cycle checking, artifact generation, and binary bake orchestration.

## 12. Build Pipeline

RelayDB v2.0 build stages:

1. **Extract** — read source files
2. **Parse** — decode JSON source
3. **Normalize** — canonicalize node structure
4. **Validate** — enforce schema and graph rules
5. **Analyze** — compute graph and audit data
6. **Compile** — emit `.relay`
7. **Verify** — validate produced artifact
8. **Emit Reports** — produce optional auxiliary artifacts

## 13. Required Build Validation

The compiler MUST validate:
- duplicate anchors
- missing required fields
- invalid reserved prefix types
- unresolved batons
- topic validity
- cycle-policy compliance
- schema violations
- malformed source documents

The current implementation already aborts on detected cycles before baking.

## 14. Compiler Diagnostics

Compiler diagnostics SHOULD include:
- source file path
- anchor ID
- offending field
- severity
- human-readable explanation
- suggested fix when practical

This is a first-class product requirement.

## 15. Build Outputs

A successful build MAY emit:
- `.relay` artifact
- audit report
- graph artifact
- stats report
- warnings report
- checksum manifest

The current compiler already emits audit markdown and graph DOT artifacts.

---

# Part III — Binary Contract

## 16. Artifact Philosophy

Runtime retrieval MUST operate from the compiled `.relay` artifact itself, not from the original JSON sources.

This matches the current architecture, where source JSON is baked into the artifact and runtime reads occur from the compiled file.

## 17. Endianness

All numeric fields MUST be little-endian.

## 18. File Layout

The `.relay` artifact is a contiguous binary blob composed of:

1. **Header**
2. **Binary Index**
3. **Node Store**
4. **Integrity Manifest**
5. **Reserved / Extension Space**

This reflects the converged v2 direction from the unified spec and Gemini’s binary-layout draft.

## 19. Header

RelayDB v2.0 defines a **64-byte header**.

### Header Layout

| Offset | Type | Field | Description |
|---|---:|---|---|
| `0x00` | `u32` | MagicBytes | ASCII `RELY` |
| `0x04` | `u16` | VersionMajor | `2` |
| `0x06` | `u16` | VersionMinor | `0` |
| `0x08` | `u64` | NodeCount | Total compiled anchors |
| `0x10` | `u64` | IndexOffset | Start of binary index |
| `0x18` | `u64` | DataOffset | Start of node store |
| `0x20` | `u64` | ManifestOffset | Start of integrity manifest |
| `0x28` | `u16` | Flags | Encoding/compression flags |
| `0x2A` | `u16` | HeaderSize | Header size in bytes |
| `0x2C` | `[20]u8` | Reserved | Future expansion |

The reserved block is intentionally preserved for forward compatibility.

## 20. Magic and Versioning

- Magic MUST be `RELY`
- Major version mismatch MUST cause reader rejection
- Minor version mismatch MAY be accepted if backward-compatible policy allows

## 21. Binary Index

The v2 index replaces the current text-based jump table with a **collision-safe sorted binary index**. The current v1.1 library still resolves addresses by scanning a textual `id:offset` table, which is acceptable for the PoC but not the v2 contract.

### Index Entry Layout

Each index entry SHALL contain:

- `Hash (u64)` — fast first-pass comparison
- `KeyLen (u16)` — anchor string length
- `KeyBytes (variable)` — exact anchor bytes for collision-safe verification
- `Address (u64)` — absolute offset to node block
- `PayloadLen (u32)` — bounded node length

### Index Rules
- entries MUST be sorted
- lookup MUST use exact-match verification after hash comparison
- collisions MUST NOT be resolved by hash alone

## 22. Node Store

Nodes are stored as **length-prefixed blocks**, replacing the current null-terminated framing model. This is a deliberate v2 upgrade for safe bounds checking, easier mmap support, and cleaner corruption handling.

### Node Block Layout

- `Checksum (u32)` — CRC32 of payload
- `BodyLen (u32)` — payload byte length
- `Payload (bytes)` — serialized node body

## 23. Payload Encoding

For v2.0, the reference payload encoding is **serialized canonical JSON bytes**.

Future versions MAY add alternate encodings behind header flags.

## 24. Integrity Manifest

The manifest section MAY contain:
- artifact checksum
- per-node checksums
- section checksums
- bake metadata
- verifier summary data

---

# Part IV — Runtime Contract

## 25. Runtime Philosophy

RelayDB runtime is intentionally small and disciplined.

It is not a general query engine.  
It is a retrieval and traversal layer for compiled relational artifacts.

## 26. Core Runtime Primitives

RelayDB v2.0 standardizes four primary operations:

- `lookup(id)`
- `traverse(id, options)`
- `context(id, options)`
- `verify()`

These four operations are the canonical handshake between the artifact and consuming applications.

## 27. `lookup(id)`

Resolves a single anchor and returns its node payload.

Behavior:
- search binary index
- verify exact key match
- load node block
- validate checksum if enabled
- decode payload
- return structured node packet

## 28. `traverse(id, options)`

Follows `@` batons recursively or iteratively.

Traversal MUST support:
- visited protection
- depth limit
- node-count limit
- byte budget
- topic filtering
- metadata filtering
- warning emission

The current runtime already follows batons recursively with a visited set.

## 29. `context(id, options)`

Returns a **structural context packet** assembled from the requested node and its relevant neighborhood.

This operation is intended for:
- UI/API payload assembly
- frontend consumption
- local knowledge bundles
- RAG pre-pruning

Structural pruning belongs in the Relay runtime; semantic ranking remains outside the core.

## 30. `verify()`

Runs a full integrity scan of the artifact and returns a structured report.

## 31. Structured Result Packets

Runtime MUST return **structured result packets**, not terminal-oriented text, when used as a library or service.

A result packet SHOULD include:
- requested anchor
- resolved nodes
- traversed edges
- visited count
- warnings
- byte counts
- truncation reason
- integrity notices

This is a deliberate improvement over the current demo pattern, where an Express wrapper shells out to the CLI and returns stdout.

---

# Part V — Integrity Contract

## 32. Defensible Trust

RelayDB v2.0 defines trust as **tiered and explicit**.

The guarantee is:

**If it bakes and passes verification, it is trustworthy within the guarantees defined by this spec.**

## 33. Integrity Tiers

### 33.1 Structural — CRITICAL
Checks the physical validity of the artifact:
- magic bytes
- version compatibility
- header sanity
- offset bounds
- readable index
- valid node framing
- manifest bounds

Failure is **fatal**. The engine refuses to ignite.

### 33.2 Semantic — WARNING
Checks localized node trust:
- checksum mismatch
- payload decode failure
- anchor mismatch between index and payload
- malformed node content

Affected nodes SHOULD be quarantined.

### 33.3 Graph — ADVISORY
Checks logical graph issues:
- broken batons
- forbidden cycles
- metadata drift
- graph inconsistencies

These SHOULD be reported in build or verification output.

## 34. Verification Modes

RelayDB SHOULD support:

### Strict Mode
Any structural or semantic failure invalidates the artifact.

### Operational Mode
Structural failure invalidates the artifact. Localized semantic failures may be quarantined while healthy branches remain traversable.

## 35. Verifier Report

Verifier output SHOULD include:
- structural status
- semantic status
- graph status
- quarantined node count
- broken baton count
- warning count
- summary metadata

## 36. Error Model

Core runtime APIs MUST return structured errors.

Recommended classes:
- `ArtifactOpenError`
- `ArtifactFormatError`
- `AnchorNotFound`
- `PayloadDecodeError`
- `ChecksumFailure`
- `BrokenBaton`
- `TraversalLimitExceeded`
- `PolicyViolation`

The v2 goal is to replace panic-first control flow with structured error handling in core paths. The current v1.1 library still uses `unwrap()` and `expect()` in several places.

---

# Part VI — Tooling Contract

## 37. CLI Surface

RelayDB v2.0 SHOULD expose:

- `relay build`
- `relay check`
- `relay lint`
- `relay inspect <anchor>`
- `relay lookup <anchor>`
- `relay traverse <anchor>`
- `relay context <anchor>`
- `relay graph`
- `relay stats`

The current CLI already provides `jump` and `check` as the early form of this interface.

## 38. Linting

Relay linting SHOULD detect:
- duplicate anchors
- dead batons
- inconsistent metadata shapes
- suspicious authoring patterns
- cycle violations
- schema drift

## 39. Explainability Artifacts

Relay tooling SHOULD generate:
- audit reports
- graph visualizations
- stats summaries
- integrity reports

The current build pipeline already demonstrates the usefulness of this direction.

---

# Part VII — Integration Contract

## 40. Integration Surfaces

RelayDB v2.0 MUST be consumable through multiple surfaces:

- Rust core library
- CLI
- service wrapper
- language bindings

Potential bindings:
- Node
- Python
- C ABI
- WASM later

## 41. Service Mode

Service wrappers SHOULD return structured JSON packets and MUST avoid shell-interpolated runtime input in production.

The current React proof-of-concept demonstrates strong developer value, but the current backend integration is prototype-grade and not the desired long-term production shape.

## 42. Topic Neutrality

Relay core MUST remain topic-neutral.

Hardcoded domain assumptions like `movies`, `actors`, and `directors` belong in demo datasets and tooling examples, not in the runtime contract. The current graph generator still contains example-specific cluster assumptions and should be generalized in v2.

---

# Part VIII — Performance and Proof Contract

## 43. Performance Language

RelayDB SHOULD use precise language:
- direct offset-based retrieval
- sorted binary index lookup
- compiled structural traversal
- reduced runtime glue code for static relational data

Relay SHOULD avoid unqualified claims like:
- “database killer”
- “O(1) for everything”
- “faster than all databases”

until benchmarked.

## 44. Benchmark Harness

RelayDB v2.0 SHOULD benchmark against:
- flat JSON scan
- SQLite
- RelayDB v1.1
- RelayDB v2.0

Metrics SHOULD include:
- cold lookup latency
- warm lookup latency
- memory footprint
- compile time
- artifact size
- traversal throughput
- service response time

This is required for measured credibility.

---

# Part IX — Compatibility and Future-Proofing

## 45. Source Compatibility

The 4-tag authoring model is preserved in v2. Existing datasets should remain conceptually compatible even though the artifact format evolves.

## 46. Artifact Compatibility

v2 `.relay` files need not be binary-compatible with v1.1 artifacts.

Migration tools MAY be provided.

## 47. Forward Compatibility

The header reserved block and flags field exist to support:
- alternate payload encodings
- compression flags
- manifest evolution
- future extension records

---

# Relay Philosophy Statement

**If data is static, relational, and known at build time, correctness and structure should be enforced before shipping, not rediscovered repeatedly at runtime.**

That is the RelayDB v2.0 project standard.

---

# Immediate Next Deliverables

With the project spec locked, the next artifacts should be:

1. **Binary Appendix**
   - exact code-adjacent header/index/node structs

2. **Diagnostics Appendix**
   - compiler and verifier output schema

3. **Structured Result Packet Schema**
   - canonical JSON/API payload contract

4. **v2 Prototype Reader/Writer**
   - smallest compliant artifact builder and reader

5. **Benchmark Harness**
   - v1.1 vs v2 vs JSON vs SQLite

This is the final project spec.
