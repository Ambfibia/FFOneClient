use super::*;

pub(super) const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;

pub(super) fn parse_cli<I>(arguments: I) -> Result<PreviewCli, &'static str>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let arguments = arguments
        .into_iter()
        .map(|value| value.to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    if arguments.len() == 1 && arguments[0].ends_with(".png") {
        return Ok(PreviewCli {
            scene: PreviewScene::Player,
            language: "en".to_owned(),
            output: PathBuf::from(&arguments[0]),
        });
    }
    if arguments.len() > 3 {
        return Err(
            "usage: email_ui_gpu_preview [player|compose|buddy|calculator] [en|ru] [OUTPUT.png]",
        );
    }
    let scene: PreviewScene = arguments
        .first()
        .map(String::as_str)
        .unwrap_or("player")
        .parse()
        .map_err(|_| {
            "usage: email_ui_gpu_preview [player|compose|buddy|calculator] [en|ru] [OUTPUT.png]"
        })?;
    let language = arguments.get(1).map(String::as_str).unwrap_or("en");
    if !matches!(language, "en" | "ru") {
        return Err(
            "usage: email_ui_gpu_preview [player|compose|buddy|calculator] [en|ru] [OUTPUT.png]",
        );
    }
    let output = arguments.get(2).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/ui-parity/email-{}-{language}-1264x681.png",
            scene.name()
        ))
    });
    if output.extension().and_then(|value| value.to_str()) != Some("png") {
        return Err(
            "usage: email_ui_gpu_preview [player|compose|buddy|calculator] [en|ru] [OUTPUT.png]",
        );
    }
    Ok(PreviewCli {
        scene,
        language: language.to_owned(),
        output,
    })
}
