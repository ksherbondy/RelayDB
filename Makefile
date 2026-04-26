# ==============================================================================
# RelayDB: Professional Orchestration Layer
# ==============================================================================
# Indentation MUST be Tabs. Place in /RelayDB/Makefile

.PHONY: all build test verify clean audit graph help

# --- [ Primary Pipeline ] ---

all: test build verify
	@echo "✅ System baked and verified. Artifacts ready in relay-compiler/builds/"

# 1. Logic Validation (The Gatekeeper)
test:
	@echo "--- [1/4] Running Protocol Logic Tests ---"
	@cd relay-compiler && cargo test --quiet

# 2. Binary Synthesis (The Forge)
build:
	@echo "--- [2/4] Soldering Binary & Generating Artifacts ---"
	@cd relay-compiler && cargo run --bin compiler --quiet

# 3. Physical Audit (The Inspector)
verify:
	@echo "--- [3/4] Performing Bit-Level Integrity Check ---"
	@cd relay-compiler && cargo run --bin relay -- check

# 4. Systems Observability (The Architect's View)
# This opens the latest generated documentation automatically.
audit:
	@echo "--- [4/4] Opening System Audit Report ---"
	@open relay-compiler/builds/$$(ls -t relay-compiler/builds | grep .md | head -n 1)

# --- [ Developer Utilities ] ---

# Generates and opens the visual graph (requires Graphviz installed)
graph:
	@echo "--- Generating Visual Topology ---"
	@dot -Tpng relay-compiler/builds/$$(ls -t relay-compiler/builds | grep .dot | head -n 1) -o relay-compiler/builds/latest_schema.png
	@open relay-compiler/builds/latest_schema.png

# Wipes the build environment to prevent "stale solder" joints
clean:
	@echo "--- Wiping Build Environment ---"
	@rm -rf relay-compiler/builds/*
	@rm -f relay-compiler/bacon_standard.relay
	@cd relay-compiler && cargo clean

help:
	@echo "RelayDB Engineering Interface"
	@echo "  make all     - Full pipeline: Test -> Build -> Verify"
	@echo "  make test    - Execute library and compiler unit tests"
	@echo "  make build   - Forge .relay binary and generate .dot/.md files"
	@echo "  make verify  - Run physical address validation on baked binary"
	@echo "  make audit   - View the latest Markdown system report"
	@echo "  make graph   - Convert .dot to PNG and open visually"
	@echo "  make clean   - Deep clean of all binaries and artifacts"