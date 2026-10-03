use super::*;

pub(in super::super) fn spawn_combi_sfx_0104(
    commands: &mut Commands,
    asset_server: &AssetServer,
    catalog: &NativeAudioCatalog,
    gain: f32,
    true_name: &'static str,
    looping: bool,
) -> Result<Entity, String> {
    let candidates = catalog
        .by_true_name(true_name)
        .into_iter()
        .filter(|asset| asset.category == NativeAudioCategory::Sfx)
        .collect::<Vec<_>>();
    let [asset] = candidates.as_slice() else {
        return Err(format!(
            "Combi SFX {true_name:?} resolved to {} native assets",
            candidates.len()
        ));
    };
    let settings = if looping {
        PlaybackSettings::LOOP.with_volume(Volume::Linear(gain))
    } else {
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain))
    };
    Ok(commands
        .spawn((
            Name::new(format!("CombiMode SFX {true_name}")),
            WorldSliceEntity,
            AudioPlayer::new(asset_server.load(asset.path.clone())),
            ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
            settings,
        ))
        .id())
}
