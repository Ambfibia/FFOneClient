use bevy::ecs::system::SystemState;
use ffone_runtime_contracts::PlayerRigGender;

use crate::character_selection_portraits::*;

#[test]
fn portrait_hat_uses_the_instance_exact_helmet_socket() {
    let mut app = App::new();
    app.init_resource::<CharacterSelectionPortraitsModel>()
        .add_systems(Update, bind_character_selection_portrait_attachments);

    let rig_root = app.world_mut().spawn_empty().id();
    let exact_socket = app
        .world_mut()
        .spawn((
            Name::new("Bip01 helmet01"),
            Transform::from_scale(Vec3::splat(0.25)),
            ChildOf(rig_root),
        ))
        .id();
    let duplicate_socket = app
        .world_mut()
        .spawn((
            Name::new("Bip01 helmet01"),
            Transform::from_scale(Vec3::splat(0.5)),
            ChildOf(rig_root),
        ))
        .id();
    let socket_full_path = player_attachment_socket_full_path(
        PlayerRigGender::Male,
        LegacyPlayerAttachmentSlot::Hat,
    );
    app.world_mut()
        .entity_mut(rig_root)
        .insert(NativePlayerRigBones::new(vec![
            crate::player_shared_rig::NativePlayerRigBoneEntity {
                actor_bone_index: 15,
                true_name: "Bip01 helmet01".to_owned(),
                full_path: socket_full_path.clone(),
                entity: exact_socket,
            },
        ]));
    let attachment = app
        .world_mut()
        .spawn((
            CharacterSelectionPortraitPart {
                slot: 0,
                kind: NativePlayerPartKind::Hat,
                exact_route: "wear/helmat_football.nif".to_owned(),
                generation: 1,
            },
            PendingCharacterSelectionPortraitAttachment {
                rig_root,
                socket_full_path,
                socket_local_scale_override: Some(Vec3::ONE),
            },
            Transform::default(),
            ChildOf(rig_root),
        ))
        .id();

    app.update();

    assert_eq!(
        app.world()
            .get::<ChildOf>(attachment)
            .expect("portrait hat remains parented")
            .parent(),
        exact_socket
    );
    assert_eq!(
        app.world()
            .get::<Transform>(exact_socket)
            .expect("exact portrait helmet socket")
            .scale,
        Vec3::ONE
    );
    assert_eq!(
        app.world()
            .get::<Transform>(duplicate_socket)
            .expect("duplicate modular helmet socket")
            .scale,
        Vec3::splat(0.5)
    );
    assert!(
        app.world()
            .get::<PendingCharacterSelectionPortraitAttachment>(attachment)
            .is_none()
    );
}

#[test]
fn selection_portrait_walk_never_visits_resident_world_entities() {
    let mut world = World::new();
    let portrait_root = world.spawn_empty().id();
    let portrait_child = world.spawn(ChildOf(portrait_root)).id();
    let world_root = world.spawn_empty().id();
    let world_child = world.spawn(ChildOf(world_root)).id();

    let mut state = SystemState::<Query<&Children>>::new(&mut world);
    let children = state.get(&world).unwrap();
    let mut scratch = Vec::new();
    let mut visited = Vec::new();
    visit_portrait_hierarchy(portrait_root, &children, &mut scratch, |entity| {
        visited.push(entity);
    });
    visited.sort();

    let mut expected = vec![portrait_root, portrait_child];
    expected.sort();
    assert_eq!(visited, expected);
    assert!(!visited.contains(&world_root));
    assert!(!visited.contains(&world_child));
}

#[test]
fn portrait_render_layers_are_repaired_across_existing_descendants() {
    let mut app = App::new();
    app.init_resource::<CharacterSelectionPortraitsRuntime>()
        .add_systems(Update, sync_character_selection_portrait_render_layers);
    let root = app
        .world_mut()
        .spawn(RenderLayers::layer(
            CHARACTER_SELECTION_PORTRAIT_RENDER_LAYERS[1],
        ))
        .id();
    let child = app.world_mut().spawn(ChildOf(root)).id();
    let grandchild = app
        .world_mut()
        .spawn((ChildOf(child), RenderLayers::layer(0)))
        .id();
    app.world_mut()
        .resource_mut::<CharacterSelectionPortraitsRuntime>()
        .slots[0]
        .root = Some(root);

    app.update();

    let expected = RenderLayers::layer(CHARACTER_SELECTION_PORTRAIT_RENDER_LAYERS[0]);
    for entity in [root, child, grandchild] {
        assert_eq!(app.world().get::<RenderLayers>(entity), Some(&expected));
    }
}

#[test]
fn legacy_portrait_rects_are_exact_inside_selection_panel() {
    let layout = CharacterSelectionLayout::for_viewport(Vec2::new(1264.0, 681.0), 1.0);
    let expected_y = [83.5, 180.5, 277.5, 373.5];
    for (slot, y) in expected_y.into_iter().enumerate() {
        assert_eq!(
            character_selection_portrait_rect(layout, slot),
            Some(LegacySelectionRect::new(688.0, y, 63.0, 85.0))
        );
    }
    assert_eq!(character_selection_portrait_rect(layout, 4), None);
}

#[test]
fn portrait_render_layers_are_unique_and_do_not_overlap_central_preview() {
    let unique = CHARACTER_SELECTION_PORTRAIT_RENDER_LAYERS
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), CHARACTER_SELECTION_PORTRAIT_COUNT);
    assert!(!unique.contains(&crate::player_preview::NATIVE_PLAYER_PREVIEW_RENDER_LAYER));
}

#[test]
fn portrait_camera_uses_serialized_slot_overrides_not_central_defaults() {
    assert_eq!(CHARACTER_SELECTION_PORTRAIT_CAMERA_DISTANCE, 0.5);
    assert_eq!(CHARACTER_SELECTION_PORTRAIT_CAMERA_HEIGHT, 0.12);
    assert_eq!(CHARACTER_SELECTION_PORTRAIT_CAMERA_FOV_DEGREES, 45.0);
    assert_eq!(CHARACTER_SELECTION_PORTRAIT_CAMERA_NEAR, 0.01);
    assert_eq!(CHARACTER_SELECTION_PORTRAIT_CAMERA_FAR, 1_000.0);
    assert!(
        CHARACTER_SELECTION_PORTRAIT_CAMERA_ORDER_BASE
            > crate::player_preview::NATIVE_PLAYER_PREVIEW_CAMERA_ORDER
    );
    assert!(
        CHARACTER_SELECTION_PORTRAIT_CAMERA_ORDER_BASE
            + CHARACTER_SELECTION_PORTRAIT_COUNT as isize
            - 1
            < crate::character_selection_ui::CHARACTER_SELECTION_MODAL_CAMERA_ORDER
    );
}

#[test]
fn gameplay_portrait_uses_an_independent_fixed_slot_and_exact_camera_contract() {
    let model = GameplayPlayerPortraitModel {
        visible: true,
        slot: Some(2),
    };
    assert_eq!(model.slot, Some(2));
    assert_eq!(GAMEPLAY_PLAYER_PORTRAIT_NECK_BONE, "Bip01 Neck");
    assert_eq!(
        (
            GAMEPLAY_PLAYER_PORTRAIT_TEXTURE_WIDTH,
            GAMEPLAY_PLAYER_PORTRAIT_TEXTURE_HEIGHT,
        ),
        (100, 120)
    );
    assert_eq!(GAMEPLAY_PLAYER_PORTRAIT_CAMERA_DISTANCE, 0.5);
    assert_eq!(GAMEPLAY_PLAYER_PORTRAIT_CAMERA_HEIGHT, 0.2);
    assert_eq!(GAMEPLAY_PLAYER_PORTRAIT_CAMERA_YAW_DEGREES, -20.0);
    assert_eq!(GAMEPLAY_PLAYER_PORTRAIT_CAMERA_FOV_DEGREES, 45.0);
    assert_eq!(GAMEPLAY_PLAYER_PORTRAIT_CAMERA_NEAR, 0.001);
    assert_eq!(GAMEPLAY_PLAYER_PORTRAIT_CAMERA_FAR, 100.0);

    let focus = Vec3::new(-3.0, 1.5, 7.0);
    let rig_rotation =
        native_scene_container_transform(NativeSceneRole::CharacterGameplay).rotation;
    let transform = gameplay_player_portrait_camera_transform(rig_rotation, focus);
    let unity_offset =
        Quat::from_rotation_y(GAMEPLAY_PLAYER_PORTRAIT_CAMERA_YAW_DEGREES.to_radians())
            * Vec3::Z
            * GAMEPLAY_PLAYER_PORTRAIT_CAMERA_DISTANCE;
    let expected_base = focus + rig_rotation * unity_to_native_vector(unity_offset);
    assert!(transform.translation.abs_diff_eq(
        expected_base + Vec3::Y * GAMEPLAY_PLAYER_PORTRAIT_CAMERA_HEIGHT,
        0.000_01
    ));
    assert!(
        (transform.rotation * -Vec3::Z)
            .abs_diff_eq((focus - expected_base).normalize(), 0.000_01)
    );
}

#[test]
fn blocked_slot_is_fail_closed_and_revises_the_instance() {
    let mut model = CharacterSelectionPortraitsModel::default();
    model.block_slot(2, "missing exact route").unwrap();
    assert_eq!(model.slots[2].revision(), 1);
    assert_eq!(
        model.slots[2].status,
        CharacterSelectionPortraitStatus::Blocked("missing exact route".to_owned())
    );
    assert!(model.slots[2].look().is_none());
    model.block_slot(2, "missing exact route").unwrap();
    assert_eq!(model.slots[2].revision(), 1);
}
