//! Nano voice coroutines: wait for the current line, load, then play a detached
//! one-shot. Pending animation requests belong to the Nano; farewell belongs
//! to its avatar and therefore survives removal of the Nano model.

use super::*;

#[derive(Debug)]
struct PendingVoice {
    anchor: Entity,
    current_owner: Option<Entity>,
    after: Option<Entity>,
    logical_key: String,
    handle: Option<Handle<AudioSource>>,
}

#[derive(Debug)]
struct Dismissal {
    owner: Entity,
    voice_owner: String,
    take: u32,
    after: Option<Entity>,
}

#[derive(Debug, Default)]
pub(super) struct NanoVoiceRuntime {
    current: HashMap<Entity, Entity>,
    pending: Vec<PendingVoice>,
    dismissals: Vec<Dismissal>,
}

impl NanoVoiceRuntime {
    #[cfg(test)]
    pub(super) fn dismissal_count(&self) -> usize {
        self.dismissals.len()
    }
    pub(super) fn queue(&mut self, nano: Entity, logical_key: String) {
        self.pending.push(PendingVoice {
            anchor: nano,
            current_owner: Some(nano),
            after: self.current.get(&nano).copied(),
            logical_key,
            handle: None,
        });
    }

    pub(super) fn dismiss(&mut self, nano: Entity, owner: Entity, voice_owner: &str, take: u32) {
        // Capture currentVO before Hide destroys the animation handler. Its
        // waiting coroutines are cancelled; the already playing source lives on.
        self.pending
            .retain(|voice| voice.current_owner != Some(nano));
        self.dismissals.push(Dismissal {
            owner,
            voice_owner: voice_owner.to_owned(),
            take,
            after: self.current.remove(&nano),
        });
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn drive(
        &mut self,
        commands: &mut Commands,
        asset_server: &AssetServer,
        sources: &Assets<AudioSource>,
        catalog: &NativeAudioCatalog,
        language: &VoiceLanguage,
        mix: &RetrobutionAudioMix,
        transforms: &Query<&GlobalTransform>,
        active: &Query<(Entity, Option<&SpatialAudioSink>), With<GameplaySfxAudio>>,
        listener: Option<Vec3>,
        maximum_distance: f32,
        active_count: usize,
    ) {
        self.current
            .retain(|owner, source| transforms.contains(*owner) && active.contains(*source));
        for dismissal in std::mem::take(&mut self.dismissals) {
            let Ok(transform) = transforms.get(dismissal.owner) else {
                continue;
            };
            // SoundUtil.IsPlayable is tested when AddNano(-1) requests VO,
            // before waiting/loading, rather than again after that wait.
            if active_count > 12
                || listener.is_some_and(|listener| {
                    transform.translation().distance(listener) >= maximum_distance
                })
            {
                continue;
            }
            let Some(audio) = catalog.choose_owner_voice(
                &dismissal.voice_owner,
                "_nandismiss",
                &language.effective,
                dismissal.take,
            ) else {
                warn!(
                    "Nano {:?} has no dismissal take {}",
                    dismissal.voice_owner, dismissal.take
                );
                continue;
            };
            self.pending.push(PendingVoice {
                anchor: dismissal.owner,
                current_owner: None,
                after: dismissal.after,
                logical_key: audio.logical_key.clone(),
                handle: None,
            });
        }
        self.pending.retain_mut(|voice| {
            let Ok(transform) = transforms.get(voice.anchor) else {
                return false;
            };
            if voice.after.is_some_and(|source| {
                active
                    .get(source)
                    .is_ok_and(|(_, sink)| sink.is_none_or(|sink| !sink.empty()))
            }) {
                return true;
            }
            let Some(audio) = catalog.by_logical_key(&voice.logical_key) else {
                return false;
            };
            let Some(path) = catalog.path_for_locale(audio, &language.effective) else {
                return false;
            };
            if voice.handle.as_ref().is_some_and(|handle| {
                asset_server
                    .get_path(handle.id())
                    .is_some_and(|loaded| loaded.path() != std::path::Path::new(path))
            }) {
                voice.handle = None;
            }
            let handle = voice
                .handle
                .get_or_insert_with(|| asset_server.load(path.to_owned()));
            if !sources.contains(handle.id()) {
                return !matches!(
                    asset_server.load_state(handle.id()),
                    bevy::asset::LoadState::Failed(_)
                );
            }
            let entity = commands
                .spawn((
                    Name::new(format!("Nano voice {}", audio.true_name)),
                    GameplayAudioChannel::new(NativeAudioCategory::Voice, 1.0),
                    GameplaySfxAudio,
                    LocalizedVoice::by_true_name(audio.true_name.clone()),
                    Transform::from_translation(transform.translation()),
                    GlobalTransform::from_translation(transform.translation()),
                    AudioPlayer::new(handle.clone()),
                    PlaybackSettings::DESPAWN
                        .with_volume(Volume::Linear(mix.voice.clamp(0.0, 1.0)))
                        .with_spatial(true)
                        .with_spatial_scale(LEGACY_SPATIAL_SCALE),
                ))
                .id();
            if let Some(owner) = voice.current_owner {
                self.current.insert(owner, entity);
            }
            false
        });
    }
}

#[cfg(test)]
fn dismissal_asset<'a>(
    catalog: &'a NativeAudioCatalog,
    owner: &str,
    take: u32,
) -> Option<&'a crate::semantic_audio::NativeAudioAsset> {
    let suffix = format!("_nandismiss{take:02}");
    // Ownership comes from the native model's semantic Nano directory. This
    // also covers authored display-name changes and differing voice prefixes
    // (BtrCup/Buttercup, Ice King/IceKing) without matching translated UI text.
    catalog.assets().iter().find(|audio| {
        audio.category == NativeAudioCategory::Voice
            && audio.owner == owner
            && audio.true_name.to_ascii_lowercase().ends_with(&suffix)
    })
}

#[cfg(test)]
#[path = "nano/tests.rs"]
mod tests;
