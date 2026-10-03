use super::*;

pub(crate) fn validate_logical_name(name: &str) -> Result<()> {
    if name.trim().is_empty() || name.len() > 1_024 {
        return invalid("logical model name is empty or too long");
    }
    if has_generated_identity(name) {
        return invalid("logical model name must be a true root name, not a hash/PathID fallback");
    }
    Ok(())
}

pub(crate) fn validate_true_asset_name(label: &str, name: &str) -> Result<()> {
    if name.trim().is_empty()
        || name.len() > 1_024
        || name.chars().any(char::is_control)
        || name.contains(['/', '\\'])
        || has_generated_identity(name)
    {
        return invalid(format!(
            "{label} name must be an exact source m_Name, not an unsafe/hash/PathID fallback"
        ));
    }
    Ok(())
}

pub(super) fn validate_path_segment(label: &str, value: &str) -> Result<()> {
    if value.trim() != value
        || value.is_empty()
        || matches!(value, "." | "..")
        || value.len() > 255
        || value
            .chars()
            .any(|character| character.is_control() || "<>:\"/\\|?*".contains(character))
    {
        return invalid(format!("{label} is not a safe true-name path segment"));
    }
    Ok(())
}
