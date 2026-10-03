use super::*;

pub(super) fn occupied(uid: i64, background: CharacterLocationBackground) -> CharacterSlotUi {
    CharacterSlotUi::Occupied(OccupiedCharacterSlotUi {
        pc_uid: uid,
        display_name: format!("Player {uid}"),
        level: 1,
        district: "TECH SQUARE".to_owned(),
        zone: "THE FUTURE".to_owned(),
        background,
    })
}

pub(super) fn only_entity<M: Component>(world: &mut World) -> Entity {
    world
        .query_filtered::<Entity, With<M>>()
        .single(world)
        .unwrap()
}

#[test]
fn clean_primary_ownership_archives_and_reached_branches_are_pinned() {
    assert_eq!(CHARACTER_SELECTION_PRIMARY_MAIN_UNITY3D_BYTES, 7_000_415);
    assert_eq!(
        CHARACTER_SELECTION_PRIMARY_MAIN_UNITY3D_SHA256,
        "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F"
    );
    assert_eq!(
        CHARACTER_SELECTION_PRIMARY_CREATION_RESOURCE_BYTES,
        8_974_798
    );
    assert_eq!(
        CHARACTER_SELECTION_PRIMARY_CREATION_RESOURCE_SHA256,
        "78785925E716027DE4BEC897C402C352BB411B7D627DE1783E6FBDA8BF34E59E"
    );
    assert_eq!(
        CHARACTER_SELECTION_PRIMARY_PLAYER_RESOURCE_BYTES,
        14_544_152
    );
    assert_eq!(
        CHARACTER_SELECTION_PRIMARY_PLAYER_RESOURCE_SHA256,
        "FE050AF079AF43458A79CB27284A243F532AD56BB79E3C526C44E313C1A72930"
    );
    assert_eq!(CHARACTER_SELECTION_GAME_OBJECT_PATH_ID, 1_334);
    assert_eq!(CHARACTER_SELECTION_GUI_COMPONENT_PATH_ID, 1_523);
    assert_eq!(CHARACTER_SELECTION_MODE_COMPONENT_PATH_ID, 1_524);
    assert_eq!(CHARACTER_SELECTION_SKIN_PATH_ID, 1_382);
    assert_eq!(CHARACTER_SELECTION_GUI_SCRIPT_PATH_ID, 1_161);
    assert_eq!(CHARACTER_SELECTION_MODE_SCRIPT_PATH_ID, 921);
    assert_eq!(CHARACTER_SELECTION_GUI_DEPTH, 10);
    assert_eq!(CHARACTER_SELECTION_CHROME_SOURCE_PATH_ID, 78);
    assert_eq!(
        CharacterLocationBackground::Future.source_path_ids(),
        [63, 37, 59, 30, 102, 118, 73, 117, 19, 38]
    );
    assert_eq!(
        CharacterLocationBackground::Suburbs.source_path_ids(),
        [129, 104, 32, 3, 62, 92, 79, 110, 34, 5]
    );
    assert_eq!(
        CharacterLocationBackground::Downtown.source_path_ids(),
        [74, 66, 109, 112, 49, 99, 80, 36, 101, 100]
    );
    assert_eq!(
        CharacterLocationBackground::Wilds.source_path_ids(),
        [95, 85, 82, 58, 41, 12, 55, 22, 14, 67]
    );
    assert_eq!(
        CharacterLocationBackground::Darklands.source_path_ids(),
        [51, 107, 2, 48, 111, 7, 6, 106, 88, 16]
    );
}

#[test]
fn create_delete_and_preview_are_honest_typed_pending_states() {
    let model = CharacterSelectionUiModel::default();
    assert_eq!(
        model.create,
        CharacterSelectionCapability::Pending(
            CharacterSelectionPending::CreateCharacterNetworkCommand
        )
    );
    assert_eq!(
        model.delete,
        CharacterSelectionCapability::Pending(
            CharacterSelectionPending::DeleteCharacterNetworkCommand
        )
    );
    assert_eq!(model.preview, CharacterPreviewStatus::PlayerAssemblyPending);
    assert_eq!(model.preview_yaw_degrees, 0.0);
}

#[test]
fn create_slot_is_first_true_empty_and_is_protocol_one_based() {
    let mut model = CharacterSelectionUiModel::default();
    model.slots = [
        occupied(7, CharacterLocationBackground::Future),
        CharacterSlotUi::SubscriptionLocked,
        CharacterSlotUi::Empty,
        CharacterSlotUi::Empty,
    ];
    assert_eq!(model.first_creatable_protocol_slot(), Some(3));

    model.slots[2] = CharacterSlotUi::SubscriptionLockedOccupied(
        occupied(8, CharacterLocationBackground::Wilds)
            .occupied()
            .unwrap()
            .clone(),
    );
    assert_eq!(model.first_creatable_protocol_slot(), Some(4));
    model.slots[3] = CharacterSlotUi::SubscriptionLocked;
    assert_eq!(model.first_creatable_protocol_slot(), None);
}

#[test]
fn source_draw_modes_use_exact_stretch_and_five_pixel_nine_slice_contracts() {
    fn assert_stretch(image: &ImageNode) {
        assert_eq!(image.image_mode, NodeImageMode::Stretch);
    }

    fn assert_sliced_five(image: &ImageNode) {
        assert_eq!(image.visual_box, bevy::ui::VisualBox::BorderBox);
        let NodeImageMode::Sliced(slicer) = &image.image_mode else {
            panic!("expected serialized five-pixel nine-slice");
        };
        assert_eq!(slicer.border, BorderRect::all(5.0));
        assert_eq!(slicer.center_scale_mode, SliceScaleMode::Stretch);
        assert_eq!(slicer.sides_scale_mode, SliceScaleMode::Stretch);
    }

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

    let world = app.world_mut();
    macro_rules! assert_marker_mode {
        ($marker:ty, $assertion:ident) => {{
            let mut query = world.query_filtered::<&ImageNode, With<$marker>>();
            let rows = query.iter(world).collect::<Vec<_>>();
            assert!(!rows.is_empty(), stringify!($marker));
            for image in rows {
                $assertion(image);
            }
        }};
    }
    assert_marker_mode!(SelectionBackgroundFrame, assert_stretch);
    assert_marker_mode!(SelectionChrome, assert_stretch);
    assert_marker_mode!(SelectionEnter, assert_stretch);
    assert_marker_mode!(SelectionMusicToggle, assert_stretch);
    assert_marker_mode!(SelectionFullscreen, assert_stretch);
    assert_marker_mode!(SelectionDeleteModal, assert_stretch);
    assert_marker_mode!(SelectionDeletePanel, assert_stretch);

    assert_marker_mode!(SelectionSlotButton, assert_sliced_five);
    assert_marker_mode!(SelectionSlotDiskBack, assert_sliced_five);
    assert_marker_mode!(SelectionSlotDiskFront, assert_sliced_five);
    assert_marker_mode!(SelectionSlotLock, assert_sliced_five);
    assert_marker_mode!(SelectionRotateLeft, assert_sliced_five);
    assert_marker_mode!(SelectionRotateRight, assert_sliced_five);
    assert_marker_mode!(SelectionCreate, assert_sliced_five);
    assert_marker_mode!(SelectionDelete, assert_sliced_five);
    assert_marker_mode!(SelectionQuit, assert_sliced_five);
    assert_marker_mode!(SelectionDeleteCancel, assert_sliced_five);
    assert_marker_mode!(SelectionDeleteConfirm, assert_sliced_five);
    let backdrop = world
        .query_filtered::<&ImageNode, With<SelectionDeleteModal>>()
        .single(world)
        .unwrap();
    assert_eq!(backdrop.color, Color::srgba(0.0, 0.0, 0.0, 0.75));
}

#[test]
fn occupied_slot_level_keeps_the_exact_server_value_in_ui_text() {
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
        model.visible = true;
        model.slots[2] = CharacterSlotUi::Occupied(OccupiedCharacterSlotUi {
            pc_uid: 6,
            display_name: "Gaia Roundbreath".to_owned(),
            level: 36,
            district: "Genius Grove".to_owned(),
            zone: "The Future".to_owned(),
            background: CharacterLocationBackground::Future,
        });
        model.selected_slot = Some(2);
    }
    app.update();

    let (level_text, empty_level_visibility) = {
        let world = app.world_mut();
        let mut query = world.query::<(&SelectionSlotLevel, &LocalizedText, &Visibility)>();
        let rows = query
            .iter(world)
            .map(|(marker, text, visibility)| (marker.0, text.clone(), *visibility))
            .collect::<Vec<_>>();
        (
            rows.iter()
                .find_map(|(slot, text, _)| (*slot == 2).then_some(text.clone()))
                .unwrap(),
            rows.iter()
                .find_map(|(slot, _, visibility)| (*slot == 1).then_some(*visibility))
                .unwrap(),
        )
    };
    assert_eq!(level_text.key, "ui.character_select.level");
    assert_eq!(level_text.fallback, "LEVEL {level}");
    assert_eq!(level_text.args.get("level").map(String::as_str), Some("36"));
    assert_eq!(empty_level_visibility, Visibility::Hidden);
}
