//! Field profiling sits between validation and physical encoding.
//!
//! A compiler can use these statistics to choose compact storage later. For
//! V1, the profile is intentionally descriptive rather than prescriptive: it
//! records what the source contains without changing the source values.

use crate::source_model::SourceModel;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct FieldProfile {
    /// Collection whose records contributed to this profile.
    pub collection: String,
    /// Original field name, including a reserved prefix such as `@` or `~`.
    pub field: String,
    /// Number of records in the collection, used as the profile denominator.
    pub record_count: usize,
    /// Number of records where the field exists, including explicit nulls.
    pub present_count: usize,
    /// Number of present values equal to JSON null.
    pub null_count: usize,
    /// Number of present non-null values that are not arrays.
    pub scalar_count: usize,
    /// Number of present values represented as JSON arrays.
    pub array_count: usize,
    /// Smallest numeric scalar or array element, when one exists.
    pub numeric_min: Option<f64>,
    /// Largest numeric scalar or array element, when one exists.
    pub numeric_max: Option<f64>,
}

/// Calculate per-collection field statistics from a validated model.
///
/// `record_count` is the denominator for a collection, while
/// `present_count` is the number of records that actually contain the field.
/// Their difference therefore preserves the important distinction between a
/// missing property and a property explicitly set to `null`.
///
/// # Output ordering
///
/// Profiles are sorted by collection and then field name. This is deliberate:
/// deterministic output makes generated manifests, snapshots, and future
/// encoding plans reproducible.
///
/// # Example
///
/// ```
/// # use relay_compiler::profile::profile_model;
/// # use relay_compiler::source_model::validate_records;
/// # use serde_json::json;
/// let model = validate_records(&[json!({"#id":"one", "^":"items", "~score":3})]).unwrap();
/// let profiles = profile_model(&model);
/// assert_eq!(profiles[0].numeric_min, Some(3.0));
/// ```
pub fn profile_model(model: &SourceModel) -> Vec<FieldProfile> {
    // BTreeMap gives callers stable output order. Stable profiles make tests,
    // generated manifests, and future release diffs much easier to review.
    let mut profiles: BTreeMap<(String, String), FieldProfile> = BTreeMap::new();
    let mut collection_counts: BTreeMap<String, usize> = BTreeMap::new();

    for record in &model.records {
        *collection_counts
            .entry(record.collection.clone())
            .or_default() += 1;
        let Some(object) = record.value.as_object() else {
            continue;
        };

        for (field, value) in object {
            // Identity and collection metadata describe the record itself;
            // they are not payload lanes that need field statistics.
            if field == "#" || field == "#id" || field == "^" {
                continue;
            }
            let key = (record.collection.clone(), field.clone());
            let profile = profiles.entry(key).or_insert_with(|| FieldProfile {
                collection: record.collection.clone(),
                field: field.clone(),
                record_count: 0,
                present_count: 0,
                null_count: 0,
                scalar_count: 0,
                array_count: 0,
                numeric_min: None,
                numeric_max: None,
            });
            profile.present_count += 1;
            if value.is_null() {
                profile.null_count += 1;
            } else if value.is_array() {
                profile.array_count += 1;
            } else {
                profile.scalar_count += 1;
            }
            update_numeric_range(profile, value);
        }
    }

    for profile in profiles.values_mut() {
        profile.record_count = collection_counts[&profile.collection];
    }
    profiles.into_values().collect()
}

fn update_numeric_range(profile: &mut FieldProfile, value: &Value) {
    // Arrays may contain numeric values too, so numeric range discovery is
    // recursive even though the top-level cardinality is tracked separately.
    match value {
        Value::Number(number) => {
            if let Some(number) = number.as_f64() {
                profile.numeric_min =
                    Some(profile.numeric_min.map_or(number, |min| min.min(number)));
                profile.numeric_max =
                    Some(profile.numeric_max.map_or(number, |max| max.max(number)));
            }
        }
        Value::Array(values) => values
            .iter()
            .for_each(|value| update_numeric_range(profile, value)),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source_model::validate_records;
    use serde_json::json;

    #[test]
    fn profiles_presence_cardinality_and_numeric_ranges() {
        let values = vec![
            json!({"#id":"one","^":"items","~score":0,"@links":[],"~optional":null}),
            json!({"#id":"two","^":"items","~score":8.5,"@links":[],"~name":"second"}),
        ];
        let model = validate_records(&values).unwrap();
        let profiles = profile_model(&model);
        let score = profiles
            .iter()
            .find(|profile| profile.field == "~score")
            .unwrap();
        assert_eq!(score.record_count, 2);
        assert_eq!(score.present_count, 2);
        assert_eq!(score.numeric_min, Some(0.0));
        assert_eq!(score.numeric_max, Some(8.5));
        let links = profiles
            .iter()
            .find(|profile| profile.field == "@links")
            .unwrap();
        assert_eq!(links.array_count, 2);
        let optional = profiles
            .iter()
            .find(|profile| profile.field == "~optional")
            .unwrap();
        assert_eq!(optional.null_count, 1);
        assert_eq!(optional.present_count, 1);
    }
}
