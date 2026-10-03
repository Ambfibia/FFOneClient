use super::*;

pub const CHARACTER_SELECTION_BUTTON_SOUND_PATHS: [&str; 5] = [
    "audio/sfx/ui/mouse_click01.ogg",
    "audio/sfx/ui/mouse_click02.ogg",
    "audio/sfx/ui/mouse_click03.ogg",
    "audio/sfx/ui/mouse_click04.ogg",
    "audio/sfx/ui/mouse_click05.ogg",
];

pub const CHARACTER_SELECTION_DELETE_YES_SOUND_PATH: &str = "audio/sfx/ui/yes_button.ogg";

pub const CHARACTER_SELECTION_DELETE_NO_SOUND_PATH: &str = "audio/sfx/ui/no_button.ogg";

pub(super) fn play_character_selection_sound(
    commands: &mut Commands,
    handle: Handle<AudioSource>,
    volume: f32,
) {
    commands.spawn((
        crate::audio_channel::GameplayAudioChannel::ui_sfx(),
        AudioPlayer::new(handle),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
    ));
}

pub(super) fn play_character_selection_button_sound(
    commands: &mut Commands,
    assets: &CharacterSelectionAssets,
    random: &mut CharacterSelectionRandom,
) {
    let handle = assets.button_sounds[random.below(assets.button_sounds.len())].clone();
    play_character_selection_sound(commands, handle, 0.7);
}
