use super::*;

pub(super) fn localized_tutorial_voice_busy(
    locale: &str,
    asset_server: &AssetServer,
    voices: &Query<(&AudioPlayer, Option<&AudioSink>), With<TutorialVoiceAudio>>,
) -> bool {
    !is_english_locale(locale)
        && voices.iter().any(|(player, sink)| {
            sink.map_or_else(
                || {
                    matches!(
                        asset_server.load_state(player.0.id()),
                        bevy::asset::LoadState::Loading
                    )
                },
                |sink| !sink.empty(),
            )
        })
}

pub(super) fn retain_localized_tutorial_subtitle(
    time: Res<Time>,
    language: Res<VoiceLanguage>,
    asset_server: Res<AssetServer>,
    voices: Query<(&AudioPlayer, Option<&AudioSink>), With<TutorialVoiceAudio>>,
    mut subtitles: ResMut<TutorialVoiceSubtitleState>,
) {
    if localized_tutorial_voice_busy(&language.effective, &asset_server, &voices) {
        subtitles.retain_until(time.elapsed_secs_f64() + f64::from(time.delta_secs()) + 0.1);
    }
}
