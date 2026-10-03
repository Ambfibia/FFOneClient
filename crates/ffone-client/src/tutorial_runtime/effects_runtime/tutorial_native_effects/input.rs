use super::*;

pub(super) fn parse_colors(value: &JsonValue) -> Result<[Vec4; 5], String> {
    let mut result = [Vec4::ONE; 5];
    for (index, target) in result.iter_mut().enumerate() {
        let packed = value
            .get(format!("colorAnimation[{index}]"))
            .and_then(|value| value.get("rgba"))
            .and_then(JsonValue::as_u64)
            .ok_or_else(|| format!("colorAnimation[{index}].rgba is absent"))?;
        let packed = u32::try_from(packed)
            .map_err(|_| format!("colorAnimation[{index}].rgba exceeds u32"))?;
        *target = Vec4::new(
            (packed & 0xff) as f32 / 255.0,
            ((packed >> 8) & 0xff) as f32 / 255.0,
            ((packed >> 16) & 0xff) as f32 / 255.0,
            ((packed >> 24) & 0xff) as f32 / 255.0,
        );
    }
    Ok(result)
}

pub(super) fn parse_curve(value: &JsonValue, field: &str) -> Result<AnimationCurve, String> {
    parse_curve_with_modes(value, field, false)
}

pub(super) fn parse_curve_with_modes(
    value: &JsonValue,
    field: &str,
    allow_once: bool,
) -> Result<AnimationCurve, String> {
    let curve = object(value, field)?;
    let valid_mode = |mode| mode == 2 || allow_once && mode == 1;
    if !valid_mode(integer(curve, "m_PreInfinity")?)
        || !valid_mode(integer(curve, "m_PostInfinity")?)
    {
        return Err(format!("{field} uses an unsupported infinity mode"));
    }
    let keys = array(curve, "m_Curve")?
        .iter()
        .map(|key| {
            Ok(CurveKey {
                time: number(key, "time")?,
                value: number(key, "value")?,
                in_slope: number(key, "inSlope")?,
                out_slope: number(key, "outSlope")?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    if keys.windows(2).any(|pair| pair[0].time > pair[1].time) {
        return Err(format!("{field} contains unordered keys"));
    }
    Ok(AnimationCurve(keys))
}

pub(super) fn collect_named(
    entity: Entity,
    target: &str,
    names: &Query<&Name>,
    children: &Query<&Children>,
    matches: &mut Vec<Entity>,
) {
    if names.get(entity).is_ok_and(|name| name.as_str() == target) {
        matches.push(entity);
    }
    if let Ok(descendants) = children.get(entity) {
        for child in descendants.iter() {
            collect_named(child, target, names, children, matches);
        }
    }
}
