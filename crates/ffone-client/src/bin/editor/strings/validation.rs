use super::*;

pub(super) fn validate(en: &BTreeMap<String, String>, ru: &BTreeMap<String, String>) -> Result<(), String> {
    if !en.keys().eq(ru.keys()) {
        return Err("EN/RU key sets differ".into());
    }
    for (key, value) in en {
        if placeholders(value) != placeholders(&ru[key]) {
            return Err(format!("{{…}}: {key}"));
        }
    }
    Ok(())
}
