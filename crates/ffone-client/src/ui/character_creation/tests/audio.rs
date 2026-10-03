use super::*;

#[test]
fn exact_audio_and_font_hooks_are_published() {
    for path in CHARACTER_CREATION_BUTTON_SOUND_PATHS.into_iter().chain([
        CHARACTER_CREATION_MUSIC_PATH,
        CHARACTER_CREATION_CONTINUE_SOUND_PATH,
        CHARACTER_CREATION_RANDOM_SOUND_PATH,
        CHARACTER_CREATION_TAB_SOUND_PATH,
        CHARACTER_CREATION_COLOR_SOUND_PATH,
        CHARACTER_CREATION_HEIGHT_DOWN_SOUND_PATH,
        CHARACTER_CREATION_HEIGHT_UP_SOUND_PATH,
        CHARACTER_CREATION_GIRTH_NARROW_SOUND_PATH,
        CHARACTER_CREATION_GIRTH_WIDE_SOUND_PATH,
        CHARACTER_CREATION_FONT_PATH,
        CHARACTER_CREATION_DISPLAY_FONT_PATH,
    ]) {
        assert!(project_asset(path).is_file(), "{path} is missing");
    }
}

#[test]
fn sound_routing_matches_the_two_legacy_creation_scripts() {
    let mut model = CharacterCreationUiModel::default();
    model.name_table = CharacterCreationCapability::Enabled;
    model.screen = CharacterCreationScreen::Appearance;
    assert_eq!(
        control_sound(CharacterCreationControl::Exit, &model),
        Some(CharacterCreationSoundCue::Button)
    );
    assert_eq!(
        control_sound(
            CharacterCreationControl::Camera(CharacterCreationCameraAction::RotateLeft),
            &model
        ),
        None
    );
    assert_eq!(
        control_sound(
            CharacterCreationControl::Step(AppearanceField::Height, -1),
            &model
        ),
        Some(CharacterCreationSoundCue::Specific(
            CharacterCreationSound::HeightDown
        ))
    );
    assert_eq!(
        control_sound(
            CharacterCreationControl::Gender(CharacterGender::Boy),
            &model
        ),
        None
    );
    model.screen = CharacterCreationScreen::Name;
    assert_eq!(control_sound(CharacterCreationControl::Exit, &model), None);
    assert_eq!(
        control_sound(
            CharacterCreationControl::NameScroll(CharacterNamePart::First, 1),
            &model
        ),
        None
    );
    assert_eq!(
        control_sound(CharacterCreationControl::RandomName, &model),
        Some(CharacterCreationSoundCue::Specific(
            CharacterCreationSound::Random
        ))
    );
}

pub(super) fn creation_audio_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin {
            file_path: project_asset("").to_string_lossy().into_owned(),
            ..default()
        },
    ))
    .init_asset::<Image>()
    .init_asset::<Font>()
    .init_asset::<AudioSource>()
    .init_resource::<CharacterCreationUiModel>()
    .init_resource::<CharacterCreationUiOutbox>()
    .init_resource::<CharacterNameLists>()
    .init_resource::<CharacterCreationRandom>()
    .init_resource::<RetrobutionAudioMix>()
    .add_systems(Update, handle_character_creation_interactions);
    let assets = CharacterCreationAssets::load(app.world().resource::<AssetServer>());
    app.insert_resource(assets);
    let mut model = app.world_mut().resource_mut::<CharacterCreationUiModel>();
    model.visible = true;
    model.screen = CharacterCreationScreen::Appearance;
    model.creation_items = CharacterCreationCapability::Enabled;
    model.starter_icons = CharacterCreationCapability::Enabled;
    model.save_appearance = CharacterCreationCapability::Enabled;
    app
}

#[test]
fn creation_audio_plays_exact_cue_once_per_press_with_effects_gain() {
    for (control, cue) in [
        (
            CharacterCreationControl::Step(AppearanceField::Height, -1),
            CharacterCreationSound::HeightDown,
        ),
        (
            CharacterCreationControl::Step(AppearanceField::Height, 1),
            CharacterCreationSound::HeightUp,
        ),
        (
            CharacterCreationControl::Step(AppearanceField::Body, -1),
            CharacterCreationSound::GirthNarrow,
        ),
        (
            CharacterCreationControl::Step(AppearanceField::Body, 1),
            CharacterCreationSound::GirthWide,
        ),
        (
            CharacterCreationControl::Skin(1),
            CharacterCreationSound::Color,
        ),
        (
            CharacterCreationControl::HairColor(1),
            CharacterCreationSound::Color,
        ),
        (
            CharacterCreationControl::EyeColor(1),
            CharacterCreationSound::Color,
        ),
        (
            CharacterCreationControl::RandomAppearance,
            CharacterCreationSound::Random,
        ),
        (
            CharacterCreationControl::ContinueAppearance,
            CharacterCreationSound::Continue,
        ),
    ] {
        let mut app = creation_audio_test_app();
        app.world_mut()
            .resource_mut::<RetrobutionAudioMix>()
            .effects = 0.4;
        app.world_mut()
            .spawn((CharacterCreationButton(control), Interaction::Pressed));
        app.update();
        app.update(); // Holding the same button does not replay its one-shot.
        let expected = app.world().resource::<CharacterCreationAssets>().sounds[&cue].id();
        let world = app.world_mut();
        let mut query = world.query::<(&AudioPlayer, &PlaybackSettings)>();
        let sounds = query.iter(world).collect::<Vec<_>>();
        assert_eq!(sounds.len(), 1, "{control:?}");
        assert_eq!(sounds[0].0.0.id(), expected, "{control:?}");
        assert_eq!(sounds[0].1.volume, Volume::Linear(0.7 * 0.4));
    }
}

#[test]
fn creation_audio_rejects_hidden_blocked_selected_and_muted_controls() {
    for scenario in 0..6 {
        let mut app = creation_audio_test_app();
        let control = match scenario {
            0 => {
                app.world_mut()
                    .resource_mut::<CharacterCreationUiModel>()
                    .visible = false;
                CharacterCreationControl::RandomAppearance
            }
            1 => CharacterCreationControl::RandomName, // Hidden appearance-stage control.
            2 => {
                app.world_mut()
                    .resource_mut::<CharacterCreationUiModel>()
                    .save_appearance = CharacterCreationCapability::Pending(
                    CharacterCreationPending::SaveAppearanceNetworkCommand,
                );
                CharacterCreationControl::ContinueAppearance
            }
            3 => CharacterCreationControl::Gender(CharacterGender::Boy), // Already selected.
            4 => {
                app.world_mut()
                    .resource_mut::<RetrobutionAudioMix>()
                    .effects = 0.0;
                CharacterCreationControl::RandomAppearance
            }
            _ => {
                let mut messages = SystemMessageUiModel::default();
                messages.set_focus_out(true);
                app.insert_resource(messages);
                CharacterCreationControl::RandomAppearance
            }
        };
        app.world_mut()
            .spawn((CharacterCreationButton(control), Interaction::Pressed));
        app.update();
        let world = app.world_mut();
        assert_eq!(
            world.query::<&AudioPlayer>().iter(world).count(),
            0,
            "scenario {scenario}"
        );
    }
}

#[test]
fn creation_audio_loop_stops_when_production_phase_leaves_creation() {
    use crate::ui_startup::NativeUiStartupPhase;
    let mut app = App::new();
    insert_test_localization(&mut app, &project_asset(""));
    app.add_plugins((
        MinimalPlugins,
        bevy::state::app::StatesPlugin,
        AssetPlugin::default(),
    ))
    .init_asset::<Image>()
    .init_asset::<Font>()
    .init_asset::<AudioSource>()
    .add_message::<KeyboardInput>()
    .insert_state(NativeUiStartupPhase::Deferred)
    .add_plugins(NativeCharacterCreationUiPlugin);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NativeUiStartupPhase>>()
        .set(NativeUiStartupPhase::CharacterCreation);
    app.world_mut()
        .resource_mut::<CharacterCreationUiModel>()
        .visible = true;
    app.update();
    let music = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<CreationMusic>>()
            .single(world)
            .unwrap()
    };
    assert!(!app.world().get::<PlaybackSettings>(music).unwrap().paused);
    for phase in [
        NativeUiStartupPhase::CharacterSelection,
        NativeUiStartupPhase::Deferred,
    ] {
        app.world_mut()
            .resource_mut::<NextState<NativeUiStartupPhase>>()
            .set(phase);
        app.update();
        assert!(app.world().get::<PlaybackSettings>(music).unwrap().paused);
    }
}
