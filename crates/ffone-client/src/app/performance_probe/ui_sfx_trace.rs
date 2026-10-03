//! Opt-in observation of real UI audio players/sinks during pointer verification.
//! Does not inject interactions or alter audio playback.
use super::*;
use bevy::audio::{AudioSink, AudioSinkPlayback};
use ffone_client::audio_channel::GameplayAudioChannel;

pub(super) fn install(app: &mut App) {
    app.add_systems(Last, trace);
}

fn trace(
    assets: Res<AssetServer>,
    players: Query<
        (
            Entity,
            &AudioPlayer<AudioSource>,
            &PlaybackSettings,
            &GameplayAudioChannel,
        ),
        Added<AudioPlayer<AudioSource>>,
    >,
    sinks: Query<(Entity, &AudioSink, &GameplayAudioChannel), Added<AudioSink>>,
    mut removed: RemovedComponents<AudioPlayer<AudioSource>>,
    mut observed: Local<std::collections::HashSet<Entity>>,
) {
    for (entity, player, settings, channel) in &players {
        let path = assets
            .get_path(player.0.id())
            .map(|p| p.to_string())
            .unwrap_or_default();
        if !path.starts_with("audio/sfx/ui/") {
            continue;
        }
        observed.insert(entity);
        println!(
            "ui-sfx player={entity:?} path={path} category={:?} base={} gain={} speed={} mode={:?}",
            channel.category,
            channel.base_gain,
            settings.volume.to_linear(),
            settings.speed,
            settings.mode
        );
    }
    for (entity, sink, _) in &sinks {
        if observed.contains(&entity) {
            println!(
                "ui-sfx sink={entity:?} gain={} paused={}",
                sink.volume().to_linear(),
                sink.is_paused()
            );
        }
    }
    for entity in removed.read() {
        if observed.remove(&entity) {
            println!("ui-sfx stopped={entity:?}");
        }
    }
}
