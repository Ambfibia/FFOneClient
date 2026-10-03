use super::*;

pub(super) const WARMUP_FRAMES_AFTER_LOAD: u32 = 80;

pub(super) fn parse_cli<I>(arguments: I) -> Result<PreviewCli, &'static str>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let mut arguments = arguments.into_iter();
    let language = arguments
        .next()
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| "en".to_owned());
    if !matches!(language.as_str(), "en" | "ru") {
        return Err("usage: vendor_ui_gpu_preview [en|ru] [OUTPUT.png]");
    }
    let output = arguments
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| default_output(&language));
    if arguments.next().is_some()
        || output.extension().and_then(|value| value.to_str()) != Some("png")
    {
        return Err("usage: vendor_ui_gpu_preview [en|ru] [OUTPUT.png]");
    }
    Ok(PreviewCli { language, output })
}
