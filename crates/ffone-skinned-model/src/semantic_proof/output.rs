use super::*;

pub(super) fn write_f64_vectors_as_f32<const N: usize>(
    out: &mut Canonical,
    values: &[[f64; N]],
) -> Result<()> {
    out.len(values.len())?;
    for value in values {
        for &component in value {
            out.f32(component as f32);
        }
    }
    Ok(())
}

pub(super) fn write_track(out: &mut Canonical, values: &TrackValues) -> Result<()> {
    match values {
        TrackValues::Translation(values) => {
            out.u8(0);
            write_f64_vectors_as_f32(out, values)
        }
        TrackValues::Rotation(values) => {
            out.u8(1);
            write_f64_vectors_as_f32(out, values)
        }
        TrackValues::Scale(values) => {
            out.u8(2);
            write_f64_vectors_as_f32(out, values)
        }
    }
}

pub(super) fn write_optional_track(out: &mut Canonical, values: Option<&TrackValues>) -> Result<()> {
    out.bool(values.is_some());
    if let Some(values) = values {
        write_track(out, values)?;
    }
    Ok(())
}

pub(super) fn write_canonical_json(out: &mut Canonical, value: &Value) -> Result<()> {
    match value {
        Value::Null => out.u8(0),
        Value::Bool(value) => {
            out.u8(1);
            out.bool(*value);
        }
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                out.u8(2);
                out.0.extend_from_slice(&value.to_le_bytes());
            } else if let Some(value) = value.as_u64() {
                out.u8(3);
                out.u64(value);
            } else if let Some(value) = value.as_f64().filter(|value| value.is_finite()) {
                out.u8(4);
                out.f64(value);
            } else {
                return invalid("animation metadata contains a non-finite JSON number");
            }
        }
        Value::String(value) => {
            out.u8(5);
            out.string(value);
        }
        Value::Array(values) => {
            out.u8(6);
            out.len(values.len())?;
            for value in values {
                write_canonical_json(out, value)?;
            }
        }
        Value::Object(values) => {
            out.u8(7);
            out.len(values.len())?;
            let mut keys = values.keys().collect::<Vec<_>>();
            keys.sort_unstable();
            for key in keys {
                out.string(key);
                write_canonical_json(out, &values[key])?;
            }
        }
    }
    Ok(())
}

pub(super) fn write_track_rows<'a>(
    out: &mut Canonical,
    kind: u8,
    values: impl Iterator<Item = &'a Vec<f32>>,
    count: usize,
) -> Result<()> {
    out.u8(kind);
    out.len(count)?;
    for row in values {
        for &component in row {
            out.f32(component);
        }
    }
    Ok(())
}

pub(super) fn write_json_f32_array<const N: usize>(
    out: &mut Canonical,
    value: Option<&Value>,
    context: &str,
) -> Result<()> {
    let values = required_array_value(value, context)?;
    if values.len() != N {
        return invalid(format!("{context} must contain exactly {N} values"));
    }
    for value in values {
        out.f32(required_f64(Some(value), context)? as f32);
    }
    Ok(())
}

pub(super) fn write_accessor_f32(
    out: &mut Canonical,
    accessors: &Accessors<'_, '_>,
    value: Option<&Value>,
    kind: &str,
) -> Result<()> {
    let rows = accessors.read_f32(required_u32(value, "attribute accessor")?, kind)?;
    out.len(rows.len())?;
    for row in rows {
        for component in row {
            out.f32(component);
        }
    }
    Ok(())
}

pub(super) fn write_optional_accessor_f32(
    out: &mut Canonical,
    accessors: &Accessors<'_, '_>,
    value: Option<&Value>,
    kind: &str,
) -> Result<()> {
    if value.is_some() {
        write_accessor_f32(out, accessors, value, kind)
    } else {
        out.len(0)
    }
}

pub(super) fn write_indices(
    out: &mut Canonical,
    accessors: &Accessors<'_, '_>,
    value: Option<&Value>,
) -> Result<()> {
    let values = accessors.read_indices(required_u32(value, "index accessor")?)?;
    out.len(values.len())?;
    for value in values {
        out.u32(value);
    }
    Ok(())
}

pub(super) fn write_optional_joints(
    out: &mut Canonical,
    accessors: &Accessors<'_, '_>,
    value: Option<&Value>,
) -> Result<()> {
    let Some(value) = value else {
        return out.len(0);
    };
    let values = accessors.read_joints(value_u32(value, "joint accessor")?)?;
    out.len(values.len())?;
    for row in values {
        for value in row {
            out.u16(value);
        }
    }
    Ok(())
}
