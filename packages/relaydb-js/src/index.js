// RelayDB v1.2 JavaScript Reader
// Matches current universal Rust implementation:
// - HEADER_SIZE = 32
// - POINTER_START = 16
// - Null-terminated UTF-8 JSON node payloads
// - Jump table supports:
//   - v1.2 universal format: "anchor<TAB>offset\n"
//   - legacy v1 format:      "anchor:offset\n"
// - Anchor fields support:
//   - universal project memory: "#"
//   - legacy RelayDB data:      "#id"

export default class RelayDB {
  constructor(buffer) {
    this.buffer = buffer;
    this.view = new DataView(buffer);
    this.bytes = new Uint8Array(buffer);

    this.pointerStart = 16;
    this.headerSize = 32;
    this.terminator = 0;

    this.indexOffset = this._readIndexOffset();
    this.jumpTable = this._parseJumpTable();
  }

  static async open(path) {
    // Browser path: fetch works directly.
    // Node 18+ can also fetch file-served or HTTP-served artifacts.
    const res = await fetch(path);

    if (!res.ok) {
      throw new Error(`RelayDB.open failed for ${path}: ${res.status} ${res.statusText}`);
    }

    const buffer = await res.arrayBuffer();
    return new RelayDB(buffer);
  }

  static fromBytes(buffer) {
    return new RelayDB(buffer);
  }

  // --- Core Parsing ---

  _readIndexOffset() {
    return Number(this.view.getBigUint64(this.pointerStart, true));
  }

  _parseJumpTable() {
    const text = new TextDecoder().decode(
      this.bytes.slice(Number(this.indexOffset))
    );

    const map = new Map();

    for (const line of text.split("\n")) {
      const trimmed = line.trim();
      if (!trimmed) continue;

      const parsed = this._parseJumpTableLine(trimmed);

      if (parsed) {
        map.set(parsed.id, parsed.offset);
      }
    }

    return map;
  }

  _parseJumpTableLine(line) {
    // Universal v1.2 format:
    // anchor<TAB>offset
    //
    // This is required because anchors may contain colons:
    // function:relay_jump_from
    // concept:self-documentation-loop
    if (line.includes("\t")) {
      const [id, offsetRaw] = line.split("\t");

      const offset = Number(offsetRaw);
      if (!id || Number.isNaN(offset)) return null;

      return {
        id: id.trim(),
        offset,
      };
    }

    // Legacy v1 fallback:
    // anchor:offset
    //
    // Use lastIndexOf so older anchors containing colons have a chance
    // to parse correctly if they ever appear in legacy files.
    const lastColon = line.lastIndexOf(":");
    if (lastColon === -1) return null;

    const id = line.slice(0, lastColon).trim();
    const offset = Number(line.slice(lastColon + 1).trim());

    if (!id || Number.isNaN(offset)) return null;

    return { id, offset };
  }

  _anchorOf(node) {
    return node?.["#"] ?? node?.["#id"] ?? null;
  }

  // --- Public API ---

  get(id) {
    const address = this.jumpTable.get(id);
    if (address === undefined) return null;
    return this._fetchEntry(address);
  }

  has(id) {
    return this.jumpTable.has(id);
  }

  anchors() {
    return Array.from(this.jumpTable.keys());
  }

  entries() {
    return Array.from(this.jumpTable.entries());
  }

  getAddress(id) {
    return this.jumpTable.get(id) ?? null;
  }

  fetchAt(address) {
    return this._fetchEntry(address);
  }

  size() {
    return this.jumpTable.size;
  }

  // --- Low-Level Fetch ---

  _fetchEntry(address) {
    if (!Number.isInteger(address) || address < this.headerSize || address >= this.bytes.length) {
      throw new Error(`Invalid RelayDB address: ${address}`);
    }

    let i = address;
    const bytes = [];

    while (i < this.bytes.length) {
      const byte = this.bytes[i++];
      if (byte === this.terminator) break;
      bytes.push(byte);
    }

    const json = new TextDecoder().decode(new Uint8Array(bytes));
    return JSON.parse(json);
  }

  // --- Traversal ---

  jump(startId, options = {}) {
    const visited = new Set();
    const nodes = [];
    const missing = [];

    const subject = options.filter ?? null;

    const traverse = (id) => {
      if (visited.has(id)) return;
      visited.add(id);

      const node = this.get(id);

      if (!node) {
        missing.push(id);
        return;
      }

      if (this._shouldDisplay(node, subject)) {
        nodes.push(node);
      }

      for (const key in node) {
        if (key.startsWith("@")) {
          this._walkRelayValue(node[key], traverse);
        }
      }
    };

    traverse(startId);

    return {
      start: startId,
      visited: visited.size,
      nodes,
      missing,
    };
  }

  _walkRelayValue(value, traverse) {
    if (Array.isArray(value)) {
      for (const item of value) {
        if (typeof item === "string") traverse(item);
      }
      return;
    }

    if (typeof value === "string") {
      traverse(value);
    }
  }

  _shouldDisplay(node, subject) {
    if (!subject) return true;

    const raw = JSON.stringify(node);
    return raw.includes(subject);
  }

  // --- Integrity Check ---

  verifyIntegrity(options = {}) {
    const failures = [];
    const warn = options.warn ?? true;

    for (const [id, offset] of this.jumpTable.entries()) {
      let node;

      try {
        node = this._fetchEntry(offset);
      } catch (error) {
        failures.push({
          id,
          offset,
          reason: "fetch_or_parse_failed",
          error: error.message,
        });
        continue;
      }

      const anchor = this._anchorOf(node);

      if (anchor !== id) {
        failures.push({
          id,
          offset,
          reason: "anchor_mismatch",
          expected: id,
          actual: anchor,
        });
      }
    }

    if (warn) {
      for (const failure of failures) {
        console.warn("RelayDB integrity failure:", failure);
      }
    }

    return failures.length === 0;
  }

  integrityReport() {
    const failures = [];

    for (const [id, offset] of this.jumpTable.entries()) {
      try {
        const node = this._fetchEntry(offset);
        const anchor = this._anchorOf(node);

        if (anchor !== id) {
          failures.push({
            id,
            offset,
            reason: "anchor_mismatch",
            expected: id,
            actual: anchor,
          });
        }
      } catch (error) {
        failures.push({
          id,
          offset,
          reason: "fetch_or_parse_failed",
          error: error.message,
        });
      }
    }

    return {
      ok: failures.length === 0,
      entries: this.jumpTable.size,
      failures,
    };
  }
}
