use super::*;

#[test]
fn every_character_selection_text_entity_has_semantic_ownership() {
    let asset_root = project_asset("");
    let mut app = App::new();
    insert_test_localization(&mut app, &asset_root);
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_message::<KeyboardInput>()
        .add_plugins(NativeCharacterSelectionUiPlugin);
    app.update();

    let mut texts = app.world_mut().query::<(&Text, Option<&LocalizedText>)>();
    let rows = texts.iter(app.world()).collect::<Vec<_>>();
    assert!(!rows.is_empty());
    assert!(rows.iter().all(|(_, localized)| localized.is_some()));
}

#[test]
fn location_separator_is_keyed_for_selected_and_unselected_characters() {
    let ordinary = OccupiedCharacterSlotUi {
        pc_uid: 1,
        display_name: "Ordinary".to_owned(),
        level: 1,
        district: "TECH SQUARE".to_owned(),
        zone: "THE FUTURE".to_owned(),
        background: CharacterLocationBackground::Future,
    };
    assert_eq!(ordinary.location_label(), "TECH SQUARE - THE FUTURE");
    assert_eq!(
        ordinary.location_label_for_state(false),
        "TECH SQUARE - THE FUTURE"
    );
    assert_eq!(
        ordinary.location_label_for_state(true),
        "TECH SQUARE - THE FUTURE"
    );
    let ordinary_unselected = ordinary.location_localized(false);
    assert_eq!(
        ordinary_unselected.key,
        "ui.character_select.location.hyphen"
    );
    assert_eq!(ordinary_unselected.fallback, "{district} - {zone}");
    assert_eq!(
        ordinary_unselected.args.get("district").map(String::as_str),
        Some("TECH SQUARE")
    );
    assert_eq!(
        ordinary_unselected.args.get("zone").map(String::as_str),
        Some("THE FUTURE")
    );

    let mut creation = ordinary.clone();
    creation.district = "CHARACTER CREATION".to_owned();
    creation.zone = "QUEUE".to_owned();
    assert_eq!(
        creation.location_label_for_state(false),
        "CHARACTER CREATION - QUEUE"
    );
    assert_eq!(
        creation.location_label_for_state(true),
        "CHARACTER CREATION  QUEUE"
    );
    assert_eq!(
        creation.location_localized(false).key,
        "ui.character_select.location.hyphen"
    );
    assert_eq!(
        creation.location_localized(true).key,
        "ui.character_select.location.spaced"
    );
}

#[test]
fn clean_control_visibility_requires_a_real_selected_or_creatable_slot() {
    let asset_root = project_asset("");
    let mut app = App::new();
    insert_test_localization(&mut app, &asset_root);
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_plugins(NativeCharacterSelectionUiPlugin);
    app.update();

    {
        let mut model = app.world_mut().resource_mut::<CharacterSelectionUiModel>();
        model.slots = [
            occupied(7, CharacterLocationBackground::Future),
            CharacterSlotUi::Empty,
            CharacterSlotUi::SubscriptionLocked,
            CharacterSlotUi::SubscriptionLocked,
        ];
        model.selected_slot = Some(0);
        model.create = CharacterSelectionCapability::Enabled;
        model.delete = CharacterSelectionCapability::Enabled;
    }
    app.update();
    let visible = |world: &mut World| {
        let enter = *world
            .query_filtered::<&Visibility, With<SelectionEnter>>()
            .single(world)
            .unwrap();
        let create = *world
            .query_filtered::<&Visibility, With<SelectionCreate>>()
            .single(world)
            .unwrap();
        let delete = *world
            .query_filtered::<&Visibility, With<SelectionDelete>>()
            .single(world)
            .unwrap();
        (enter, create, delete)
    };
    assert_eq!(
        visible(app.world_mut()),
        (
            Visibility::Inherited,
            Visibility::Inherited,
            Visibility::Inherited
        )
    );

    {
        let mut model = app.world_mut().resource_mut::<CharacterSelectionUiModel>();
        model.slots = [
            occupied(7, CharacterLocationBackground::Future),
            CharacterSlotUi::SubscriptionLocked,
            CharacterSlotUi::SubscriptionLocked,
            CharacterSlotUi::SubscriptionLocked,
        ];
        model.selected_slot = None;
    }
    app.update();
    assert_eq!(
        visible(app.world_mut()),
        (Visibility::Hidden, Visibility::Hidden, Visibility::Hidden)
    );
}

#[test]
fn subscription_locked_occupied_slot_keeps_player_data_but_cannot_be_selected() {
    let locked = OccupiedCharacterSlotUi {
        pc_uid: 41,
        display_name: "Coop".to_owned(),
        level: 36,
        district: "Genius Grove".to_owned(),
        zone: "The Future".to_owned(),
        background: CharacterLocationBackground::Future,
    };
    let mut model = CharacterSelectionUiModel::default();
    model.set_slots(
        [
            CharacterSlotUi::SubscriptionLockedOccupied(locked.clone()),
            occupied(7, CharacterLocationBackground::Suburbs),
            CharacterSlotUi::Empty,
            CharacterSlotUi::SubscriptionLocked,
        ],
        Some(41),
    );

    assert_eq!(model.slots[0].occupied(), Some(&locked));
    assert!(model.slots[0].subscription_locked());
    assert!(!model.is_selectable(0));
    assert_eq!(model.selected_slot, Some(1));
    assert!(!model.select_slot(0));
    assert_eq!(model.selected_slot, Some(1));
}

#[test]
fn selection_assets_are_semantic_pngs_with_exact_dimensions() {
    for spec in CHARACTER_SELECTION_IMAGE_SPECS {
        assert!(
            !spec.path.contains("--"),
            "runtime path contains a hash suffix"
        );
        let path = project_asset(spec.path);
        let bytes = fs::read(&path).unwrap();
        assert_eq!(bytes.len() as u64, spec.bytes, "{}", spec.role);
        assert_eq!(
            format!("{:X}", Sha256::digest(&bytes)),
            spec.sha256,
            "{}",
            spec.role
        );
        assert_eq!(
            image::image_dimensions(path).unwrap(),
            (spec.width, spec.height),
            "{}",
            spec.role
        );
    }
    for (location_index, location) in CharacterLocationBackground::ALL.into_iter().enumerate() {
        assert_eq!(
            CHARACTER_SELECTION_BACKGROUND_SPECS[location_index]
                .map(|spec| spec.source_path_id),
            location.source_path_ids()
        );
        for frame in 1..=CHARACTER_SELECTION_BACKGROUND_FRAME_COUNT {
            let path = location.asset_path(frame);
            assert!(!path.contains("--"), "runtime path contains a hash suffix");
            let expected = CHARACTER_SELECTION_BACKGROUND_SPECS[location_index][frame - 1];
            let bytes = fs::read(project_asset(&path)).unwrap();
            assert_eq!(bytes.len() as u64, expected.bytes, "{path}");
            assert_eq!(
                format!("{:X}", Sha256::digest(&bytes)),
                expected.sha256,
                "{path}"
            );
            assert_eq!(
                image::image_dimensions(project_asset(&path)).unwrap(),
                (542, 477),
                "{path}"
            );
        }
    }
    for path in CHARACTER_SELECTION_BUTTON_SOUND_PATHS.into_iter().chain([
        CHARACTER_SELECTION_DELETE_YES_SOUND_PATH,
        CHARACTER_SELECTION_DELETE_NO_SOUND_PATH,
        CHARACTER_SELECTION_MUSIC_PATH,
    ]) {
        assert!(project_asset(path).is_file(), "{path}");
    }
    assert!(project_asset(CHARACTER_SELECTION_CHALET_FONT_PATH).is_file());
    assert!(project_asset(CHARACTER_SELECTION_JEFFE_FONT_PATH).is_file());
}

#[test]
fn production_defers_selection_assets_until_selection_phase() {
    let asset_root = project_asset("");
    let mut app = App::new();
    insert_test_localization(&mut app, &asset_root);
    app.add_plugins(MinimalPlugins)
        .add_plugins(bevy::state::app::StatesPlugin)
        .add_plugins(AssetPlugin {
            file_path: asset_root.to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_asset::<AudioSource>()
        .add_message::<KeyboardInput>()
        .insert_state(crate::ui_startup::NativeUiStartupPhase::Deferred)
        .add_plugins(NativeCharacterSelectionUiPlugin);

    app.update();
    assert!(!app.world().contains_resource::<CharacterSelectionAssets>());

    app.world_mut()
        .resource_mut::<NextState<crate::ui_startup::NativeUiStartupPhase>>()
        .set(crate::ui_startup::NativeUiStartupPhase::CharacterSelection);
    app.update();
    assert!(app.world().contains_resource::<CharacterSelectionAssets>());
    app.world_mut()
        .resource_mut::<CharacterSelectionUiModel>()
        .visible = true;
    app.update();
    let music = {
        let world = app.world_mut();
        world
            .query_filtered::<Entity, With<SelectionMusic>>()
            .single(world)
            .unwrap()
    };
    assert!(!app.world().get::<PlaybackSettings>(music).unwrap().paused);
    app.world_mut()
        .resource_mut::<CharacterSelectionUiModel>()
        .visible = false;
    app.world_mut()
        .resource_mut::<NextState<crate::ui_startup::NativeUiStartupPhase>>()
        .set(crate::ui_startup::NativeUiStartupPhase::CharacterCreation);
    app.update();
    assert!(
        app.world().get::<PlaybackSettings>(music).unwrap().paused,
        "selection music must not overlap creation after its startup set stops"
    );
}
