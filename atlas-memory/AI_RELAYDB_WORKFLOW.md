# RelayDB AI Workflow Guide

This file is a suggested instruction template for AI coding tools such as Claude Code, Cursor, GitHub Copilot Chat, VS Code AI assistants, ChatGPT, or any agentic development workflow that can read Markdown instructions.

RelayDB can be used as a project-memory workflow:

```text
human + AI work on project
        ↓
AI generates tagged JSONL project memory
        ↓
RelayDB compiles JSONL into a .relay artifact
        ↓
.relay artifact is verified
        ↓
future AI sessions read the project memory first
        ↓
AI starts with context instead of guessing
```

RelayDB does not make AI correct by magic. It gives AI a structured project memory to inspect before touching code.

---

## The Problem

AI coding assistants often start cold.

They may not know:

- the project purpose
- current architecture
- why certain decisions were made
- which files/functions are connected
- what risks already exist
- what side effects a change may create
- what was done on prior branches
- what should not be changed casually

This causes repeated explanation, stale documentation, accidental rewrites, and bugs from missing context.

RelayDB helps by turning project facts into structured memory.

---

## The Core Idea

Author project memory as tagged JSONL.

Compile that JSONL into a `.relay` artifact.

Use the `.relay` artifact and/or source JSONL as the first thing an AI reads before making changes.

```text
project_memory.jsonl
        ↓
RelayDB compiler
        ↓
project-memory.relay
        ↓
AI handoff / docs / CLI / JS reader
```

---

## The 4 Tags

RelayDB project memory uses four core tags:

| Tag | Meaning | Purpose |
|---|---|---|
| `#` | Anchor | Stable identity for a node |
| `^` | Provenance | Source, parent, upstream context, or origin |
| `@` | Relationship | Related nodes, affected areas, side effects |
| `~` | Aliases | Search terms, alternate names, retrieval hints |

Example:

```jsonl
{"#":"function:extract_links_from_node","type":"function","name":"extract_links_from_node","file":"src/lib.rs","summary":"Extracts relationship links from @ and ^ tagged fields, supporting both string values and arrays of strings.","^":["module:relay_compiler_lib","concept:relaydb-relationship-graph"],"@":["function:ingest_data","function:relay_jump_from","risk:compiler-runtime-relationship-mismatch"],"~":["relationship extractor","tag parser","shared link parser"]}
```

---

## Manual Workflow

This is the current human-supervised workflow.

### Step 1: Build or update the project normally

Work with your AI assistant as usual, but tell it to track only project-relevant engineering facts.

Do not ask it to record raw chat.

Track things like:

- functions changed
- files changed
- design decisions
- risks discovered
- tests added
- side effects
- relationship changes
- breaking changes
- known limitations

---

### Step 2: Ask the AI to generate branch memory

Before committing a branch, ask the AI to create a tagged JSONL memory file for that branch.

Example filename:

```text
atlas-memory/branches/feat-shared-relationship-extractor.jsonl
```

The branch memory should contain distilled facts, not conversation logs.

---

### Step 3: Compile project memory into a `.relay` artifact

Example:

```bash
make build \
  INPUT=atlas-memory/relaydb_v1_self_documentation.jsonl \
  OUTPUT=relay-compiler/builds/relaydb-v1-self-docs.relay
```

For multiple JSONL files, use a memory merge step first, or compile a directory if your RelayDB workflow supports it:

```bash
make build \
  INPUT=atlas-memory \
  OUTPUT=relay-compiler/builds/project-memory.relay
```

---

### Step 4: Verify the `.relay` artifact

```bash
make verify \
  OUTPUT=relay-compiler/builds/project-memory.relay
```

Physical verification should pass before treating the `.relay` file as trustworthy.

---

### Step 5: Jump into important anchors

Use RelayDB to inspect the compiled memory:

```bash
make jump \
  OUTPUT=relay-compiler/builds/project-memory.relay \
  ANCHOR=project:your-project
```

Other useful anchors:

```bash
make jump OUTPUT=relay-compiler/builds/project-memory.relay ANCHOR=risk:some-risk
make jump OUTPUT=relay-compiler/builds/project-memory.relay ANCHOR=function:some-function
make jump OUTPUT=relay-compiler/builds/project-memory.relay ANCHOR=decision:some-decision
```

---

### Step 6: Fix what the memory reveals

RelayDB may expose:

- missing anchors
- stale descriptions
- incomplete relationships
- undocumented constants
- risks without mitigations
- functions without related tests
- concepts referenced but not defined

Those warnings become the documentation repair checklist.

This is the important loop:

```text
compile memory
→ inspect / jump / audit
→ find holes
→ patch JSONL
→ recompile
→ verify again
```

---

## The Process We Used

RelayDB’s own v1.2 foundation was improved using this process.

1. We generated an initial JSONL memory file from the RelayDB source.
2. The structured memory exposed design problems, including compiler/runtime relationship drift.
3. We used those discovered risks as the guide for refactoring.
4. We updated RelayDB to support universal JSON/JSONL input and custom output files.
5. We compiled RelayDB’s own self-documentation into a `.relay` artifact.
6. We verified the artifact.
7. We jumped into `project:relaydb` and traversed the compiled project graph.
8. We patched stale/missing memory nodes.
9. The updated `.relay` became a better guide for the next development step.

That is the core pattern:

```text
AI helps create memory
memory exposes problems
problems guide code changes
code changes update memory
memory compiles into .relay
.relay guides the next AI session
```

---

## Suggested AI Instruction File

For tools that support Markdown instruction files, such as Claude Code or other agentic AI coding assistants, create something like:

```text
AI_RELAYDB_WORKFLOW.md
CLAUDE.md
AGENTS.md
.cursor/rules/relaydb-memory.md
.github/copilot-instructions.md
```

Then paste the following instruction block.

---

# AI Instructions: RelayDB Project-Memory Workflow

Before editing code, read the project memory first.

Project memory may exist as:

- tagged JSONL source files
- compiled `.relay` artifacts
- Markdown specifications
- audit reports
- branch memory files

Your job is not only to write code. Your job is to preserve and improve project memory.

## Required Startup Process

Before making code changes:

1. Read the project README.
2. Read any RelayDB/ATLAS memory instructions.
3. Inspect the tagged JSONL memory files.
4. Inspect the compiled `.relay` artifact if a reader/tool is available.
5. Read the current branch goal.
6. Identify likely affected files/functions before editing.

## Before Editing

Summarize:

- project purpose
- relevant architecture
- known risks
- related functions/modules
- likely side effects
- tests that may need to be updated
- documentation/memory nodes that may need to change

Do not edit files until this impact summary is complete.

## During Work

Track only project-relevant engineering facts:

- files changed
- functions changed
- public APIs changed
- binary format changes
- CLI changes
- decisions made
- risks discovered
- tests added
- tests needed
- side effects
- documentation updates needed

Do not record:

- raw chat
- private conversation
- irrelevant commentary
- emotional/casual discussion
- secrets, credentials, or personal data

## Before Commit

Generate a branch memory JSONL file.

Suggested location:

```text
atlas-memory/branches/<branch-name>.jsonl
```

Each JSONL line must be one valid JSON object.

Use these tags:

```text
# = stable anchor identity
^ = provenance / source / upstream context
@ = relationships / affected nodes / side effects
~ = aliases / retrieval terms
```

Include nodes for:

- branch
- changed functions
- changed modules
- decisions
- risks
- tests
- TODOs
- migration notes, if any

## Required Branch Memory Shape

At minimum, generate:

```jsonl
{"#":"branch:feat-example","type":"branch","name":"feat/example","summary":"Short description of the branch goal.","^":["project:your-project"],"@":["function:changed_function","decision:some-decision","risk:some-risk"],"~":["branch memory","feature branch"]}
{"#":"decision:example-decision","type":"decision","status":"accepted","summary":"A decision made during the branch.","^":["branch:feat-example"],"@":["function:changed_function"],"~":["decision alias"]}
{"#":"risk:example-risk","type":"risk","summary":"A risk discovered or mitigated during the branch.","^":["branch:feat-example"],"@":["function:changed_function","test:example-test"],"~":["risk alias"]}
```

## After Branch Memory Is Generated

Recommend the next commands:

```bash
make build INPUT=atlas-memory OUTPUT=relay-compiler/builds/project-memory.relay
make verify OUTPUT=relay-compiler/builds/project-memory.relay
make jump OUTPUT=relay-compiler/builds/project-memory.relay ANCHOR=project:your-project
```

If missing anchors or stale nodes appear, suggest a JSONL patch before code is merged.

---

## Feature Planning Workflow

RelayDB can also be used before a feature is implemented.

### Step 1: Compile current project memory

```bash
make build INPUT=atlas-memory OUTPUT=relay-compiler/builds/project-memory.relay
make verify OUTPUT=relay-compiler/builds/project-memory.relay
```

### Step 2: Ask the AI to read the current memory

Prompt:

```text
Before planning this feature, inspect the project memory first.

Read:
1. README.md
2. current JSONL memory files
3. compiled .relay artifact if accessible
4. relevant source files

Then tell me:
- what parts of the system this feature is likely to affect
- what risks exist
- what tests should be added
- what memory nodes should be updated
- whether this should be one branch or multiple branches
```

### Step 3: Ask the AI to generate a feature-plan JSONL

Example:

```text
atlas-memory/plans/feat-new-index-format.plan.jsonl
```

This feature-plan JSONL becomes the guide for the branch.

### Step 4: Implement against the plan

Use the `.relay` artifact and feature-plan JSONL as the working guide.

This is similar to how RelayDB itself was hardened:

```text
initial memory
→ discovered risks
→ refactor plan
→ implementation
→ updated memory
→ verified .relay
```

---

## Automation Path

The manual process can later be automated.

### Current manual process

```text
AI writes/updates JSONL
human reviews JSONL
RelayDB compiles JSONL
human runs verify/jump
human fixes missing nodes
human commits source + memory
```

### Near-term semi-automated process

Add Makefile targets:

```bash
make merge-memory
make build-memory
make verify-memory
make jump-project
make audit-memory
```

### Future automated process

A future RelayDB/ATLAS tool could:

1. detect branch name
2. collect changed files
3. ask AI to summarize branch memory
4. generate JSONL patch
5. run RelayDB compile
6. run verification
7. report missing anchors
8. block commit if critical memory errors exist
9. package `.relay` release cartridge

Potential future Git hook:

```text
pre-commit:
  generate branch memory
  merge JSONL
  compile .relay
  verify .relay
  fail if duplicate anchors or broken internal references exist
```

---

## Recommended Repo Structure

```text
project-root/
├── README.md
├── AI_RELAYDB_WORKFLOW.md
├── atlas-memory/
│   ├── project_standard.jsonl
│   ├── branches/
│   │   └── feat-example.jsonl
│   ├── patches/
│   │   └── patch-001-missing-anchors.jsonl
│   └── compiled/
│       └── project_full.jsonl
├── relay-artifacts/
│   ├── project-current.relay
│   └── releases/
│       └── project-v1.0.0.relay
└── src/
```

---

## Suggested Human Prompt for Daily Use

```text
We are working on this project using the RelayDB project-memory workflow.

Before changing code:
1. Read the project memory files.
2. Read the current .relay artifact if possible.
3. Read the relevant source files.
4. Summarize what this change will affect.

During the branch:
- track changed files/functions
- track decisions
- track risks
- track tests
- track documentation/memory updates

Before commit:
Generate a branch JSONL memory file using the RelayDB 4-tag format.

Do not record raw chat.
Do not record irrelevant discussion.
Only record durable engineering facts.
```

---

## Benefits

This process helps:

- reduce repeated project explanation
- give future AI sessions better startup context
- preserve branch decisions
- reduce architecture drift
- expose documentation holes
- improve code review context
- make releases easier to understand
- make AI-assisted development more disciplined

---

## Current Limitations

This workflow is still early.

RelayDB does not guarantee:

- AI correctness
- complete documentation
- bug-free implementation
- automatic understanding of every source file
- replacement for human review

The memory is only as good as the reviewed JSONL source.

Review AI-generated memory before compiling it.

---

## The Main Principle

Do not let AI write code and disappear.

Make it leave behind structured project memory for the next developer, the next branch, and the next AI session.
