//! A small isolated 3D stage inside the type picker, using the native NPC spawners.
use super::*;
use bevy::camera::RenderTarget;
use bevy::{camera::visibility::RenderLayers, render::render_resource::TextureFormat};

#[derive(Default, Resource)]
pub(super) struct Preview {
    root: Option<Entity>,
    current: Option<(usize, usize)>,
    camera: Option<Entity>,
}
#[derive(Component)]
pub(super) struct PickerCamera;
const LAYER: usize = 30;
pub(super) fn update(
    mut commands: Commands,
    mut e: ResMut<WorldEditor>,
    mut preview: ResMut<Preview>,
    catalog: Res<EditorCatalog>,
    server: Res<AssetServer>,
    mut rig: ResMut<NativePlayerRigAssetCache>,
    mut images: ResMut<Assets<Image>>,
    children: Query<&Children>,
    meshes: Query<
        (
            &GlobalTransform,
            Option<&bevy::camera::primitives::Aabb>,
            Option<&RenderLayers>,
        ),
        With<Mesh3d>,
    >,
    mut cameras: Query<(&mut Camera, &mut Transform), With<PickerCamera>>,
    mut primitives: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let active = e.type_picker.is_some();
    if active && e.type_image.is_none() {
        let target = images.add(Image::new_target_texture(
            320,
            320,
            TextureFormat::Rgba8UnormSrgb,
            None,
        ));
        preview.camera = Some(
            commands
                .spawn((
                    PickerCamera,
                    Camera3d::default(),
                    Camera {
                        order: -15,
                        is_active: true,
                        clear_color: ClearColorConfig::Custom(Color::srgb(0.02, 0.035, 0.05)),
                        ..default()
                    },
                    RenderTarget::from(target.clone()),
                    Msaa::Off,
                    Transform::from_xyz(3., 2., -5.).looking_at(Vec3::Y, Vec3::Y),
                    RenderLayers::layer(LAYER),
                ))
                .id(),
        );
        commands.spawn((
            DirectionalLight {
                illuminance: 8_000.,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_xyz(-3., 5., -3.).looking_at(Vec3::ZERO, Vec3::Y),
            RenderLayers::layer(LAYER),
        ));
        e.type_image = Some(target);
        e.revision += 1;
    }
    for (mut camera, _) in &mut cameras {
        camera.is_active = active;
    }
    let next = e
        .type_picker
        .as_ref()
        .and_then(|p| p.selected.map(|i| (p.kind, i)));
    if next != preview.current {
        if let Some(root) = preview.root.take() {
            commands.entity(root).despawn();
        }
        preview.current = next;
        if let Some(choice) = e
            .type_picker
            .as_ref()
            .and_then(|p| p.selected.and_then(|i| p.choices.get(i)))
        {
            let root = commands
                .spawn((
                    Name::new("Type picker preview"),
                    Transform::IDENTITY,
                    Visibility::Inherited,
                    RenderLayers::layer(LAYER),
                ))
                .id();
            preview.root = Some(root);
            if let Some(entry) = choice.entry().map(|i| &catalog.entries[i]) {
                if let (Some(definition), Some(hnpc)) = (&entry.hnpc_visual, &catalog.hnpc) {
                    match spawn_network_hnpc_visual_0104(
                        &mut commands,
                        &server,
                        &mut rig,
                        hnpc,
                        root,
                        definition,
                        e.revision,
                    ) {
                        Ok(visual) => {
                            commands.entity(root).insert(visual);
                        }
                        Err(error) => eprintln!("Type preview: {error}"),
                    }
                } else if let Some(definition) = &entry.npc_visual {
                    let visual = spawn_network_npc_visual_0104(
                        &mut commands,
                        &server,
                        root,
                        definition,
                        "editor type preview",
                    );
                    commands.entity(root).insert(visual);
                } else {
                    commands.spawn((
                        ChildOf(root),
                        Mesh3d(primitives.add(Cuboid::new(2., 2., 2.))),
                        MeshMaterial3d(materials.add(StandardMaterial {
                            base_color: Color::srgb(0.25, 0.76, 1.),
                            unlit: true,
                            ..default()
                        })),
                        Transform::from_xyz(0., 1., 0.),
                        RenderLayers::layer(LAYER),
                    ));
                }
            } else if let Some(path) = choice.model(&catalog).filter(|p| !p.is_empty()) {
                commands.entity(root).insert(WorldAssetRoot(
                    server.load(GltfAssetLabel::Scene(0).from_asset(path)),
                ));
            }
        }
        e.revision += 1;
    }
    let Some(root) = preview.root else {
        return;
    };
    let mut low = Vec3::splat(f32::INFINITY);
    let mut high = Vec3::splat(f32::NEG_INFINITY);
    for entity in children.iter_descendants(root) {
        if let Ok((transform, bounds, layers)) = meshes.get(entity) {
            if layers.is_none_or(|l| !l.intersects(&RenderLayers::layer(LAYER))) {
                commands.entity(entity).insert(RenderLayers::layer(LAYER));
            }
            let Some(bounds) = bounds else {
                continue;
            };
            let center = Vec3::from(bounds.center);
            let half = Vec3::from(bounds.half_extents);
            for x in [-1., 1.] {
                for y in [-1., 1.] {
                    for z in [-1., 1.] {
                        let at = transform.transform_point(center + half * Vec3::new(x, y, z));
                        low = low.min(at);
                        high = high.max(at);
                    }
                }
            }
        }
    }
    if low.is_finite() && high.is_finite() {
        let center = (low + high) * 0.5;
        let radius = (high - low).length().max(1.) * 0.5;
        for (_, mut transform) in &mut cameras {
            *transform = Transform::from_translation(
                center + Vec3::new(0.5, 0.25, -1.).normalize() * radius * 2.8,
            )
            .looking_at(center, Vec3::Y);
        }
    }
}
