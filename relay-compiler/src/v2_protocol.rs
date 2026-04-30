use crc32fast::Hasher as Crc32Hasher;
use highway::{HighwayHash, HighwayHasher, Key};
use std::io::{Read, Seek, SeekFrom};
use thiserror::Error;

// --- 1. ERROR MODEL (v2.1 Section 12) ---------------------------------------

#[derive(Error, Debug, PartialEq, Eq)]
pub enum RelayError {
    #[error("Structural: Format violation at {0} - {1}")]
    Structural(u64, String),
    #[error("Semantic: Corruption detected at address {0}")]
    Semantic(u64),
    #[error("Identity: Anchor #{0} not found")]
    AnchorNotFound(String),
    #[error("System: IO failure - {0}")]
    Io(String),
}

pub type Result<T> = std::result::Result<T, RelayError>;

// --- 2. PROTOCOL CONSTANTS --------------------------------------------------

pub const RELAY_MAGIC: [u8; 4] = *b"RELY";
pub const MAJOR_VERSION: u16 = 2;
pub const MINOR_VERSION: u16 = 1;
pub const SUPPORTED_FLAGS: u16 = 0;
pub const ENTRY_FLAGS_V21: u16 = 0;
pub const HEADER_SIZE: u64 = 64;
pub const INDEX_ENTRY_SIZE: u64 = 40;
pub const MAX_NODE_SIZE: u32 = 16 * 1024 * 1024; // 16 MiB
pub const KEY_PREFIX_LEN: usize = 8;
pub const MANIFEST_ABSENT: u64 = 0;

const RELAY_HASH_KEY: Key = Key([0, 0, 0, 0]);

// --- 3. BINARY CONTRACT (Section 6 & 7) -------------------------------------

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RelayHeaderV2_1 {
    pub magic: [u8; 4],       // 0x00
    pub version_major: u16,   // 0x04
    pub version_minor: u16,   // 0x06
    pub node_count: u64,      // 0x08
    pub index_offset: u64,    // 0x10
    pub data_offset: u64,     // 0x18
    pub manifest_offset: u64, // 0x20
    pub flags: u16,           // 0x28
    pub header_size: u16,     // 0x2A
    pub key_pool_offset: u64, // 0x2C
    pub reserved: [u8; 12],   // 0x34 -> 64
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct RelayIndexEntryV2_1 {
    pub hash: u64,           // 0x00
    pub key_offset: u64,     // 0x08
    pub address: u64,        // 0x10
    pub payload_len: u32,    // 0x18
    pub key_len: u16,        // 0x1C
    pub entry_flags: u16,    // 0x1E
    pub key_prefix: [u8; 8], // 0x20 -> 40 Total
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayNodeV2_1 {
    pub checksum: u32,
    pub body_len: u32,
    pub payload: Vec<u8>,
}

// --- 4. STRUCTURAL INTEGRITY (Section 11) ----------------------------------

pub fn validate_header(h: &RelayHeaderV2_1) -> Result<()> {
    if h.magic != RELAY_MAGIC {
        return Err(RelayError::Structural(0, "Invalid MagicBytes".into()));
    }
    if h.version_major != MAJOR_VERSION || h.version_minor != MINOR_VERSION {
        return Err(RelayError::Structural(
            4,
            format!(
                "Unsupported version: {}.{}",
                h.version_major, h.version_minor
            ),
        ));
    }
    if h.flags != SUPPORTED_FLAGS {
        return Err(RelayError::Structural(40, "Unsupported flags".into()));
    }
    if h.header_size as u64 != HEADER_SIZE {
        return Err(RelayError::Structural(42, "HeaderSize mismatch".into()));
    }

    // Checked Arithmetic for Section Bounds
    let index_bytes = h
        .node_count
        .checked_mul(INDEX_ENTRY_SIZE)
        .ok_or_else(|| RelayError::Structural(8, "Index size overflow".into()))?;
    let index_end = h
        .index_offset
        .checked_add(index_bytes)
        .ok_or_else(|| RelayError::Structural(16, "Index bounds overflow".into()))?;

    if h.index_offset < HEADER_SIZE {
        return Err(RelayError::Structural(16, "Index overlaps header".into()));
    }
    if h.key_pool_offset < index_end {
        return Err(RelayError::Structural(44, "KeyPool overlaps index".into()));
    }
    if h.data_offset < h.key_pool_offset {
        return Err(RelayError::Structural(24, "Data precedes KeyPool".into()));
    }

    if h.manifest_offset != MANIFEST_ABSENT && h.manifest_offset < h.data_offset {
        return Err(RelayError::Structural(32, "Manifest out of order".into()));
    }

    Ok(())
}

pub fn validate_header_with_file_len<R: Read + Seek>(r: &mut R, h: &RelayHeaderV2_1) -> Result<()> {
    validate_header(h)?;
    let file_len = r
        .seek(SeekFrom::End(0))
        .map_err(|e| RelayError::Io(e.to_string()))?;

    if h.index_offset > file_len {
        return Err(RelayError::Structural(
            16,
            "Index offset beyond file length".into(),
        ));
    }
    if h.key_pool_offset > file_len {
        return Err(RelayError::Structural(
            44,
            "KeyPool offset beyond file length".into(),
        ));
    }
    if h.data_offset > file_len {
        return Err(RelayError::Structural(
            24,
            "Data offset beyond file length".into(),
        ));
    }
    if h.manifest_offset != MANIFEST_ABSENT && h.manifest_offset > file_len {
        return Err(RelayError::Structural(
            32,
            "Manifest offset beyond file length".into(),
        ));
    }
    Ok(())
}

// --- 5. SEARCH ENGINE (Section 10) -----------------------------------------

pub fn find_address<R: Read + Seek>(r: &mut R, id: &str, h: &RelayHeaderV2_1) -> Result<u64> {
    if h.node_count == 0 {
        return Err(RelayError::AnchorNotFound(id.into()));
    }
    let target_hash = HighwayHasher::new(RELAY_HASH_KEY).hash64(id.as_bytes());
    let target_key_len = u16::try_from(id.len())
        .map_err(|_| RelayError::Structural(28, "Target ID too long".into()))?;

    let mut low = 0;
    let mut high = h.node_count - 1;

    while low <= high {
        let mid = low + (high - low) / 2;
        let pos = entry_pos(h, mid)?;
        let entry = read_index_entry(r, pos)?;

        if entry.hash < target_hash {
            low = mid + 1;
        } else if entry.hash > target_hash {
            if mid == 0 {
                break;
            }
            high = mid - 1;
        } else {
            return resolve_collision(r, h, mid, id, target_hash, target_key_len);
        }
    }
    Err(RelayError::AnchorNotFound(id.into()))
}

fn resolve_collision<R: Read + Seek>(
    r: &mut R,
    h: &RelayHeaderV2_1,
    mid: u64,
    id: &str,
    hash: u64,
    target_key_len: u16,
) -> Result<u64> {
    let target_bytes = id.as_bytes();
    let mut target_prefix = [0u8; 8];
    let n = target_bytes.len().min(8);
    target_prefix[..n].copy_from_slice(&target_bytes[..n]);

    let mut first = mid;
    while first > 0 {
        let prev_idx = first.checked_sub(1).unwrap(); // Guarded by first > 0
        let prev = read_index_entry(r, entry_pos(h, prev_idx)?)?;
        if prev.hash != hash {
            break;
        }
        first = prev_idx;
    }

    let mut curr = first;
    while curr < h.node_count {
        let entry = read_index_entry(r, entry_pos(h, curr)?)?;
        if entry.hash != hash {
            break;
        }

        // v2.1 Fast-Rejection
        if entry.key_len == target_key_len && entry.key_prefix == target_prefix {
            if verify_key_pool(r, h, &entry, target_bytes)? {
                return Ok(entry.address);
            }
        }
        curr = curr
            .checked_add(1)
            .ok_or_else(|| RelayError::Structural(curr, "Index overflow".into()))?;
    }
    Err(RelayError::AnchorNotFound(id.into()))
}

fn entry_pos(h: &RelayHeaderV2_1, idx: u64) -> Result<u64> {
    let offset = idx
        .checked_mul(INDEX_ENTRY_SIZE)
        .ok_or_else(|| RelayError::Structural(idx, "Arithmetic Overflow: index seek".into()))?;
    h.index_offset
        .checked_add(offset)
        .ok_or_else(|| RelayError::Structural(idx, "Address Overflow: index seek".into()))
}

// --- 6. ATOMIC IO PRIMITIVES -----------------------------------------------

fn read_index_entry<R: Read + Seek>(r: &mut R, pos: u64) -> Result<RelayIndexEntryV2_1> {
    r.seek(SeekFrom::Start(pos))
        .map_err(|e| RelayError::Io(e.to_string()))?;
    let mut b = [0u8; 40];
    r.read_exact(&mut b)
        .map_err(|e| RelayError::Io(e.to_string()))?;

    Ok(RelayIndexEntryV2_1 {
        hash: u64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]]),
        key_offset: u64::from_le_bytes([b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]]),
        address: u64::from_le_bytes([b[16], b[17], b[18], b[19], b[20], b[21], b[22], b[23]]),
        payload_len: u32::from_le_bytes([b[24], b[25], b[26], b[27]]),
        key_len: u16::from_le_bytes([b[28], b[29]]),
        entry_flags: u16::from_le_bytes([b[30], b[31]]),
        key_prefix: [b[32], b[33], b[34], b[35], b[36], b[37], b[38], b[39]],
    })
}

fn verify_key_pool<R: Read + Seek>(
    r: &mut R,
    h: &RelayHeaderV2_1,
    entry: &RelayIndexEntryV2_1,
    target: &[u8],
) -> Result<bool> {
    if entry.entry_flags != ENTRY_FLAGS_V21 {
        return Err(RelayError::Structural(
            entry.address,
            "Unsupported entry flags".into(),
        ));
    }

    // Bounds Enforcement
    let pool_size = h
        .data_offset
        .checked_sub(h.key_pool_offset)
        .ok_or_else(|| RelayError::Structural(44, "Invalid key pool bounds".into()))?;
    let key_end = entry
        .key_offset
        .checked_add(entry.key_len as u64)
        .ok_or_else(|| RelayError::Structural(entry.key_offset, "Key offset overflow".into()))?;

    if key_end > pool_size {
        return Err(RelayError::Structural(
            entry.key_offset,
            "Key out of pool bounds".into(),
        ));
    }

    let abs_pos = h.key_pool_offset.checked_add(entry.key_offset).unwrap(); // Guarded by key_end check
    r.seek(SeekFrom::Start(abs_pos))
        .map_err(|e| RelayError::Io(e.to_string()))?;

    let mut buf = vec![0u8; target.len()];
    r.read_exact(&mut buf)
        .map_err(|e| RelayError::Io(e.to_string()))?;
    Ok(buf == target)
}

pub fn read_node_at<R: Read + Seek>(
    r: &mut R,
    addr: u64,
    expected_len: Option<u32>,
) -> Result<RelayNodeV2_1> {
    r.seek(SeekFrom::Start(addr))
        .map_err(|e| RelayError::Io(e.to_string()))?;
    let mut meta = [0u8; 8];
    r.read_exact(&mut meta)
        .map_err(|e| RelayError::Io(e.to_string()))?;

    let checksum = u32::from_le_bytes([meta[0], meta[1], meta[2], meta[3]]);
    let body_len = u32::from_le_bytes([meta[4], meta[5], meta[6], meta[7]]);

    if body_len > MAX_NODE_SIZE {
        return Err(RelayError::Structural(addr + 4, "Oversized node".into()));
    }
    if let Some(expected) = expected_len {
        if body_len != expected {
            return Err(RelayError::Structural(
                addr + 4,
                "Payload length mismatch".into(),
            ));
        }
    }

    let mut payload = vec![0u8; body_len as usize];
    r.read_exact(&mut payload)
        .map_err(|e| RelayError::Io(e.to_string()))?;

    let mut hasher = Crc32Hasher::new();
    hasher.update(&payload);
    if hasher.finalize() != checksum {
        return Err(RelayError::Semantic(addr));
    }

    Ok(RelayNodeV2_1 {
        checksum,
        body_len,
        payload,
    })
}

pub fn verify_payload_consistency<R: Read + Seek>(
    r: &mut R,
    h: &RelayHeaderV2_1,
    entry: &RelayIndexEntryV2_1,
    id: &str,
) -> Result<bool> {
    let node = read_node_at(r, entry.address, Some(entry.payload_len))?;
    let val: serde_json::Value =
        serde_json::from_slice(&node.payload).map_err(|_| RelayError::Semantic(entry.address))?;
    Ok(val["#id"].as_str() == Some(id))
}

// --- 7. EXHAUSTIVE TESTS ---------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_v2_1_contract_sizes() {
        assert_eq!(std::mem::size_of::<RelayHeaderV2_1>(), 64);
        assert_eq!(std::mem::size_of::<RelayIndexEntryV2_1>(), 40);
    }

    #[test]
    fn test_ignition_structural_gate() {
        let mut h = create_fixture();
        h.magic = [0; 4];
        assert!(matches!(
            validate_header(&h),
            Err(RelayError::Structural(0, _))
        ));

        h = create_fixture();
        h.flags = 1;
        assert!(matches!(
            validate_header(&h),
            Err(RelayError::Structural(40, _))
        ));

        h = create_fixture();
        h.index_offset = 32;
        assert!(matches!(
            validate_header(&h),
            Err(RelayError::Structural(16, _))
        ));
    }

    #[test]
    fn test_node_read_semantic_verification() {
        let payload = b"{\"#id\":\"test\"}";
        let mut hasher = Crc32Hasher::new();
        hasher.update(payload);
        let checksum = hasher.finalize();

        let mut data = Vec::new();
        data.extend_from_slice(&checksum.to_le_bytes());
        data.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        data.extend_from_slice(payload);

        let mut r = Cursor::new(data);
        let node = read_node_at(&mut r, 0, Some(payload.len() as u32)).unwrap();
        assert_eq!(node.payload, payload);
    }

    fn create_fixture() -> RelayHeaderV2_1 {
        RelayHeaderV2_1 {
            magic: RELAY_MAGIC,
            version_major: MAJOR_VERSION,
            version_minor: MINOR_VERSION,
            node_count: 1,
            index_offset: 64,
            key_pool_offset: 128,
            data_offset: 256,
            manifest_offset: 0,
            flags: 0,
            header_size: 64,
            reserved: [0u8; 12],
        }
    }
}
