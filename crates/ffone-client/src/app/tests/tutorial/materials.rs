use super::*;

#[test]
fn dexter_hologram_preserves_rgb_only_es740_render_target_pixels() {
    let shader = include_str!("../../../dexter_hologram.wgsl");
    let clear = DEXTER_SHIP_RENDER_TARGET_CLEAR.to_srgba();
    assert_eq!(
        clear.alpha, 0.0,
        "uncovered render-target pixels must remain transparent"
    );
    assert!(
        clear.red > 0.0 && clear.green > 0.0 && clear.blue > 0.0,
        "the authored camera RGB clear must remain available to additive passes"
    );
    assert!(
        shader.contains("scene.a * scanline.a * material.color.a"),
        "RGB-only ES740 passes must not expand the actor-shaped hologram alpha"
    );
    assert!(
        !shader.contains("additive_coverage"),
        "computer RGB must never become rectangular UI coverage"
    );
}

#[test]
fn dexter_hologram_embedded_registration_matches_material_shader_path() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<bevy::shader::Shader>()
        .init_asset_loader::<bevy::shader::ShaderLoader>();
    register_dexter_hologram_asset(&mut app);

    let shader: Handle<bevy::shader::Shader> = app
        .world()
        .resource::<AssetServer>()
        .load(AssetPath::from_path_buf(dexter_hologram_asset_path()).with_source("embedded"));
    for _ in 0..1_000 {
        app.update();
        if !matches!(
            app.world()
                .resource::<AssetServer>()
                .load_state(shader.id()),
            LoadState::Loading | LoadState::NotLoaded
        ) {
            break;
        }
        std::thread::sleep(Duration::from_millis(1));
    }

    let state = app
        .world()
        .resource::<AssetServer>()
        .load_state(shader.id());
    assert!(matches!(state, LoadState::Loaded), "load state: {state:?}");
}

#[test]
fn tutorial_residency_never_plain_preloads_xdt_npc_material_textures() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&asset_root).unwrap();
    let catalog = NetworkNpcVisualCatalog0104::open(&locator).unwrap();
    let plain_preloads = tutorial_residency_plain_image_paths();

    for npc_type in tutorial_actor_npc_types() {
        let Some(definition) = catalog.get(npc_type) else {
            continue;
        };
        for texture in [
            definition.main_texture.as_ref(),
            definition.sub_texture.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            assert!(
                !plain_preloads.contains(&texture.path),
                "tutorial residency must leave XDT texture {} to the exact-sampler material binder",
                texture.path
            );
        }
    }
}
