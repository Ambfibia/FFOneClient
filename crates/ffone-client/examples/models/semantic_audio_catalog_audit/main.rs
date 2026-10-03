use std::{env, path::PathBuf};

use ffone_client::semantic_audio::NativeAudioCatalog;

fn main() -> Result<(), String> {
    let mut args = std::env::args_os();
    let executable = args
        .next()
        .and_then(|value| PathBuf::from(value).file_name().map(|name| name.to_owned()))
        .unwrap_or_else(|| "semantic_audio_catalog_audit".into());
    let Some(asset_root) = args.next() else {
        return Err(format!(
            "usage: {} <ASSET_ROOT>",
            executable.to_string_lossy()
        ));
    };
    if args.next().is_some() {
        return Err(format!(
            "usage: {} <ASSET_ROOT>",
            executable.to_string_lossy()
        ));
    }

    let catalog = NativeAudioCatalog::open(PathBuf::from(asset_root), false)?;
    let diagnostics = catalog.diagnostics();
    println!(
        "opened editable semantic audio catalog: assets={} bytes={} music={} ambient={} voice={} \
         sfx={} localizedFiles={} englishVoiceFiles={} russianVoiceFiles={}",
        diagnostics.assets,
        diagnostics.bytes,
        diagnostics.music,
        diagnostics.ambient,
        diagnostics.voice,
        diagnostics.sfx,
        diagnostics.localized_files,
        diagnostics.english_voice_files,
        diagnostics.russian_voice_files,
    );
    Ok(())
}
