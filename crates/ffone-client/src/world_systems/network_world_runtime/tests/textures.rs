use super::*;

#[test]
fn npc_companion_pass_inherits_the_sources_final_xdt_texture() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .insert_resource(Assets::<LegacyModelMaterial>::default())
        .add_systems(Update, bind_network_npc_texture_variants_0104);

    let root = app.world_mut().spawn_empty().id();
    app.world_mut()
        .entity_mut(root)
        .insert(NetworkNpcVisual0104 {
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
        });

    let params = LegacyModelMaterialParams::for_shader(LegacyShaderKind::SkinnedToon);
    let mut source_material = params
        .material_for_pass(
            params.render_plan().passes[0],
            &LegacyModelTextures::default(),
        )
        .unwrap();
    let xdt_texture = Handle::<Image>::default();
    source_material.base_texture = Some(xdt_texture.clone());
    let source_handle = app
        .world_mut()
        .resource_mut::<Assets<LegacyModelMaterial>>()
        .add(source_material);
    let source = app
        .world_mut()
        .spawn((
            ChildOf(root),
            MeshMaterial3d(source_handle),
            NetworkNpcTextureVariantBound0104,
            NetworkNpcMaterialSurface0104 { root },
        ))
        .id();

    let companion_material = params
        .material_for_pass(
            params.render_plan().passes[0],
            &LegacyModelTextures::default(),
        )
        .unwrap();
    let companion_handle = app
        .world_mut()
        .resource_mut::<Assets<LegacyModelMaterial>>()
        .add(companion_material);
    let companion = app
        .world_mut()
        .spawn((
            ChildOf(source),
            MeshMaterial3d(companion_handle),
            LegacyMaterialPassCompanion {
                source_mesh_entity: source,
                pass: LegacyPassKind::TransparentColor,
            },
        ))
        .id();

    app.update();

    assert!(
        app.world()
            .get::<NetworkNpcTextureVariantBound0104>(companion)
            .is_some()
    );
    assert_eq!(
        app.world()
            .resource::<Assets<LegacyModelMaterial>>()
            .get(
                &app.world()
                    .get::<MeshMaterial3d<LegacyModelMaterial>>(companion)
                    .unwrap()
                    .0
            )
            .unwrap()
            .base_texture
            .as_ref(),
        Some(&xdt_texture)
    );
}

pub(super) fn texture_catalog(textures: Value, blocked: Value) -> Value {
    serde_json::json!({
        "schema": NPC_TEXTURE_CATALOG_SCHEMA,
        "textures": textures,
        "blocked": blocked
    })
}
