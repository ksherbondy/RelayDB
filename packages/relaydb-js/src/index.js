// RelayDB v1 JavaScript Reader
// Matches current Rust implementation:
// - HEADER_SIZE = 32
// - POINTER_START = 16
// - Null-terminated JSON nodes
// - Jump table: "id:offset\n"

export default class RelayDB {
  constructor(buffer) {
    this.buffer = buffer;
    this.view = new DataView(buffer);
    this.bytes = new Uint8Array(buffer);

    this.pointerStart = 16;
    this.headerSize = 32;

    this.indexOffset = this._readIndexOffset();
    this.jumpTable = this._parseJumpTable();
  }

  static async open(path) {
    const res = await fetch(path);
    const buffer = await res.arrayBuffer();
    return new RelayDB(buffer);
  }

  static fromBytes(buffer) {
    return new RelayDB(buffer);
  }

  // --- Core Parsing ---

  _readIndexOffset() {
    return this.view.getBigUint64(this.pointerStart, true);
  }

  _parseJumpTable() {
    const text = new TextDecoder().decode(
      this.bytes.slice(Number(this.indexOffset))
    );

    const map = new Map();

    for (const line of text.split("\n")) {
      if (!line.trim()) continue;

      const [id, offset] = line.split(":");
      if (id && offset) {
        map.set(id.trim(), Number(offset));
      }
    }

    return map;
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

  // --- Low-Level Fetch ---

  _fetchEntry(address) {
    let i = address;
    const bytes = [];

    while (i < this.bytes.length) {
      const byte = this.bytes[i++];
      if (byte === 0) break;
      bytes.push(byte);
    }

    const json = new TextDecoder().decode(new Uint8Array(bytes));
    return JSON.parse(json);
  }

  // --- Traversal (matches relay_jump behavior) ---

  jump(startId, options = {}) {
    const visited = new Set();
    const results = [];

    const subject = options.filter ?? null;

    const traverse = (id) => {
      if (visited.has(id)) return;
      visited.add(id);

      const node = this.get(id);
      if (!node) return;

      if (this._shouldDisplay(node, subject)) {
        results.push(node);
      }

      for (const key in node) {
        if (key.startsWith("@")) {
          const val = node[key];

          if (Array.isArray(val)) {
            val.forEach(traverse);
          } else if (typeof val === "string") {
            traverse(val);
          }
        }
      }
    };

    traverse(startId);

    return {
      nodes: results,
      visited: visited.size,
    };
  }

  _shouldDisplay(node, subject) {
    if (!subject) return true;

    const raw = JSON.stringify(node);
    return raw.includes(subject) && node["^"] === "movies";
  }

  // --- Integrity Check (matches Rust logic) ---

  verifyIntegrity() {
    let failures = 0;

    for (const [id, offset] of this.jumpTable.entries()) {
      const node = this._fetchEntry(offset);

      if (node["#id"] !== id) {
        console.warn(
          `Integrity failure: ${id} mismatch at offset ${offset}`
        );
        failures++;
      }
    }

    return failures === 0;
  }
}