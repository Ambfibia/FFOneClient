use super::*;

/// Independently compares the in-memory source values with values read back
/// from the actual GLB bytes. Publication fails on the first unequal section.
pub fn prove_semantic_roundtrip(model: &NativeModel, glb: &[u8]) -> Result<SemanticRoundtripProof> {
    let source = SemanticSections::from_model(model)?;
    let emitted = SemanticSections::from_glb(glb)?;
    for (name, left, right) in [
        ("hierarchy", &source.hierarchy, &emitted.hierarchy),
        ("geometry", &source.geometry, &emitted.geometry),
        ("skins", &source.skins, &emitted.skins),
        (
            "standard animations",
            &source.standard_animations,
            &emitted.standard_animations,
        ),
        (
            "animation metadata",
            &source.animation_metadata,
            &emitted.animation_metadata,
        ),
        ("materials", &source.materials, &emitted.materials),
    ] {
        if left != right {
            let detail = if name == "animation metadata" {
                animation_metadata_mismatch_detail(model, glb)
                    .unwrap_or_else(|error| format!("diagnostic failed: {error}"))
            } else {
                "canonical byte streams differ".to_owned()
            };
            return invalid(format!(
                "semantic round-trip mismatch in {name} ({detail}); equal feature counts are not accepted as value-level proof"
            ));
        }
    }
    let source = source.digests();
    let emitted = emitted.digests();
    debug_assert_eq!(source, emitted);
    Ok(SemanticRoundtripProof {
        schema: SEMANTIC_ROUNDTRIP_PROOF_SCHEMA.to_owned(),
        status: "native-model-matches-redecoded-glb".to_owned(),
        matched: true,
        canonical_encoding: "ffone-semantic-le-v1-length-prefixed-ieee754-bits".to_owned(),
        floating_point_contract: "NativeModel numeric values are compared as the exact IEEE-754 f32 bits emitted by glTF accessors/TRS; JSON-only clip settings retain IEEE-754 f64 bits"
            .to_owned(),
        covered_scopes: COVERED_SCOPES.into_iter().map(str::to_owned).collect(),
        excluded_scopes: EXCLUDED_SCOPES.into_iter().map(str::to_owned).collect(),
        source,
        emitted,
    })
}

pub(super) fn vec3_f64(rows: Vec<Vec<f32>>, context: &str) -> Result<Vec<[f64; 3]>> {
    rows.into_iter()
        .map(|row| {
            let [x, y, z] = row.as_slice() else {
                return invalid(format!("{context} row is not VEC3"));
            };
            Ok([f64::from(*x), f64::from(*y), f64::from(*z)])
        })
        .collect()
}

pub(super) fn first_json_difference(left: &Value, right: &Value, path: &str) -> Option<String> {
    match (left, right) {
        (Value::Null, Value::Null) => None,
        (Value::Bool(left), Value::Bool(right)) if left == right => None,
        (Value::String(left), Value::String(right)) if left == right => None,
        (Value::Number(left), Value::Number(right)) => {
            let left = canonical_json_number(left);
            let right = canonical_json_number(right);
            (left != right).then(|| format!("{path} number {left:?} != {right:?}"))
        }
        (Value::Array(left), Value::Array(right)) => {
            if left.len() != right.len() {
                return Some(format!(
                    "{path} array length {} != {}",
                    left.len(),
                    right.len()
                ));
            }
            left.iter()
                .zip(right)
                .enumerate()
                .find_map(|(index, (left, right))| {
                    first_json_difference(left, right, &format!("{path}[{index}]"))
                })
        }
        (Value::Object(left), Value::Object(right)) => {
            let mut left_keys = left.keys().collect::<Vec<_>>();
            let mut right_keys = right.keys().collect::<Vec<_>>();
            left_keys.sort_unstable();
            right_keys.sort_unstable();
            if left_keys != right_keys {
                return Some(format!("{path} object keys differ"));
            }
            left_keys.into_iter().find_map(|key| {
                first_json_difference(&left[key], &right[key], &format!("{path}.{key}"))
            })
        }
        _ => Some(format!("{path} {left:?} != {right:?}")),
    }
}

pub(super) fn canonical_json_number(value: &serde_json::Number) -> CanonicalJsonNumber {
    if let Some(value) = value.as_i64() {
        CanonicalJsonNumber::Signed(value)
    } else if let Some(value) = value.as_u64() {
        CanonicalJsonNumber::Unsigned(value)
    } else if let Some(value) = value.as_f64().filter(|value| value.is_finite()) {
        CanonicalJsonNumber::Float(value.to_bits())
    } else {
        CanonicalJsonNumber::Invalid
    }
}

pub(super) fn interpolation_tag(value: Interpolation) -> u8 {
    match value {
        Interpolation::Linear => 0,
        Interpolation::Step => 1,
        Interpolation::CubicSpline => 2,
    }
}

pub(super) fn source_encoding_tag(value: crate::EmptyTrsSourceEncoding) -> u8 {
    match value {
        crate::EmptyTrsSourceEncoding::Plain => 0,
        crate::EmptyTrsSourceEncoding::Compressed => 1,
    }
}

pub(super) fn required_array<'a>(root: &'a Value, key: &str, context: &str) -> Result<&'a Vec<Value>> {
    required_array_value(root.get(key), context)
}

pub(super) fn optional_root_array<'a>(root: &'a Value, key: &str) -> Result<&'a [Value]> {
    match root.get(key) {
        None => Ok(&[]),
        Some(value) => value
            .as_array()
            .map(Vec::as_slice)
            .ok_or_else(|| ModelError::Invalid(format!("semantic proof {key} is not an array"))),
    }
}

pub(super) fn required_array_value<'a>(value: Option<&'a Value>, context: &str) -> Result<&'a Vec<Value>> {
    value
        .and_then(Value::as_array)
        .ok_or_else(|| ModelError::Invalid(format!("semantic proof {context} is not an array")))
}

pub(super) fn optional_array<'a>(value: Option<&'a Value>, context: &str) -> Result<&'a [Value]> {
    match value {
        None => Ok(&[]),
        Some(value) => value.as_array().map(Vec::as_slice).ok_or_else(|| {
            ModelError::Invalid(format!("semantic proof {context} is not an array"))
        }),
    }
}

pub(super) fn indexed<'a>(values: &'a [Value], index: u32, context: &str) -> Result<&'a Value> {
    values
        .get(usize_from_u32(index)?)
        .ok_or_else(|| ModelError::Invalid(format!("semantic proof {context} is out of range")))
}

pub(super) fn required_string<'a>(value: Option<&'a Value>, context: &str) -> Result<&'a str> {
    value
        .and_then(Value::as_str)
        .ok_or_else(|| ModelError::Invalid(format!("semantic proof {context} is not a string")))
}

pub(super) fn required_bool(value: Option<&Value>, context: &str) -> Result<bool> {
    value
        .and_then(Value::as_bool)
        .ok_or_else(|| ModelError::Invalid(format!("semantic proof {context} is not a bool")))
}

pub(super) fn required_u32(value: Option<&Value>, context: &str) -> Result<u32> {
    value
        .ok_or_else(|| ModelError::Invalid(format!("semantic proof has no {context}")))
        .and_then(|value| value_u32(value, context))
}

pub(super) fn optional_u32(value: Option<&Value>) -> Result<Option<u32>> {
    value
        .filter(|value| !value.is_null())
        .map(|value| value_u32(value, "optional u32"))
        .transpose()
}

pub(super) fn value_u32(value: &Value, context: &str) -> Result<u32> {
    value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| ModelError::Invalid(format!("semantic proof {context} is not a u32")))
}

pub(super) fn value_i32(value: &Value, context: &str) -> Result<i32> {
    value
        .as_i64()
        .and_then(|value| i32::try_from(value).ok())
        .ok_or_else(|| ModelError::Invalid(format!("semantic proof {context} is not an i32")))
}

pub(super) fn optional_i32(value: Option<&Value>) -> Result<Option<i32>> {
    value
        .filter(|value| !value.is_null())
        .map(|value| value_i32(value, "optional i32"))
        .transpose()
}

pub(super) fn required_f64(value: Option<&Value>, context: &str) -> Result<f64> {
    value
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite())
        .ok_or_else(|| ModelError::Invalid(format!("semantic proof {context} is not finite f64")))
}

pub(super) fn optional_f64(value: Option<&Value>) -> Result<Option<f64>> {
    value
        .filter(|value| !value.is_null())
        .map(|value| required_f64(Some(value), "optional f64"))
        .transpose()
}

pub(super) fn required_usize(value: Option<&Value>, context: &str) -> Result<usize> {
    value
        .and_then(Value::as_u64)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| ModelError::Invalid(format!("semantic proof {context} is not a usize")))
}

pub(super) fn optional_usize(value: Option<&Value>) -> Result<Option<usize>> {
    value
        .filter(|value| !value.is_null())
        .map(|value| required_usize(Some(value), "optional usize"))
        .transpose()
}

pub(super) fn component_size(component_type: u32) -> Result<usize> {
    match component_type {
        5_123 => Ok(2),
        5_125 | 5_126 => Ok(4),
        value => invalid(format!("unsupported accessor componentType {value}")),
    }
}

pub(super) fn checked_add(left: usize, right: usize, context: &'static str) -> Result<usize> {
    left.checked_add(right).ok_or(ModelError::Overflow(context))
}

pub(super) fn usize_from_u32(value: u32) -> Result<usize> {
    usize::try_from(value).map_err(|_| ModelError::Overflow("u32 to usize"))
}

pub(super) fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub(super) fn invalid<T>(message: impl Into<String>) -> Result<T> {
    Err(ModelError::Invalid(message.into()))
}
