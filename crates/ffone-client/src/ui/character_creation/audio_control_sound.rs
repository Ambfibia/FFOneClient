use super::*;

/// `SoundUtil.ButtonSound()` does not use the CharacterCreation bundle's
/// unrelated `CC Sound/Button.wav`. It randomly selects one of these five
/// `ui sound/mouse_click0N.wav` clips on every accepted click.
pub const CHARACTER_CREATION_BUTTON_SOUND_PATHS: [&str; 5] = [
    "audio/sfx/ui/mouse_click01.ogg",
    "audio/sfx/ui/mouse_click02.ogg",
    "audio/sfx/ui/mouse_click03.ogg",
    "audio/sfx/ui/mouse_click04.ogg",
    "audio/sfx/ui/mouse_click05.ogg",
];

pub const CHARACTER_CREATION_CONTINUE_SOUND_PATH: &str = "audio/sfx/ui/continue_01.ogg";

pub const CHARACTER_CREATION_RANDOM_SOUND_PATH: &str = "audio/sfx/ui/randomize.ogg";

pub const CHARACTER_CREATION_TAB_SOUND_PATH: &str = "audio/sfx/ui/tab_click01.ogg";

pub const CHARACTER_CREATION_COLOR_SOUND_PATH: &str = "audio/sfx/ui/select_color.ogg";

pub const CHARACTER_CREATION_HEIGHT_DOWN_SOUND_PATH: &str = "audio/sfx/ui/height_down.ogg";

pub const CHARACTER_CREATION_HEIGHT_UP_SOUND_PATH: &str = "audio/sfx/ui/height_up.ogg";

pub const CHARACTER_CREATION_GIRTH_NARROW_SOUND_PATH: &str = "audio/sfx/ui/girth_narrow.ogg";

pub const CHARACTER_CREATION_GIRTH_WIDE_SOUND_PATH: &str = "audio/sfx/ui/girth_wide.ogg";

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum CharacterCreationSound {
    Continue,
    Random,
    Tab,
    Color,
    HeightDown,
    HeightUp,
    GirthNarrow,
    GirthWide,
}

impl CharacterCreationSound {
    pub(super) const fn path(self) -> &'static str {
        match self {
            Self::Continue => CHARACTER_CREATION_CONTINUE_SOUND_PATH,
            Self::Random => CHARACTER_CREATION_RANDOM_SOUND_PATH,
            Self::Tab => CHARACTER_CREATION_TAB_SOUND_PATH,
            Self::Color => CHARACTER_CREATION_COLOR_SOUND_PATH,
            Self::HeightDown => CHARACTER_CREATION_HEIGHT_DOWN_SOUND_PATH,
            Self::HeightUp => CHARACTER_CREATION_HEIGHT_UP_SOUND_PATH,
            Self::GirthNarrow => CHARACTER_CREATION_GIRTH_NARROW_SOUND_PATH,
            Self::GirthWide => CHARACTER_CREATION_GIRTH_WIDE_SOUND_PATH,
        }
    }
}

pub(super) fn play_specific_sound(
    commands: &mut Commands,
    assets: &CharacterCreationAssets,
    sound: CharacterCreationSound,
    gain: f32,
) {
    if let Some(handle) = assets.sounds.get(&sound) {
        commands.spawn((
            crate::audio_channel::GameplayAudioChannel::ui_sfx(),
            AudioPlayer::new(handle.clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)),
        ));
    }
}

pub(super) fn play_button_sound(
    commands: &mut Commands,
    assets: &CharacterCreationAssets,
    random: &mut CharacterCreationRandom,
    gain: f32,
) {
    let handle = assets.button_sounds[random.below(assets.button_sounds.len())].clone();
    commands.spawn((
        crate::audio_channel::GameplayAudioChannel::ui_sfx(),
        AudioPlayer::new(handle),
        PlaybackSettings::DESPAWN.with_volume(Volume::Linear(gain)),
    ));
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CharacterCreationSoundCue {
    Button,
    Specific(CharacterCreationSound),
}

pub(super) fn control_sound(
    control: CharacterCreationControl,
    model: &CharacterCreationUiModel,
) -> Option<CharacterCreationSoundCue> {
    use CharacterCreationSoundCue::{Button, Specific};

    if !control_on_current_screen(control, model) {
        return None;
    }
    // A blocked native capability is not an accepted legacy GUI.Button click.
    // Keep local name-validation feedback audible: the source plays Continue
    // before validating the entered text, but not for unavailable controls.
    let enabled = match control {
        CharacterCreationControl::ContinueAppearance => model.save_appearance.enabled(),
        CharacterCreationControl::ContinueName => {
            model.reserve_name.enabled()
                && (model.name_mode != CharacterNameMode::Custom
                    || model.custom_name_filter.enabled())
        }
        CharacterCreationControl::RandomName => model.name_table.enabled(),
        CharacterCreationControl::RandomAppearance => model.creation_items.enabled(),
        CharacterCreationControl::ClothingChoice(_, delta) => {
            delta != 0 && model.creation_items.enabled() && model.starter_icons.enabled()
        }
        CharacterCreationControl::Step(field, _) => {
            matches!(field, AppearanceField::Height | AppearanceField::Body)
                || model.creation_items.enabled()
        }
        _ => true,
    };
    if !enabled {
        return None;
    }

    match control {
        CharacterCreationControl::ContinueAppearance | CharacterCreationControl::ContinueName => {
            Some(Specific(CharacterCreationSound::Continue))
        }
        CharacterCreationControl::RandomAppearance | CharacterCreationControl::RandomName => {
            Some(Specific(CharacterCreationSound::Random))
        }
        CharacterCreationControl::NameMode(mode) if mode != model.name_mode => {
            Some(Specific(CharacterCreationSound::Tab))
        }
        CharacterCreationControl::NameMode(_) => None,
        CharacterCreationControl::ColorPage(_, _)
        | CharacterCreationControl::Skin(_)
        | CharacterCreationControl::HairColor(_)
        | CharacterCreationControl::EyeColor(_) => Some(Specific(CharacterCreationSound::Color)),
        CharacterCreationControl::NameScroll(_, _)
        | CharacterCreationControl::Camera(_)
        | CharacterCreationControl::FocusCustomName => None,
        CharacterCreationControl::Step(AppearanceField::Height, delta) if delta < 0 => {
            Some(Specific(CharacterCreationSound::HeightDown))
        }
        CharacterCreationControl::Step(AppearanceField::Height, _) => {
            Some(Specific(CharacterCreationSound::HeightUp))
        }
        CharacterCreationControl::Step(AppearanceField::Body, delta) if delta < 0 => {
            Some(Specific(CharacterCreationSound::GirthNarrow))
        }
        CharacterCreationControl::Step(AppearanceField::Body, _) => {
            Some(Specific(CharacterCreationSound::GirthWide))
        }
        CharacterCreationControl::Gender(gender) if gender == model.appearance.gender => None,
        CharacterCreationControl::Exit if model.screen == CharacterCreationScreen::Name => None,
        CharacterCreationControl::ToggleFullscreen
            if model.screen == CharacterCreationScreen::Name =>
        {
            None
        }
        CharacterCreationControl::Step(_, _)
        | CharacterCreationControl::ClothingChoice(_, _)
        | CharacterCreationControl::Gender(_)
        | CharacterCreationControl::Exit
        | CharacterCreationControl::ToggleFullscreen => Some(Button),
    }
}

pub(super) fn play_sound_cue(
    commands: &mut Commands,
    assets: &CharacterCreationAssets,
    random: &mut CharacterCreationRandom,
    cue: CharacterCreationSoundCue,
    mix: &RetrobutionAudioMix,
) {
    let gain = 0.7 * mix.effects;
    if gain <= 0.0 || !gain.is_finite() {
        return;
    }
    match cue {
        CharacterCreationSoundCue::Button => play_button_sound(commands, assets, random, gain),
        CharacterCreationSoundCue::Specific(sound) => {
            play_specific_sound(commands, assets, sound, gain)
        }
    }
}
