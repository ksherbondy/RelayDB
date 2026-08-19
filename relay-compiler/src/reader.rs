//! Read-only runtime for a compiled `.relay` artifact.
//!
//! The current layout is intentionally simple: a fixed header stores the
//! jump-table offset, JSON records occupy the bytes after the header, and each
//! record ends with a zero byte. `open` loads those records once; the public
//! methods then operate on the in-memory values.
//!
//! This mirrors the JavaScript prototype's logical API while keeping the
//! physical parser small enough to audit. A future dictionary or columnar
//! format can replace this parser without changing the query concepts below.

use crate::{HEADER_SIZE, POINTER_START, TERMINATOR, extract_anchor_id};
use serde_json::Value;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(Debug)]
pub enum ReaderError {
    /// The artifact could not be read from the filesystem.
    Io(std::io::Error),
    /// The artifact bytes violate the current structural or JSON rules.
    InvalidArtifact(String),
    /// An unqualified ID matched records in more than one collection.
    AmbiguousId(String),
}

impl fmt::Display for ReaderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "I/O error: {}", error),
            Self::InvalidArtifact(message) => write!(f, "invalid artifact: {}", message),
            Self::AmbiguousId(id) => write!(f, "ambiguous ID: {}", id),
        }
    }
}

impl std::error::Error for ReaderError {}

impl From<std::io::Error> for ReaderError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RelayDb {
    /// Parsed source records. The current reader favors simple predictable
    /// reads; a future large-artifact reader can replace this with lazy sections.
    records: Vec<Value>,
}

/// Comparison operation used by [`QueryFilter`].
#[derive(Debug, Clone, PartialEq)]
pub enum QueryOperator {
    /// Equality; strings compare case-insensitively.
    Eq,
    /// Logical inverse of equality.
    Ne,
    /// Numeric greater-than.
    Gt,
    /// Numeric greater-than-or-equal.
    Gte,
    /// Numeric less-than.
    Lt,
    /// Numeric less-than-or-equal.
    Lte,
}

/// One field predicate applied by [`RelayDb::query`].
#[derive(Debug, Clone, PartialEq)]
pub struct QueryFilter {
    /// Exact JSON object key to inspect, such as `~year` or `name`.
    pub field: String,
    /// Comparison operation.
    pub operator: QueryOperator,
    /// Value compared against the record field.
    pub expected: Value,
}

/// Bounded result set returned by [`RelayDb::query_page`].
#[derive(Debug, Clone, PartialEq)]
pub struct QueryPage {
    /// Records in this page, optionally projected.
    pub results: Vec<Value>,
    /// Number of records matching before pagination.
    pub total: usize,
    /// Requested starting offset.
    pub offset: usize,
    /// Requested maximum result count.
    pub limit: usize,
    /// Whether another page exists.
    pub has_more: bool,
    /// Offset to use for the next page, or `None` at the end.
    pub next_offset: Option<usize>,
}

impl RelayDb {
    /// Open an artifact and validate its structural boundaries before parsing.
    ///
    /// Offsets come from a file, so they are untrusted input. Checking the
    /// header pointer before slicing is what prevents malformed artifacts from
    /// turning into out-of-bounds reads.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use relay_compiler::reader::RelayDb;
    /// let db = RelayDb::open("movies.relay")?;
    /// let movie = db.get("film-1", Some("films"))?;
    /// # let _: Option<serde_json::Value> = movie;
    /// # Ok::<(), relay_compiler::reader::ReaderError>(())
    /// ```
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, ReaderError> {
        let bytes = fs::read(path)?;
        if bytes.len() < HEADER_SIZE as usize {
            return Err(ReaderError::InvalidArtifact("truncated header".into()));
        }
        let index_offset = u64::from_le_bytes(
            bytes[POINTER_START as usize..(POINTER_START as usize + 8)]
                .try_into()
                .map_err(|_| ReaderError::InvalidArtifact("invalid index pointer".into()))?,
        ) as usize;
        if index_offset < HEADER_SIZE as usize || index_offset > bytes.len() {
            return Err(ReaderError::InvalidArtifact(
                "index pointer out of bounds".into(),
            ));
        }

        let mut records = Vec::new();
        for payload in bytes[HEADER_SIZE as usize..index_offset].split(|byte| *byte == TERMINATOR) {
            if payload.is_empty() {
                continue;
            }
            let value = serde_json::from_slice(payload)
                .map_err(|error| ReaderError::InvalidArtifact(error.to_string()))?;
            if extract_anchor_id(&value).is_none()
                || value.get("^").and_then(Value::as_str).is_none()
            {
                return Err(ReaderError::InvalidArtifact(
                    "record is missing identity or collection".into(),
                ));
            }
            records.push(value);
        }
        Ok(Self { records })
    }

    pub fn get(&self, id: &str, collection: Option<&str>) -> Result<Option<Value>, ReaderError> {
        // An unqualified ID is allowed only when it identifies one record
        // globally. Returning the first match would make results depend on
        // source-file order, so ambiguity is an explicit runtime error.
        let matches: Vec<&Value> = self
            .records
            .iter()
            .filter(|record| {
                extract_anchor_id(record) == Some(id)
                    && collection.is_none_or(|collection| {
                        record.get("^").and_then(Value::as_str) == Some(collection)
                    })
            })
            .collect();
        if collection.is_none() && matches.len() > 1 {
            return Err(ReaderError::AmbiguousId(id.to_string()));
        }
        Ok(matches.first().map(|record| (*record).clone()))
    }

    /// Test whether a public ID exists, optionally within one collection.
    pub fn has(&self, id: &str, collection: Option<&str>) -> Result<bool, ReaderError> {
        Ok(self.get(id, collection)?.is_some())
    }

    /// Return every record matching a collection and all supplied filters.
    ///
    /// Missing fields do not match. Array fields use existential matching: an
    /// equality or numeric comparison succeeds when any element matches.
    pub fn query(&self, collection: Option<&str>, filters: &[QueryFilter]) -> Vec<Value> {
        self.records
            .iter()
            .filter(|record| {
                collection.is_none_or(|collection| {
                    record.get("^").and_then(Value::as_str) == Some(collection)
                })
            })
            .filter(|record| filters.iter().all(|filter| matches_filter(record, filter)))
            .cloned()
            .collect()
    }

    pub fn query_page(
        &self,
        collection: Option<&str>,
        filters: &[QueryFilter],
        offset: usize,
        limit: usize,
        fields: Option<&[String]>,
    ) -> QueryPage {
        // Pagination is applied after filtering. `total` describes all
        // matches, while `results` contains only the requested window.
        let matches = self.query(collection, filters);
        let total = matches.len();
        let results = matches
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|record| project_record(record, fields))
            .collect::<Vec<_>>();
        let next_offset = offset
            .checked_add(results.len())
            .filter(|next| *next < total);
        QueryPage {
            results,
            total,
            offset,
            limit,
            has_more: next_offset.is_some(),
            next_offset,
        }
    }

    /// Return public IDs for the same match set used by [`RelayDb::query`].
    pub fn select_ids(&self, collection: Option<&str>, filters: &[QueryFilter]) -> Vec<String> {
        self.query(collection, filters)
            .iter()
            .filter_map(|record| extract_anchor_id(record).map(str::to_string))
            .collect()
    }

    /// Release the reader. The current implementation owns memory only, so
    /// dropping `RelayDb` is sufficient; this method exists for JS API parity.
    pub fn close(self) {}

    /// Fetch a record and recursively replace `@` IDs with records up to
    /// `depth` edges away. Depth zero returns relationship IDs unchanged.
    ///
    /// Cycles terminate through a path-local visited set. Missing relationship
    /// targets remain as their original IDs rather than causing hydration to
    /// fail, which is useful when reading legacy or externally linked data.
    pub fn get_hydrated(
        &self,
        id: &str,
        collection: Option<&str>,
        depth: usize,
    ) -> Result<Option<Value>, ReaderError> {
        let Some(record) = self.get(id, collection)? else {
            return Ok(None);
        };
        let mut visited = Vec::new();
        Ok(Some(self.hydrate(record, depth, &mut visited)?))
    }

    pub fn records(&self) -> &[Value] {
        &self.records
    }

    fn hydrate(
        &self,
        mut record: Value,
        depth: usize,
        visited: &mut Vec<String>,
    ) -> Result<Value, ReaderError> {
        if depth == 0 {
            return Ok(record);
        }
        let Some(id) = extract_anchor_id(&record).map(str::to_string) else {
            return Ok(record);
        };
        // Relationships form a graph, not necessarily a tree. The path-local
        // visited list prevents infinite recursion while still allowing a
        // record to appear through another independent branch.
        if visited.contains(&id) {
            return Ok(record);
        }
        visited.push(id);
        if let Some(object) = record.as_object_mut() {
            let fields = object
                .keys()
                .filter(|field| field.starts_with('@'))
                .cloned()
                .collect::<Vec<_>>();
            for field in fields {
                let value = object.get(&field).cloned().unwrap_or(Value::Null);
                object.insert(field, self.hydrate_relationship(value, depth, visited)?);
            }
        }
        visited.pop();
        Ok(record)
    }

    fn hydrate_relationship(
        &self,
        value: Value,
        depth: usize,
        visited: &mut Vec<String>,
    ) -> Result<Value, ReaderError> {
        if depth == 0 || value.is_null() {
            return Ok(value);
        }
        if let Some(id) = value.as_str() {
            if let Some(target) = self.get(id, None)? {
                return self.hydrate(target, depth - 1, visited);
            }
            return Ok(value);
        }
        if let Some(values) = value.as_array() {
            return Ok(Value::Array(
                values
                    .iter()
                    .cloned()
                    .map(|value| self.hydrate_relationship(value, depth, visited))
                    .collect::<Result<Vec<_>, _>>()?,
            ));
        }
        Ok(value)
    }
}

fn project_record(mut record: Value, fields: Option<&[String]>) -> Value {
    // A projection never removes identity or collection metadata; otherwise
    // the returned object could no longer be addressed by the reader API.
    let Some(fields) = fields else {
        return record;
    };
    let Some(object) = record.as_object_mut() else {
        return record;
    };
    let requested = fields.iter().collect::<std::collections::HashSet<_>>();
    object.retain(|field, _| {
        field == "#" || field == "#id" || field == "^" || requested.contains(field)
    });
    record
}

fn matches_filter(record: &Value, filter: &QueryFilter) -> bool {
    // Array fields use existential matching: one matching element is enough,
    // matching the JavaScript reference behavior for tag arrays.
    let Some(actual) = record.get(&filter.field) else {
        return false;
    };
    let values = actual
        .as_array()
        .map_or_else(|| vec![actual], |values| values.iter().collect());
    values
        .iter()
        .any(|actual| compare_value(actual, &filter.expected, &filter.operator))
}

fn compare_value(actual: &Value, expected: &Value, operator: &QueryOperator) -> bool {
    if matches!(operator, QueryOperator::Eq | QueryOperator::Ne) {
        let equal = match (actual, expected) {
            (Value::String(actual), Value::String(expected)) => {
                actual.eq_ignore_ascii_case(expected)
            }
            _ => actual == expected,
        };
        return if matches!(operator, QueryOperator::Eq) {
            equal
        } else {
            !equal
        };
    }
    let (Some(actual), Some(expected)) = (actual.as_f64(), expected.as_f64()) else {
        return false;
    };
    match operator {
        QueryOperator::Gt => actual > expected,
        QueryOperator::Gte => actual >= expected,
        QueryOperator::Lt => actual < expected,
        QueryOperator::Lte => actual <= expected,
        QueryOperator::Eq | QueryOperator::Ne => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_fixture() -> std::path::PathBuf {
        let path =
            std::env::temp_dir().join(format!("relaydb-reader-{}.relay", std::process::id()));
        let records = [
            serde_json::json!({"#id":"123","^":"people","name":"Person 123"}),
            serde_json::json!({"#id":"123","^":"movies","name":"Movie 123"}),
        ];
        let mut bytes = vec![0u8; HEADER_SIZE as usize];
        for record in records {
            bytes.extend(serde_json::to_vec(&record).unwrap());
            bytes.push(TERMINATOR);
        }
        let index_offset = bytes.len() as u64;
        bytes.extend(format!("123\t{}\n", HEADER_SIZE).as_bytes());
        bytes[POINTER_START as usize..POINTER_START as usize + 8]
            .copy_from_slice(&index_offset.to_le_bytes());
        let mut file = fs::File::create(&path).unwrap();
        file.write_all(&bytes).unwrap();
        path
    }

    #[test]
    fn reads_collection_scoped_records_and_rejects_ambiguous_lookup() {
        let path = write_fixture();
        let db = RelayDb::open(&path).unwrap();
        assert!(matches!(
            db.get("123", None),
            Err(ReaderError::AmbiguousId(_))
        ));
        assert_eq!(
            db.get("123", Some("movies")).unwrap().unwrap()["name"],
            "Movie 123"
        );
        assert!(db.has("123", Some("people")).unwrap());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn queries_case_insensitively_and_hydrates_to_a_depth_limit() {
        let db = RelayDb {
            records: vec![
                serde_json::json!({"#id":"actor","^":"actors","name":"Ada"}),
                serde_json::json!({"#id":"film","^":"films","name":"Example","~year":2024,"@director":"actor"}),
            ],
        };
        let results = db.query(
            Some("films"),
            &[QueryFilter {
                field: "name".into(),
                operator: QueryOperator::Eq,
                expected: serde_json::json!("EXAMPLE"),
            }],
        );
        assert_eq!(results.len(), 1);
        let hydrated = db.get_hydrated("film", Some("films"), 1).unwrap().unwrap();
        assert_eq!(hydrated["@director"]["#id"], "actor");
    }

    #[test]
    fn pages_projects_and_selects_ids() {
        let db = RelayDb {
            records: vec![
                serde_json::json!({"#id":"one","^":"films","name":"One","~year":2000}),
                serde_json::json!({"#id":"two","^":"films","name":"Two","~year":2024}),
            ],
        };
        let fields = vec!["name".to_string()];
        let page = db.query_page(Some("films"), &[], 0, 1, Some(&fields));
        assert_eq!(page.total, 2);
        assert!(page.has_more);
        assert_eq!(page.next_offset, Some(1));
        assert_eq!(page.results[0]["#id"], "one");
        assert_eq!(page.results[0]["name"], "One");
        assert!(page.results[0].get("~year").is_none());
        assert_eq!(db.select_ids(Some("films"), &[]), vec!["one", "two"]);
    }
}
