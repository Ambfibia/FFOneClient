use std::path::Path;

use bevy::ecs::system::SystemState;

use crate::gameplay_nano_portraits::*;

#[test]
fn portrait_alpha_copy_preserves_world_material_and_is_applied_once() {
    use crate::legacy_model_material::{LegacyModelMaterialParams, LegacyModelTextures};
    let params = LegacyModelMaterialParams::from_shader_name(
        "Skin_FusionEffect_blendSrcalphaInvsrcalpha",
    )
    .unwrap();
    let original = params
        .material_for_pass(
            params.render_plan().passes[0],
            &LegacyModelTextures::default(),
        )
        .unwrap();
    assert_eq!(original.render_mode.color_write, LegacyColorWriteMask::Rgb);
    let mut app = App::new();
    app.init_resource::<Assets<LegacyModelMaterial>>()
        .add_systems(Update, bind_gameplay_nano_portrait_alpha);
    let shared = app
        .world_mut()
        .resource_mut::<Assets<LegacyModelMaterial>>()
        .add(original.clone());
    let root = app
        .world_mut()
        .spawn(GameplayNanoPortraitRoot {
            slot: 0,
            nano_id: 41,
            model_path: String::new(),
            gltf: Handle::default(),
        })
        .id();
    let portrait = app
        .world_mut()
        .spawn((ChildOf(root), MeshMaterial3d(shared.clone())))
        .id();
    let world_mesh = app.world_mut().spawn(MeshMaterial3d(shared.clone())).id();
    app.update();
    let portrait_handle = app
        .world()
        .get::<MeshMaterial3d<LegacyModelMaterial>>(portrait)
        .unwrap()
        .0
        .clone();
    assert_ne!(portrait_handle, shared);
    let materials = app.world().resource::<Assets<LegacyModelMaterial>>();
    assert_eq!(materials.get(&shared).unwrap(), &original);
    let mut expected = original.clone();
    expected.render_mode.color_write = LegacyColorWriteMask::Rgba;
    assert_eq!(materials.get(&portrait_handle).unwrap(), &expected);
    assert_eq!(
        app.world()
            .get::<MeshMaterial3d<LegacyModelMaterial>>(world_mesh)
            .unwrap()
            .0,
        shared
    );
    app.world_mut().clear_trackers();
    app.update();
    assert_eq!(
        app.world().resource::<Assets<LegacyModelMaterial>>().len(),
        2
    );
    assert!(
        !app.world()
            .entity(portrait)
            .get_ref::<MeshMaterial3d<LegacyModelMaterial>>()
            .unwrap()
            .is_changed()
    );
}

#[test]
fn portrait_hierarchy_walk_never_visits_an_unrelated_world_subtree() {
    let mut world = World::new();
    let portrait_root = world.spawn_empty().id();
    let portrait_child = world.spawn(ChildOf(portrait_root)).id();
    let portrait_grandchild = world.spawn(ChildOf(portrait_child)).id();
    let world_root = world.spawn_empty().id();
    let world_child = world.spawn(ChildOf(world_root)).id();

    let mut state = SystemState::<Query<&Children>>::new(&mut world);
    let children = state.get(&world).unwrap();
    let mut scratch = Vec::new();
    let mut visited = Vec::new();
    visit_hierarchy(portrait_root, &children, &mut scratch, |entity| {
        visited.push(entity);
    });
    visited.sort();

    let mut expected = vec![portrait_root, portrait_child, portrait_grandchild];
    expected.sort();
    assert_eq!(visited, expected);
    assert!(!visited.contains(&world_root));
    assert!(!visited.contains(&world_child));
}

#[test]
fn unchanged_portrait_camera_does_not_dirty_its_transform() {
    let mut app = App::new();
    app.init_resource::<GameplayUiModel>()
        .add_systems(Update, update_gameplay_nano_portrait_cameras);
    let camera = app
        .world_mut()
        .spawn((GameplayNanoPortraitCamera { slot: 0 }, Transform::IDENTITY))
        .id();

    app.update();
    app.world_mut().clear_trackers();
    app.update();

    let transform = app.world().entity(camera).get_ref::<Transform>().unwrap();
    assert!(!transform.is_changed());
}

#[test]
fn exact_retrobution_nano_camera_contract_is_stable() {
    let (full, rotation) = authored_nano_camera_transform(0.0);
    let (depleted, depleted_rotation) =
        authored_nano_camera_transform(GAMEPLAY_NANO_DEPLETED_CAMERA_OFFSET);
    assert!((full.z + 0.787_846_2).abs() < 0.000_001);
    assert!((full.y - 0.438_918_53).abs() < 0.000_001);
    assert!((depleted.y - full.y - 0.48).abs() < 0.000_001);
    assert_eq!(rotation, depleted_rotation);
    assert_eq!(GAMEPLAY_NANO_CAMERA_FOV_DEGREES, 45.0);
    assert_eq!(GAMEPLAY_NANO_CAMERA_NEAR, 0.3);
}

#[test]
fn taller_hud_view_preserves_original_pixel_positions_and_reveals_above() {
    use bevy::camera::CameraProjection;

    let projection = PerspectiveProjection {
        fov: GAMEPLAY_NANO_CAMERA_FOV_DEGREES.to_radians(),
        near: GAMEPLAY_NANO_CAMERA_NEAR,
        aspect_ratio: 1.0,
        ..default()
    };
    let original = projection.get_clip_from_view();
    let view = portrait_sub_camera_view(0).unwrap();
    let expanded = projection.get_clip_from_view_for_sub(&view);
    for rect in crate::gameplay_ui::NANO_ANIMATED_PORTRAIT_RECTS {
        assert_eq!(rect.width / rect.height, view.size.x as f32 / view.size.y as f32);
        assert_eq!(rect.y + rect.height, 84.0);
        assert_eq!(rect.y, 12.0 + view.offset.y / 2.0);
    }
    let pixel = |matrix: Mat4, point: Vec3, size: UVec2| {
        let ndc = matrix.project_point3(point);
        Vec2::new((ndc.x + 1.0) * 0.5, (1.0 - ndc.y) * 0.5) * size.as_vec2()
    };
    for point in [Vec3::new(0.1, 0.2, -0.8), Vec3::new(-0.1, -0.2, -1.2)] {
        let before = pixel(original, point, view.full_size);
        let after = pixel(expanded, point, view.size) + view.offset;
        assert!(before.abs_diff_eq(after, 0.0001));
    }
    let tall_head = Vec3::new(0.0, 0.5, -0.8);
    assert!(pixel(original, tall_head, view.full_size).y < 0.0);
    assert!(pixel(expanded, tall_head, view.size).y > 0.0);
    assert_eq!(portrait_texture_height(0), view.size.y);
    assert_eq!(portrait_texture_height(GAMEPLAY_NANO_PORTRAIT_COUNT), 128);
    assert!(portrait_sub_camera_view(GAMEPLAY_NANO_PORTRAIT_COUNT).is_none());
}

#[test]
fn journal_portrait_owns_a_distinct_exact_size_target() {
    assert_eq!(
        portrait_render_layer(GAMEPLAY_NANO_PORTRAIT_COUNT),
        JOURNAL_NANO_PORTRAIT_RENDER_LAYER
    );
    assert_eq!(
        portrait_texture_size(GAMEPLAY_NANO_PORTRAIT_COUNT),
        JOURNAL_NANO_PORTRAIT_TEXTURE_SIZE
    );
    assert!(
        !GAMEPLAY_NANO_PORTRAIT_RENDER_LAYERS.contains(&JOURNAL_NANO_PORTRAIT_RENDER_LAYER)
    );
    let (translation, rotation) = authored_journal_nano_camera_transform();
    assert!(translation.abs_diff_eq(Vec3::new(0.0, 0.3, -0.8), 0.000_001));
    assert!(
        (rotation * Vec3::NEG_Z).abs_diff_eq(Vec3::Z, 0.000_001),
        "native camera looks horizontally from -Z before applying fHeight"
    );

    let mut request = JournalNanoPortraitRequest::default();
    assert_eq!(request.desired(), None);
    request.set(1, "characters/nanos/nano_buttercup/nano_buttercup.glb");
    assert_eq!(
        request.desired(),
        Some((1, "characters/nanos/nano_buttercup/nano_buttercup.glb"))
    );
    request.clear();
    assert_eq!(request.desired(), None);
}

#[test]
fn real_assets_resolve_table_nanos_to_native_glbs_when_available() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("game");
    if !root.join(TABLE_SET_PATH).is_file() {
        return;
    }
    let assets = AssetLocator::open(&root).unwrap();
    let catalog = GameplayNanoPortraitCatalog::open(&assets).unwrap();
    assert_eq!(
        catalog.model_path(1),
        Some("characters/nanos/nano_buttercup/nano_buttercup.glb")
    );
    assert_eq!(
        catalog.model_path(2),
        Some("characters/nanos/nano_numbuhtwo/nano_numbuhtwo.glb")
    );
    assert_eq!(
        catalog.model_path(42),
        Some("characters/nanos/nano_belladonna/nano_belladonnaN.glb")
    );
    assert_eq!(
        catalog.model_path(24),
        Some("characters/nanos/nano_swampfire/nano_swampfire.glb")
    );
    assert_eq!(
        catalog.model_path(43),
        Some("characters/nanos/nano_computress/nano_computress.glb")
    );
    assert!(catalog.len() >= 40);
}
