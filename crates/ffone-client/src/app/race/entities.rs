use super::*;

pub(in super::super) fn spawn_race_npc_voice(
    commands: &mut Commands,
    asset_server: &AssetServer,
    production: &mut RaceProductionRuntime,
    catalog: &NativeAudioCatalog,
    voice_language: &VoiceLanguage,
    voice_gain: f32,
    suffix: &str,
) -> Result<(), String> {
    let (npc_id, owner) = production
        .source_npc
        .as_ref()
        .map(|source| (source.runtime_npc_id, source.voice_owner.clone()))
        .ok_or_else(|| "RaceMode voice has no correlated source NPC".to_owned())?;
    if owner.is_empty() {
        return Err(format!(
            "RaceMode voice rejected NPC {npc_id} without a TableData voice owner"
        ));
    }
    let take = production.next_voice_take();
    let true_name = format!("{owner}_{suffix}0{take}");
    let candidates = catalog
        .by_true_name(&true_name)
        .into_iter()
        .filter(|asset| asset.category == NativeAudioCategory::Voice)
        .collect::<Vec<_>>();
    let [asset] = candidates.as_slice() else {
        return Err(format!(
            "RaceMode voice {true_name:?} resolved to {} native voice assets",
            candidates.len()
        ));
    };
    let Some(path) = catalog
        .path_for_locale(asset, &voice_language.effective)
        .map(str::to_owned)
    else {
        return Ok(());
    };
    commands.spawn((
        Name::new(format!("RaceMode NPC {npc_id} voice {true_name}")),
        WorldSliceEntity,
        LocalizedVoice::by_true_name(true_name),
        ffone_client::audio_channel::GameplayAudioChannel::new(NativeAudioCategory::Voice, 1.0),
        AudioPlayer::new(asset_server.load(path)),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(voice_gain)),
    ));
    Ok(())
}
