# ==============================================================================
# RelayDB: Universal Orchestration Layer
# ==============================================================================
# Indentation MUST be Tabs.
# Place this file in /RelayDB/Makefile
#
# Expected project layout:
#
# RelayDB/
# ├── Makefile
# ├── relay-compiler/
# │   ├── Cargo.toml
# │   ├── src/
# │   └── builds/
# └── atlas-memory/
#     └── relaydb_atlas_tagged_memory.jsonl
#
# The new universal compiler supports:
# - .json files
# - .jsonl files
# - custom input path
# - custom output .relay filename
# - custom relay file for check/jump commands
# ==============================================================================

.PHONY: all test build verify jump audit graph clean clean-build clean-cargo demo help

# --- [ Configurable Paths ] ----------------------------------------------------

COMPILER_DIR ?= relay-compiler
INPUT       ?= atlas-memory/relaydb_atlas_tagged_memory.jsonl
OUTPUT      ?= $(COMPILER_DIR)/builds/relaydb-docs.relay
BUILDS      ?= $(COMPILER_DIR)/builds
ANCHOR      ?= project:relaydb
FILTER      ?=

# --- [ Primary Pipeline ] ------------------------------------------------------

all: test build verify
	@echo "✅ System baked and verified."
	@echo "📦 Relay artifact: $(OUTPUT)"
	@echo "📁 Audit artifacts: $(BUILDS)"

# 1. Logic Validation
test:
	@echo "--- [1/4] Running Protocol Logic Tests ---"
	@cd $(COMPILER_DIR) && cargo test --quiet

# 2. Binary Synthesis
build:
	@echo "--- [2/4] Compiling JSON/JSONL into RelayDB Artifact ---"
	@mkdir -p $(BUILDS)
	@cd $(COMPILER_DIR) && cargo run --bin compiler --quiet -- \
		--input ../$(INPUT) \
		--output ../$(OUTPUT) \
		--builds ../$(BUILDS)

# 3. Physical Audit
verify:
	@echo "--- [3/4] Performing Relay Integrity Check ---"
	@cd $(COMPILER_DIR) && cargo run --bin relay --quiet -- \
		check \
		--file ../$(OUTPUT)

# 4. Jump / Query Demo
jump:
	@echo "--- [4/4] Jumping to Anchor: $(ANCHOR) ---"
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

# Full ATLAS / RelayDB documentation demo pipeline
demo: all jump
	@echo "🎥 Demo complete: JSONL -> .relay -> verify -> jump"

# --- [ Developer Utilities ] ---------------------------------------------------

# Opens the latest generated Markdown audit report.
audit:
	@echo "--- Opening Latest System Audit Report ---"
	@open $(BUILDS)/$$(ls -t $(BUILDS) | grep '\.md$$' | head -n 1)

# Generates and opens the visual graph.
# Requires Graphviz installed:
# brew install graphviz
graph:
	@echo "--- Generating Visual Topology ---"
	@dot -Tpng $(BUILDS)/$$(ls -t $(BUILDS) | grep '\.dot$$' | head -n 1) \
		-o $(BUILDS)/latest_schema.png
	@open $(BUILDS)/latest_schema.png

# Remove generated relay/docs artifacts only.
clean-build:
	@echo "--- Wiping RelayDB Build Artifacts ---"
	@rm -rf $(BUILDS)/*
	@rm -f $(OUTPUT)

# Deep clean Rust target files too.
clean-cargo:
	@echo "--- Running Cargo Clean ---"
	@cd $(COMPILER_DIR) && cargo clean

# Full clean
clean: clean-build clean-cargo
	@echo "✅ Clean complete."

help:
	@echo "RelayDB Universal Engineering Interface"
	@echo ""
	@echo "Primary commands:"
	@echo "  make all       - Full pipeline: test -> build -> verify"
	@echo "  make test      - Execute Rust unit/integration tests"
	@echo "  make build     - Compile .json/.jsonl source memory into .relay"
	@echo "  make verify    - Verify physical integrity of the .relay artifact"
	@echo "  make jump      - Jump to an anchor in the .relay artifact"
	@echo "  make demo      - Run full MVP demo pipeline"
	@echo ""
	@echo "Utility commands:"
	@echo "  make audit     - Open latest Markdown audit report"
	@echo "  make graph     - Generate/open latest Graphviz PNG"
	@echo "  make clean     - Remove artifacts and cargo build output"
	@echo ""
	@echo "Config variables:"
	@echo "  INPUT=$(INPUT)"
	@echo "  OUTPUT=$(OUTPUT)"
	@echo "  ANCHOR=$(ANCHOR)"
	@echo ""
	@echo "Examples:"
	@echo "  make build INPUT=atlas-memory/relaydb_atlas_tagged_memory.jsonl OUTPUT=relay-compiler/builds/relaydb-docs.relay"
	@echo "  make jump ANCHOR=function:fetch_entry"
	@echo "  make jump ANCHOR=project:relaydb FILTER=RelayDB"
