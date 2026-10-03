use std::{fs, path::Path};

use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::network_world_runtime::transportation::*;

fn gltf_with_named_animation(name: &str) -> Gltf {
    let mut gltf = Gltf {
        scenes: Vec::new(),
        named_scenes: Default::default(),
        meshes: Vec::new(),
        named_meshes: Default::default(),
        materials: Vec::new(),
        named_materials: Default::default(),
        nodes: Vec::new(),
        named_nodes: Default::default(),
        skins: Vec::new(),
        named_skins: Default::default(),
        default_scene: None,
        animations: Vec::new(),
        named_animations: Default::default(),
        source: None,
    };
    let animation = Handle::<AnimationClip>::default();
    gltf.animations.push(animation.clone());
    gltf.named_animations.insert(name.into(), animation);
    gltf
}

#[test]
fn slider_contract_is_exact_in_runtime_data() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let runtime = fs::read(root.join("assets/game/data/transportation/downtown_bus.json"))
        .expect("runtime transportation contract");
    let contract: Value = serde_json::from_slice(&runtime).unwrap();
    assert_eq!(contract["transportationType"], SLIDER_TRANSPORTATION_TYPE);
    assert_eq!(contract["logicalName"], SLIDER_LOGICAL_NAME);
    assert_eq!(contract["glb"], SLIDER_GLTF);
    assert_eq!(contract["animation"], SLIDER_STAND_ANIMATION);
    assert_eq!(contract["collision"]["glb"], SLIDER_COLLISION_GLTF);
    assert_eq!(contract["collision"]["node"], SLIDER_COLLISION_NODE);
    assert_eq!(
        contract["collision"]["expectedVertexCount"],
        SLIDER_COLLISION_VERTEX_COUNT
    );
    assert_eq!(
        contract["collision"]["expectedIndexCount"],
        SLIDER_COLLISION_INDEX_COUNT
    );
    assert_eq!(contract["source"]["alias"], "primary");
    assert_eq!(contract["source"]["meshColliderPathId"], 6977);
    assert_eq!(contract["source"]["rigidbody"]["isKinematic"], true);
    assert_eq!(contract["serverOwnership"]["routePointCount"], 64);
    assert_eq!(contract["serverOwnership"]["looped"], true);
    assert_eq!(
        contract["materialBindings"]["main"]["materialTrueName"],
        SLIDER_MAIN_MATERIAL
    );
    assert_eq!(
        contract["materialBindings"]["sub"]["materialTrueName"],
        SLIDER_SUB_MATERIAL
    );
    assert_eq!(
        contract["materialBindings"]["shader"]["trueName"],
        "normal_glow_blendSrcalphaInvsrcalpha"
    );
    assert_eq!(
        contract["materialBindings"]["shader"]["sourceScriptSha256"],
        "6ebaf7007733225536951c3be40e445c643c181c65d71494a678ebb364e87ffe"
    );
    assert_eq!(contract["materialBindings"]["shader"]["colorMask"], "RGB");
    assert_eq!(
        contract["materialBindings"]["shader"]["bumpMapSemantic"],
        "fixedFunctionGlowMask"
    );
    assert_eq!(
        contract["materialBindings"]["shader"]["rgbCombiner"],
        "MainTex.rgb * lerp(Previous.rgb, Constant0.5, BumpMap.a)"
    );
    assert_eq!(contract["materialBindings"]["shader"]["rgbScale"], 1.0);
    assert_eq!(
        contract["materialBindings"]["shader"]["alphaCombiner"],
        "MainTex.a * Primary.a"
    );
    assert_eq!(
        contract["materialBindings"]["shader"]["gltfNormalTextureUsage"],
        "loaderHintOnly"
    );
}

#[test]
fn slider_assets_bind_the_recorded_exact_hashes_and_collider_counts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let contract: Value = serde_json::from_slice(
        &fs::read(root.join("data/transportation/downtown_bus.json")).unwrap(),
    )
    .unwrap();
    let visual = fs::read(root.join(SLIDER_GLTF)).unwrap();
    assert_eq!(
        blake3::hash(&visual).to_hex().as_str(),
        contract["glbBlake3"].as_str().unwrap()
    );
    let visual_gltf = gltf::Gltf::from_slice(&visual).expect("exact Slider visual GLB");
    let collision_nodes = visual_gltf
        .nodes()
        .filter(|node| node.name() == Some(SLIDER_COLLISION_NODE))
        .collect::<Vec<_>>();
    let [collision_node] = collision_nodes.as_slice() else {
        panic!(
            "visual GLB must retain exactly one empty collision node, found {}",
            collision_nodes.len()
        );
    };
    assert!(collision_node.mesh().is_none());
    let (translation, rotation, scale) = collision_node.transform().decomposed();
    for (actual, expected) in
        translation
            .into_iter()
            .zip([-0.030127736, 2.8161774, 0.011581391])
    {
        assert!((actual - expected).abs() < 1.0e-6);
    }
    for (actual, expected) in rotation
        .into_iter()
        .zip([-0.70710677, 0.0, 0.0, 0.70710677])
    {
        assert!((actual - expected).abs() < 1.0e-6);
    }
    assert_eq!(scale, [1.0, 1.0, 1.0]);
    let collision =
        fs::read(root.join(SLIDER_COLLISION_GLTF)).expect("exact Slider collision GLB bytes");
    assert_eq!(
        format!("{:x}", Sha256::digest(&collision)),
        contract["collision"]["sha256"].as_str().unwrap()
    );
    let collision = gltf::Gltf::from_slice(&collision).expect("exact Slider collision GLB");
    let primitive = collision
        .meshes()
        .next()
        .unwrap()
        .primitives()
        .next()
        .unwrap();
    assert_eq!(
        primitive.get(&gltf::Semantic::Positions).unwrap().count(),
        134
    );
    assert_eq!(primitive.indices().unwrap().count(), 531);
    for (slot, path) in [("main", SLIDER_MAIN_TEXTURE), ("sub", SLIDER_SUB_TEXTURE)] {
        let bytes = fs::read(root.join(path)).unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(bytes)),
            contract["textures"][slot]["sha256"].as_str().unwrap()
        );
    }
    assert_eq!(contract["textures"]["sampler"]["legacyFilterMode"], 1);
    assert_eq!(contract["textures"]["sampler"]["legacyWrapMode"], 0);
}

#[test]
fn slider_preserves_authored_atlases_unless_source_name_matches_table_role() {
    assert_eq!(slider_texture_role(SLIDER_MAIN_MATERIAL), None);
    assert_eq!(slider_texture_role(SLIDER_SUB_MATERIAL), None);
    for untouched in [
        "dt_etc_downtownbus_a_00-car_for0411-k_resurrection_c.dds",
        "dt_etc_downtownbus_a_00-19 - default-car_for08.dds",
        "Main",
        "Sub",
    ] {
        assert_eq!(slider_texture_role(untouched), None, "{untouched}");
    }

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let bytes = fs::read(root.join(SLIDER_GLTF)).expect("exact Slider visual GLB");
    let document = gltf::Gltf::from_slice(&bytes).expect("exact Slider visual GLB");
    let materials = document
        .materials()
        .map(|material| material.name().unwrap_or_default().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        materials
            .iter()
            .filter_map(|name| slider_texture_role(name).map(|role| (name.as_str(), role)))
            .collect::<Vec<_>>(),
        vec![]
    );
    assert_eq!(
        slider_texture_role("main (Instance)"),
        Some(LegacyNpcTextureRole::Main)
    );
    assert_eq!(
        slider_texture_role("sub (Instance)"),
        Some(LegacyNpcTextureRole::Sub)
    );
    assert_eq!(
        slider_texture_role("sub_main"),
        Some(LegacyNpcTextureRole::Main)
    );
}

#[test]
fn slider_glow_mask_is_native_fixed_function_data_not_a_pbr_normal_map() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let bytes = fs::read(root.join(SLIDER_GLTF)).expect("exact Slider visual GLB");
    let json_length = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let document: Value = serde_json::from_slice(&bytes[20..20 + json_length]).unwrap();
    let glow = document["materials"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|material| {
            material["extras"]["ffone"]["legacyShaderName"]
                == "normal_glow_blendSrcalphaInvsrcalpha"
        })
        .collect::<Vec<_>>();
    assert_eq!(glow.len(), 2);
    for material in glow {
        let name = material["name"].as_str().unwrap();
        assert!([SLIDER_MAIN_MATERIAL, SLIDER_SUB_MATERIAL].contains(&name));
        assert_eq!(slider_texture_role(name), None);
        assert_eq!(
            material["normalTexture"]["extras"]["ffone"]["slot"],
            "_BumpMap"
        );
        assert_eq!(
            material["normalTexture"]["extras"]["ffone"]["standardTextureRefIsLoaderHint"],
            true
        );
        let extras = serde_json::to_string(&serde_json::json!({
            "ffone": material["extras"]["ffone"].clone()
        }))
        .unwrap();
        let pending = PendingLegacyModelMaterial::from_gltf_extras(Some(name), &extras)
            .expect("Slider glow material has an exact native runtime contract");
        assert_eq!(
            pending.params.shader,
            crate::legacy_model_material::LegacyShaderKind::AlphaBlendNormalGlow
        );
        assert!(pending.params.glow_mask);
        assert!(
            pending
                .texture_bindings
                .iter()
                .any(|binding| binding.slot == "_BumpMap")
        );
        let bump = Handle::<Image>::default();
        let textures = crate::legacy_model_material::LegacyModelTextures {
            base: Some(Handle::default()),
            bump: Some(bump.clone()),
            ..default()
        };
        assert!(textures.is_complete_for(pending.params.shader));
        let native = pending
            .params
            .material_for_pass(pending.params.render_plan().passes[0], &textures)
            .expect("normal_glow is a native LegacyModelMaterial pass");
        assert_eq!(native.bump_texture, Some(bump));
        assert_eq!(native.uniform.legacy_effect.z, 1.0);
        assert_eq!(
            native.render_mode.color_write,
            crate::legacy_model_material::LegacyColorWriteMask::Rgb
        );
    }
}

#[test]
fn slider_glow_surfaces_preserve_uv0_normals_and_do_not_require_tangents() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let bytes = fs::read(root.join(SLIDER_GLTF)).expect("exact Slider visual GLB");
    let document = gltf::Gltf::from_slice(&bytes).expect("exact Slider visual GLB");
    let mut glow_primitives = 0;
    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let Some(name) = primitive.material().name() else {
                continue;
            };
            if ![SLIDER_MAIN_MATERIAL, SLIDER_SUB_MATERIAL].contains(&name) {
                continue;
            }
            glow_primitives += 1;
            assert!(
                primitive.get(&gltf::Semantic::TexCoords(0)).is_some(),
                "{name} must retain source UV0"
            );
            assert!(
                primitive.get(&gltf::Semantic::Normals).is_some(),
                "{name} must retain source normals"
            );
            assert!(
                primitive.get(&gltf::Semantic::Tangents).is_none(),
                "{name} uses a glow mask, not tangent-space normal mapping"
            );
        }
    }
    assert_eq!(glow_primitives, 5);
}

#[test]
fn streaming_budget_is_intentionally_bounded() {
    assert_eq!(MAX_TRANSPORTATION_VISUAL_SPAWNS_PER_FRAME, 2);
    assert!(MAX_TRANSPORTATION_VISUAL_SPAWNS_PER_FRAME < 8);
}

#[test]
fn slider_animation_retries_only_its_pending_player_until_gltf_is_ready() {
    let mut app = App::new();
    app.init_resource::<Assets<Gltf>>()
        .init_resource::<Assets<AnimationGraph>>()
        .add_systems(Update, play_network_transportation_animation_0104);
    let gltf = app.world().resource::<Assets<Gltf>>().reserve_handle();
    let root = app.world_mut().spawn_empty().id();
    app.world_mut()
        .entity_mut(root)
        .insert(NetworkTransportationVisual0104 {
            transportation_type: SLIDER_TRANSPORTATION_TYPE,
            visual_container: root,
            gltf: gltf.clone(),
        });
    let player = app
        .world_mut()
        .spawn((AnimationPlayer::default(), ChildOf(root)))
        .id();

    app.update();
    assert!(
        app.world()
            .get::<PendingSliderAnimation0104>(player)
            .is_some()
    );
    assert!(app.world().get::<AnimationGraphHandle>(player).is_none());

    app.world_mut()
        .resource_mut::<Assets<Gltf>>()
        .insert(gltf.id(), gltf_with_named_animation(SLIDER_STAND_ANIMATION))
        .expect("reserved Slider glTF handle remains valid");
    app.update();
    assert!(
        app.world()
            .get::<PendingSliderAnimation0104>(player)
            .is_none()
    );
    assert!(app.world().get::<AnimationGraphHandle>(player).is_some());
}

#[test]
fn slider_preserves_clean_prefab_facing_and_source_root_transform() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let contract: Value = serde_json::from_slice(
        &fs::read(root.join("data/transportation/downtown_bus.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        contract["source"]["rootTransform"],
        serde_json::json!({
            "translation": [0.0, 0.0, 0.0],
            "rotation": [0.0, 0.0, 0.0, 1.0],
            "scale": [1.0, 1.0, 1.0]
        })
    );
    assert_eq!(
        contract["serverOwnership"]["clientController"],
        "cnBusMoveController"
    );
    assert_eq!(contract["serverOwnership"]["clientChangesRotation"], false);
    assert_eq!(
        crate::character_scene::native_scene_container_transform(
            crate::character_scene::NativeSceneRole::TransportationGameplay
        ),
        Transform::IDENTITY
    );
    assert_eq!(
        crate::coordinates::LegacyCharacterRootPolicy::Transportation
            .resolve_root(Transform::IDENTITY),
        Transform::IDENTITY
    );

    let original_rotation = Quat::from_rotation_y(0.37);
    let mut transform = Transform::from_rotation(original_rotation);
    let arrived = super::super::advance_server_entity_motion(
        &mut transform,
        Vec3::new(5.0, 0.0, -2.0),
        1.0,
        0.25,
        false,
    );
    assert!(!arrived);
    assert_eq!(transform.rotation, original_rotation);
}
