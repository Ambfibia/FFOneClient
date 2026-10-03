//! Live channel gain for sounds whose volume has no separate fade owner.
use crate::{gameplay_audio::RetrobutionAudioMix, semantic_audio::NativeAudioCategory};
use bevy::{
    audio::{AudioSink, AudioSinkPlayback, GlobalVolume, SpatialAudioSink, Volume},
    prelude::*,
    transform::TransformSystems,
};

#[derive(Clone, Copy, Debug, Component)]
pub struct GameplayAudioChannel {
    pub category: NativeAudioCategory,
    pub base_gain: f32,
}

impl GameplayAudioChannel {
    /// `SoundUtil.ButtonSound`, `Playsound` and `PlayUIModeSound` all use
    /// 0.7 * GetSFXVolume(). Keep the unscaled gain so mute can be reversed.
    pub const fn ui_sfx() -> Self {
        Self::new(NativeAudioCategory::Sfx, 0.7)
    }

    pub const fn new(category: NativeAudioCategory, base_gain: f32) -> Self {
        Self {
            category,
            base_gain,
        }
    }

    pub fn gain(self, mix: &RetrobutionAudioMix) -> f32 {
        let channel = match self.category {
            NativeAudioCategory::Music => mix.music,
            NativeAudioCategory::Ambient => mix.ambient,
            NativeAudioCategory::Voice => mix.voice,
            NativeAudioCategory::Sfx => mix.effects,
        };
        finite_gain(self.base_gain) * finite_gain(channel).min(1.0)
    }
}

fn finite_gain(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

pub struct GameplayChannelMixPlugin;

impl Plugin for GameplayChannelMixPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RetrobutionAudioMix>()
            .init_resource::<GlobalVolume>()
            // Bevy starts queued audio after transform propagation. Pending
            // players must receive channel gain before master is applied there.
            .add_systems(
                PostUpdate,
                sync_channel_gain.before(TransformSystems::Propagate),
            );
    }
}

fn sync_channel_gain(
    mix: Res<RetrobutionAudioMix>,
    master: Res<GlobalVolume>,
    mut sources: Query<(
        &GameplayAudioChannel,
        &mut PlaybackSettings,
        Option<&mut AudioSink>,
        Option<&mut SpatialAudioSink>,
    )>,
) {
    for (channel, mut settings, regular, spatial) in &mut sources {
        let gain = channel.gain(&mix);
        let pending = Volume::Linear(gain);
        if settings.volume != pending {
            settings.volume = pending;
        }
        // PlaybackSettings and GlobalVolume affect queued playback only.
        // Existing sinks need the full gain, including master exactly once.
        let playing = Volume::Linear(gain * finite_gain(master.volume.to_linear()));
        if let Some(mut sink) = regular {
            if sink.volume() != playing {
                sink.set_volume(playing);
            }
        }
        if let Some(mut sink) = spatial {
            if sink.volume() != playing {
                sink.set_volume(playing);
            }
        }
    }
}

#[cfg(test)]
mod tests;
