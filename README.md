<table>
  <tr>
    <td width="260" valign="top">
      <img src="RelayDB-Logo.png" width="220" alt="RelayDB logo" />
    </td>
    <td valign="middle">
      <h1>RelayDB</h1>
      <p><strong>Universal 4-Tag Memory Compiler</strong><br/>
      <em>A compiled read layer for static relational data and structured project memory.</em></p>
    </td>
  </tr>
</table>

RelayDB is a Rust-based compiler-and-runtime system for **static, relational, read-heavy data**.

It is designed for situations where data is already known at build time and does **not** need the full overhead of a live-query database in production. Instead of repeatedly importing, mapping, and manually stitching together scattered JSON files at runtime, RelayDB lets you:

- author related data with a simple **4-tag model**
- use either `.json` or `.jsonl` source files
- validate structure and topology at build time
- compile that source into a portable, read-only `.relay` artifact
- verify physical artifact integrity
- retrieve and traverse compiled memory through explicit anchors and relationships

RelayDB is **not** a database replacement. It is a **compiled read layer** for data and project memory that are effectively finished before deployment.

---

## Current status

RelayDB v1 is a working Rust compiler and read-only runtime. The archived JavaScript implementation is retained only as historical compatibility evidence.

The current foundation supports:

- universal `.json` / `.jsonl` ingestion
- configurable input paths
- configurable `.relay` output filenames
- physical integrity verification
- recursive anchor traversal
- generated Markdown audit reports
- generated Graphviz DOT topology files
- self-documentation through compiled RelayDB memory
- V1 identity, collection, relationship, cardinality, ambiguity, and numeric validation
- logical field profiling and relationship index compilation
- collection-aware Rust reads, queries, pagination, projection, and depth-limited hydration
- compatibility tests against the archived V1 fixture corpus

The current proof loop is:

```text
RelayDB project facts
→ tagged JSONL source memory
→ RelayDB compiler
→ verified .relay artifact
→ relay jump traversal
→ self-documenting project graph
```

RelayDB can now compile its own structured documentation into a `.relay` file and navigate that compiled project memory.

---

## Before you start

### Prerequisites

You need these installed **before** running RelayDB:

- **Rust + Cargo**
- **make**
- **Graphviz** (`dot`) for graph rendering

On macOS, Graphviz can be installed with:

```bash
brew install graphviz
```

### Important directory note

Run all `make` commands from the **top-level `RelayDB/` directory**, not from `relay-compiler/`.

```text
RelayDB/          = repo root
relay-compiler/  = Rust subproject
data/            = legacy/demo JSON files
atlas-memory/    = tagged JSONL project-memory files
Makefile         = top-level workflow orchestrator
```

If you are on Linux or Windows, you may need to adjust any `open` commands used by the current `Makefile`.

---

## Minimal repo map

```text
RelayDB/
├── Makefile
├── README.md
├── RelayDB-Logo.png
├── data/
│   ├── actors.json
│   ├── directors.json
│   └── movies.json
├── atlas-memory/
│   └── relaydb_v1_self_documentation.jsonl
├── relay-compiler/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   └── bin/
│   │       ├── compiler.rs
│   │       ├── reader.rs
│   │       ├── relay.rs
│   │       └── verify.rs
│   └── builds/
└── RelayDB_v2_Final_Project_Specification.md
```

---

## Quick start

### Full default pipeline

From the top-level `RelayDB/` directory:

```bash
make all
```

This runs:

```text
test → build → verify
```

Generated artifacts are written under:

```text
relay-compiler/builds/
```

---

## Compile the RelayDB self-documentation artifact

The current self-documentation source file is:

```text
atlas-memory/relaydb_v1_self_documentation.jsonl
```

Compile it into a `.relay` artifact:

```bash
make build \
  INPUT=atlas-memory/relaydb_v1_self_documentation.jsonl \
  OUTPUT=relay-compiler/builds/relaydb-v1-self-docs.relay
```

Verify the compiled artifact:

```bash
make verify \
  OUTPUT=relay-compiler/builds/relaydb-v1-self-docs.relay
```

Jump into the project root memory node:

```bash
make jump \
  OUTPUT=relay-compiler/builds/relaydb-v1-self-docs.relay \
  ANCHOR=project:relaydb
```

Jump to a specific function:

```bash
make jump \
  OUTPUT=relay-compiler/builds/relaydb-v1-self-docs.relay \
  ANCHOR=function:relay_jump_from
```

Jump to a specific concept:

```bash
make jump \
  OUTPUT=relay-compiler/builds/relaydb-v1-self-docs.relay \
  ANCHOR=concept:self-documentation-loop
```

---

## Individual Makefile commands

From the top-level `RelayDB/` directory:

### Run tests

```bash
make test
```

### Build a `.relay` artifact

```bash
make build
```

You can override the input and output:

```bash
make build INPUT=atlas-memory/relaydb_v1_self_documentation.jsonl OUTPUT=relay-compiler/builds/relaydb-v1-self-docs.relay
```

### Verify a `.relay` artifact

```bash
make verify OUTPUT=relay-compiler/builds/relaydb-v1-self-docs.relay
```

### Jump to an anchor

```bash
make jump OUTPUT=relay-compiler/builds/relaydb-v1-self-docs.relay ANCHOR=project:relaydb
```

### Optional filter

```bash
make jump OUTPUT=relay-compiler/builds/relaydb-v1-self-docs.relay ANCHOR=project:relaydb FILTER=compiler
```

### Open the latest Markdown audit report

```bash
make audit
```

### Generate and open the graph PNG

```bash
make graph
```

### Clean build products

```bash
make clean
```

### Show available commands

```bash
make help
```

---

## Direct Cargo usage

If you want to run the Rust tools manually, switch into the Rust subproject first:

```bash
cd relay-compiler
```

### Compile / bake

```bash
cargo run --bin compiler -- \
  --input ../atlas-memory/relaydb_v1_self_documentation.jsonl \
  --output ./builds/relaydb-v1-self-docs.relay \
  --builds ./builds
```

### Run tests

```bash
cargo test
```

### Verify a `.relay` file

```bash
cargo run --bin relay -- \
  check \
  --file ./builds/relaydb-v1-self-docs.relay
```

### Jump to an anchor

```bash
cargo run --bin relay -- \
  jump \
  --file ./builds/relaydb-v1-self-docs.relay \
  project:relaydb
```

### Strict acyclic mode

By default, RelayDB allows semantic graph cycles because project-memory graphs are often relational.

To fail the build if a cycle exists:

```bash
cargo run --bin compiler -- \
  --input ../atlas-memory/relaydb_v1_self_documentation.jsonl \
  --output ./builds/relaydb-v1-self-docs.relay \
  --strict-acyclic
```

---

## What RelayDB is

RelayDB is:

- a **source authoring model**
- a **compiler / verifier pipeline**
- a **portable `.relay` binary artifact**
- a **read-only runtime retrieval engine**
- a **toolchain for audit, graphing, and validation**
- a **project-memory foundation for documentation and AI-assisted development**

RelayDB is optimized for:

- static knowledge bundles
- structured documentation
- self-documenting projects
- localization / i18n data
- product or content catalogs
- reference sites
- frontend applications that need structured related data without backend complexity
- RAG prefiltering / structural context assembly
- AI project-memory handoff

---

## What RelayDB is not

RelayDB is intentionally narrow in scope.

It is **not**:

- a transactional database
- a live write system
- a system of record
- a query planner
- a CRUD backend
- a full-text search engine
- a replacement for SQLite, Postgres, Redis, MongoDB, or graph databases

The source files are the authored truth.  
The `.relay` file is the **compiled truth**.

---

## Why RelayDB exists

RelayDB grew out of a real frontend problem: too many JSON files, too much manual wiring, and too much repetitive mapping just to display related static content on a page.

If the data is:

- static
- relational
- known ahead of time
- and read-heavy in production

then RelayDB asks a simple question:

**Why pay runtime database overhead to rediscover structure that could have been compiled once?**

RelayDB now extends that same idea to project memory:

**Why should humans and AI assistants repeatedly rediscover project structure, design decisions, side effects, and documentation relationships when those facts can be compiled once into a navigable artifact?**

---

## The 4-Tag Model

Relay source memory uses four reserved prefixes:

| Prefix | Name | Purpose |
|---|---|---|
| `#` | Anchor | Unique, stable node identity |
| `^` | Provenance | Source, parent, upstream context, or classification |
| `@` | Relay Link | Traversable relationship edges |
| `~` | Alias | Search terms, labels, aliases, or retrieval hints |

RelayDB also supports the legacy `#id` field for older RelayDB data.

### JSON example

```json
[
  {
    "#id": "gladiator",
    "^": "movies",
    "name": "Gladiator",
    "release_year": 2000,
    "@cast": ["russell_crowe"],
    "@director": "ridley_scott",
    "~genres": "Action"
  }
]
```

### JSONL project-memory example

```jsonl
{"#":"function:relay_jump_from","type":"function","name":"relay_jump_from","file":"src/lib.rs","summary":"Recursively navigates a chosen .relay file starting from an anchor, displays matching nodes, follows @ links, and uses a visited set to avoid infinite recursion.","^":["module:relay_compiler_lib","concept:recursive-jump-traversal"],"@":["function:get_address_from","function:fetch_entry_from","function:process_baton"],"~":["relay jump","recursive traversal","graph navigation"]}
```

### Meaning

- `#` gives the node a stable identity.
- `^` records where the node comes from or what it belongs to.
- `@` defines graph relationships.
- `~` provides aliases and retrieval terms.

---

## Current project shape

The current RelayDB codebase includes:

- a **universal compiler** that ingests `.json` and `.jsonl`
- a **library crate** that handles protocol constants, tag extraction, retrieval, integrity checks, and traversal
- a **CLI** for jumping and verifying
- a **verifier** for physical integrity checks
- generated **audit** and **graph** artifacts during build
- a working **self-documentation memory file**

The current implementation is the working foundation for the RelayDB v2 direction.

---

## Current workflow

RelayDB currently follows this pattern:

1. Author related JSON or JSONL source files
2. Run tests and validation
3. Compile source into a chosen `.relay` artifact
4. Verify the artifact
5. Use the CLI or service layer to retrieve / traverse compiled memory

At runtime, Relay reads from the compiled `.relay` artifact itself.

```text
source .json / .jsonl
        ↓
relay compiler
        ↓
compiled .relay artifact
        ↓
verify / jump / audit / docs
```

---

## What the Makefile does

The top-level `Makefile` provides the main developer workflow:

- `make all` → full pipeline: **test → build → verify**
- `make test` → run Rust unit tests
- `make build` → compile `.json` / `.jsonl` source into `.relay`
- `make verify` → perform physical integrity checks on the baked artifact
- `make jump` → jump into a compiled `.relay` artifact by anchor
- `make audit` → open the latest Markdown audit report
- `make graph` → convert the latest `.dot` file into a PNG and open it
- `make clean` → wipe generated artifacts and Rust build outputs
- `make help` → show workflow help

Longer term, the Makefile will become RelayDB’s audit and compliance interface for graph-health checks such as:

- duplicate anchors
- missing internal anchors
- orphan nodes
- external references
- strict cycle checks
- release packaging
- checksums

---

## Current implementation notes

## Learn the implementation

The easiest way to understand RelayDB is to follow one record through the
pipeline rather than starting with the binary format:

```text
JSON / JSONL bytes
  |
  v
compiler.rs: ingest files into serde_json::Value
  |
  v
source_model.rs: validate identity, collection, numbers, and @ links
  |
  v
profile.rs: measure fields for future compact encoding decisions
  |
  v
compiled_model.rs: map public relationship IDs to integer record positions
  |
  +----------------------+
  |                      |
  v                      v
legacy .relay writer      reader.rs runtime API
  |                      |
  +-----------> get / query / hydrate
```

### 1. Source records

Every source object needs exactly one non-empty `#` or `#id` and a non-empty
`^` collection string. Ordinary fields are preserved as payload. `@field`
values are relationship references; `~field` values are searchable or indexed
payload by convention.

The source validator deliberately runs in two passes. The first pass checks
each record and builds lookup tables. The second pass resolves relationships
after every input file is known. This is what makes a movie in one JSON file
able to reference an actor in another file.

### 2. Profiling

The profiler does not rewrite records. It counts field presence, explicit nulls,
scalar values, arrays, and numeric ranges per collection. A missing field and a
present `null` are different states, so both `record_count` and
`present_count` matter. `BTreeMap` keeps profile output deterministic, which
makes generated metadata and tests stable.

### 3. Logical compilation

The logical compiler keeps developer IDs in the public record while creating
integer positions for relationships. Integer positions are cheaper to follow
than repeatedly comparing strings, but they are an implementation detail. A
relationship remains `Null`, `Scalar`, or `Array`, so cardinality is not lost.

This layer is intentionally separate from the physical writer. It lets the
project improve the `.relay` layout later without changing the source contract
or runtime semantics.

### 4. Runtime reads

`reader.rs` opens the current artifact, validates the header pointer, parses the
record region, and exposes collection-aware lookup. Unqualified duplicate IDs
produce an ambiguity error instead of silently returning whichever record was
encountered first.

Queries filter before pagination. Projections always retain identity and
collection metadata. Hydration follows `@` links only to the requested depth
and keeps a path-local visited set so cycles terminate.

### 5. How to extend the system

When adding a feature, keep the ownership boundaries intact:

1. Add or clarify a source rule in `source_model.rs` and a fixture test.
2. Add statistics in `profile.rs` only if encoding decisions need them.
3. Add runtime relationship structures in `compiled_model.rs`.
4. Add reader behavior in `reader.rs`, including a focused test.
5. Change the physical artifact only after logical behavior is verified.
6. Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `cargo test`, and
   `make all` before publishing.

The archived JavaScript material under `deprecated/javascript-reference/` is
historical compatibility evidence. The Rust modules are the production
compiler and reader. Compare behavior and fixtures, not incidental
implementation details or provisional byte layouts.

The current working implementation:

- compiles `.json` and `.jsonl` source into a single `.relay` artifact
- supports configurable input and output paths
- supports both `#` and `#id` anchors
- extracts relationship links from `@` and `^` fields
- handles string links and array links
- records byte offsets for compiled nodes
- writes a tab-separated jump table so anchors may contain colons
- resolves anchors through the jump table
- retrieves node payloads from the compiled file
- follows `@` relay links recursively
- uses a visited set to avoid infinite recursion
- verifies index-to-payload integrity

This means runtime retrieval is based on the compiled artifact, not on reopening the original source JSON or JSONL files.

---

## Current limitations

The logical V1 behavior is implemented. The physical `.relay` bytes remain the provisional legacy layout described in `deprecated/javascript-reference/js_ref_code/REFERENCE_FORMAT_V1.md`; final byte-format hardening is intentionally a later milestone.

Known limitations:

- Anchor lookup currently scans the text jump table before seeking to the byte offset.
- The byte seek is direct after anchor resolution, but lookup should not be described as strict O(1) yet.
- Graph cycles are allowed by default and reported as warnings unless strict mode is enabled.
- The current CLI reader binary is still a legacy traversal demo; applications should use the `relay_compiler::reader::RelayDb` library API.
- The audit suite is not complete yet.
- Missing-anchor, duplicate-anchor, orphan-node, and external-reference checks are separate audit concerns from V1 source validation.
- The docs viewer has not been built yet.

These limitations are being kept explicit to avoid overstating the current implementation.

---

## Why read-only is a feature

Read-only is not a limitation in RelayDB. It is part of the design.

Because the data is compiled ahead of time:

- there are no runtime writes to coordinate
- there are no race conditions on production data
- there is no mutable database state to protect
- there is a cleaner trust boundary between build time and runtime

RelayDB deliberately lets databases do database things, while Relay handles stable relational data that can be validated and baked before shipping.

---

## Current strengths

RelayDB already shows value in a few key areas:

- reducing repetitive frontend / backend glue code
- centralizing relationship traversal logic
- turning scattered JSON or JSONL into a coherent compiled artifact
- generating explainability artifacts during build
- making static relational data easier to consume from apps and services
- compiling structured project documentation into navigable memory
- exposing documentation holes through missing-anchor warnings
- providing a foundation for AI project-memory handoff

A React bootstrap prototype was able to consume Relay with very little code, which is exactly the kind of developer experience Relay is intended to improve.

---

## Self-documentation proof

RelayDB can now document itself.

The current self-documentation pipeline is:

```text
RelayDB implementation facts
→ relaydb_v1_self_documentation.jsonl
→ relaydb-v1-self-docs.relay
→ make verify
→ make jump ANCHOR=project:relaydb
```

The self-documentation artifact currently demonstrates:

- project identity
- tagged source-memory rules
- binary format
- protocol constants
- compiler functions
- library functions
- CLI binaries
- recursive traversal
- physical integrity checking
- audit artifacts
- risks
- decisions
- test evidence

This is the foundation for a future docs viewer powered by the compiled `.relay` artifact itself.

---

## RelayDB v2 direction

The project has a formal **RelayDB v2.0 Master Specification**.

The v2 direction centers on:

- stronger build-time certainty
- collision-safe binary indexing
- length-prefixed node storage
- structured result packets instead of terminal-only output
- tiered integrity verification
- benchmark-driven performance claims
- topic-neutral core behavior
- stronger graph-health auditing
- versioned release cartridges

The high-level philosophy is:

> If data is static, relational, and known at build time, correctness and structure should be enforced before shipping, not rediscovered repeatedly at runtime.

---

## Immediate next deliverables

The next major artifacts for the project are:

1. **Makefile Audit Suite**
   - duplicate anchor checks
   - missing internal anchor checks
   - orphan node checks
   - external reference policy
   - release/checksum targets

2. **Graph Health Audit Command**
   - separate physical artifact verification from semantic graph quality

3. **Memory Patch Workflow**
   - standard JSONL
   - patch JSONL files
   - merged full memory file

4. **Docs Viewer**
   - webpage that reads the compiled `.relay` self-documentation artifact

5. **Binary Appendix**
   - exact code-adjacent header / index / node structs

6. **Diagnostics Appendix**
   - compiler and verifier output schema

7. **Structured Result Packet Schema**
   - canonical API / JSON response shape

8. **Benchmark Harness**
   - RelayDB v1 vs RelayDB v2 vs JSON scan vs SQLite

---

## Contributing

RelayDB is still evolving, but contributions are welcome from developers interested in:

- compiler diagnostics
- verification tooling
- service wrappers
- language bindings
- visualization tools
- benchmark harnesses
- schema / lint tooling
- frontend integration examples
- WASM exploration
- CI / CD automation
- documentation tooling
- AI-assisted development workflows

Good contribution targets include:

- audit suite hardening
- cleaner structured API output
- generalized graph generation
- hardening error handling
- replacing prototype-grade shell integrations with proper service layers
- benchmarking and profiling the runtime
- docs viewer development
- missing-anchor and orphan-node reporting

---

## Development philosophy

RelayDB should stay:

- narrow in scope
- strong in guarantees
- easy to explain
- useful to frontend and app developers
- useful to AI-assisted development workflows
- evidence-driven in performance claims
- honest about current limitations

The goal is not to become everything.

The goal is to become **very good at compiling static relational data and project memory into a trusted read artifact**.
