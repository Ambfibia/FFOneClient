use super::*;

pub(super) const POST_CAPTURE_ERROR_GRACE_FRAMES: u64 = 3;

pub(super) fn validate_json_path(path: &Path) -> Result<(), String> {
    let is_json = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("json"));
    if !is_json {
        return Err("--report must end in .json".to_owned());
    }
    Ok(())
}
