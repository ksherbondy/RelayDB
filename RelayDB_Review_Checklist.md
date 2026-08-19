# RelayDB Review Checklist

## Review Snapshot

- **Repository:** Rust compiler/runtime with archived JavaScript compatibility material and project-memory workflow.
- **Purpose:** Compile static JSON/JSONL relational memory into a read-only `.relay` artifact, verify it, and retrieve/traverse nodes.
- **Snapshot:** branch `test`, commit `f037b2978d0396370fe93f584845261450ca8040`, commit date `2026-06-15T18:56:16-05:00`. Review run: `2026-08-19`.
- **Scope identity:** This is not a website or SQL/NoSQL extractor. It contains the Rust compiler/runtime, CLI tools, archived compatibility material, and project-memory sources.
- **Status:** v1.2 proof of concept is operational. v2 is primarily specified/documented; `relay-compiler/src/v2_protocol.rs` is isolated and not wired into the crate or build CLI.

## Project Identity

- **Implemented:** JSON/JSONL ingestion, configurable output, v1 `.relay` writing, physical integrity checking, recursive `@` traversal, Markdown/DOT audit artifacts, Rust source-memory audit, and the Rust reader API.
- **Planned/experimental:** v2 binary format, binary collision-safe index, manifest, structured runtime packets, `lookup`/`context` APIs, limits, mmap, bindings, service mode, lint CLI, merge-memory, and patch generation.
- **Observed commands:** `make test`, `make fmt-check`, `make all`, and `make memory-audit` are the active Rust workflow. `make merge-memory` and `make patch-next` are intentional TODO failures.

## Architecture

- **Source layer:** `.json` accepts one object or an array; `.jsonl` accepts one object per non-empty line. `data/` is legacy/demo data; `atlas-memory/` is project-memory source.
- **Compiler:** `relay-compiler/src/bin/compiler.rs` collects files, parses them, extracts anchors/links, warns or fails on cycles, emits Markdown/DOT, then writes the v1 artifact.
- **Shared library:** `relay-compiler/src/lib.rs` exposes v1 constants, anchor/link helpers, text-index lookup, payload fetch, integrity verification, traversal, node writing, and memory audit.
- **CLI/demo binaries:** `relay` provides `jump`, `check`, and `audit-memory`; `reader` is a fixed-anchor demo; `verify` is a standalone check wrapper.
- **Production reader:** `relay-compiler/src/reader.rs` loads compiled artifacts and exposes lookup, traversal, query, pagination, and hydration APIs.
- **Data flow:** JSON/JSONL -> `serde_json::Value` -> graph analysis/audit artifacts -> 32-byte-header `.relay` -> Rust reader.
- **Important invariants:** anchors should identify nodes; jump-table offsets must point to matching payload anchors; `@` traversal uses a visited set; numeric pointer data is little-endian; source files are sorted before compilation.
- **Repository relationship:** No separate RelayDB repositories are wired into this workspace. The README/specification and `AI_RELAYDB_WORKFLOW.md` describe the broader project concept; this checkout is the implementation/proof repository.

## 4-Tag Contract

- `#` is the canonical anchor in current project memory; legacy `#id` is also accepted.
- `^` is documented here as provenance/topic context. The v1 compiler/audit treats string and array values under `^` as graph references, while the runtime does not traverse them. The v2 specification instead defines `^` as a single string topic, so the contracts are not currently aligned.
- `@` is the traversable baton/relationship field. Runtime traversal supports scalar strings and arrays of strings.
- `~` is metadata/aliases. It is preserved as payload but not validated, indexed, or traversed.
- **Legacy support:** `#id` anchors and colon-separated legacy jump-table lines are supported alongside `#` and tab-separated lines.
- **Validation actually implemented:** JSON/JSONL parseability, object records, presence of anchors in memory audit, duplicate reporting, missing internal-reference reporting, cycle reporting, and payload-anchor matching during artifact verification.
- **Validation missing from compilation:** anchor type/uniqueness, exactly-one anchor, topic type, reserved-field shapes, baton type, metadata shape, schema mapping, and reference resolution are not enforced by `compiler`; duplicate IDs are inserted into a `HashMap` and can overwrite one another.
- **Cycles:** allowed by default with a warning; `--strict-acyclic` makes detected internal graph cycles fatal. Runtime visited sets prevent infinite recursive traversal.

## `.relay` Format

- **Produced/read:** the active compiler and Rust reader use the provisional v1 artifact. Archived JavaScript code is not part of the active build. `v2_protocol.rs` describes a separate v2.1-style format but is not compiled as a module.
- **v1 layout:** 32 zero-filled header bytes; minified UTF-8 JSON payloads; a null byte after each payload; a text jump table at EOF; a little-endian `u64` index offset at header byte 16.
- **Sections:** no declared node count, data offset, payload lengths, manifest, alignment policy, or reserved extension protocol. The index is `anchor<TAB>absolute_offset` lines.
- **Lookup:** Rust `get_address_from` scans the entire text jump table linearly for each lookup. JS parses the whole table once into a `Map`, then performs constant-time map lookup, but the artifact bytes and table are loaded/copied in memory.
- **Hydration/storage:** no mmap. Rust opens/seeks/reads each request; JS holds the complete artifact bytes and copies each fetched payload before JSON decoding.
- **Integrity:** v1 verification checks each indexed payload parses and its `#`/`#id` matches the indexed anchor. It does not validate magic/version/header bounds, index bounds, duplicate index entries, payload lengths, checksums, truncation beyond parse failure, or stale-source provenance.
- **Compatibility:** legacy `#id` and colon-separated table parsing exist, but there is no v1-to-v2 migration, version negotiation, or reader compatibility layer for the v2 binary format. The v2 spec itself says 2.0 while the isolated protocol module declares 2.1.

## Extraction and Compilation

- **Inputs:** JSON and JSONL only. No SQL, NoSQL, database connector, schema introspection, or generic extraction adapter exists.
- **Mapping:** input objects are preserved as JSON; reserved prefixes are interpreted by convention. There is no source-specific normalization layer beyond object/array and JSONL parsing.
- **Relationship validation:** graph edges are collected and cycles are checked, but unresolved batons are not fatal in the compiler. The separate memory-audit command can report missing `@`/`^` references.
- **Determinism:** supported input files are sorted and the jump table is sorted by anchor. Same source should produce the same v1 payload/index bytes, but timestamped Markdown/DOT names and local environment metadata make the overall build directory non-reproducible.
- **Transformations:** generic JSON serialization, graph extraction, audit generation, and binary framing; no SQL/NoSQL-specific transformations are present.

## Reader and APIs

- **Rust public APIs:** anchor extraction, link extraction, address lookup, payload fetch, jump-table parsing, integrity verification, recursive terminal traversal, node writing, and memory audit. Traversal is print-oriented (`()`), not a structured library result.
- **JavaScript APIs:** `open`, `fromBytes`, `get`, `has`, `anchors`, `entries`, `getAddress`, `fetchAt`, `size`, `jump`, `verifyIntegrity`, and `integrityReport`.
- **Language access:** Rust and JavaScript only. No FFI, C ABI, Python binding, WASM package, or service API is implemented in this checkout.
- **Missing behavior:** no lazy mmap hydration, depth/node/byte limits, topic/metadata filter semantics, context packet, structured Rust errors, or machine-readable CLI result protocol.
- **Missing anchors:** Rust prints a warning and returns; JS lookup returns `null`; JS traversal records missing IDs in its result. Malformed payloads generally become I/O/parse errors or thrown JS exceptions.

## Performance

- **Measured in this review:** no representative artifact-size, compile/open-time, memory, lookup-latency, traversal-latency, or baseline benchmark suite exists.
- **Known cost:** Rust direct lookup is O(n) over index lines and repeatedly opens/scans the file. Traversal performs repeated linear lookups. JS improves repeated lookup after opening by retaining a `Map`, at the cost of loading the full artifact.
- **Indexes/caches:** sorted text table, JS in-memory `Map`, and a visited set only. No typed lanes, caches, bitsets, binary search index, or mmap.
- **Reproducibility:** benchmark results are not currently reproducible because they are not recorded or automated.

## Testing and Verification

- **Commands:** `make test`, `make fmt-check`, `make clippy`, `make build`, `make verify`, `make inspect`, `make memory-audit`, and `make strict` exist. Unit tests cover source validation, profiling, logical compilation, reader behavior, helpers, cycle detection, and audit construction.
- **Current results:** six library tests and two compiler tests pass; formatting passes; core build/verify/JS smoke/memory audit pass; Clippy fails on one lint.
- **Fixtures:** checked-in `.relay` artifacts exist, but there is no explicit golden-byte fixture test or source/artifact consistency test.
- **Coverage gaps:** malformed/truncated/incompatible artifacts, duplicate compiler anchors, missing compiler references, invalid tag types, cycles as runtime fixtures, large files, boundary offsets, repeated builds, and source-vs-reader cross-checks are not comprehensively tested.

## Documentation and Project Memory

- **Authoritative candidates:** `README.md` for v1 usage, `RelayDB_v2_Final_Project_Specification.md` for intended v2 contract, Rust/JS source for actual behavior, `Makefile` for commands, and `atlas-memory/relaydb_v1_self_documentation.jsonl` for project-memory source truth.
- **Generated artifacts:** `relay-compiler/builds/*.md`, `*.dot`, PNGs, and `.relay` outputs. Existing artifacts are not automatically tagged with source commit, source hash, compiler version, or schema version.
- **Decisions/provenance:** design decisions and risks are recorded in JSONL, with timestamps and branch references in some records. The memory also contains stale claims, including prior `make stable` evidence and branch names that do not match this review snapshot.
- **Agent/project memory:** `AI_RELAYDB_WORKFLOW.md` describes the workflow; no separate executable agent profile format is present. Project memory is separate from source code but not cryptographically tied to it.
- **Compilation/verification:** the self-documentation JSONL compiles and verifies; current run contained 80 nodes, 361 references, no duplicates/missing refs, and 179 reported cycles.
- **Staleness detection:** warnings and audits exist, but no source hash, generated-file manifest, freshness check, or CI gate detects stale summaries/artifacts.

## Additional Findings and Priorities

### Findings

1. **High: compiler does not enforce identity or graph safety.** Duplicate anchors overwrite the jump-table map while all duplicate payloads are still written. Missing references and invalid reserved-field types do not fail compilation. This conflicts with the v2 trust-gate requirements.
2. **High: active artifact format is incompatible with the v2 contract.** The shipped path uses a 32-byte header, text index, null termination, and no manifest; v2 requires a 64-byte header, binary index, length-prefixed/checksummed nodes, and version validation.
3. **High: malformed artifacts are weakly bounded.** v1 readers trust the header pointer and offsets, scan until EOF for a terminator, and lack payload-size/checksum bounds. Treating untrusted `.relay` files as input can cause excessive reads or parsing work.
4. **Medium: quality gate is red.** `make clippy` fails at `relay-compiler/src/bin/compiler.rs:262` under `-D warnings`. The core pipeline does not include formatting or Clippy, so `make all` can pass while the documented lint gate fails.
5. **Medium: lookup scales linearly in Rust.** The jump table is scanned for every lookup, and traversal repeats that scan. This is the clearest current performance bottleneck.
6. **Medium: build outputs are not reproducible as a release set.** Timestamped reports, unrecorded compiler/source versions, and no source/artifact manifest make provenance and stale-artifact detection difficult.
7. **Medium: graph artifact generation does not escape DOT identifiers.** Anchors and relationships are interpolated into quoted DOT strings without escaping. Untrusted source values could corrupt the graph or inject DOT directives.
8. **Low: runtime contract is terminal-oriented.** Rust traversal prints warnings and nodes and returns no structured result; the v2 specification’s result packets and error classes are not implemented.

### Recommended next priorities

1. Make compilation reject duplicate anchors, missing required anchors, invalid reserved-field shapes, and unresolved non-external batons; add focused tests for each.
2. Choose and wire one artifact contract. Either formally freeze v1 or implement v2 end to end, including magic/version/bounds checks, binary index, length framing, checksums, and migration policy.
3. Fix the Clippy failure and make `make all` run `fmt-check`, Clippy, artifact verification, and JS smoke as one CI gate.
4. Add golden artifacts, malformed-file tests, large/boundary tests, determinism checks, and source-to-reader consistency tests.
5. Replace Rust linear lookup with a bounded binary index or a loaded index map, then add reproducible benchmarks.
6. Add source/compiler hashes and artifact metadata, and escape DOT output.

### Open questions

- Is v1.2 the intended release contract, or is v2.0/v2.1 now the target?
- Should `^` be a topic scalar, provenance relation, or both with an explicit compatibility rule?
- Are cycles valid for all RelayDB datasets or only project-memory artifacts?
- Are unresolved `@` references allowed, and how are external references declared canonically?
- Which runtime surfaces are required for release: Rust library, CLI, JS package, service, FFI, or WASM?