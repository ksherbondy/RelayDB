//! Source validation is the first real stage of the RelayDB pipeline.
//!
//! The compiler deliberately keeps this layer separate from file I/O and from
//! the `.relay` byte writer. That separation makes the behavior easy to test:
//! JSON/JSONL parsing produces `serde_json::Value` records, this module checks
//! the V1 rules, and later stages can assume they received a valid model.
//!
//! The JavaScript reference compiler performs the same conceptual work. The
//! important design choice is that validation is global: relationships can
//! point into another input file, so target resolution happens only after all
//! records have been collected.

use serde_json::{Map, Value};
use std::collections::{HashMap, HashSet};
use std::fmt;

// JavaScript represents numbers as IEEE-754 doubles. Integers above this value
// cannot be represented exactly, so V1 rejects them instead of silently
// changing a developer's data during a JS/Rust round trip.
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// The normalized identity and collection information for one source object.
/// `value` remains intact because the reader must later reconstruct the
/// developer-facing record, not just an internal index table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRecord {
    /// Developer-owned public identity from `#` or `#id`.
    pub id: String,
    /// Non-empty `^` collection/type string used to scope identity.
    pub collection: String,
    /// Original JSON object, retained for reconstruction and projection.
    pub value: Value,
}

/// The validated in-memory source representation passed to profiling and
/// logical compilation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceModel {
    /// Records retain source order because it is useful for deterministic
    /// output and makes internal relationship indexes stable between builds.
    pub records: Vec<SourceRecord>,
}

/// A human-readable validation failure. Keeping the record index lets the CLI
/// report several errors in one pass instead of making the user fix them one
/// at a time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceDiagnostic {
    /// Zero-based source record position, if the error belongs to one record.
    pub record_index: Option<usize>,
    /// Human-readable explanation suitable for a CLI error report.
    pub message: String,
}

impl fmt::Display for SourceDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.record_index {
            Some(index) => write!(f, "record {}: {}", index + 1, self.message),
            None => f.write_str(&self.message),
        }
    }
}

/// Validate raw JSON records against the RelayDB V1 source contract.
///
/// # Arguments
///
/// * `values` - Objects already parsed from JSON or JSONL. File parsing is
///   intentionally outside this function so the same validator works for
///   arrays, line-delimited input, and tests.
///
/// # Returns
///
/// `Ok(SourceModel)` only when every record and relationship is valid.
/// `Err(Vec<SourceDiagnostic>)` aggregates all discovered failures.
///
/// # Validation rules
///
/// * Exactly one non-empty `#` or `#id` is required.
/// * `^` must be a non-empty string.
/// * IDs are unique inside a collection but may repeat across collections.
/// * `@` values are null, a string, or an array of strings.
/// * Non-null relationship cardinality cannot change within a collection.
/// * Relationship targets must exist and must not be globally ambiguous.
/// * Integers must fit JavaScript's safe integer range.
///
/// # Example
///
/// ```
/// # use relay_compiler::source_model::validate_records;
/// # use serde_json::json;
/// let model = validate_records(&[
///     json!({"#id":"actor-1", "^":"actors"}),
///     json!({"#id":"film-1", "^":"films", "@director":"actor-1"}),
/// ]).expect("valid source");
/// assert_eq!(model.records.len(), 2);
/// ```
pub fn validate_records(values: &[Value]) -> Result<SourceModel, Vec<SourceDiagnostic>> {
    // These indexes are temporary compiler structures. They are built from
    // borrowed strings so validation does not needlessly allocate a second
    // copy of every identity while it is checking the input.
    let mut records = Vec::with_capacity(values.len());
    let mut diagnostics = Vec::new();
    let mut identities: HashMap<(&str, &str), usize> = HashMap::new();
    let mut targets: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut relationship_shapes: HashMap<(&str, &str), bool> = HashMap::new();

    // Pass one validates local shape and builds indexes needed for global
    // checks: collection-scoped uniqueness, all IDs by name, and relationship
    // cardinality by (collection, field).
    for (record_index, value) in values.iter().enumerate() {
        let Some(object) = value.as_object() else {
            diagnostics.push(error(record_index, "record must be a JSON object"));
            continue;
        };

        let identity = read_identity(object, record_index, &mut diagnostics);
        let collection = read_collection(object, record_index, &mut diagnostics);
        validate_numbers(value, record_index, &mut diagnostics);

        if let (Some(id), Some(collection)) = (identity, collection) {
            if identities.insert((collection, id), record_index).is_some() {
                diagnostics.push(error(
                    record_index,
                    format!(
                        "duplicate identity '{}' in collection '^{}'",
                        id, collection
                    ),
                ));
            }
            targets.entry(id).or_default().insert(collection);
            records.push(SourceRecord {
                id: id.to_string(),
                collection: collection.to_string(),
                value: value.clone(),
            });
        }

        for (key, value) in object {
            if !key.starts_with('@') {
                continue;
            }
            let Some((collection, _)) = collection.zip(identity) else {
                continue;
            };
            let Some(is_array) = relationship_shape(value) else {
                if !value.is_null() {
                    diagnostics.push(error(
                        record_index,
                        format!(
                            "relationship '{}' must be null, a string, or an array of strings",
                            key
                        ),
                    ));
                }
                continue;
            };
            if let Some(previous) = relationship_shapes.insert((collection, key), is_array)
                && previous != is_array
            {
                diagnostics.push(error(
                    record_index,
                    format!(
                        "relationship '{}' mixes scalar and array cardinality in '^{}'",
                        key, collection
                    ),
                ));
            }
        }
    }

    // Pass two resolves every @ target now that records from every file are
    // known. A bare ID is valid only when it identifies one record globally;
    // duplicate IDs in different collections must be disambiguated by a
    // higher-level API or future qualified-reference syntax.
    for (record_index, record) in records.iter().enumerate() {
        let Some(object) = record.value.as_object() else {
            continue;
        };
        for (key, value) in object.iter().filter(|(key, _)| key.starts_with('@')) {
            for target in relationship_targets(value) {
                match targets.get(target) {
                    None => diagnostics.push(error(
                        record_index,
                        format!(
                            "relationship '{}' targets missing identity '{}'",
                            key, target
                        ),
                    )),
                    Some(collections) if collections.len() > 1 => diagnostics.push(error(
                        record_index,
                        format!(
                            "relationship '{}' targets ambiguous identity '{}'",
                            key, target
                        ),
                    )),
                    Some(_) => {}
                }
            }
        }
    }

    if diagnostics.is_empty() {
        Ok(SourceModel { records })
    } else {
        Err(diagnostics)
    }
}

fn read_identity<'a>(
    object: &'a Map<String, Value>,
    index: usize,
    diagnostics: &mut Vec<SourceDiagnostic>,
) -> Option<&'a str> {
    let has_hash = object.contains_key("#");
    let has_hash_id = object.contains_key("#id");
    if has_hash == has_hash_id {
        diagnostics.push(error(
            index,
            "record requires exactly one non-empty '#' or '#id' identity",
        ));
        return None;
    }
    let key = if has_hash { "#" } else { "#id" };
    match object
        .get(key)
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
    {
        Some(id) => Some(id),
        None => {
            diagnostics.push(error(
                index,
                format!("'{}' must be a non-empty string", key),
            ));
            None
        }
    }
}

fn read_collection<'a>(
    object: &'a Map<String, Value>,
    index: usize,
    diagnostics: &mut Vec<SourceDiagnostic>,
) -> Option<&'a str> {
    match object
        .get("^")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    {
        Some(collection) => Some(collection),
        None => {
            diagnostics.push(error(index, "'^' must be a non-empty string"));
            None
        }
    }
}

fn relationship_shape(value: &Value) -> Option<bool> {
    // `None` means null: null is allowed but intentionally does not establish
    // whether a relationship field is scalar or array cardinality.
    if value.is_null() {
        return None;
    }
    if value.as_str().is_some() {
        return Some(false);
    }
    value
        .as_array()
        .and_then(|items| items.iter().all(Value::is_string).then_some(true))
}

fn relationship_targets(value: &Value) -> Vec<&str> {
    value
        .as_str()
        .map(|target| vec![target])
        .unwrap_or_else(|| {
            value
                .as_array()
                .map(|items| items.iter().filter_map(Value::as_str).collect())
                .unwrap_or_default()
        })
}

fn validate_numbers(value: &Value, index: usize, diagnostics: &mut Vec<SourceDiagnostic>) {
    // Walk nested arrays and objects too. A large number hidden inside a
    // payload must be rejected just like a top-level numeric field.
    match value {
        Value::Number(number) => {
            if let Some(integer) = number.as_u64() {
                if integer > MAX_SAFE_INTEGER {
                    diagnostics.push(error(
                        index,
                        "integer exceeds JavaScript safe integer range",
                    ));
                }
            } else if let Some(integer) = number.as_i64() {
                if integer.unsigned_abs() > MAX_SAFE_INTEGER {
                    diagnostics.push(error(
                        index,
                        "integer exceeds JavaScript safe integer range",
                    ));
                }
            } else if number.as_f64().is_none_or(|number| !number.is_finite()) {
                diagnostics.push(error(index, "number must be finite"));
            }
        }
        Value::Array(items) => items
            .iter()
            .for_each(|item| validate_numbers(item, index, diagnostics)),
        Value::Object(object) => object
            .values()
            .for_each(|item| validate_numbers(item, index, diagnostics)),
        _ => {}
    }
}

fn error(index: usize, message: impl Into<String>) -> SourceDiagnostic {
    SourceDiagnostic {
        record_index: Some(index),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture(name: &str) -> Vec<Value> {
        let contents = match name {
            "reference-v1.jsonl" => include_str!(
                "../../deprecated/javascript-reference/js_ref_code/fixtures/reference-v1.jsonl"
            ),
            "invalid-identity.jsonl" => {
                include_str!(
                    "../../deprecated/javascript-reference/js_ref_code/fixtures/invalid-identity.jsonl"
                )
            }
            "invalid-mixed-cardinality.jsonl" => {
                include_str!(
                    "../../deprecated/javascript-reference/js_ref_code/fixtures/invalid-mixed-cardinality.jsonl"
                )
            }
            "ambiguous-ids.jsonl" => include_str!(
                "../../deprecated/javascript-reference/js_ref_code/fixtures/ambiguous-ids.jsonl"
            ),
            "cycle.jsonl" => include_str!(
                "../../deprecated/javascript-reference/js_ref_code/fixtures/cycle.jsonl"
            ),
            _ => panic!("unknown fixture: {name}"),
        };
        contents
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    #[test]
    fn accepts_collection_scoped_ids_and_null_relationships() {
        let values = vec![
            json!({"#id":"same","^":"people","@friend":null}),
            json!({"#":"same","^":"companies"}),
        ];
        assert_eq!(validate_records(&values).unwrap().records.len(), 2);
    }

    #[test]
    fn rejects_mixed_cardinality_and_ambiguous_targets() {
        let values = vec![
            json!({"#id":"same","^":"people"}),
            json!({"#id":"same","^":"companies"}),
            json!({"#id":"movie","^":"movies","@cast":"same"}),
            json!({"#id":"other","^":"movies","@cast":["same"]}),
        ];
        let errors = validate_records(&values).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("ambiguous"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("cardinality"))
        );
    }

    #[test]
    fn rejects_unsafe_integers() {
        let errors =
            validate_records(&[json!({"#id":"x","^":"items","value":9007199254740992u64})])
                .unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("safe integer"))
        );
    }

    #[test]
    fn accepts_reference_v1_fixture_values() {
        let model = validate_records(&fixture("reference-v1.jsonl")).unwrap();
        assert_eq!(model.records.len(), 3);
        assert_eq!(model.records[2].value["~large"], json!(2147483648u64));
        assert_eq!(model.records[2].value["~optional"], Value::Null);
        assert!(
            !model.records[0]
                .value
                .as_object()
                .unwrap()
                .contains_key("~missing")
        );
    }

    #[test]
    fn matches_reference_fixture_validation_cases() {
        assert!(validate_records(&fixture("invalid-identity.jsonl")).is_err());
        assert!(validate_records(&fixture("invalid-mixed-cardinality.jsonl")).is_err());
        assert!(validate_records(&fixture("ambiguous-ids.jsonl")).is_ok());
        assert!(validate_records(&fixture("cycle.jsonl")).is_ok());
    }
}
