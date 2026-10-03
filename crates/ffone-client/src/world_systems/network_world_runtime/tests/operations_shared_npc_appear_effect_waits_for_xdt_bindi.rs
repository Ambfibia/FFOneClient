use super::*;

#[test]
fn npc_appear_effect_matches_the_primary_two_second_alpha_curve() {
    assert_eq!(network_npc_appear_alpha_0104(0.0), 0.0);
    assert_eq!(network_npc_appear_alpha_0104(1.0), 0.25);
    assert_eq!(network_npc_appear_alpha_0104(1.999), 0.49975);
    assert_eq!(network_npc_appear_alpha_0104(2.0), 1.0);
    assert_eq!(network_npc_appear_alpha_0104(5.0), 1.0);
}

#[test]
fn spawn11_green_sub_slot_uses_the_green_fusion_matter_contract() {
    assert!(network_npc_texture_uses_fusion_matter_0104(
        NetworkNpcTextureSlot0104::Sub,
        "spawn11_green"
    ));
    assert!(!network_npc_texture_uses_fusion_matter_0104(
        NetworkNpcTextureSlot0104::Main,
        "spawn11_green"
    ));

    let mut old = LegacyModelMaterialParams::for_shader(LegacyShaderKind::FusionEffect);
    old.uv_scale = Vec2::new(0.75, 1.25);
    old.uv_offset = Vec2::new(0.125, -0.25);
    let promoted = network_npc_fusion_matter_params_0104(&old);

    assert_eq!(promoted.shader, LegacyShaderKind::FusionMatterLightDir);
    assert_eq!(promoted.tint_color, LinearRgba::new(0.0, 1.0, 0.0, 1.0));
    assert_eq!(promoted.base_color, LinearRgba::new(0.6, 0.6, 0.6, 1.0));
    assert_eq!(promoted.uv_scale, old.uv_scale);
    assert_eq!(promoted.uv_offset, old.uv_offset);
    assert!(promoted.render_plan().outline().is_some());
}

#[test]
fn npc_scene_reveals_after_source_binding_without_waiting_for_optional_effect_passes() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, finalize_network_npc_material_visibility_0104);

    let root = app.world_mut().spawn_empty().id();
    let scene = app
        .world_mut()
        .spawn((
            ChildOf(root),
            Visibility::Hidden,
            LegacyCharacterSceneDeferredReveal,
            LegacyCharacterSceneStatus::Ready {
                root,
                authored_root: Transform::IDENTITY,
                applied_root: Transform::IDENTITY,
            },
        ))
        .id();
    app.world_mut().entity_mut(root).insert((
        NetworkNpcVisual0104 {
            npc_type: 2674,
            logical_name: "mob_sneakyspawn".to_owned(),
            glb_path: "characters/mobs/mob_sneakyspawn/mob_sneakyspawn.glb".to_owned(),
            table_scale: 1.5,
            visual_container: root,
            scene,
            gltf: Handle::default(),
            main_texture: None,
            sub_texture: None,
            walk_animation_speed: 1.0,
            run_animation_speed: 1.0,
            animation_effect_events: Arc::from([]),
            animation_sound_events: Arc::from([]),
            animation_ends: Arc::default(),
            material_animation_clips: Arc::from([]),
        },
        NetworkNpcAppearEffect0104::default(),
    ));
    let surface = app
        .world_mut()
        .spawn((
            ChildOf(scene),
            MeshMaterial3d::<StandardMaterial>(Handle::default()),
        ))
        .id();

    app.update();

    assert!(matches!(
        app.world().get::<Visibility>(scene),
        Some(Visibility::Hidden)
    ));
    assert!(
        app.world()
            .get::<NetworkNpcVisualMaterialReady0104>(root)
            .is_none()
    );

    app.world_mut()
        .entity_mut(surface)
        .remove::<MeshMaterial3d<StandardMaterial>>()
        .insert((
            MeshMaterial3d::<LegacyModelMaterial>(Handle::default()),
            LegacyMaterialApplied,
        ));
    let companion = app
        .world_mut()
        .spawn((
            ChildOf(surface),
            MeshMaterial3d::<LegacyModelMaterial>(Handle::default()),
            LegacyMaterialPassCompanion {
                source_mesh_entity: surface,
                pass: LegacyPassKind::TransparentColor,
            },
        ))
        .id();
    let outline = app
        .world_mut()
        .spawn((
            ChildOf(surface),
            MeshMaterial3d::<LegacyOutlineMaterial>(Handle::default()),
        ))
        .id();
    app.update();

    assert!(matches!(
        app.world().get::<Visibility>(scene),
        Some(Visibility::Hidden)
    ));
    assert!(
        app.world()
            .get::<LegacyCharacterSceneDeferredReveal>(scene)
            .is_some(),
        "the scene must also wait for its exact XDT texture slot"
    );

    app.world_mut()
        .entity_mut(surface)
        .insert(NetworkNpcTextureVariantBound0104);
    app.update();

    assert!(matches!(
        app.world().get::<Visibility>(scene),
        Some(Visibility::Inherited)
    ));
    assert!(
        app.world()
            .get::<LegacyCharacterSceneDeferredReveal>(scene)
            .is_none(),
        "optional companion and AppearEffect work must not keep a gameplay NPC invisible"
    );
    assert!(
        app.world()
            .get::<NetworkNpcTextureVariantBound0104>(companion)
            .is_none()
    );
    assert!(
        app.world()
            .get::<NetworkNpcAppearMaterialBound0104>(surface)
            .is_none()
    );
    assert!(
        app.world()
            .get::<NetworkNpcAppearOutlineBound0104>(outline)
            .is_none()
    );
    assert!(
        app.world()
            .get::<NetworkNpcVisualMaterialReady0104>(root)
            .is_some()
    );
}

#[test]
fn shared_npc_appear_effect_waits_for_xdt_binding_then_finishes() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            250,
        )))
        .insert_resource(Assets::<LegacyModelMaterial>::default())
        .insert_resource(Assets::<LegacyOutlineMaterial>::default())
        .insert_resource(Assets::<Image>::default())
        .add_systems(Update, animate_network_npc_appear_effect_0104);

    let root = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(root).insert((
        NetworkNpcVisual0104 {
            npc_type: 2674,
            logical_name: "mob_sneakyspawn".to_owned(),
            glb_path: "characters/mobs/mob_sneakyspawn/mob_sneakyspawn.glb".to_owned(),
            table_scale: 1.5,
            visual_container: root,
            scene: root,
            gltf: Handle::default(),
            main_texture: None,
            sub_texture: None,
            walk_animation_speed: 1.0,
            run_animation_speed: 1.0,
            animation_effect_events: Arc::from([]),
            animation_sound_events: Arc::from([]),
            animation_ends: Arc::default(),
            material_animation_clips: Arc::from([]),
        },
        NetworkNpcAppearEffect0104::default(),
    ));
    let params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::SkinnedToon);
    let authored_material = params
        .material_for_pass(
            params.render_plan().passes[0],
            &LegacyModelTextures::default(),
        )
        .unwrap();
    let authored_handle = app
        .world_mut()
        .resource_mut::<Assets<LegacyModelMaterial>>()
        .add(authored_material);
    let surface = app
        .world_mut()
        .spawn((MeshMaterial3d(authored_handle.clone()), ChildOf(root)))
        .id();
    let authored_outline_handle = app
        .world_mut()
        .resource_mut::<Assets<LegacyOutlineMaterial>>()
        .add(params.outline_material().unwrap());
    let outline_surface = app
        .world_mut()
        .spawn((
            MeshMaterial3d(authored_outline_handle.clone()),
            ChildOf(root),
        ))
        .id();

    app.update();

    assert!(
        app.world()
            .get::<NetworkNpcAppearMaterialBound0104>(surface)
            .is_none(),
        "AppearEffect must not clone an unbound authored material"
    );
    assert_eq!(
        app.world()
            .get::<MeshMaterial3d<LegacyModelMaterial>>(surface)
            .unwrap()
            .0,
        authored_handle
    );
    assert!(
        app.world()
            .get::<NetworkNpcAppearOutlineBound0104>(outline_surface)
            .is_none(),
        "AppearEffect must wait for the fill surface's XDT binding before cloning its outline"
    );
    assert_eq!(
        app.world()
            .get::<MeshMaterial3d<LegacyOutlineMaterial>>(outline_surface)
            .unwrap()
            .0,
        authored_outline_handle
    );

    let xdt_texture = app
        .world_mut()
        .resource_mut::<Assets<Image>>()
        .add(Image::default());
    let mut xdt_material = app
        .world()
        .resource::<Assets<LegacyModelMaterial>>()
        .get(&authored_handle)
        .unwrap()
        .clone();
    xdt_material.base_texture = Some(xdt_texture.clone());
    let xdt_handle = app
        .world_mut()
        .resource_mut::<Assets<LegacyModelMaterial>>()
        .add(xdt_material);
    app.world_mut().entity_mut(surface).insert((
        MeshMaterial3d(xdt_handle.clone()),
        NetworkNpcTextureVariantBound0104,
    ));

    app.update();

    assert!(
        app.world()
            .get::<NetworkNpcAppearMaterialBound0104>(surface)
            .is_some()
    );
    let appeared_handle = app
        .world()
        .get::<MeshMaterial3d<LegacyModelMaterial>>(surface)
        .unwrap()
        .0
        .clone();
    assert_ne!(appeared_handle, xdt_handle);
    let materials = app.world().resource::<Assets<LegacyModelMaterial>>();
    let appeared_material = materials.get(&appeared_handle).unwrap();
    assert_eq!(appeared_material.base_texture.as_ref(), Some(&xdt_texture));
    assert_eq!(appeared_material.uniform.base_color.alpha, 0.0625);
    let appeared_outline_handle = app
        .world()
        .get::<MeshMaterial3d<LegacyOutlineMaterial>>(outline_surface)
        .unwrap()
        .0
        .clone();
    assert_ne!(appeared_outline_handle, authored_outline_handle);
    let outline_materials = app.world().resource::<Assets<LegacyOutlineMaterial>>();
    assert_eq!(
        outline_materials
            .get(&appeared_outline_handle)
            .unwrap()
            .uniform
            .color
            .alpha,
        0.0625
    );
    assert!(
        app.world()
            .get::<NetworkNpcAppearEffect0104>(root)
            .is_some(),
        "the shared effect must remain active until the two-second restore"
    );

    for _ in 0..7 {
        app.update();
    }

    let appeared_material = app
        .world()
        .resource::<Assets<LegacyModelMaterial>>()
        .get(&appeared_handle)
        .unwrap();
    assert_eq!(appeared_material.uniform.base_color.alpha, 1.0);
    let appeared_outline_material = app
        .world()
        .resource::<Assets<LegacyOutlineMaterial>>()
        .get(&appeared_outline_handle)
        .unwrap();
    assert_eq!(appeared_outline_material.uniform.color.alpha, 1.0);
    assert!(
        app.world()
            .get::<NetworkNpcAppearEffect0104>(root)
            .is_none(),
        "the completed effect must stop mutating the NPC material"
    );
}

pub(super) fn table_set(npc_rows: Value, mesh_rows: Value) -> Value {
    serde_json::json!({
        "tables": [{
            "name": CONSOLIDATED_TABLE,
            "value": {
                "m_pNpcTable": {
                    "m_pNpcData": npc_rows,
                    "m_pNpcMeshData": mesh_rows
                }
            }
        }]
    })
}

pub(super) fn remote_pc_test_look(parts: &[(NativePlayerPartKind, &str)]) -> NativePlayerLook {
    NativePlayerLook {
        identity: "Remote Test".to_owned(),
        gender: ffone_runtime_contracts::PlayerRigGender::Male,
        parts: parts
            .iter()
            .enumerate()
            .map(
                |(index, (kind, route))| crate::player_preview::NativePlayerPartLook {
                    kind: *kind,
                    assembly: if matches!(
                        kind,
                        NativePlayerPartKind::Face
                            | NativePlayerPartKind::Hair
                            | NativePlayerPartKind::Shirt
                            | NativePlayerPartKind::Pants
                            | NativePlayerPartKind::Shoes
                    ) {
                        crate::player_preview::NativePlayerPartAssembly::SharedSkin
                    } else {
                        crate::player_preview::NativePlayerPartAssembly::RigidAttachment
                    },
                    exact_route: (*route).to_owned(),
                    glb: format!("characters/test/part-{index}.glb"),
                    primary_texture: None,
                    secondary_texture: None,
                },
            )
            .collect(),
        skin_texture: None,
        skin_color: LinearRgba::WHITE,
        hair_color: LinearRgba::WHITE,
        weapon_animation_profile: None,
        height_selector: 0,
        body_selector: 0,
    }
}

#[test]
fn production_traffic_car_sidecars_bind_exact_primary_meshcolliders() {
    let locator = AssetLocator::open(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"),
    )
    .unwrap();
    let registry: Value = locator.read_character_models().unwrap();
    let catalog = NetworkNpcVisualCatalog0104::open(&locator).unwrap();
    for (
        id,
        expected_npc_numbers,
        expected_mesh_path_id,
        expected_component_path_id,
        expected_vertices,
        expected_indices,
    ) in [
        (
            "npc/dt_etc_passengercar_a",
            2917..=2921,
            210,
            2663,
            226,
            1_200,
        ),
        ("npc/dt_etc_pickupcar_a", 2922..=2926, 316, 2660, 194, 942),
    ] {
        let model = registry["models"]
            .as_array()
            .unwrap()
            .iter()
            .find(|model| model["id"] == id)
            .expect("traffic car registry entry");
        let glb = model["glb"].as_str().unwrap();
        let collision_path = model["collision"].as_str().unwrap();
        let visual_hash = blake3::hash(&locator.read(glb).unwrap())
            .to_hex()
            .to_string();
        let contract: NativeCharacterCollisionContract0104 =
            locator.read_json(collision_path).unwrap();
        let collider_glb = contract.collider_glb.as_deref().unwrap();
        let collider_hash = blake3::hash(&locator.read(collider_glb).unwrap())
            .to_hex()
            .to_string();
        contract
            .validate(collision_path, glb, &visual_hash, Some(&collider_hash))
            .unwrap();
        assert_eq!(
            contract.source["meshCollider"]["meshPathId"],
            expected_mesh_path_id
        );
        assert_eq!(
            contract.source["meshCollider"]["componentPathId"],
            expected_component_path_id
        );
        assert_eq!(
            contract.source["meshCollider"]["ownerLocalTransform"],
            serde_json::json!({
                "translation": [0.0, 0.0, 0.0],
                "rotation": [0.0, 0.0, 0.0, 1.0],
                "scale": [1.0, 1.0, 1.0]
            })
        );
        assert!(
            contract.source["meshCollider"]["geometrySpace"]
                .as_str()
                .is_some_and(|detail| detail.contains("collision-node local"))
        );
        assert_eq!(contract.source["rigidbody"]["isKinematic"], true);
        assert_eq!(contract.colliders.len(), 1);
        assert_eq!(contract.colliders[0].node, "collision");
        assert_eq!(
            contract.colliders[0].expected_vertex_count,
            expected_vertices
        );
        assert_eq!(contract.colliders[0].expected_index_count, expected_indices);

        let visual_gltf = gltf::Gltf::from_slice(&locator.read(glb).unwrap()).unwrap();
        let collision_nodes = visual_gltf
            .nodes()
            .filter(|node| node.name() == Some("collision"))
            .collect::<Vec<_>>();
        let [collision_node] = collision_nodes.as_slice() else {
            panic!("{id} must retain exactly one collision node");
        };
        assert!(collision_node.mesh().is_none());
        let (translation, rotation, scale) = collision_node.transform().decomposed();
        let expected_node = &contract.source["collisionNode"];
        for (actual, expected) in translation.iter().zip(
            expected_node["translation"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_f64().unwrap() as f32),
        ) {
            assert!((*actual - expected).abs() < 1.0e-6);
        }
        for (actual, expected) in rotation.iter().zip(
            expected_node["rotation"]
                .as_array()
                .unwrap()
                .iter()
                .map(|value| value.as_f64().unwrap() as f32),
        ) {
            assert!((*actual - expected).abs() < 1.0e-6);
        }
        assert_eq!(scale, [1.0, 1.0, 1.0]);

        let collision_gltf = gltf::Gltf::from_slice(&locator.read(collider_glb).unwrap())
            .expect("exact traffic car collider GLB");
        let primitive = collision_gltf
            .meshes()
            .next()
            .unwrap()
            .primitives()
            .next()
            .unwrap();
        assert_eq!(
            primitive.get(&gltf::Semantic::Positions).unwrap().count(),
            expected_vertices
        );
        assert_eq!(primitive.indices().unwrap().count(), expected_indices);

        for npc_number in expected_npc_numbers {
            let definition = catalog.get(npc_number).expect("traffic car XDT route");
            assert_eq!(definition.collision_path.as_deref(), Some(collision_path));
            assert!(definition.collision_contract.is_some());
        }
    }
}

#[test]
fn setup_npc_hides_location_marker_classes_without_loading_the_objectnpc1_cube() {
    let table = table_set(
        serde_json::json!([
            {
                "m_iNpcNumber": 1175,
                "m_iNpcType": 23,
                "m_iMesh": 0,
                "m_iHeight": 200,
                "m_fScale": 0.01,
                "m_fWalkAnimationSpeed": 1.0,
                "m_fRunAnimationSpeed": 1.0
            },
            {
                "m_iNpcNumber": 798,
                "m_iNpcType": 100
            }
        ]),
        serde_json::json!([{"m_pstrMMeshModelString": "ObjectNPC1"}]),
    );
    let registry = registry(serde_json::json!([{
        "logicalName": "ObjectNPC1",
        "category": "npc",
        "glb": "characters/npcs/objectnpc1/ObjectNPC1.glb"
    }]));
    let mut required = Vec::new();
    let catalog = NetworkNpcVisualCatalog0104::from_documents(
        &table,
        &registry,
        &texture_catalog(serde_json::json!([]), serde_json::json!([])),
        |path| {
            required.push(path.to_owned());
            Ok(())
        },
        |_| Ok(()),
    )
    .unwrap();

    assert_eq!(required, ["characters/npcs/objectnpc1/ObjectNPC1.glb"]);
    assert!(catalog.get(1175).is_some(), "class 23 remains visible");
    assert!(catalog.get(798).is_none(), "class 100 is a hidden marker");
    assert!(catalog.issues.is_empty());
}

#[test]
fn resolves_xdt_main_and_sub_textures_with_exact_source_samplers() {
    let table = table_set(
        serde_json::json!([{
            "m_iNpcNumber": 55,
            "m_iNpcType": 0,
            "m_iMesh": 0,
            "m_iHeight": 200,
            "m_fScale": 1.0,
            "m_fWalkAnimationSpeed": 1.0,
            "m_fRunAnimationSpeed": 1.0
        }]),
        serde_json::json!([{
            "m_pstrMMeshModelString": "mob_spawn",
            "m_pstrMTextureString": "spawn11_green",
            "m_pstrMTextureString2": "spawn_eye"
        }]),
    );
    let registry = registry(serde_json::json!([{
        "logicalName": "mob_spawn",
        "category": "mob",
        "glb": "characters/mobs/mob_spawn/mob_spawn.glb"
    }]));
    let sampler = |name: &str| {
        serde_json::json!({
            "name": name,
            "magFilter": "linear",
            "minFilter": "linearMipmapLinear",
            "wrapS": "repeat",
            "wrapT": "repeat",
            "legacyFilterMode": 1,
            "legacyWrapMode": 0,
            "anisotropyLevel": 1,
            "mipMapBias": 0.0
        })
    };
    let textures = texture_catalog(
        serde_json::json!([
            {
                "trueName": "spawn11_green",
                "path": "textures/spawn11_green.png",
                "sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "sampler": sampler("spawn11_green")
            },
            {
                "trueName": "spawn_eye",
                "path": "textures/spawn_eye.png",
                "sha256": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "sampler": sampler("spawn_eye")
            }
        ]),
        serde_json::json!([]),
    );
    let mut required_textures = Vec::new();
    let catalog = NetworkNpcVisualCatalog0104::from_documents(
        &table,
        &registry,
        &textures,
        |_| Ok(()),
        |path| {
            required_textures.push(path.to_owned());
            Ok(())
        },
    )
    .unwrap();

    assert_eq!(
        required_textures,
        ["textures/spawn11_green.png", "textures/spawn_eye.png"]
    );
    assert!(catalog.issues.is_empty());
    let definition = catalog.get(55).unwrap();
    assert_eq!(
        definition.main_texture.as_ref().unwrap().true_name,
        "spawn11_green"
    );
    assert_eq!(
        definition.sub_texture.as_ref().unwrap().path,
        "textures/spawn_eye.png"
    );
    assert_eq!(
        definition.main_texture.as_ref().unwrap().sampler.min_filter,
        ffone_skinned_model::SamplerMinFilter::LinearMipmapLinear
    );
}

#[test]
fn npc_motion_stream_keeps_walk_until_packets_stop() {
    let mut app = App::new();
    app.insert_resource(Time::<()>::default());
    app.add_systems(Update, advance_network_npc_motion_0104);
    let entity = app
        .world_mut()
        .spawn((
            NetworkNpc0104 {
                npc_id: 7,
                npc_type: 1007,
            },
            Transform::default(),
        ))
        .id();
    for segment in 1..=20 {
        app.world_mut()
            .entity_mut(entity)
            .insert(NetworkNpcMotion0104 {
                destination: Vec3::new(segment as f32 * 0.8, 0.0, 0.0),
                speed: 8.0,
                move_style: 0,
            });
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(100));
        app.update();
        assert!(
            app.world().get::<NetworkNpcMotion0104>(entity).is_some(),
            "segment boundary must not request idle and restart walking"
        );
    }
    let stopped = app.world().get::<Transform>(entity).unwrap().translation;
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(Duration::from_millis(160));
    app.update();
    assert!(app.world().get::<NetworkNpcMotion0104>(entity).is_none());
    assert_eq!(
        app.world().get::<Transform>(entity).unwrap().translation,
        stopped
    );
}

#[test]
fn remote_pc_visible_cosmetics_use_exact_slots_without_claiming_weapon_ownership() {
    for (kind, slot, suffix) in [
        (
            NativePlayerPartKind::Hat,
            LegacyPlayerAttachmentSlot::Hat,
            "Bip01 helmet01",
        ),
        (
            NativePlayerPartKind::Glasses,
            LegacyPlayerAttachmentSlot::Glasses,
            "Bip01 glass01",
        ),
        (
            NativePlayerPartKind::Back,
            LegacyPlayerAttachmentSlot::Back,
            "Bip01 back01",
        ),
    ] {
        assert_eq!(network_pc_appearance_attachment_slot(kind), Some(slot));
        let exact_path = player_attachment_socket_full_path(
            ffone_runtime_contracts::PlayerRigGender::Male,
            slot,
        );
        assert_eq!(exact_path, format!("m/{}", slot.socket_path()));
        assert!(exact_path.ends_with(suffix));
    }
    assert_eq!(
        network_pc_appearance_attachment_slot(NativePlayerPartKind::Weapon),
        None,
        "the existing remote-player weapon owner must remain unique"
    );
    assert_eq!(
        network_pc_appearance_attachment_slot(NativePlayerPartKind::Shirt),
        None,
        "shared-skin parts must not be rigidly attached"
    );
}

#[test]
fn remote_pc_appearance_readiness_waits_for_exact_routes_and_pending_sockets() {
    let look = remote_pc_test_look(&[
        (NativePlayerPartKind::Face, "wear/face"),
        (NativePlayerPartKind::Hat, "wear/hat"),
        (NativePlayerPartKind::Glasses, "wear/glasses"),
        (NativePlayerPartKind::Back, "wear/back"),
        (NativePlayerPartKind::Weapon, "wear/hand"),
    ]);
    let expected = network_pc_expected_appearance_routes(&look);
    assert_eq!(expected.len(), 4);
    assert!(!expected.contains(&(NativePlayerPartKind::Weapon, "wear/hand".to_owned())));

    assert!(network_pc_appearance_routes_ready(
        &expected,
        1,
        1,
        expected.len(),
        &expected,
        false,
    ));
    assert!(!network_pc_appearance_routes_ready(
        &expected,
        1,
        1,
        expected.len(),
        &expected,
        true,
    ));
    let mut missing_back = expected.clone();
    missing_back.remove(&(NativePlayerPartKind::Back, "wear/back".to_owned()));
    assert!(!network_pc_appearance_routes_ready(
        &expected,
        1,
        1,
        missing_back.len(),
        &missing_back,
        false,
    ));
    assert!(!network_pc_appearance_routes_ready(
        &expected,
        1,
        1,
        expected.len() + 1,
        &expected,
        false,
    ));
}
