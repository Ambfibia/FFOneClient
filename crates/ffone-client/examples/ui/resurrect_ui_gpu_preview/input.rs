use super::*;

pub(super) const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;

pub(super) fn parse_preview_args(
    args: impl IntoIterator<Item = OsString>,
) -> Result<(PreviewMode, String, PathBuf), String> {
    let mut mode = PreviewMode::PhoenixSelf;
    let mut language = "en".to_owned();
    let mut output = None;
    let mut mode_seen = false;
    let mut args = args.into_iter();
    while let Some(argument) = args.next() {
        if argument == "--language" {
            let value = args
                .next()
                .and_then(|value| value.into_string().ok())
                .ok_or_else(|| "--language requires en or ru".to_owned())?;
            if value != "en" && value != "ru" {
                return Err("--language requires en or ru".to_owned());
            }
            language = value;
        } else if !mode_seen && argument == "self" {
            mode = PreviewMode::PhoenixSelf;
            mode_seen = true;
        } else if !mode_seen && argument == "group-item" {
            mode = PreviewMode::GroupItem;
            mode_seen = true;
        } else if output.is_none() {
            output = Some(PathBuf::from(argument));
        } else {
            return Err("too many positional arguments".to_owned());
        }
    }
    let output = output.unwrap_or_else(|| mode.default_output(&language));
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err(format!("output must be a .png path: {}", output.display()));
    }
    Ok((mode, language, output))
}
