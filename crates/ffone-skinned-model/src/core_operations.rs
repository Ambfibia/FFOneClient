use super::*;

pub fn exact_native_coordinate_contract() -> NativeCoordinateContract {
    NativeCoordinateContract {
        schema: NATIVE_COORDINATE_CONTRACT_SCHEMA.to_owned(),
        published_space: "gltf-right-handed-y-up".to_owned(),
        position: "[-unity.x,unity.y,unity.z]".to_owned(),
        normal: "[-unity.x,unity.y,unity.z]".to_owned(),
        translation: "[-unity.x,unity.y,unity.z]".to_owned(),
        rotation: "[unity.x,-unity.y,-unity.z,unity.w]".to_owned(),
        scale: "unchanged".to_owned(),
        uv: "[unity.u,1-unity.v]".to_owned(),
        inverse_bind_matrix: "H*unity*H where H=diag(-1,1,1,1)".to_owned(),
        source_triangle_winding: "unity-source-indices-in-h-reflected-native-space".to_owned(),
        published_triangle_winding: "gltf-counter-clockwise-per-triangle-normal-aligned".to_owned(),
        winding_conversion_owner: "ffone-asset-pipeline-triangle-normal-audit".to_owned(),
        unit_scale: "1-unity-unit-equals-1-bevy-unit".to_owned(),
        origin_policy: "source-root-trs-unchanged-no-auto-centering".to_owned(),
        auto_centered: false,
        auto_scaled: false,
    }
}

pub(super) fn is_false(value: &bool) -> bool {
    !*value
}

pub(super) fn minimal_windows_filename(source_stem: &str, extension: &str) -> Result<String> {
    let mut stem = source_stem
        .chars()
        .map(|character| {
            if character.is_control() || "<>:\"/\\|?*".contains(character) {
                '_'
            } else {
                character
            }
        })
        .collect::<String>();
    stem.truncate(stem.trim_end_matches([' ', '.']).len());
    if stem.is_empty() {
        stem.push('_');
    }
    let device_stem = stem
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    let reserved = matches!(device_stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || device_stem.strip_prefix("COM").is_some_and(|value| {
            matches!(value, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
        })
        || device_stem.strip_prefix("LPT").is_some_and(|value| {
            matches!(value, "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9")
        });
    if reserved {
        stem.insert(0, '_');
    }
    let filename = format!("{stem}.{extension}");
    if filename.encode_utf16().count() > 255 {
        return invalid("sanitized asset filename exceeds 255 UTF-16 code units");
    }
    Ok(filename)
}

pub(crate) fn has_generated_identity(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    let has_hash_suffix = lower.rsplit_once("--").is_some_and(|(_, suffix)| {
        suffix.len() >= 8 && suffix.chars().all(|value| value.is_ascii_hexdigit())
    });
    let path_id_token = lower.find("pathid").is_some_and(|offset| {
        lower[offset + "pathid".len()..]
            .trim_start_matches(['-', '_', '#', ' '])
            .chars()
            .next()
            .is_some_and(|value| value.is_ascii_digit())
    });
    let generated_object = ["mesh#", "transform#", "texture#", "texture2d#", "material#"]
        .iter()
        .any(|prefix| {
            lower.strip_prefix(prefix).is_some_and(|suffix| {
                !suffix.is_empty() && suffix.chars().all(|c| c.is_ascii_digit())
            })
        });
    has_hash_suffix || path_id_token || generated_object
}

pub(crate) fn invalid<T>(message: impl Into<String>) -> Result<T> {
    Err(ModelError::Invalid(message.into()))
}
