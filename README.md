<p align="left">
  <img src="RelayDB-Logo.png" width="400" />
</p>

# RelayDB
**The Bacon Standard**  
*A compiled read layer for static relational knowledge.*

RelayDB is a Rust-based compiler-and-runtime system for **static, relational, read-heavy data**.

It is designed for situations where data is already known at build time and does **not** need the full overhead of a live-query database in production. Instead of repeatedly importing, mapping, and manually stitching together scattered JSON files at runtime, RelayDB lets you:

- author related data with a simple **4-tag model**
- validate structure and topology at build time
- compile that data into a portable, read-only `.relay` artifact
- retrieve and traverse it through explicit anchors and relationships

RelayDB is **not** a database replacement. It is a **compiled read layer** for data that is effectively finished before deployment.

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

That is the problem RelayDB is built to solve.

---

## What RelayDB is

RelayDB is:

- a **source authoring model**
- a **compiler / verifier pipeline**
- a **portable `.relay` binary artifact**
- a **read-only runtime retrieval engine**
- a **toolchain for audit, graphing, and validation**

RelayDB is optimized for:

- static knowledge bundles
- documentation engines
- localization/i18n data
- product or content catalogs
- reference sites
- frontend applications that need structured related data without backend complexity
- RAG prefiltering / structural context assembly

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

The source files are the authored truth.  
The `.relay` file is the **compiled truth**.

---

## The 4-Tag Model

Relay source data uses four reserved prefixes:

| Prefix | Name | Purpose |
|---|---|---|
| `#` | Anchor | Unique, stable node identity |
| `^` | Topic | Primary classification / type |
| `@` | Baton | Traversable relationship edges |
| `~` | Metadata | Non-traversed descriptive/filterable fields |

### Example

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

### Meaning

- `#id` gives the node a stable identity.
- `^` tells Relay what kind of node it is.
- `@cast` and `@director` define graph relationships.
- `~genres` provides metadata that can be filtered or inspected but is not traversed as an edge.

---

## Current project shape

The current RelayDB codebase already includes:

- a **compiler** that ingests JSON and bakes a `.relay` artifact
- a **library crate** that handles retrieval and traversal logic
- a **CLI** for jumping and verifying
- a **verifier** for integrity checks
- generated **audit** and **graph** artifacts during build

The current implementation is the working foundation for the RelayDB v2 direction.

---

## Current workflow

RelayDB currently follows this pattern:

1. Author related JSON source files
2. Run tests and validation
3. Compile source into `bacon_standard.relay`
4. Verify the artifact
5. Use the CLI or service layer to retrieve/traverse data

At runtime, Relay reads from the compiled `.relay` artifact itself.

---

## Project layout

This README assumes the current repository layout where the Rust project lives under `relay-compiler/` and the top-level `Makefile` orchestrates the full workflow.

---

## Requirements

- Rust / Cargo
- `make`
- Graphviz (`dot`) for graph rendering
- macOS `open` command is currently used by the provided `Makefile` targets for `audit` and `graph`

If you are on Linux or Windows, you may need to adjust those `open` commands.

---

## Quick start

### 1. Run the full pipeline

```bash
make all
```

This runs:

1. tests
2. build
3. verification

and leaves the generated artifacts in:

```text
relay-compiler/builds/
```

### 2. Run steps individually

#### Run tests

```bash
make test
```

#### Build the `.relay` artifact and reports

```bash
make build
```

#### Verify the baked binary

```bash
make verify
```

#### Open the latest markdown audit report

```bash
make audit
```

#### Generate and open the graph PNG

```bash
make graph
```

#### Clean all build products

```bash
make clean
```

#### Show available commands

```bash
make help
```

---

## What the Makefile does

The current `Makefile` provides a professional orchestration layer:

- `make all` → full pipeline: **Test → Build → Verify**
- `make test` → run Rust unit tests
- `make build` → compile the `.relay` file and generate `.md` / `.dot` artifacts
- `make verify` → perform integrity checks on the baked artifact
- `make audit` → open the latest markdown audit report
- `make graph` → convert the latest `.dot` file into a PNG and open it
- `make clean` → wipe generated artifacts and Rust build outputs

---

## Direct Cargo usage

If you want to run the tools manually without `make`, the current commands are:

### Compile / bake

```bash
cd relay-compiler
cargo run --bin compiler --quiet
```

### Run tests

```bash
cd relay-compiler
cargo test --quiet
```

### Verify the artifact

```bash
cd relay-compiler
cargo run --bin relay -- check
```

### Jump to an anchor

```bash
cd relay-compiler
cargo run --bin relay -- jump the_terminal -f Drama
```

> The current CLI uses `jump` and `check`.  
> The longer-term v2 direction expands this into a more formal command surface.

---

## Current implementation notes

The current working implementation:

- compiles source JSON into a single `.relay` artifact
- records byte offsets for compiled nodes
- resolves anchors through an index
- retrieves node payloads from the compiled file
- follows `@` batons recursively
- verifies index-to-payload integrity

This means runtime retrieval is based on the compiled artifact, not on reopening the original source JSON files.

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

- reducing repetitive frontend/backend glue code
- centralizing relationship traversal logic
- turning scattered JSON into a coherent compiled artifact
- generating explainability artifacts during build
- making static relational data easier to consume from apps and services

A React bootstrap prototype was able to consume Relay with very little code, which is exactly the kind of developer experience Relay is intended to improve.

---

## RelayDB v2 direction

The project now has a formal **RelayDB v2.0 Master Specification**.

The v2 direction centers on:

- stronger build-time certainty
- collision-safe binary indexing
- length-prefixed node storage
- structured result packets instead of terminal-only output
- tiered integrity verification
- benchmark-driven performance claims
- topic-neutral core behavior

The high-level philosophy is:

> If data is static, relational, and known at build time, correctness and structure should be enforced before shipping, not rediscovered repeatedly at runtime.

---

## Immediate next deliverables

The next major artifacts for the project are:

1. **Binary Appendix**
   - exact code-adjacent header / index / node structs

2. **Diagnostics Appendix**
   - compiler and verifier output schema

3. **Structured Result Packet Schema**
   - canonical API / JSON response shape

4. **v2 Prototype Reader / Writer**
   - smallest compliant v2 artifact builder and reader

5. **Benchmark Harness**
   - RelayDB v1.1 vs RelayDB v2 vs JSON scan vs SQLite

---

## Contributing

RelayDB is still evolving, but contributions are welcome from developers interested in:

- compiler diagnostics
- verification tooling
- service wrappers
- language bindings
- visualization tools
- benchmark harnesses
- schema/lint tooling
- frontend integration examples
- WASM exploration
- CI/CD automation

Good contribution targets include:

- cleaner structured API output
- generalized graph generation
- hardening error handling
- replacing prototype-grade shell integrations with proper service layers
- benchmarking and profiling the runtime

---

## Development philosophy

RelayDB should stay:

- narrow in scope
- strong in guarantees
- easy to explain
- useful to frontend and app developers
- evidence-driven in performance claims

The goal is not to become everything.

The goal is to become **very good at compiling static relational data into a trusted read artifact**.