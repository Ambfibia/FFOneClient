//! Legacy skybox, sky zones and fusion star.

use super::{LocalPlayer, WorldSliceEntity};
use bevy::{camera::visibility::RenderLayers, math::Affine2, prelude::*};
use ffone_client::{
    coordinates::{native_to_unity_vector, unity_to_native_rotation, unity_to_native_vector},
    movement::LegacyOrbitCamera,
    world::NativeWorldCatalog,
};

pub(super) const LEGACY_FUSION_STAR_TEXTURE: &str = "map/shared/effects/textures/fusionstar.png";
pub(super) const LEGACY_FUSION_STAR_POSITION_UNITY: Vec3 = Vec3::new(0.0, 8.0, 12.0);
pub(super) const LEGACY_FUSION_STAR_SIZE: f32 = 10.0;
pub(super) const LEGACY_SKYBOX_SIZE: f32 = 900.0;
pub(super) const LEGACY_FUSION_STAR_BACKGROUND_DISTANCE: f32 = LEGACY_SKYBOX_SIZE * 0.45;
// The Retrobution reference capture has the user-facing Graphics > Glow
// option disabled. `mainData` still serializes a GlowEffect component, but
// cnPlayerCamera.SetupRenderOptions() calls SetEnable(GetGlow()) after loading
// cnOption. Keeping the post-process attached here unconditionally multiplied
// the whole frame by DefaultAmbience.filterColor and made additive tutorial
// markers bloom even though the reference profile does neither.
pub(super) const LEGACY_REFERENCE_GLOW_ENABLED: bool = false;
pub(super) const LEGACY_SKYBOX_CAMERA_FAR: f32 = 1_000.0;
pub(super) const LEGACY_SKYBOX_CAMERA_ORDER: isize = -2;

pub(super) const GAMEPLAY_SKY_RENDER_LAYER: usize = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LegacySkyZone {
    Past,
    Future,
}

#[derive(Component)]
pub(super) struct LegacySkyboxRoot {
    pub(super) zone: LegacySkyZone,
}

#[derive(Component)]
pub(super) struct LegacySkyboxFace {
    pub(super) past: Handle<StandardMaterial>,
    pub(super) future: Handle<StandardMaterial>,
}

#[derive(Component)]
pub(super) struct LegacySkyboxCamera;

#[derive(Component)]
pub(super) struct LegacyFusionStarRoot;

#[derive(Component)]
pub(super) struct LegacyFusionStarPlane;

pub(super) const LEGACY_PAST_SKY_TEXTURES: [&str; 6] = [
    "map/shared/environment/skyboxes/past/front.png",
    "map/shared/environment/skyboxes/past/back.png",
    "map/shared/environment/skyboxes/past/left.png",
    "map/shared/environment/skyboxes/past/right.png",
    "map/shared/environment/skyboxes/past/top.png",
    "map/shared/environment/skyboxes/past/bottom.png",
];
pub(super) const LEGACY_FUTURE_SKY_TEXTURES: [&str; 6] = [
    "map/shared/environment/skyboxes/future/front.png",
    "map/shared/environment/skyboxes/future/back.png",
    "map/shared/environment/skyboxes/future/left.png",
    "map/shared/environment/skyboxes/future/right.png",
    "map/shared/environment/skyboxes/future/top.png",
    "map/shared/environment/skyboxes/future/bottom.png",
];

/// Exact `cnPlayerCamera` rule: Tutorial and Future use
/// `TutorialAssets/Skybox_FreeZone.mat`; every other region uses
/// `TutorialAssets/Skybox.mat`.
pub(super) fn legacy_sky_zone(native_position: Vec3) -> LegacySkyZone {
    let unity = native_to_unity_vector(native_position);
    let in_rect = |x: f32, z: f32, width: f32, height: f32| {
        unity.x >= x && unity.x <= x + width && unity.z >= z && unity.z <= z + height
    };
    if in_rect(512.0, 512.0, 512.0, 512.0) || in_rect(5_632.0, 512.0, 2_048.0, 2_048.0) {
        LegacySkyZone::Future
    } else {
        LegacySkyZone::Past
    }
}

pub(super) fn legacy_sky_material(
    asset_server: &AssetServer,
    materials: &mut Assets<StandardMaterial>,
    path: &'static str,
    tint: Color,
    uv_transform: Affine2,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color: tint,
        base_color_texture: Some(asset_server.load(path)),
        perceptual_roughness: 1.0,
        unlit: true,
        fog_enabled: false,
        cull_mode: None,
        uv_transform,
        ..default()
    })
}

pub(super) fn legacy_sky_uv_transform(zone: LegacySkyZone, face_index: usize) -> Affine2 {
    if zone == LegacySkyZone::Future {
        if face_index < 4 {
            // `Skybox_FreeZone.mat` serializes scale=(1, 2) and pivot=(0, 1)
            // for all four 512x256 side textures. With the source textures'
            // Clamp addressing, the legacy sky renderer traverses the full
            // texture over half a face and holds its cyan edge after that.
            return Affine2::from_scale(Vec2::new(1.0, 2.0));
        }
        if face_index == 4 {
            // The authored `_UpTex` rotation is 270 degrees. Bevy's inward
            // rectangle basis already accounts for the legacy shader's
            // 180-degree top-face basis, leaving a 90-degree UV rotation here.
            // Rotating about the texture center makes all four source borders
            // meet their matching side-texture borders.
            let center = Vec2::splat(0.5);
            return Affine2::from_translation(center)
                * Affine2::from_angle(std::f32::consts::FRAC_PI_2)
                * Affine2::from_translation(-center);
        }
    }
    Affine2::IDENTITY
}

pub(super) fn legacy_sky_shader_tint(fog_color: [f32; 4], light_color: [f32; 4]) -> Color {
    // `cnPlayerCamera.Update` overwrites the serialized material `_Tint` every
    // frame with `DefaultAmbience`'s post-processed fog color. Unity's built-in
    // six-sided sky shader evaluates `tex2D(_Tex) * _Tint * 2`. The enabled
    // `GlowEffect` subsequently applies `filterColor` to the completed 3D
    // frame. Keep that camera-owned operation out of the sky material itself
    // so the native post-process does not apply it twice.
    let _ = light_color;
    Color::srgba(
        fog_color[0] * 2.0,
        fog_color[1] * 2.0,
        fog_color[2] * 2.0,
        1.0,
    )
}

pub(super) fn legacy_skybox_camera_transform(world_camera: &Transform) -> Transform {
    // Unity removes the translation from the view matrix for a camera-owned
    // Skybox. Rendering the background camera at the origin is the direct
    // native equivalent and prevents any terrain-scale parallax or one-frame
    // follow lag while preserving the gameplay camera's view rotation.
    Transform::from_rotation(world_camera.rotation)
}

pub(super) fn legacy_sky_face_transforms() -> [Transform; 6] {
    let half = LEGACY_SKYBOX_SIZE * 0.5;
    [
        // Unity front/back remain native +Z/-Z under the H reflection.
        Transform::from_xyz(0.0, 0.0, half)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::PI)),
        Transform::from_xyz(0.0, 0.0, -half),
        // Unity left/right exchange native X sides under H.
        Transform::from_xyz(half, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2)),
        Transform::from_xyz(-half, 0.0, 0.0)
            .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
        Transform::from_xyz(0.0, half, 0.0)
            .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
        Transform::from_xyz(0.0, -half, 0.0)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ]
}

pub(super) fn legacy_skybox_projection(world_projection: &Projection) -> Projection {
    let mut projection = world_projection.clone();
    if let Projection::Perspective(perspective) = &mut projection {
        perspective.far = LEGACY_SKYBOX_CAMERA_FAR;
    }
    projection
}

/// Restores the separate camera-owned `FusionStar` prefab. The source script
/// follows camera translation while forcing the root to world rotation zero.
/// The native background pass has translation stripped, so an origin-rooted
/// star is the equivalent relative transform.
pub(super) fn legacy_fusion_star_plane_transform(scale: f32) -> Transform {
    let source_distance = LEGACY_FUSION_STAR_POSITION_UNITY.length();
    let background_ratio = LEGACY_FUSION_STAR_BACKGROUND_DISTANCE / source_distance;
    let source_plane_rotation = unity_to_native_rotation(Quat::from_xyzw(
        -3.090_862e-8,
        0.707_106_8,
        -0.707_106_8,
        -3.090_862e-8,
    ));
    let rectangle_to_unity_plane = Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
    Transform::from_translation(unity_to_native_vector(
        LEGACY_FUSION_STAR_POSITION_UNITY * background_ratio,
    ))
    .with_rotation(source_plane_rotation * rectangle_to_unity_plane)
    // Moving the source plane outward without this matching scale would make
    // the celestial disc shrink. The ratio preserves its exact angular size.
    .with_scale(Vec3::splat(scale * background_ratio))
}

pub(super) fn ensure_and_update_legacy_fusion_star(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cameras: Query<
        &Transform,
        (
            With<Camera3d>,
            With<LegacyOrbitCamera>,
            Without<LegacyFusionStarRoot>,
            Without<LegacyFusionStarPlane>,
        ),
    >,
    players: Query<
        &Transform,
        (
            With<LocalPlayer>,
            Without<LegacyOrbitCamera>,
            Without<LegacyFusionStarRoot>,
            Without<LegacyFusionStarPlane>,
        ),
    >,
    mut roots: Query<&mut Transform, With<LegacyFusionStarRoot>>,
    mut planes: Query<&mut Transform, (With<LegacyFusionStarPlane>, Without<LegacyFusionStarRoot>)>,
) {
    let Ok(_camera) = cameras.single() else {
        return;
    };
    let Ok(player) = players.single() else {
        return;
    };
    let target_scale = match legacy_sky_zone(player.translation) {
        LegacySkyZone::Future => 1.0,
        LegacySkyZone::Past => 0.5,
    };

    if let Some(mut root) = roots.iter_mut().next() {
        *root = Transform::IDENTITY;
        let target_transform = legacy_fusion_star_plane_transform(target_scale);
        for mut plane in &mut planes {
            plane.translation = target_transform.translation;
            plane.rotation = target_transform.rotation;
            let current = plane.scale.x;
            let target = target_transform.scale.x;
            let next = current * (1.0 - time.delta_secs()) + target * time.delta_secs();
            plane.scale = Vec3::splat(next);
        }
        return;
    }

    let mesh = meshes.add(Rectangle::new(
        LEGACY_FUSION_STAR_SIZE,
        LEGACY_FUSION_STAR_SIZE,
    ));
    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(asset_server.load(LEGACY_FUSION_STAR_TEXTURE)),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        fog_enabled: false,
        cull_mode: None,
        ..default()
    });
    commands
        .spawn((
            Name::new("Retrobution FusionStar"),
            WorldSliceEntity,
            LegacyFusionStarRoot,
            Transform::IDENTITY,
            Visibility::Inherited,
        ))
        .with_child((
            Name::new("Retrobution FusionStar Plane"),
            LegacyFusionStarPlane,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            legacy_fusion_star_plane_transform(target_scale),
            RenderLayers::layer(GAMEPLAY_SKY_RENDER_LAYER),
        ));
}

pub(super) fn ensure_and_update_legacy_skybox(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    catalog: Res<NativeWorldCatalog>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    cameras: Query<
        (&Transform, &Projection, &LegacyOrbitCamera),
        (
            With<Camera3d>,
            Without<LocalPlayer>,
            Without<LegacySkyboxRoot>,
            Without<LegacySkyboxCamera>,
        ),
    >,
    mut sky_cameras: Query<
        (&mut Transform, &mut Projection),
        (
            With<Camera3d>,
            With<LegacySkyboxCamera>,
            Without<LocalPlayer>,
            Without<LegacyOrbitCamera>,
            Without<LegacySkyboxRoot>,
        ),
    >,
    players: Query<
        &Transform,
        (
            With<LocalPlayer>,
            Without<LegacyOrbitCamera>,
            Without<LegacySkyboxRoot>,
            Without<LegacySkyboxCamera>,
        ),
    >,
    mut roots: Query<
        (&mut Transform, &mut LegacySkyboxRoot),
        (
            Without<LocalPlayer>,
            Without<LegacyOrbitCamera>,
            Without<LegacySkyboxCamera>,
        ),
    >,
    mut faces: Query<(&LegacySkyboxFace, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    let Ok((camera_transform, world_projection, orbit)) = cameras.single() else {
        return;
    };
    let background_transform = legacy_skybox_camera_transform(camera_transform);
    if let Ok((mut sky_transform, mut sky_projection)) = sky_cameras.single_mut() {
        sky_transform.set_if_neq(background_transform);
        *sky_projection = legacy_skybox_projection(world_projection);
    } else if sky_cameras.is_empty() {
        commands.spawn((
            Name::new("Retrobution skybox background camera"),
            WorldSliceEntity,
            LegacySkyboxCamera,
            Camera3d::default(),
            Msaa::Off,
            Camera {
                order: LEGACY_SKYBOX_CAMERA_ORDER,
                clear_color: ClearColorConfig::Custom(Color::BLACK),
                ..default()
            },
            legacy_skybox_projection(world_projection),
            background_transform,
            RenderLayers::layer(GAMEPLAY_SKY_RENDER_LAYER),
        ));
    }
    let position = players
        .get(orbit.target)
        .map(|transform| transform.translation)
        .unwrap_or(camera_transform.translation);
    let zone = legacy_sky_zone(position);
    let ambience = catalog.applied_ambience(camera_transform.translation, true);
    let tint = legacy_sky_shader_tint(ambience.fog_color, ambience.light_color);

    if let Some((mut root_transform, mut root)) = roots.iter_mut().next() {
        root_transform.set_if_neq(Transform::IDENTITY);
        if root.zone != zone {
            root.zone = zone;
            for (face, mut material) in &mut faces {
                material.0 = match zone {
                    LegacySkyZone::Past => face.past.clone(),
                    LegacySkyZone::Future => face.future.clone(),
                };
            }
        }
        for (_, material_handle) in &faces {
            if materials
                .get(&material_handle.0)
                .is_some_and(|material| material.base_color != tint)
            {
                materials.get_mut(&material_handle.0).unwrap().base_color = tint;
            }
        }
        return;
    }

    let mesh = meshes.add(Rectangle::new(LEGACY_SKYBOX_SIZE, LEGACY_SKYBOX_SIZE));
    let transforms = legacy_sky_face_transforms();
    let mut root = commands.spawn((
        Name::new("Retrobution Past/Future skybox"),
        WorldSliceEntity,
        LegacySkyboxRoot { zone },
        Transform::IDENTITY,
        Visibility::Inherited,
    ));
    root.with_children(|parent| {
        for face_index in 0..6 {
            let past = legacy_sky_material(
                &asset_server,
                &mut materials,
                LEGACY_PAST_SKY_TEXTURES[face_index],
                tint,
                legacy_sky_uv_transform(LegacySkyZone::Past, face_index),
            );
            let future = legacy_sky_material(
                &asset_server,
                &mut materials,
                LEGACY_FUTURE_SKY_TEXTURES[face_index],
                tint,
                legacy_sky_uv_transform(LegacySkyZone::Future, face_index),
            );
            let selected = match zone {
                LegacySkyZone::Past => past.clone(),
                LegacySkyZone::Future => future.clone(),
            };
            parent.spawn((
                Name::new(format!("Legacy sky face {face_index}")),
                LegacySkyboxFace { past, future },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(selected),
                transforms[face_index],
                RenderLayers::layer(GAMEPLAY_SKY_RENDER_LAYER),
            ));
        }
    });
}
