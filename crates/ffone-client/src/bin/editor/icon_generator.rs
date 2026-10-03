//! Square native model renders and reversible icon authoring, separate from published routes.
use super::*;
use bevy::{
    asset::RenderAssetUsages,
    camera::{RenderTarget, visibility::RenderLayers},
    render::render_resource::TextureFormat,
};
use ffone_client::{
    legacy_model_material::{
        LegacyColorWriteMask, LegacyMaterialPassCompanion, LegacyModelMaterial,
        PendingLegacyModelMaterial,
    },
    player_preview::{
        NATIVE_PLAYER_PREVIEW_RENDER_LAYER, NativePlayerPreviewModel, NativePlayerPreviewStatus,
    },
};
use ffone_runtime_contracts::AvatarItemCategory;
use image::{DynamicImage, RgbaImage};
#[path = "icon_generator/composition.rs"]
mod composition;
#[path = "icon_generator/view.rs"]
mod view;
use composition::{Layers, compose};

fn readable_path(path: &std::path::Path) -> String {
    let text = path.to_string_lossy();
    text.strip_prefix("\\\\?\\").unwrap_or(&text).to_owned()
}
fn preview_image(output: RgbaImage) -> Image {
    let mut image = Image::from_dynamic(
        DynamicImage::ImageRgba8(output),
        true,
        RenderAssetUsages::default(),
    );
    image.sampler = bevy::image::ImageSampler::nearest();
    image
}

pub(super) struct IconGeneratorPlugin {
    root: PathBuf,
}
impl IconGeneratorPlugin {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
}
impl Plugin for IconGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(IconGenerator::new(self.root.clone()))
            .add_systems(
                Update,
                actions
                    .before(sync_equipment_look)
                    .before(update_orbit_camera),
            )
            .add_systems(
                Update,
                (
                    camera_input,
                    sync_camera,
                    bind_alpha,
                    hide_exposed_skin,
                    capture,
                    bind_chrome,
                    view::draw,
                )
                    .chain()
                    .after(update_orbit_camera)
                    .after(sync_equipment_camera),
            )
            .add_systems(
                PostUpdate,
                freeze_equipment
                    .after(bevy::app::AnimationSystems)
                    .before(apply_editor_t_pose),
            );
    }
}

#[derive(Resource)]
pub(super) struct IconGenerator {
    pub active: bool,
    pub(super) has_subject: bool,
    root: PathBuf,
    output: PathBuf,
    target: Handle<Image>,
    preview: Handle<Image>,
    identity: Option<(usize, bool)>,
    layers: Layers,
    source_badge: Option<RgbaImage>,
    source_color: Option<image::Rgba<u8>>,
    badges: Result<composition::BadgeLibrary, String>,
    pub(super) roll: f32,
    pub(super) fov: f32,
    pub(super) scale: u8,
    pub(super) level_text: String,
    pub(super) level_focused: bool,
    pub(super) level_edit: ffone_client::text_edit::TextEdit,
    pub(super) dirty: bool,
    busy: bool,
    frame: u32,
    ready_frames: u32,
    export: bool,
    both: Option<bool>,
    pub(super) status: String,
    saved_pose: Option<(EditorPoseMode, bool, bool)>,
    pub(super) revision: u64,
}
impl IconGenerator {
    pub(super) fn new(root: PathBuf) -> Self {
        let output = root
            .parent()
            .and_then(|p| p.parent())
            .unwrap_or(&root)
            .join("target/editor/icons");
        let badges = composition::BadgeLibrary::open(&root);
        Self {
            root,
            output,
            badges,
            active: false,
            has_subject: false,
            target: default(),
            preview: default(),
            identity: None,
            layers: Layers::default(),
            source_badge: None,
            source_color: None,
            roll: 0.,
            fov: 45.,
            scale: 3,
            level_text: "1".into(),
            level_focused: false,
            level_edit: default(),
            dirty: true,
            busy: false,
            frame: 0,
            ready_frames: 0,
            export: false,
            both: None,
            status: String::new(),
            saved_pose: None,
            revision: 0,
        }
    }
    fn invalidate(&mut self) {
        self.dirty = true;
        self.has_subject = false;
        self.revision += 1;
    }
    fn load_source(&mut self, entry: &EditorCatalogEntry) {
        self.source_badge = None;
        self.source_color = None;
        if entry.kind == CatalogKind::Nano {
            if let Some(path) = entry.icon_path.as_ref() {
                if let Ok(src) = image::open(self.root.join(path)) {
                    let mut color = *src.to_rgba8().get_pixel(0, 0);
                    // Native icon padding retains the affinity RGB with zero alpha.
                    color[3] = 255;
                    self.source_color = Some(color);
                }
                if let Some(file) = PathBuf::from(path).file_name().and_then(|s| s.to_str()) {
                    let ready = self
                        .root
                        .join("icons/entities/nanos/ready")
                        .join(file.replacen("nanoicon_", "nanoready_", 1));
                    if let Ok(src) = image::open(ready) {
                        self.source_badge = Some(composition::extract_badge(&src.to_rgba8()));
                    }
                }
            }
        }
    }
}
#[derive(Component)]
struct IconCamera;
#[derive(Component)]
struct AlphaBound;
#[derive(Component)]
pub(super) struct EditorChrome;

fn bind_chrome(icons: Res<IconGenerator>, mut nodes: Query<&mut Node, With<EditorChrome>>) {
    for mut node in &mut nodes {
        node.display = if icons.active {
            Display::None
        } else {
            Display::Flex
        };
    }
}
#[derive(Component, Clone, Copy)]
pub(super) enum Action {
    Close,
    Front,
    Back,
    Left,
    Right,
    Top,
    Fit,
    Restore,
    Scale(u8),
    Roll(f32),
    Fov(f32),
    Background,
    Outline,
    Locked,
    Badge,
    Affinity,
    BadgeKind,
    Level(i32),
    LevelInput,
    Export,
    Both,
    Gender(bool),
    Folder,
}

fn actions(
    mut commands: Commands,
    launch: Query<(&Interaction, &EditorAction), Changed<Interaction>>,
    buttons: Query<(&Interaction, &Action), Changed<Interaction>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut icons: ResMut<IconGenerator>,
    mut state: ResMut<EditorState>,
    catalog: Res<EditorCatalog>,
    mut images: ResMut<Assets<Image>>,
    mut orbit: ResMut<OrbitCamera>,
    mut preview: ResMut<ModelPreview>,
    mut keyboard: MessageReader<KeyboardInput>,
) {
    let open = launch
        .iter()
        .any(|(i, a)| *i == Interaction::Pressed && *a == EditorAction::IconGenerator)
        || (!icons.active
            && env::args().any(|a| a == "--icon-generator")
            && icons.target == Handle::default());
    if open {
        icons.saved_pose = Some((state.pose_mode, state.paused, state.turntable));
        icons.active = true;
        icons.export = env::args().any(|a| a == "--icon-export");
        icons.layers.locked = env::args().any(|a| a == "--icon-locked");
        state.turntable = false;
        state.paused = true;
        if state.kind == CatalogKind::Equipment {
            state.activate_t_pose();
        }
        icons.identity = None;
        if icons.target == Handle::default() {
            icons.target = images.add(Image::new_target_texture(
                512,
                512,
                TextureFormat::Rgba8UnormSrgb,
                None,
            ));
            icons.preview = images.add(Image::from_dynamic(
                DynamicImage::ImageRgba8(RgbaImage::new(128, 128)),
                true,
                RenderAssetUsages::default(),
            ));
            commands.spawn((
                IconCamera,
                Camera3d::default(),
                Msaa::Off,
                Camera {
                    order: -20,
                    is_active: false,
                    clear_color: ClearColorConfig::Custom(Color::NONE),
                    ..default()
                },
                RenderTarget::from(icons.target.clone()),
                Transform::default(),
                RenderLayers::layer(0),
            ));
        }
        icons.invalidate();
    }
    if !icons.active {
        keyboard.clear();
        return;
    }
    let was_focused = icons.level_focused;
    if icons.level_focused {
        let (control, shift) = ffone_client::text_edit::modifiers(Some(&keys));
        for event in keyboard.read().filter(|e| e.state == ButtonState::Pressed) {
            if matches!(
                event.key_code,
                KeyCode::Escape | KeyCode::Enter | KeyCode::NumpadEnter
            ) {
                icons.level_focused = false;
                icons.level_text = icons.layers.level.to_string();
                continue;
            }
            let IconGenerator {
                level_text,
                level_edit,
                ..
            } = &mut *icons;
            if control && event.key_code == KeyCode::KeyV {
                if let Ok(text) = arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                    let digits = text
                        .chars()
                        .filter(char::is_ascii_digit)
                        .collect::<String>();
                    level_edit.insert(level_text, &digits, 2, false);
                }
            } else if control && event.key_code == KeyCode::KeyC {
                let selected = level_text
                    .chars()
                    .skip(level_edit.range().start)
                    .take(level_edit.range().len())
                    .collect::<String>();
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(selected);
                }
            } else if !level_edit.key(level_text, event.key_code, control, shift) && !control {
                if let Some(text) = event.text.as_ref() {
                    let digits = text
                        .chars()
                        .filter(char::is_ascii_digit)
                        .collect::<String>();
                    level_edit.insert(level_text, &digits, 2, false);
                }
            }
            if let Ok(level) = icons.level_text.parse::<i32>() {
                icons.layers.level = level.clamp(0, 99);
            }
            icons.invalidate();
        }
    } else {
        keyboard.clear();
    }
    let identity = (state.selected, state.equipment_female);
    if icons.identity != Some(identity) {
        icons.identity = Some(identity);
        icons.load_source(&catalog.entries[state.selected]);
        icons.ready_frames = 0;
        orbit.fit_requested = true;
        icons.roll = 0.;
        icons.fov = 45.;
        icons.layers.nano = state.kind == CatalogKind::Nano;
        icons.invalidate();
    }
    let mut todo = buttons
        .iter()
        .filter(|(i, _)| **i == Interaction::Pressed && mouse.just_pressed(MouseButton::Left))
        .map(|(_, a)| *a)
        .collect::<Vec<_>>();
    if keys.just_pressed(KeyCode::Escape) && !was_focused {
        todo.push(Action::Close)
    }
    for action in todo {
        if icons.busy || icons.both.is_some() {
            continue;
        }
        if !matches!(action, Action::LevelInput) {
            icons.level_focused = false;
            icons.level_text = icons.layers.level.to_string();
        }
        match action {
            Action::Close => {
                icons.active = false;
                icons.export = false;
                icons.identity = None;
                if let Some((pose, paused, turntable)) = icons.saved_pose.take() {
                    state.pose_mode = pose;
                    state.paused = paused;
                    state.turntable = turntable;
                    state.bump_playback_revision();
                }
                preview.current_index = None;
            }
            Action::Fit => orbit.fit_requested = true,
            Action::Scale(scale) => icons.scale = scale,
            Action::Restore => {
                let entry = &catalog.entries[state.selected];
                match restore_framing(
                    &icons.output,
                    &entry.semantic_id,
                    state.equipment_female,
                    &mut orbit,
                ) {
                    Ok((roll, fov, layers)) => {
                        icons.roll = roll;
                        icons.fov = fov;
                        icons.layers = layers;
                    }
                    Err(error) => icons.status = error,
                }
            }
            Action::Front => {
                orbit.yaw = 0.;
                orbit.pitch = 0.;
                icons.roll = 0.;
            }
            Action::Back => {
                orbit.yaw = std::f32::consts::PI;
                orbit.pitch = 0.;
                icons.roll = 0.;
            }
            Action::Left => {
                orbit.yaw = std::f32::consts::FRAC_PI_2;
                orbit.pitch = 0.;
                icons.roll = 0.;
            }
            Action::Right => {
                orbit.yaw = -std::f32::consts::FRAC_PI_2;
                orbit.pitch = 0.;
                icons.roll = 0.;
            }
            Action::Top => {
                orbit.pitch = 1.56;
                icons.roll = 0.;
            }
            Action::Roll(delta) => icons.roll += delta,
            Action::Fov(delta) => icons.fov = (icons.fov + delta).clamp(5., 100.),
            Action::Background => icons.layers.background = !icons.layers.background,
            Action::Outline => icons.layers.outline = !icons.layers.outline,
            Action::Locked => icons.layers.locked = !icons.layers.locked,
            Action::Badge => icons.layers.badge = !icons.layers.badge,
            Action::Affinity => icons.layers.affinity = !icons.layers.affinity,
            Action::BadgeKind => icons.layers.badge_kind = (icons.layers.badge_kind + 1) % 4,
            Action::Level(delta) => {
                icons.layers.level = (icons.layers.level + delta).clamp(0, 99);
                icons.level_text = icons.layers.level.to_string();
            }
            Action::LevelInput => {
                icons.level_text = icons.layers.level.to_string();
                icons.level_focused = true;
                let len = icons.level_text.len();
                icons.level_edit.cursor = len;
                icons.level_edit.anchor = 0;
            }
            Action::Gender(female) => state.equipment_female = female,
            Action::Export => icons.export = true,
            Action::Both => {
                icons.both = Some(state.equipment_female);
                state.equipment_female = false;
                icons.export = true;
            }
            Action::Folder => {
                let _ = fs::create_dir_all(&icons.output);
                let _ = std::process::Command::new("explorer.exe")
                    .arg(readable_path(&icons.output))
                    .spawn();
            }
        }
        icons.invalidate();
    }
}

fn camera_input(
    mut icons: ResMut<IconGenerator>,
    mut orbit: ResMut<OrbitCamera>,
    mut motions: MessageReader<MouseMotion>,
    mut wheels: MessageReader<MouseWheel>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    canvas: Query<&RelativeCursorPosition, With<view::Canvas>>,
) {
    let over = icons.active
        && icons.both.is_none()
        && canvas.iter().any(RelativeCursorPosition::cursor_over);
    let mut changed = false;
    for motion in motions.read() {
        if !over {
            continue;
        }
        if mouse.pressed(MouseButton::Left) {
            if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                icons.roll -= motion.delta.x * 0.3;
            } else {
                orbit.yaw -= motion.delta.x * 0.008;
                orbit.pitch = (orbit.pitch + motion.delta.y * 0.008).clamp(-1.56, 1.56);
            }
            changed = true;
        } else if mouse.pressed(MouseButton::Right) || mouse.pressed(MouseButton::Middle) {
            let rotation = camera_transform(&orbit, icons.roll).rotation;
            let scale = 2. * orbit.distance * (icons.fov.to_radians() * 0.5).tan()
                / (128. * f32::from(icons.scale));
            let shift = rotation * Vec3::new(-motion.delta.x, motion.delta.y, 0.) * scale;
            orbit.target_xz += Vec2::new(shift.x, shift.z);
            orbit.target_y += shift.y;
            changed = true;
        }
    }
    for w in wheels.read() {
        if over {
            let amount = if w.unit == MouseScrollUnit::Line {
                w.y
            } else {
                w.y / 40.
            };
            if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                icons.fov = (icons.fov - amount).clamp(5., 100.);
            } else {
                orbit.distance = (orbit.distance * (-amount * 0.08).exp()).clamp(0.02, 250.);
            }
            changed = true;
        }
    }
    if changed {
        orbit.fit_requested = false;
        icons.invalidate();
    }
}
fn camera_transform(orbit: &OrbitCamera, roll: f32) -> Transform {
    let target = Vec3::new(orbit.target_xz.x, orbit.target_y, orbit.target_xz.y);
    let offset = Vec3::new(
        orbit.yaw.sin() * orbit.pitch.cos(),
        orbit.pitch.sin(),
        -orbit.yaw.cos() * orbit.pitch.cos(),
    ) * orbit.distance;
    let mut transform = Transform::from_translation(target + offset).looking_at(target, Vec3::Y);
    transform.rotation *= Quat::from_rotation_z(roll.to_radians());
    transform
}

fn restore_framing(
    folder: &std::path::Path,
    semantic: &str,
    female: bool,
    orbit: &mut OrbitCamera,
) -> Result<(f32, f32, Layers), String> {
    let files = fs::read_dir(folder).map_err(|e| e.to_string())?;
    let mut candidates = files
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .collect::<Vec<_>>();
    candidates
        .sort_by_key(|e| std::cmp::Reverse(e.metadata().ok().and_then(|m| m.modified().ok())));
    for entry in candidates {
        let Ok(bytes) = fs::read(entry.path()) else {
            continue;
        };
        let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
            continue;
        };
        if value["schema"] != "ffone.editor-icon.v1"
            || value["semantic_id"] != semantic
            || value["female"] != female
        {
            continue;
        }
        let number = |key: &str| -> Result<f32, String> {
            value[key]
                .as_f64()
                .filter(|v| v.is_finite() && v.abs() < 1e6)
                .map(|v| v as f32)
                .ok_or_else(|| format!("Invalid framing field: {key}"))
        };
        let target = value["target"]
            .as_array()
            .filter(|a| a.len() == 3)
            .ok_or("Invalid camera target")?;
        let target = target
            .iter()
            .map(|v| {
                v.as_f64()
                    .filter(|v| v.is_finite() && v.abs() < 1e6)
                    .map(|v| v as f32)
                    .ok_or("Invalid camera target")
            })
            .collect::<Result<Vec<_>, _>>()?;
        let yaw = number("yaw")?;
        let pitch = number("pitch")?.clamp(-1.56, 1.56);
        let distance = number("distance")?.clamp(0.02, 250.);
        let roll = number("roll")?;
        let fov = number("fov")?.clamp(5., 100.);
        let layers =
            serde_json::from_value::<Layers>(value["layers"].clone()).map_err(|e| e.to_string())?;
        if layers.badge_kind > 3 || !(0..=99).contains(&layers.level) {
            return Err("Invalid icon layers".into());
        }
        orbit.yaw = yaw;
        orbit.pitch = pitch;
        orbit.distance = distance;
        orbit.target_xz = Vec2::new(target[0], target[2]);
        orbit.target_y = target[1];
        orbit.fit_requested = false;
        return Ok((roll, fov, layers));
    }
    Err("No exported framing for this model and gender".into())
}
fn sync_camera(
    icons: Res<IconGenerator>,
    orbit: Res<OrbitCamera>,
    state: Res<EditorState>,
    mut camera: Query<
        (
            &mut Camera,
            &mut Transform,
            &mut Projection,
            &mut RenderLayers,
        ),
        With<IconCamera>,
    >,
) {
    for (mut camera, mut transform, mut projection, mut layers) in &mut camera {
        camera.is_active = icons.active;
        *transform = camera_transform(&orbit, icons.roll);
        *layers = RenderLayers::layer(if state.kind == CatalogKind::Equipment {
            NATIVE_PLAYER_PREVIEW_RENDER_LAYER
        } else {
            0
        });
        *projection = Projection::Perspective(PerspectiveProjection {
            fov: icons.fov.to_radians(),
            near: 0.005,
            far: 500.,
            aspect_ratio: 1.,
            ..default()
        });
    }
}
fn bind_alpha(
    mut commands: Commands,
    icons: Res<IconGenerator>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    mut surfaces: Query<(Entity, &mut MeshMaterial3d<LegacyModelMaterial>), Without<AlphaBound>>,
) {
    if !icons.active {
        return;
    }
    for (entity, mut handle) in &mut surfaces {
        let Some(material) = materials.get(&handle.0) else {
            continue;
        };
        if material.render_mode.color_write == LegacyColorWriteMask::Rgb {
            let mut copy = material.clone();
            copy.render_mode.color_write = LegacyColorWriteMask::Rgba;
            handle.0 = materials.add(copy);
        }
        commands.entity(entity).insert(AlphaBound);
    }
}
fn freeze_equipment(
    icons: Res<IconGenerator>,
    state: Res<EditorState>,
    mut players: Query<&mut AnimationPlayer>,
) {
    if icons.active && state.kind == CatalogKind::Equipment {
        for mut player in &mut players {
            player.pause_all();
        }
    }
}

fn hide_exposed_skin(
    icons: Res<IconGenerator>,
    state: Res<EditorState>,
    library: Res<EquipmentLibrary>,
    metadata: Query<&PendingLegacyModelMaterial>,
    mut surfaces: Query<
        (
            &mut Visibility,
            &RenderLayers,
            Option<&PendingLegacyModelMaterial>,
            Option<&LegacyMaterialPassCompanion>,
        ),
        With<Mesh3d>,
    >,
) {
    if !icons.active || state.kind != CatalogKind::Equipment {
        return;
    }
    let clothing = library.entries.get(&state.selected).is_some_and(|item| {
        matches!(
            item.category,
            AvatarItemCategory::Shirt | AvatarItemCategory::Pants | AvatarItemCategory::Shoes
        )
    });
    if !clothing {
        return;
    }
    for (mut visibility, layers, own, companion) in &mut surfaces {
        if !layers.intersects(&RenderLayers::layer(NATIVE_PLAYER_PREVIEW_RENDER_LAYER)) {
            continue;
        }
        let pending =
            own.or_else(|| companion.and_then(|pass| metadata.get(pass.source_mesh_entity).ok()));
        // Same authored material role used by the production SetBody binder.
        // Skin submeshes (hands, exposed legs) are not part of the item icon.
        if pending.is_some_and(|p| p.true_name.contains("sub")) {
            *visibility = Visibility::Hidden;
        }
    }
}

fn capture(
    mut commands: Commands,
    mut icons: ResMut<IconGenerator>,
    mut state: ResMut<EditorState>,
    runtime: Res<EditorRuntimeStatus>,
    player: Res<NativePlayerPreviewModel>,
    catalog: Res<EditorCatalog>,
    orbit: Res<OrbitCamera>,
    library: Res<EquipmentLibrary>,
) {
    if !icons.active || icons.busy {
        return;
    }
    icons.frame += 1;
    let error = if state.kind == CatalogKind::Equipment {
        library.capture_error(&player)
    } else {
        runtime.error.clone()
    };
    if let Some(error) = error {
        if icons.status != error {
            icons.status = error;
            icons.revision += 1;
        }
        icons.export = false;
        if let Some(original) = icons.both.take() {
            state.equipment_female = original;
        }
        return;
    }
    let ready = if state.kind == CatalogKind::Equipment {
        matches!(
            player.status,
            NativePlayerPreviewStatus::ReadyAnimated { .. }
        )
    } else {
        runtime.ready
    };
    if !ready {
        icons.ready_frames = 0;
        return;
    }
    icons.ready_frames += 1;
    if icons.ready_frames < 8 || (!icons.export && (!icons.dirty || icons.frame % 6 != 0)) {
        return;
    }
    let entry = &catalog.entries[state.selected];
    let layers = icons.layers.clone();
    if layers.nano && layers.badge && layers.badge_kind > 0 {
        if let Err(error) = icons.badges.as_ref() {
            let error = error.clone();
            icons.status = error;
            icons.export = false;
            return;
        }
    }
    let badge = if layers.badge_kind == 0 {
        icons.source_badge.clone()
    } else {
        icons
            .badges
            .as_ref()
            .ok()
            .map(|lib| lib.badge(layers.badge_kind, layers.level))
    };
    let color = icons.source_color;
    let affinity = if layers.nano && layers.affinity {
        entry
            .style
            .and_then(|style| ["a", "b", "c"].get(style as usize))
            .and_then(|letter| {
                image::open(
                    icons
                        .root
                        .join(format!("ui/en/gameplay/nano/affinity/{letter}_icon.png")),
                )
                .ok()
            })
            .map(|i| {
                image::imageops::resize(
                    &i.to_rgba8(),
                    32,
                    32,
                    image::imageops::FilterType::Lanczos3,
                )
            })
    } else {
        None
    };
    let revision = icons.revision;
    let export = icons.export;
    let female = state.equipment_female;
    let identity = (state.selected, female);
    let kind = state.kind;
    let name = entry.semantic_id.replace(['/', '\\', ':'], "_");
    let suffix = if kind == CatalogKind::Equipment {
        if female { "female" } else { "male" }
    } else if layers.locked {
        "locked"
    } else {
        "color"
    };
    let folder = icons.output.clone();
    let file = format!("{name}_{suffix}");
    let framing = serde_json::json!({"schema":"ffone.editor-icon.v1","semantic_id":entry.semantic_id,"female":female,"size":[128,128],"yaw":orbit.yaw,"pitch":orbit.pitch,"distance":orbit.distance,"target":[orbit.target_xz.x,orbit.target_y,orbit.target_xz.y],"roll":icons.roll,"fov":icons.fov,"layers":layers});
    icons.busy = true;
    icons.dirty = false;
    commands
        .spawn(Screenshot::image(icons.target.clone()))
        .observe(
            move |event: On<ScreenshotCaptured>,
                  mut icons: ResMut<IconGenerator>,
                  mut images: ResMut<Assets<Image>>,
                  mut state: ResMut<EditorState>| {
                icons.busy = false;
                let result = (|| -> Result<RgbaImage, String> {
                    let src = event
                        .image
                        .clone()
                        .try_into_dynamic()
                        .map_err(|e| e.to_string())?
                        .to_rgba8();
                    if !src.pixels().any(|p| p[3] > 16) {
                        return Err("empty-render".into());
                    }
                    let mut output = compose(&src, &layers, color, badge.as_ref());
                    if let Some(symbol) = affinity.as_ref() {
                        image::imageops::overlay(&mut output, symbol, 96, 0);
                    }
                    Ok(output)
                })();
                match result {
                    Ok(output) => {
                        if icons.identity == Some(identity) {
                            if revision == icons.revision {
                                icons.has_subject = true;
                            }
                            if let Some(mut img) = images.get_mut(&icons.preview) {
                                *img = preview_image(output.clone());
                            }
                        }
                        if export {
                            let saved = (|| -> Result<PathBuf, String> {
                                fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
                                let path = composition::unused_path(&folder, &file);
                                output.save(&path).map_err(|e| e.to_string())?;
                                fs::write(
                                    path.with_extension("json"),
                                    serde_json::to_vec_pretty(&framing).unwrap(),
                                )
                                .map_err(|e| e.to_string())?;
                                Ok(path)
                            })();
                            match saved {
                                Ok(path) => icons.status = path.to_string_lossy().into_owned(),
                                Err(e) => icons.status = e,
                            }
                            icons.export = false;
                            if let Some(original) = icons.both {
                                if !female {
                                    state.equipment_female = true;
                                    icons.export = true;
                                } else {
                                    state.equipment_female = original;
                                    icons.both = None;
                                }
                            }
                            icons.invalidate();
                        }
                    }
                    Err(e) => {
                        if e == "empty-render" {
                            if icons.identity == Some(identity) {
                                if let Some(mut image) = images.get_mut(&icons.preview) {
                                    *image = preview_image(compose(
                                        &RgbaImage::new(128, 128),
                                        &layers,
                                        color,
                                        badge.as_ref(),
                                    ));
                                }
                            }
                            icons.dirty = true;
                            icons.ready_frames = 0;
                            return;
                        }
                        icons.status = e;
                        icons.export = false;
                        icons.both = None;
                        icons.invalidate();
                    }
                }
            },
        );
}
