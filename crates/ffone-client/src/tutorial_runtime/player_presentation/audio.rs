use super::*;

pub(super) fn weapon_sound_variants(row: &Value, index: usize) -> Result<Vec<String>, String> {
    let mut variants = Vec::new();
    for field in [
        "m_pstrSoundString1",
        "m_pstrSoundString2",
        "m_pstrSoundString3",
    ] {
        let source = row
            .get(field)
            .and_then(Value::as_str)
            .ok_or_else(|| format!("weapon sound row {index} has no string {field}"))?
            .trim();
        if source.is_empty() || source.eq_ignore_ascii_case("null") {
            continue;
        }
        let filename = source.rsplit(['/', '\\']).next().unwrap_or(source);
        let (true_name, extension) = filename.rsplit_once('.').ok_or_else(|| {
            format!("weapon sound row {index} {field} has no file extension: {source:?}")
        })?;
        if !extension.eq_ignore_ascii_case("wav") || true_name.is_empty() {
            return Err(format!(
                "weapon sound row {index} {field} is not a WAV true name: {source:?}"
            ));
        }
        // Primary rows 27 and 69 contain the same `ThrownUesr-05.wav` typo,
        // while no correspondingly named (or correctly spelled) AudioClip is
        // present in the primary containers. Treating that orphan as absent
        // lets charged users of those rows fall back explicitly to Sound1
        // instead of making every affected grenade silently lose its attack.
        if true_name == "ThrownUesr-05" {
            continue;
        }
        if !variants.iter().any(|variant| variant == true_name) {
            variants.push(true_name.to_owned());
        }
    }
    Ok(variants)
}
