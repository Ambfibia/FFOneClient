use bevy::asset::AssetPlugin;

use crate::player_preview::*;
use crate::player_shared_rig::NativePlayerRigBoneEntity;

fn published_runtime_texture(true_name: &str) -> NativePlayerTexture {
    let document: ffone_runtime_contracts::CharacterCreationRuntimeTextures =
        serde_json::from_str(include_str!(
            "../../../../../assets/game/data/character_creation/runtime_textures.json"
        ))
        .expect("parse published character runtime textures");
    let contract = document
        .textures
        .into_iter()
        .find(|contract| contract.true_name == true_name)
        .unwrap_or_else(|| panic!("published runtime texture {true_name:?}"));
    NativePlayerTexture {
        path: contract.native_asset.path.clone(),
        contract,
    }
}

fn part(kind: NativePlayerPartKind, glb: &str) -> NativePlayerPartLook {
    NativePlayerPartLook {
        kind,
        assembly: if matches!(
            kind,
            NativePlayerPartKind::Face
                | NativePlayerPartKind::Hair
                | NativePlayerPartKind::Shirt
                | NativePlayerPartKind::Pants
                | NativePlayerPartKind::Shoes
        ) {
            NativePlayerPartAssembly::SharedSkin
        } else {
            NativePlayerPartAssembly::RigidAttachment
        },
        exact_route: format!("wear/{kind:?}.nif").to_ascii_lowercase(),
        glb: glb.to_owned(),
        primary_texture: None,
        secondary_texture: None,
    }
}

#[test]
fn weapon_attachment_uses_the_instance_bone_map_not_a_duplicate_joint_name() {
    let mut app = App::new();
    app.init_resource::<NativePlayerPreviewModel>();
    app.add_systems(Update, bind_native_player_preview_attachments);

    let rig_root = app.world_mut().spawn_empty().id();
    let live_socket = app
        .world_mut()
        .spawn((
            Name::new("Bip01 Rweapon01"),
            Transform::from_scale(Vec3::splat(0.364_763_5)),
            ChildOf(rig_root),
        ))
        .id();
    let duplicate_part_socket = app
        .world_mut()
        .spawn((
            Name::new("Bip01 Rweapon01"),
            Transform::from_scale(Vec3::splat(0.125)),
            ChildOf(rig_root),
        ))
        .id();
    let socket_full_path = player_attachment_socket_full_path(
        PlayerRigGender::Male,
        LegacyPlayerAttachmentSlot::RightPistol,
    );
    app.world_mut()
        .entity_mut(rig_root)
        .insert(NativePlayerRigBones::new(vec![NativePlayerRigBoneEntity {
            actor_bone_index: 92,
            true_name: "Bip01 Rweapon01".to_owned(),
            full_path: socket_full_path.clone(),
            entity: live_socket,
        }]));
    let attachment = app
        .world_mut()
        .spawn((
            Transform::default(),
            PendingNativePlayerPreviewAttachment {
                rig_root,
                socket_full_path,
                socket_local_scale_override: Some(Vec3::ONE),
            },
            ChildOf(rig_root),
        ))
        .id();

    app.update();

    assert_eq!(
        app.world()
            .get::<ChildOf>(attachment)
            .expect("attachment remains parented")
            .parent(),
        live_socket
    );
    assert_eq!(
        app.world()
            .get::<Transform>(live_socket)
            .expect("live socket transform")
            .scale,
        Vec3::ONE
    );
    assert_eq!(
        app.world()
            .get::<Transform>(duplicate_part_socket)
            .expect("duplicate part socket transform")
            .scale,
        Vec3::splat(0.125)
    );
    assert!(
        app.world()
            .get::<PendingNativePlayerPreviewAttachment>(attachment)
            .is_none()
    );
}

#[test]
fn native_look_rejects_duplicate_parts_and_legacy_paths() {
    let mut look = NativePlayerLook {
        identity: "Test Ser".to_owned(),
        gender: PlayerRigGender::Male,
        parts: vec![
            part(
                NativePlayerPartKind::Face,
                "characters/player/items/mask/face/models/face/model.glb",
            ),
            part(
                NativePlayerPartKind::Face,
                "characters/player/items/mask/other/models/other/model.glb",
            ),
        ],
        skin_texture: None,
        skin_color: LinearRgba::WHITE,
        hair_color: LinearRgba::WHITE,
        weapon_animation_profile: None,
        height_selector: 2,
        body_selector: 1,
    };
    assert!(look.validate().is_err());
    look.parts.truncate(1);
    look.parts[0].glb = "../legacy.unity3d".to_owned();
    assert!(look.validate().is_err());
}

#[test]
fn exact_actor_skin_combiner_tints_are_linear_and_half_strength() {
    let tint = half_tint(LinearRgba::new(0.8, 0.4, 0.2, 1.0));
    assert_eq!(tint, LinearRgba::new(0.4, 0.2, 0.1, 0.5));
    assert_eq!(
        multiply_rgb(
            LinearRgba::new(0.5, 0.25, 1.0, 1.0),
            LinearRgba::new(0.8, 0.4, 0.2, 1.0)
        ),
        LinearRgba::new(0.4, 0.1, 0.2, 1.0)
    );
}

#[test]
fn exact_actor_skin_combiner_texture_branches_are_ordered() {
    assert_eq!(
        actor_skin_texture_role("shirt-sub-hair"),
        ActorSkinTextureRole::SecondarySkin
    );
    assert_eq!(
        actor_skin_texture_role("shirt-override"),
        ActorSkinTextureRole::PreserveOverride
    );
    assert_eq!(
        actor_skin_texture_role("m_head_001_type01-hair-link_a.dds"),
        ActorSkinTextureRole::Primary
    );
}

#[test]
fn only_set_body_parts_receive_global_skin_on_sub_materials() {
    for kind in [
        NativePlayerPartKind::Face,
        NativePlayerPartKind::Hair,
        NativePlayerPartKind::Shirt,
        NativePlayerPartKind::Pants,
        NativePlayerPartKind::Shoes,
    ] {
        assert!(uses_global_skin_secondary(kind), "{kind:?}");
    }
    for kind in [
        NativePlayerPartKind::Hat,
        NativePlayerPartKind::Glasses,
        NativePlayerPartKind::Back,
        NativePlayerPartKind::Weapon,
    ] {
        assert!(!uses_global_skin_secondary(kind), "{kind:?}");
    }
}

#[test]
fn base_level_publication_accepts_audited_nine_level_source_mips() {
    let texture = published_runtime_texture("f_pants_stylistdandy");
    assert!(texture.contract.source.mip_map);
    assert_eq!(texture.contract.source.source_mip_count, 9);
    assert_eq!(
        texture.contract.published_mip_policy,
        PublishedMipPolicy::BaseLevelOnly
    );
    validate_player_texture(&texture).expect("valid exact source-mip evidence");
}

#[test]
fn runtime_texture_rejects_contradictory_source_mip_evidence() {
    let mut texture = published_runtime_texture("f_pants_stylistdandy");
    texture.contract.source.mip_map = false;
    assert!(validate_player_texture(&texture).is_err());

    texture.contract.source.mip_map = true;
    texture.contract.source.source_mip_count = 1;
    assert!(validate_player_texture(&texture).is_err());
}

#[test]
fn native_look_rejects_invalid_palette_channels() {
    let mut look = NativePlayerLook {
        identity: "Test Ser".to_owned(),
        gender: PlayerRigGender::Male,
        parts: vec![part(
            NativePlayerPartKind::Face,
            "characters/player/items/mask/m_face_001_type01/models/m_face_001_type01/model.glb",
        )],
        skin_texture: None,
        skin_color: LinearRgba::new(f32::NAN, 0.0, 0.0, 1.0),
        hair_color: LinearRgba::WHITE,
        weapon_animation_profile: None,
        height_selector: 2,
        body_selector: 1,
    };
    assert!(look.validate().is_err());
    look.skin_color = LinearRgba::WHITE;
    look.hair_color = LinearRgba::new(1.1, 0.0, 0.0, 1.0);
    assert!(look.validate().is_err());
}

#[test]
fn preview_camera_renders_after_opaque_panorama_and_below_modal() {
    assert!(NATIVE_PLAYER_PREVIEW_CAMERA_ORDER > GAMEPLAY_UI_CAMERA_ORDER);
    assert!(
        NATIVE_PLAYER_PREVIEW_CAMERA_ORDER
            < crate::character_selection_ui::CHARACTER_SELECTION_MODAL_CAMERA_ORDER
    );
    assert!(NATIVE_PLAYER_INVENTORY_PREVIEW_CAMERA_ORDER < GAMEPLAY_UI_CAMERA_ORDER);
}

#[test]
fn preview_camera_uses_exact_stage_values_and_raises_only_when_close() {
    let mut model = NativePlayerPreviewModel::default();
    assert_eq!(
        native_player_preview_camera_distance_and_height(&model, Some(1.6)),
        (
            NATIVE_PLAYER_SELECTION_CAMERA_DISTANCE,
            NATIVE_PLAYER_SELECTION_CAMERA_HEIGHT
        )
    );

    model.stage = NativePlayerPreviewStage::TryOn;
    assert_eq!(
        native_player_preview_camera_distance_and_height(&model, None),
        (2.2, 0.7)
    );
    model.stage = NativePlayerPreviewStage::Creation;
    model.camera_distance = NATIVE_PLAYER_CREATION_CAMERA_DISTANCE;
    assert_eq!(
        native_player_preview_camera_distance_and_height(&model, Some(1.6)),
        (
            NATIVE_PLAYER_CREATION_CAMERA_DISTANCE,
            NATIVE_PLAYER_CREATION_CAMERA_HEIGHT
        )
    );
    model.camera_distance = NATIVE_PLAYER_CREATION_CAMERA_MIN_DISTANCE;
    assert_eq!(
        native_player_preview_camera_distance_and_height(&model, Some(1.6)),
        (NATIVE_PLAYER_CREATION_CAMERA_MIN_DISTANCE, 1.6)
    );
    model.camera_distance = NATIVE_PLAYER_CREATION_CAMERA_MAX_DISTANCE;
    assert_eq!(
        native_player_preview_camera_distance_and_height(&model, Some(1.6)),
        (
            NATIVE_PLAYER_CREATION_CAMERA_MAX_DISTANCE,
            NATIVE_PLAYER_CREATION_CAMERA_HEIGHT
        )
    );

    model.stage = NativePlayerPreviewStage::Inventory;
    model.camera_distance = f32::NAN;
    assert_eq!(
        native_player_preview_camera_distance_and_height(&model, Some(99.0)),
        (
            NATIVE_PLAYER_INVENTORY_CAMERA_DISTANCE,
            NATIVE_PLAYER_INVENTORY_CAMERA_HEIGHT
        ),
        "UserEquip must use AvatarClothes' fixed camera rather than creator zoom/head height"
    );
}

#[test]
fn selection_preview_viewport_is_centered_on_the_authored_frame() {
    let window = Window {
        resolution: (1264, 681).into(),
        ..default()
    };
    let selection = CharacterSelectionUiModel::default();
    let model = NativePlayerPreviewModel::default();

    let viewport =
        native_player_preview_viewport(&model, None, Some(&selection), &window).unwrap();

    assert_eq!(viewport.physical_position, UVec2::new(122, 26));
    assert_eq!(
        viewport.physical_size,
        UVec2::new(
            CHARACTER_SELECTION_PREVIEW_WIDTH as u32,
            CHARACTER_SELECTION_PREVIEW_HEIGHT as u32,
        )
    );
    assert_eq!(
        viewport.physical_position.x as f32 + viewport.physical_size.x as f32 * 0.5,
        397.0,
        "the player must use the 550 px preview-frame center, not the 600 px control group"
    );
}

#[test]
fn inventory_preview_preserves_the_full_ownuser_target_for_ui_clipping() {
    let window = Window {
        resolution: (1264, 681).into(),
        ..default()
    };
    let mut model = NativePlayerPreviewModel::default();
    model.stage = NativePlayerPreviewStage::Inventory;

    assert_eq!(NATIVE_PLAYER_INVENTORY_PREVIEW_WIDTH, 500);
    assert_eq!(NATIVE_PLAYER_INVENTORY_PREVIEW_HEIGHT, 564);
    assert!(
        native_player_preview_viewport(&model, None, None, &window).is_none(),
        "Inventory renders the full logical rect off-screen; Panel_UserClothes clips y=-50 without changing camera aspect"
    );
}

#[test]
fn geometry_key_ignores_texture_and_palette_only_changes() {
    let mut look = NativePlayerLook {
        identity: "Creator".to_owned(),
        gender: PlayerRigGender::Male,
        parts: vec![part(
            NativePlayerPartKind::Hair,
            "characters/player/items/head/hair/models/hair/model.glb",
        )],
        skin_texture: None,
        skin_color: LinearRgba::WHITE,
        hair_color: LinearRgba::WHITE,
        weapon_animation_profile: None,
        height_selector: 2,
        body_selector: 1,
    };
    let before = NativePlayerPreviewGeometry::from(&look);
    look.hair_color = LinearRgba::BLACK;
    look.parts[0].primary_texture = Some(published_runtime_texture("m_skin"));
    let after = NativePlayerPreviewGeometry::from(&look);
    assert_eq!(before, after);
}

#[test]
fn appearance_rebind_preserves_late_face_render_order_for_both_genders() {
    use crate::legacy_model_material::LegacyModelTextures;

    for gender in ["m", "f"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../../assets/game/characters/player/items/mask/{gender}_face_001_type01/models/{gender}_face_001_type01/model.glb"
        ));
        let bytes = std::fs::read(path).expect("published face GLB");
        let json_len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let document: serde_json::Value =
            serde_json::from_slice(&bytes[20..20 + json_len]).unwrap();
        let mut pass_count = 0;
        for source in document["materials"].as_array().unwrap() {
            let pending = PendingLegacyModelMaterial::from_gltf_extras(
                source["name"].as_str(),
                &source["extras"].to_string(),
            )
            .unwrap();
            for pass in pending.params.render_plan().passes {
                let Some(mut material) = pending
                    .params
                    .material_for_pass(pass, &LegacyModelTextures::default())
                else {
                    continue;
                };
                let baseline = NativePlayerPreviewMaterialBaseline::capture(&material);
                // Reproduce first binding before the renderer-order system.
                material.sort_bias = 0.125 + pass_count as f32;
                let assigned_order = material.sort_bias;
                for _ in 0..3 {
                    material.uniform.base_color = LinearRgba::BLACK;
                    material.uniform.emission = LinearRgba::BLACK;
                    baseline.restore(&mut material);
                    assert_eq!(material.sort_bias, assigned_order, "{gender} face pass");
                    assert_eq!(material.uniform.base_color, baseline.base_color);
                    assert_eq!(material.uniform.emission, baseline.emission);
                }
                pass_count += 1;
            }
        }
        assert!(
            pass_count >= 2,
            "face skin and overlay must both be covered"
        );
    }
}

#[test]
fn geometry_key_rebuilds_when_the_weapon_animation_family_changes() {
    let mut look = NativePlayerLook {
        identity: "Weapon Preview".to_owned(),
        gender: PlayerRigGender::Male,
        parts: vec![part(
            NativePlayerPartKind::Weapon,
            "characters/player/items/weapon/models/weapon/model.glb",
        )],
        skin_texture: None,
        skin_color: LinearRgba::WHITE,
        hair_color: LinearRgba::WHITE,
        weapon_animation_profile: Some(PlayerWeaponAnimationProfile::Pistol),
        height_selector: 2,
        body_selector: 1,
    };
    let pistol = NativePlayerPreviewGeometry::from(&look);
    look.weapon_animation_profile = Some(PlayerWeaponAnimationProfile::Rifle);
    let rifle = NativePlayerPreviewGeometry::from(&look);
    assert_ne!(pistol, rifle);
}

#[test]
fn preview_double_buffer_discards_only_pending_and_swaps_after_ready() {
    let displayed = Entity::from_raw_u32(1).unwrap();
    let pending = Entity::from_raw_u32(2).unwrap();
    assert_eq!(
        superseded_pending_root(Some(displayed), Some(displayed)),
        None,
        "a displayed root must survive while its replacement loads"
    );
    assert_eq!(
        superseded_pending_root(Some(pending), Some(displayed)),
        Some(pending),
        "a rapid appearance change replaces only the superseded pending root"
    );
    let next_ready = Entity::from_raw_u32(3).unwrap();
    let mut displayed_root = Some(displayed);
    let previous = displayed_root.replace(next_ready);
    assert_eq!(previous, Some(displayed));
    assert_eq!(displayed_root, Some(next_ready));
    assert_eq!(
        displayed_root,
        Some(next_ready),
        "a later pending failure does not clear the last ready root"
    );
}

#[test]
fn central_preview_recovers_its_idle_graph_after_foreign_animation_replacement() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<AnimationGraph>()
        .add_systems(Update, recover_native_player_preview_idle_animation);

    let clip = Handle::<AnimationClip>::default();
    let (idle_graph, idle_node) = AnimationGraph::from_clip(clip.clone());
    let (foreign_graph, foreign_node) = AnimationGraph::from_clip(clip);
    let idle_graph = app
        .world_mut()
        .resource_mut::<Assets<AnimationGraph>>()
        .add(idle_graph);
    let foreign_graph = app
        .world_mut()
        .resource_mut::<Assets<AnimationGraph>>()
        .add(foreign_graph);

    let mut player = AnimationPlayer::default();
    player
        .start(foreign_node)
        .set_repeat(RepeatAnimation::Never)
        .pause();
    let player = app
        .world_mut()
        .spawn((player, AnimationGraphHandle(foreign_graph)))
        .id();
    app.world_mut().spawn((
        NativePlayerPreviewRoot,
        NativePlayerRigStand1Playback {
            animation_player: player,
            animation_graph: idle_graph.clone(),
            animation_node: idle_node,
        },
    ));

    app.update();

    let recovered = app.world().get::<AnimationPlayer>(player).unwrap();
    assert_eq!(recovered.playing_animations().count(), 1);
    let idle = recovered.animation(idle_node).unwrap();
    assert_eq!(idle.repeat_mode(), RepeatAnimation::Forever);
    assert!(!idle.is_paused());
    assert_eq!(idle.speed(), 1.0);
    assert_eq!(
        app.world().get::<AnimationGraphHandle>(player),
        Some(&AnimationGraphHandle(idle_graph))
    );
}
