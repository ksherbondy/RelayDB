# ==============================================================================
# RelayDB v1 Engineering Control Panel - Rust-First Version
# ==============================================================================
# Place this file at the top-level RelayDB/ directory.
#
# Philosophy:
#   Make orchestrates.
#   Rust owns RelayDB logic.
#
# This Makefile intentionally routes RelayDB validation/audit work through Rust
# binaries instead of Python scripts. Some audit targets are future-facing and
# will fail until the corresponding Rust CLI commands are implemented.
# ==============================================================================

SHELL := /bin/bash
.DEFAULT_GOAL := help

# --- Configurable Paths --------------------------------------------------------

COMPILER_DIR     ?= relay-compiler
BUILDS           ?= $(COMPILER_DIR)/builds

MEMORY_DIR       ?= atlas-memory
MEMORY_BASE      ?= $(MEMORY_DIR)/relaydb_v1_self_documentation.jsonl
MEMORY_PATCHES   ?= $(MEMORY_DIR)/patches
MEMORY_BRANCHES  ?= $(MEMORY_DIR)/branches
MEMORY_PLANS     ?= $(MEMORY_DIR)/plans
MEMORY_COMPILED  ?= $(MEMORY_DIR)/compiled/relaydb_v1_full.jsonl

SELF_DOC_OUTPUT  ?= $(BUILDS)/relaydb-v1-self-docs.relay

INPUT            ?= $(MEMORY_BASE)
OUTPUT           ?= $(SELF_DOC_OUTPUT)
LEGACY_PROVENANCE ?= 1
ANCHOR           ?= project:relaydb
FILTER           ?=

VERSION          ?= v1.2.0
RELEASE_DIR      ?= releases
RELEASE_NAME     ?= relaydb-$(VERSION)

OPEN             ?= open

# --- Phony Targets -------------------------------------------------------------

.PHONY: \
	help \
	all stable demo \
	init-dirs \
	test fmt fmt-check clippy \
	build self-docs strict verify jump jump-project jump-compiler jump-self-doc \
	audit graph graph-png size checksum inspect \
	validate-memory memory-summary duplicates missing orphans external cycles memory-audit \
	merge-memory patch-next \
	release package \
	clean clean-build clean-cargo

# ==============================================================================
# Primary Pipelines
# ==============================================================================

all: test build verify
	@echo "✅ RelayDB core pipeline passed."
	@echo "📦 Relay artifact: $(OUTPUT)"

stable: clean init-dirs test self-docs verify memory-audit graph-png checksum
	@echo ""
	@echo "✅ RelayDB stable foundation check complete."
	@echo "📦 Relay artifact: $(OUTPUT)"
	@echo "🧠 Memory source: $(INPUT)"
	@echo "📁 Build artifacts: $(BUILDS)"

demo: self-docs verify jump-project inspect
	@echo ""
	@echo "🎥 Demo pipeline complete: JSONL -> .relay -> verify -> jump -> Rust reader"

init-dirs:
	@mkdir -p $(BUILDS)
	@mkdir -p $(MEMORY_PATCHES)
	@mkdir -p $(MEMORY_BRANCHES)
	@mkdir -p $(MEMORY_PLANS)
	@mkdir -p $(dir $(MEMORY_COMPILED))
	@mkdir -p $(RELEASE_DIR)
	@echo "✅ RelayDB working directories are present."

# ==============================================================================
# Rust Quality Gates
# ==============================================================================

test:
	@echo "--- [Rust] Running unit tests ---"
	@cd $(COMPILER_DIR) && cargo test --quiet

fmt:
	@echo "--- [Rust] Formatting source ---"
	@cd $(COMPILER_DIR) && cargo fmt

fmt-check:
	@echo "--- [Rust] Checking formatting ---"
	@cd $(COMPILER_DIR) && cargo fmt -- --check

clippy:
	@echo "--- [Rust] Running Clippy ---"
	@cd $(COMPILER_DIR) && cargo clippy -- -D warnings

# ==============================================================================
# Build / Verify / Jump
# ==============================================================================

build: init-dirs
	@echo "--- [Build] Compiling JSON/JSONL into RelayDB artifact ---"
	@echo "Input:  $(INPUT)"
	@echo "Output: $(OUTPUT)"
	@cd $(COMPILER_DIR) && cargo run --bin compiler --quiet -- \
		--input ../$(INPUT) \
		--output ../$(OUTPUT) \
		--builds ../$(BUILDS) \
		$(if $(LEGACY_PROVENANCE),--allow-legacy-provenance,)

self-docs:
	@$(MAKE) build \
		INPUT=$(MEMORY_BASE) \
		OUTPUT=$(SELF_DOC_OUTPUT) \
		LEGACY_PROVENANCE=1

strict: init-dirs
	@echo "--- [Build] Strict acyclic compile ---"
	@cd $(COMPILER_DIR) && cargo run --bin compiler --quiet -- \
		--input ../$(INPUT) \
		--output ../$(OUTPUT) \
		--builds ../$(BUILDS) \
		--strict-acyclic \
		$(if $(LEGACY_PROVENANCE),--allow-legacy-provenance,)

verify:
	@echo "--- [Verify] Performing physical .relay integrity check ---"
	@cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- \
		check \
		--file ../$(OUTPUT)

jump:
	@echo "--- [Jump] Anchor: $(ANCHOR) ---"
	@if [ -z "$(FILTER)" ]; then \
		cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- \
			jump \
			--file ../$(OUTPUT) \
			$(ANCHOR); \
	else \
		cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- \
			jump \
			--file ../$(OUTPUT) \
			$(ANCHOR) \
			--filter "$(FILTER)"; \
	fi

jump-project:
	@$(MAKE) jump OUTPUT=$(OUTPUT) ANCHOR=project:relaydb

jump-compiler:
	@$(MAKE) jump OUTPUT=$(OUTPUT) ANCHOR=binary:compiler

jump-self-doc:
	@$(MAKE) jump OUTPUT=$(OUTPUT) ANCHOR=concept:self-documentation-loop

# ==============================================================================
# Build Artifacts / Visuals / Inspection
# ==============================================================================

audit:
	@echo "--- [Audit] Opening latest Markdown audit artifact ---"
	@$(OPEN) $(BUILDS)/$$(ls -t $(BUILDS) | grep '\.md$$' | head -n 1)

graph:
	@echo "--- [Graph] Opening latest DOT artifact ---"
	@$(OPEN) $(BUILDS)/$$(ls -t $(BUILDS) | grep '\.dot$$' | head -n 1)

graph-png:
	@echo "--- [Graph] Generating latest_schema.png from latest DOT artifact ---"
	@dot -Tpng $(BUILDS)/$$(ls -t $(BUILDS) | grep '\.dot$$' | head -n 1) \
		-o $(BUILDS)/latest_schema.png
	@echo "✅ Graph PNG written to $(BUILDS)/latest_schema.png"

size:
	@echo "--- [Inspect] Artifact size ---"
	@ls -lh $(OUTPUT)

checksum:
	@echo "--- [Inspect] SHA-256 checksum ---"
	@shasum -a 256 $(OUTPUT)

inspect: size checksum
	@echo "--- [Inspect] First 20 anchors via Rust reader ---"
	@cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- anchors --file ../$(OUTPUT) --limit 20

# ==============================================================================
# Rust-Backed Memory / Graph Health Audits
# ==============================================================================
# These define the interface we want.
# Next implementation work: add `audit-memory` subcommands to src/bin/relay.rs
# or create src/bin/audit_memory.rs.
#
# Proposed Rust CLI:
#
# cargo run --bin relay -- audit-memory --input ../atlas-memory/relaydb_v1_self_documentation.jsonl --mode all
# cargo run --bin relay -- audit-memory --input ../atlas-memory/relaydb_v1_self_documentation.jsonl --mode duplicates
# cargo run --bin relay -- audit-memory --input ../atlas-memory/relaydb_v1_self_documentation.jsonl --mode missing
# cargo run --bin relay -- audit-memory --input ../atlas-memory/relaydb_v1_self_documentation.jsonl --mode orphans
# ==============================================================================

validate-memory:
	@echo "--- [Memory] Validating JSON/JSONL parseability and anchor presence ---"
	@cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- \
		audit-memory \
		--input ../$(INPUT) \
		--mode validate

memory-summary:
	@echo "--- [Memory] Summary ---"
	@cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- \
		audit-memory \
		--input ../$(INPUT) \
		--mode summary

duplicates:
	@echo "--- [Memory] Duplicate anchor audit ---"
	@cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- \
		audit-memory \
		--input ../$(INPUT) \
		--mode duplicates

missing:
	@echo "--- [Memory] Missing internal anchor audit ---"
	@cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- \
		audit-memory \
		--input ../$(INPUT) \
		--mode missing

orphans:
	@echo "--- [Memory] Orphan node audit ---"
	@cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- \
		audit-memory \
		--input ../$(INPUT) \
		--mode orphans

external:
	@echo "--- [Memory] External reference audit ---"
	@cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- \
		audit-memory \
		--input ../$(INPUT) \
		--mode external

cycles:
	@echo "--- [Memory] Cycle summary ---"
	@cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- \
		audit-memory \
		--input ../$(INPUT) \
		--mode cycles

memory-audit:
	@echo "--- [Memory] Full graph-health audit ---"
	@cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- \
		audit-memory \
		--input ../$(INPUT) \
		--mode all

# ==============================================================================
# Memory Merge / Patch Generation
# ==============================================================================
# Rust-first direction:
#   These should become Rust commands too.
#
# Proposed future CLI:
#   cargo run --bin relay -- merge-memory --base ... --patches ... --branches ... --plans ... --output ...
#   cargo run --bin relay -- generate-patch --kind makefile-audit-suite --output ...
# ==============================================================================

merge-memory: init-dirs
	@echo "--- [Memory] TODO: implement Rust merge-memory command ---"
	@echo "Target interface:"
	@echo "cd $(COMPILER_DIR) && cargo run --bin relay -- merge-memory --base ../$(MEMORY_BASE) --patches ../$(MEMORY_PATCHES) --branches ../$(MEMORY_BRANCHES) --plans ../$(MEMORY_PLANS) --output ../$(MEMORY_COMPILED)"
	@false

patch-next: init-dirs
	@echo "--- [Memory] TODO: implement Rust generate-patch command ---"
	@echo "Target interface:"
	@echo "cd $(COMPILER_DIR) && cargo run --bin relay -- generate-patch --kind makefile-audit-suite --output ../$(MEMORY_PATCHES)/patch_001_makefile_audit_suite.jsonl"
	@false

# ==============================================================================
# Release Packaging
# ==============================================================================

release: stable package
	@echo "🚀 Release package ready in $(RELEASE_DIR)/$(RELEASE_NAME)"

package: init-dirs
	@echo "--- [Release] Packaging current .relay artifact and memory source ---"
	@mkdir -p $(RELEASE_DIR)/$(RELEASE_NAME)
	@cp $(OUTPUT) $(RELEASE_DIR)/$(RELEASE_NAME)/$(notdir $(OUTPUT))
	@if [ -f "$(INPUT)" ]; then cp $(INPUT) $(RELEASE_DIR)/$(RELEASE_NAME)/$(notdir $(INPUT)); fi
	@if ls $(BUILDS)/*.md >/dev/null 2>&1; then cp $$(ls -t $(BUILDS)/*.md | head -n 1) $(RELEASE_DIR)/$(RELEASE_NAME)/; fi
	@if ls $(BUILDS)/*.dot >/dev/null 2>&1; then cp $$(ls -t $(BUILDS)/*.dot | head -n 1) $(RELEASE_DIR)/$(RELEASE_NAME)/; fi
	@if [ -f "$(BUILDS)/latest_schema.png" ]; then cp $(BUILDS)/latest_schema.png $(RELEASE_DIR)/$(RELEASE_NAME)/; fi
	@shasum -a 256 $(RELEASE_DIR)/$(RELEASE_NAME)/* > $(RELEASE_DIR)/$(RELEASE_NAME)/SHA256SUMS.txt
	@echo "✅ Release files copied to $(RELEASE_DIR)/$(RELEASE_NAME)"

# ==============================================================================
# Cleaning
# ==============================================================================

clean-build:
	@echo "--- [Clean] Wiping RelayDB build artifacts ---"
	@rm -rf $(BUILDS)/*
	@rm -f $(OUTPUT)

clean-cargo:
	@echo "--- [Clean] Running cargo clean ---"
	@cd $(COMPILER_DIR) && cargo clean

clean: clean-build clean-cargo
	@echo "✅ Clean complete."

# ==============================================================================
# Help
# ==============================================================================

help:
	@echo "RelayDB v1 Engineering Control Panel"
	@echo ""
	@echo "Primary:"
	@echo "  make all              test -> build -> verify"
	@echo "  make stable           clean -> test -> self-docs -> verify -> memory-audit -> graph-png -> checksum"
	@echo "  make demo             self-docs -> verify -> jump-project -> inspect"
	@echo ""
	@echo "Build / Run:"
	@echo "  make build            Compile INPUT into OUTPUT"
	@echo "  make self-docs        Compile relaydb_v1_self_documentation.jsonl"
	@echo "  make verify           Verify physical .relay integrity"
	@echo "  make jump             Jump to ANCHOR in OUTPUT"
	@echo "  make inspect          List the first 20 anchors with the Rust reader"
	@echo ""
	@echo "Memory Audits:"
	@echo "  make validate-memory  Validate JSON/JSONL and anchors"
	@echo "  make duplicates       Fail on duplicate anchors"
	@echo "  make missing          Fail on missing internal references"
	@echo "  make orphans          Warn about unreferenced nodes"
	@echo "  make external         List external: references"
	@echo "  make cycles           Report cycles"
	@echo "  make memory-audit     Full memory audit"
	@echo ""
	@echo "Memory Workflow:"
	@echo "  make merge-memory     TODO: Rust implementation"
	@echo "  make patch-next       TODO: Rust implementation"
	@echo ""
	@echo "Artifacts:"
	@echo "  make audit            Open latest Markdown audit artifact"
	@echo "  make graph            Open latest DOT artifact"
	@echo "  make graph-png        Generate latest_schema.png"
	@echo "  make inspect          Size/checksum/anchor preview"
	@echo "  make release          Build and package release cartridge"
	@echo ""
	@echo "Config:"
	@echo "  INPUT=$(INPUT)"
	@echo "  OUTPUT=$(OUTPUT)"
	@echo "  ANCHOR=$(ANCHOR)"
