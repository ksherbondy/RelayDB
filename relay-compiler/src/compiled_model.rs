//! Logical compilation converts validated source records into runtime-friendly
//! relationship indexes.
//!
//! This is intentionally not the `.relay` writer. Keeping a logical model
//! first lets us change the physical format later without changing validation,
//! relationship semantics, or reader behavior.

use crate::source_model::SourceModel;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompiledRelationship {
    /// The source explicitly contained JSON null.
    Null,
    /// One public ID became one internal record position.
    Scalar(usize),
    /// An array of public IDs became an array of internal positions.
    Array(Vec<usize>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledRecord {
    /// Public developer-owned identity; never replaced by the internal index.
    pub id: String,
    /// Collection used for collection-qualified lookup.
    pub collection: String,
    /// Keep the original object for reconstruction and projection. The index
    /// table accelerates traversal; it does not replace public data.
    pub value: Value,
    /// Relationship fields mapped to internal record positions.
    pub relationships: HashMap<String, CompiledRelationship>,
}

/// Runtime-oriented logical database representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledModel {
    /// Records indexed by their stable compilation position.
    pub records: Vec<CompiledRecord>,
    /// `(collection, public_id) -> record position` lookup table.
    pub by_collection_id: HashMap<(String, String), usize>,
}

/// Compile a validated source model into integer-indexed relationships.
///
/// # Why integer indexes?
///
/// Public IDs are strings because they belong to the developer-facing data
/// model. Runtime traversal repeatedly follows those links, so this phase
/// resolves each target once and stores a compact `usize` position instead.
/// The original JSON remains available on each `CompiledRecord`.
///
/// # Errors
///
/// Normally validation catches missing and ambiguous targets first. This
/// function repeats those checks at its boundary so it remains safe when used
/// independently by another Rust caller.
///
/// # Example
///
/// ```
/// # use relay_compiler::compiled_model::{compile_model, CompiledRelationship};
/// # use relay_compiler::source_model::validate_records;
/// # use serde_json::json;
/// let source = validate_records(&[
///     json!({"#id":"actor", "^":"actors"}),
///     json!({"#id":"film", "^":"films", "@director":"actor"}),
/// ]).unwrap();
/// let compiled = compile_model(&source).unwrap();
/// assert_eq!(compiled.records[1].relationships["@director"], CompiledRelationship::Scalar(0));
/// ```
pub fn compile_model(model: &SourceModel) -> Result<CompiledModel, String> {
    // `by_collection_id` preserves the identity rule used by public lookup.
    // `by_id` is separate because relationship strings are currently
    // unqualified and therefore must be rejected when an ID is ambiguous.
    let mut by_collection_id = HashMap::new();
    let mut by_id: HashMap<&str, Vec<usize>> = HashMap::new();

    for (index, record) in model.records.iter().enumerate() {
        by_collection_id.insert((record.collection.clone(), record.id.clone()), index);
        by_id.entry(&record.id).or_default().push(index);
    }

    let mut records = Vec::with_capacity(model.records.len());
    // Build one compiled record at a time, retaining the exact relationship
    // shape while replacing only its targets with integer positions.
    for record in &model.records {
        let mut relationships = HashMap::new();
        if let Some(object) = record.value.as_object() {
            for (field, value) in object.iter().filter(|(field, _)| field.starts_with('@')) {
                let relationship = compile_relationship(value, field, &by_id)?;
                relationships.insert(field.clone(), relationship);
            }
        }
        records.push(CompiledRecord {
            id: record.id.clone(),
            collection: record.collection.clone(),
            value: record.value.clone(),
            relationships,
        });
    }

    Ok(CompiledModel {
        records,
        by_collection_id,
    })
}

fn compile_relationship(
    value: &Value,
    field: &str,
    by_id: &HashMap<&str, Vec<usize>>,
) -> Result<CompiledRelationship, String> {
    if value.is_null() {
        return Ok(CompiledRelationship::Null);
    }
    if let Some(target) = value.as_str() {
        return Ok(CompiledRelationship::Scalar(resolve_target(
            target, field, by_id,
        )?));
    }
    if let Some(targets) = value.as_array() {
        let indexes = targets
            .iter()
            .map(|target| {
                target
                    .as_str()
                    .ok_or_else(|| format!("relationship '{}' contains a non-string target", field))
                    .and_then(|target| resolve_target(target, field, by_id))
            })
            .collect::<Result<Vec<_>, _>>()?;
        return Ok(CompiledRelationship::Array(indexes));
    }
    Err(format!("relationship '{}' has an invalid value", field))
}

fn resolve_target(
    target: &str,
    field: &str,
    by_id: &HashMap<&str, Vec<usize>>,
) -> Result<usize, String> {
    match by_id.get(target) {
        Some(indexes) if indexes.len() == 1 => Ok(indexes[0]),
        Some(_) => Err(format!(
            "relationship '{}' targets ambiguous identity '{}'",
            field, target
        )),
        None => Err(format!(
            "relationship '{}' targets missing identity '{}'",
            field, target
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_model::validate_records;
    use serde_json::json;

    #[test]
    fn compiles_relationships_to_internal_indexes() {
        let values = vec![
            json!({"#id":"actor","^":"actors"}),
            json!({"#id":"film","^":"films","@director":"actor","@cast":["actor"],"@optional":null}),
        ];
        let model = validate_records(&values).unwrap();
        let compiled = compile_model(&model).unwrap();
        assert_eq!(
            compiled.records[1].relationships["@director"],
            CompiledRelationship::Scalar(0)
        );
        assert_eq!(
            compiled.records[1].relationships["@cast"],
            CompiledRelationship::Array(vec![0])
        );
        assert_eq!(
            compiled.records[1].relationships["@optional"],
            CompiledRelationship::Null
        );
    }
}
