//! GPU acceptance harness for both native character-creation UI screens.
//!
//! The harness consumes only semantic assets below `assets/game`, captures the
//! Name screen first, then switches the same native UI to Appearance. A capture
//! is accepted only after CPU asset readiness, UI layout, a GPU-upload grace
//! period, and a visible-pixel check. Starter-item icons are resolved through
//! the same manifest-verified native creation data as the production client.

use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    mesh::skinning::SkinnedMesh,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    time::TimeUpdateStrategy,
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    character_creation_data::CharacterCreationData,
    character_creation_ui::{
        AppearanceField, CHARACTER_CREATION_DISPLAY_FONT_PATH, CHARACTER_CREATION_FONT_PATH,
        CHARACTER_CREATION_IMAGE_SPECS, CHARACTER_CREATION_SHARED_IMAGE_SPECS,
        CharacterCreationCapability, CharacterCreationPreviewStatus, CharacterCreationScreen,
        CharacterCreationUiModel, CharacterGender, CharacterNameLists, CharacterNameMode,
        NativeCharacterCreationRoot, NativeCharacterCreationUiPlugin,
    },
    legacy_model_material::{
        LegacyMaterialPassCompanion, LegacyMaterialRendererOrder, LegacyMaterialSortOrderApplied,
        LegacyModelMaterial, LegacyModelMaterialPlugin, PendingLegacyModelMaterial,
    },
    localization::{Localization, LocalizationPlugin},
    player_preview::{
        NativePlayerPreviewModel, NativePlayerPreviewPlugin, NativePlayerPreviewSet,
        NativePlayerPreviewStage, NativePlayerPreviewStatus,
    },
    player_shared_rig::{
        NativePlayerRigCatalog, NativePlayerRigSkinBound, NativePlayerSharedRigPlugin,
    },
};

const WARMUP_FRAMES: u32 = 120;
const RETRY_FRAMES: u32 = 120;
const ANIMATION_FRAME_DURATION: Duration = Duration::from_nanos(16_666_667);
const APPEARANCE_CAPTURE_SEPARATION_FRAMES: u32 = 24;
const MIN_APPEARANCE_ANIMATION_SEEK_DELTA: f32 = 0.20;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(2);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(30);
const MIN_VISIBLE_PIXEL_RATIO: f32 = 0.65;
const FACE_CHEEK_CENTERS: [(u32, u32); 4] = [(335, 165), (342, 174), (366, 174), (369, 165)];
const FACE_PATCH_RADIUS: u32 = 3;
const FACE_NECK_BOUNDS: (u32, u32, u32, u32) = (346, 357, 189, 197);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CaptureStage {
    Name,
    Appearance,
}

impl CaptureStage {
    const fn label(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Appearance => "appearance",
        }
    }
}

#[derive(Resource)]
struct PreviewConfig {
    name_output: PathBuf,
    appearance_output: PathBuf,
    appearance_capture_count: u32,
    images: Vec<Handle<Image>>,
    fonts: Vec<Handle<Font>>,
}

#[derive(Resource)]
struct PreviewCharacterData(CharacterCreationData);

/// Optional regression lane: change the face on the existing animated preview.
#[derive(Resource)]
struct FaceCycle(Option<CharacterGender>);

#[derive(Resource)]
struct PreviewState {
    frames: u32,
    assets_ready_frame: Option<u32>,
    assets_ready_at: Option<Instant>,
    appearance_started_frame: Option<u32>,
    appearance_started_at: Option<Instant>,
    last_rejected_frame: Option<u32>,
    capture_issued: bool,
    capture_target: Option<CaptureStage>,
    capture_animation_seek_time: Option<f32>,
    name_saved: bool,
    appearance_saved: bool,
    appearance_captures_saved: u32,
    last_appearance_saved_frame: Option<u32>,
    last_appearance_animation_seek_time: Option<f32>,
    started_at: Instant,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            frames: 0,
            assets_ready_frame: None,
            assets_ready_at: None,
            appearance_started_frame: None,
            appearance_started_at: None,
            last_rejected_frame: None,
            capture_issued: false,
            capture_target: None,
            capture_animation_seek_time: None,
            name_saved: false,
            appearance_saved: false,
            appearance_captures_saved: 0,
            last_appearance_saved_frame: None,
            last_appearance_animation_seek_time: None,
            started_at: Instant::now(),
        }
    }
}

fn main() {
    let mut args = std::env::args_os().skip(1);
    let name_output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/ffone-character-creation-name-current.png"));
    let appearance_output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/ffone-character-creation-appearance-current.png"));
    let appearance_capture_count = args
        .next()
        .map(|value| {
            value
                .to_string_lossy()
                .parse::<u32>()
                .expect("APPEARANCE_FRAMES must be an integer")
        })
        .unwrap_or(1);
    let face_cycle = args.next().map(|value| match value.to_str() {
        Some("boy") => CharacterGender::Boy,
        Some("girl") => CharacterGender::Girl,
        _ => panic!("FACE_CYCLE must be boy or girl"),
    });
    if args.next().is_some() {
        eprintln!(
            "usage: character_creation_gpu_preview [NAME.png] [APPEARANCE.png] \
             [APPEARANCE_FRAMES] [FACE_CYCLE: boy|girl]"
        );
        std::process::exit(2);
    }
    if !(1..=16).contains(&appearance_capture_count) {
        eprintln!("APPEARANCE_FRAMES must be between 1 and 16");
        std::process::exit(2);
    }
    for output in [&name_output, &appearance_output] {
        if output.extension().and_then(|value| value.to_str()) != Some("png") {
            eprintln!("output must end in .png: {}", output.display());
            std::process::exit(2);
        }
    }

    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let character_data =
        CharacterCreationData::open(&asset_root).expect("open native character-creation data");
    let rig_catalog =
        NativePlayerRigCatalog::open(&asset_root).expect("open native shared player-rig catalog");
    let (localization, language) = Localization::open(
        &asset_root,
        &env::var("FFONE_REVIEW_LANGUAGE").unwrap_or_else(|_| "en".into()),
    )
    .expect("open native localization");

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(TimeUpdateStrategy::ManualDuration(ANIMATION_FRAME_DURATION))
        .insert_resource(PreviewState::default())
        .insert_resource(PreviewOutputs {
            name_output,
            appearance_output,
            appearance_capture_count,
        })
        .insert_resource(PreviewCharacterData(character_data))
        .insert_resource(FaceCycle(face_cycle))
        .insert_resource(rig_catalog)
        .insert_resource(localization)
        .insert_resource(language)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne character creation acceptance".into(),
                        resolution: WindowResolution::new(1264, 681),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            LocalizationPlugin,
            LegacyModelMaterialPlugin,
            NativeCharacterCreationUiPlugin,
            NativePlayerPreviewPlugin,
            NativePlayerSharedRigPlugin,
        ))
        .add_systems(Startup, setup_preview)
        .add_systems(Update, (cycle_preview_face, drive_capture).chain())
        .add_systems(
            Update,
            audit_face_cycle_order.after(NativePlayerPreviewSet::Rebuild),
        )
        .run();
}

#[derive(Resource)]
struct PreviewOutputs {
    name_output: PathBuf,
    appearance_output: PathBuf,
    appearance_capture_count: u32,
}

fn setup_preview(
    mut commands: Commands,
    outputs: Res<PreviewOutputs>,
    assets: Res<AssetServer>,
    data: Res<PreviewCharacterData>,
    mut model: ResMut<CharacterCreationUiModel>,
    mut player_preview: ResMut<NativePlayerPreviewModel>,
    mut names: ResMut<CharacterNameLists>,
    face_cycle: Res<FaceCycle>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));

    model.visible = true;
    model.screen = CharacterCreationScreen::Name;
    model.music_enabled = false;
    model.name_mode = CharacterNameMode::Generated;
    model.name_indices = [3, 3, 3];
    if let Some(gender) = face_cycle.0 {
        model.appearance.gender = gender;
    }
    model.appearance.height = 0;
    model.appearance.body = 0;
    model.name_table = CharacterCreationCapability::Enabled;
    model.creation_items = CharacterCreationCapability::Enabled;
    model.starter_icons = CharacterCreationCapability::Enabled;
    model.color_palettes = data.0.ui_palettes();
    if env::var_os("FFONE_REVIEW_COLOR_PAGES").is_some() {
        model.color_pages = [2, 2, 1];
        model.appearance.skin_color = 36;
        model.appearance.hair_color = 54;
        model.appearance.eye_color = 10;
    }

    model.preview = CharacterCreationPreviewStatus::PlayerAssemblyPending;
    model.hair_label = data
        .0
        .appearance_label(&model.appearance, AppearanceField::Hair)
        .expect("native hair label");
    model.face_label = data
        .0
        .appearance_label(&model.appearance, AppearanceField::Face)
        .expect("native face label");
    let mut icon_handles = Vec::new();
    for (row, field) in [
        AppearanceField::Shirt,
        AppearanceField::Pants,
        AppearanceField::Shoes,
    ]
    .into_iter()
    .enumerate()
    {
        let paths = data
            .0
            .clothing_icon_window(&model.appearance, field)
            .expect("native creator icon window");
        for (column, path) in paths.into_iter().enumerate() {
            icon_handles.push(assets.load(path.clone()));
            model.starter_icon_paths[row][column] = Some(path);
        }
    }
    let resolved = data
        .0
        .resolve_creator(0, 0, "Test", "Hero", &model.appearance)
        .expect("resolve native creator player look");
    player_preview
        .set_look(resolved.look)
        .expect("valid native creator player look");
    player_preview.visible = false;
    player_preview.stage = NativePlayerPreviewStage::Creation;

    *names = CharacterNameLists {
        first: ["", "Zoom", "Zora", "Abbey", "Abner", "Acacia"]
            .map(str::to_owned)
            .into(),
        middle: ["", "Zoom", "Zort", " ", "Able", "Acorn"]
            .map(str::to_owned)
            .into(),
        last: ["", "Zombie", "Zon", " ", "Abnormal", "Abyss"]
            .map(str::to_owned)
            .into(),
    };

    let mut images = CHARACTER_CREATION_IMAGE_SPECS
        .iter()
        .chain(CHARACTER_CREATION_SHARED_IMAGE_SPECS.iter())
        .map(|spec| assets.load(spec.path))
        .collect::<Vec<_>>();
    images.extend(icon_handles);
    commands.insert_resource(PreviewConfig {
        name_output: outputs.name_output.clone(),
        appearance_output: outputs.appearance_output.clone(),
        appearance_capture_count: outputs.appearance_capture_count,
        images,
        fonts: vec![
            assets.load(CHARACTER_CREATION_FONT_PATH),
            assets.load(CHARACTER_CREATION_DISPLAY_FONT_PATH),
        ],
    });
}

fn audit_face_cycle_order(
    cycle: Res<FaceCycle>,
    materials: Res<Assets<LegacyModelMaterial>>,
    surfaces: Query<(
        Entity,
        &MeshMaterial3d<LegacyModelMaterial>,
        Option<&LegacyMaterialPassCompanion>,
    )>,
    sources: Query<(
        &LegacyMaterialRendererOrder,
        &LegacyMaterialSortOrderApplied,
    )>,
    mut orders: Local<HashMap<Entity, (u16, f32)>>,
) {
    if cycle.0.is_none() {
        return;
    }
    for (entity, handle, companion) in &surfaces {
        let source = companion.map_or(entity, |pass| pass.source_mesh_entity);
        let Ok((order, applied)) = sources.get(source) else {
            continue;
        };
        if order.renderer_index != applied.renderer_index {
            continue;
        }
        let Some(material) = materials.get(&handle.0) else {
            continue;
        };
        let current = (order.renderer_index, material.sort_bias);
        if let Some(previous) = orders.insert(entity, current)
            && previous.0 == current.0
        {
            assert_eq!(
                previous.1, current.1,
                "appearance change erased renderer/pass order on {entity:?}"
            );
        }
    }
}

fn cycle_preview_face(
    cycle: Res<FaceCycle>,
    state: Res<PreviewState>,
    data: Res<PreviewCharacterData>,
    mut model: ResMut<CharacterCreationUiModel>,
    mut preview: ResMut<NativePlayerPreviewModel>,
    mut completed: Local<u32>,
) {
    let body_cycle = env::var_os("FFONE_REVIEW_BODY_SHAPE").is_some();
    if (cycle.0.is_none() && !body_cycle) || state.appearance_saved || *completed == state.appearance_captures_saved
    {
        return;
    }
    *completed = state.appearance_captures_saved;
    if body_cycle {
        model.appearance.height = (*completed % 5) as u8;
        model.appearance.body = (*completed / 5 % 3) as u8;
        println!("bodyShape height={} body={}", model.appearance.height, model.appearance.body);
    } else {
        model.appearance.face = 2 + (*completed % 4) as u8;
        model.appearance.eye_color = 1 + (*completed % 5) as u8;
    }
    model.face_label = data
        .0
        .appearance_label(&model.appearance, AppearanceField::Face)
        .unwrap();
    let resolved = data
        .0
        .resolve_creator(0, 0, "Test", "Hero", &model.appearance)
        .unwrap();
    println!(
        "faceCycle={:?} face={} eye={}",
        model.appearance.gender, model.appearance.face, model.appearance.eye_color
    );
    preview.set_look(resolved.look).unwrap();
}

fn drive_capture(
    mut commands: Commands,
    assets: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    config: Res<PreviewConfig>,
    mut state: ResMut<PreviewState>,
    mut model: ResMut<CharacterCreationUiModel>,
    mut player_preview: ResMut<NativePlayerPreviewModel>,
    roots: Query<(&Visibility, &ComputedNode), With<NativeCharacterCreationRoot>>,
    pass_companions: Query<(
        Entity,
        &LegacyMaterialPassCompanion,
        &SkinnedMesh,
        Option<&NativePlayerRigSkinBound>,
    )>,
    pass_sources: Query<(
        &SkinnedMesh,
        Option<&NativePlayerRigSkinBound>,
        Option<&PendingLegacyModelMaterial>,
        Option<&Name>,
    )>,
    animation_players: Query<&AnimationPlayer>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);

    let source_assets_loaded = config
        .images
        .iter()
        .all(|handle| matches!(assets.load_state(handle.id()), LoadState::Loaded))
        && config
            .fonts
            .iter()
            .all(|handle| matches!(assets.load_state(handle.id()), LoadState::Loaded));
    let cpu_assets_present = config
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && config
            .fonts
            .iter()
            .all(|handle| fonts.get(handle).is_some());
    let root_laid_out = roots.single().is_ok_and(|(visibility, computed)| {
        *visibility != Visibility::Hidden
            && computed.size().x >= 1264.0
            && computed.size().y >= 681.0
    });
    let ready = source_assets_loaded && cpu_assets_present && root_laid_out;
    if ready && state.assets_ready_frame.is_none() {
        state.assets_ready_frame = Some(state.frames);
        state.assets_ready_at = Some(Instant::now());
    }

    if state.name_saved && state.appearance_started_frame.is_none() {
        model.screen = CharacterCreationScreen::Appearance;
        player_preview.visible = true;
        state.appearance_started_frame = Some(state.frames);
        state.appearance_started_at = Some(Instant::now());
        state.last_rejected_frame = None;
    }

    let stage = if !state.name_saved {
        CaptureStage::Name
    } else {
        CaptureStage::Appearance
    };
    let (stage_frame, stage_started_at) = match stage {
        CaptureStage::Name => (state.assets_ready_frame, state.assets_ready_at),
        CaptureStage::Appearance => (state.appearance_started_frame, state.appearance_started_at),
    };
    let player_ready = stage == CaptureStage::Name
        || matches!(
            player_preview.status,
            NativePlayerPreviewStatus::ReadyAnimated { .. }
        );
    let stage_ready = ready
        && player_ready
        && stage_frame.is_some_and(|frame| state.frames.saturating_sub(frame) >= WARMUP_FRAMES)
        && stage_started_at.is_some_and(|started_at| started_at.elapsed() >= GPU_UPLOAD_GRACE);
    let retry_ready = state
        .last_rejected_frame
        .is_none_or(|frame| state.frames.saturating_sub(frame) >= RETRY_FRAMES);
    let animation_seek_time = (stage == CaptureStage::Appearance)
        .then(|| exact_active_animation_seek_time(&animation_players))
        .flatten();
    let animation_pose_ready = stage != CaptureStage::Appearance
        || (state.last_appearance_saved_frame.is_none_or(|frame| {
            state.frames.saturating_sub(frame) >= APPEARANCE_CAPTURE_SEPARATION_FRAMES
        }) && animation_seek_time.is_some_and(|current| {
            state
                .last_appearance_animation_seek_time
                .is_none_or(|previous| {
                    (current - previous).abs() >= MIN_APPEARANCE_ANIMATION_SEEK_DELTA
                })
        }));
    let pass_palettes_ready = if stage == CaptureStage::Appearance && stage_ready {
        match validate_material_pass_palettes(&pass_companions, &pass_sources) {
            Ok(diagnostic) => {
                if !state.capture_issued {
                    println!("{diagnostic}");
                }
                true
            }
            Err(error) => {
                if !state.capture_issued {
                    eprintln!("waiting for exact material-pass skin palettes: {error}");
                }
                false
            }
        }
    } else {
        true
    };
    if !state.capture_issued
        && !state.appearance_saved
        && stage_ready
        && retry_ready
        && animation_pose_ready
        && pass_palettes_ready
    {
        state.capture_issued = true;
        state.capture_target = Some(stage);
        state.capture_animation_seek_time = animation_seek_time;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }

    if state.name_saved && state.appearance_saved {
        exit.write(AppExit::Success);
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        eprintln!(
            "character-creation capture timed out: name_saved={} appearance_saved={} preview={:?} detail={:?}",
            state.name_saved,
            state.appearance_saved,
            player_preview.status,
            player_preview.loading_detail
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn exact_active_animation_seek_time(players: &Query<&AnimationPlayer>) -> Option<f32> {
    let mut seek_times = players.iter().flat_map(|player| {
        player
            .playing_animations()
            .filter(|(_, animation)| !animation.is_paused())
            .map(|(_node, animation)| animation.seek_time())
    });
    let seek_time = seek_times.next()?;
    seek_times.next().is_none().then_some(seek_time)
}

fn validate_material_pass_palettes(
    companions: &Query<(
        Entity,
        &LegacyMaterialPassCompanion,
        &SkinnedMesh,
        Option<&NativePlayerRigSkinBound>,
    )>,
    sources: &Query<(
        &SkinnedMesh,
        Option<&NativePlayerRigSkinBound>,
        Option<&PendingLegacyModelMaterial>,
        Option<&Name>,
    )>,
) -> Result<String, String> {
    let mut count = 0_usize;
    let mut face = Vec::new();
    for (entity, companion, companion_skin, companion_binding) in companions {
        count += 1;
        let (source_skin, source_binding, pending, name) =
            sources.get(companion.source_mesh_entity).map_err(|_| {
                format!(
                    "companion {entity:?}/{:?} has no skinned source {:?}",
                    companion.pass, companion.source_mesh_entity
                )
            })?;
        if source_skin.joints != companion_skin.joints
            || source_skin.inverse_bindposes != companion_skin.inverse_bindposes
        {
            return Err(format!(
                "companion {entity:?}/{:?} palette differs from source {:?}: \
                 joints {}!={}, inverseBindposesEqual={}",
                companion.pass,
                companion.source_mesh_entity,
                companion_skin.joints.len(),
                source_skin.joints.len(),
                source_skin.inverse_bindposes == companion_skin.inverse_bindposes,
            ));
        }
        if source_binding != companion_binding || source_binding.is_none() {
            return Err(format!(
                "companion {entity:?}/{:?} binding {companion_binding:?} differs from \
                 source {:?} binding {source_binding:?}",
                companion.pass, companion.source_mesh_entity
            ));
        }
        let material = pending.map_or("<missing>", |pending| pending.true_name.as_str());
        if material.contains("face") {
            face.push(format!(
                "{material}/{:?}/renderer={}/joints={}/binding={source_binding:?}",
                companion.pass,
                name.map_or("<unnamed>", Name::as_str),
                source_skin.joints.len(),
            ));
        }
    }
    Ok(format!(
        "materialPassPalettes=exact companions={count} face=[{}]",
        face.join(", ")
    ))
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    config: Res<PreviewConfig>,
    face_cycle: Res<FaceCycle>,
    mut state: ResMut<PreviewState>,
) {
    let stage = state
        .capture_target
        .expect("capture target must be recorded before requesting a screenshot");
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    let rgba = image.to_rgba8();
    let visible_pixels = rgba
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _alpha] = pixel.0;
            u16::from(red) + u16::from(green) + u16::from(blue) > 20
        })
        .count();
    let visible_ratio = visible_pixels as f32 / (rgba.width() * rgba.height()) as f32;
    if visible_ratio < MIN_VISIBLE_PIXEL_RATIO {
        eprintln!(
            "rejecting incomplete {} capture: visible pixel ratio \
             {visible_ratio:.3} < {MIN_VISIBLE_PIXEL_RATIO:.3}",
            stage.label()
        );
        state.capture_issued = false;
        state.capture_target = None;
        state.capture_animation_seek_time = None;
        state.last_rejected_frame = Some(state.frames);
        return;
    }
    // The fixed cheek/neck sample boxes below belong to the default boy.
    // Other face shapes and expanded skin/hair colors are captured for visual
    // review: the fixed RGB thresholds describe only the original default look.
    if stage == CaptureStage::Appearance
        && face_cycle.0.is_none()
        && env::var_os("FFONE_REVIEW_COLOR_PAGES").is_none()
    {
        match validate_face_render(&rgba) {
            Ok(diagnostic) => println!("{diagnostic}"),
            Err(error) => {
                eprintln!("rejecting incorrect appearance face: {error}");
                state.capture_issued = false;
                state.capture_target = None;
                state.capture_animation_seek_time = None;
                state.last_rejected_frame = Some(state.frames);
                return;
            }
        }
    }

    let output = match stage {
        CaptureStage::Name => config.name_output.clone(),
        CaptureStage::Appearance => {
            numbered_appearance_output(&config.appearance_output, state.appearance_captures_saved)
        }
    };
    if let Some(parent) = output.parent().filter(|path| !path.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(&output)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", output.display()));
    println!(
        "{}={}",
        stage.label(),
        output
            .canonicalize()
            .unwrap_or_else(|_| output.to_path_buf())
            .display()
    );

    match stage {
        CaptureStage::Name => state.name_saved = true,
        CaptureStage::Appearance => {
            let seek_time = state
                .capture_animation_seek_time
                .expect("appearance capture must record stand1 seek time");
            println!("appearanceAnimationSeekTime={seek_time:.3}");
            state.appearance_captures_saved = state.appearance_captures_saved.saturating_add(1);
            state.last_appearance_saved_frame = Some(state.frames);
            state.last_appearance_animation_seek_time = Some(seek_time);
            state.appearance_saved =
                state.appearance_captures_saved >= config.appearance_capture_count;
        }
    }
    state.capture_issued = false;
    state.capture_target = None;
    state.capture_animation_seek_time = None;
    state.last_rejected_frame = None;
}

fn validate_face_render(image: &image::RgbaImage) -> Result<String, String> {
    let neck = median_rgb(
        pixels_in_box(
            image,
            FACE_NECK_BOUNDS.0,
            FACE_NECK_BOUNDS.1,
            FACE_NECK_BOUNDS.2,
            FACE_NECK_BOUNDS.3,
        )?
        .into_iter()
        .filter(|color| luminance(*color) >= 25.0)
        .collect(),
    )
    .ok_or_else(|| "neck reference contains no non-black skin pixels".to_owned())?;
    let neck_luma = luminance(neck);
    if neck_luma < 25.0 {
        return Err(format!("neck reference is too dark: Y={neck_luma:.2}"));
    }

    let mut all_cheek_pixels = Vec::with_capacity(
        FACE_CHEEK_CENTERS.len() * ((FACE_PATCH_RADIUS * 2 + 1).pow(2) as usize),
    );
    let mut valid = [false; FACE_CHEEK_CENTERS.len()];
    let mut diagnostics = Vec::with_capacity(FACE_CHEEK_CENTERS.len());
    for (index, (x, y)) in FACE_CHEEK_CENTERS.into_iter().enumerate() {
        let pixels = pixels_in_box(
            image,
            x.saturating_sub(FACE_PATCH_RADIUS),
            x.saturating_add(FACE_PATCH_RADIUS),
            y.saturating_sub(FACE_PATCH_RADIUS),
            y.saturating_add(FACE_PATCH_RADIUS),
        )?;
        let cheek = median_rgb(pixels.clone())
            .ok_or_else(|| format!("cheek patch {index} contains no pixels"))?;
        let cheek_luma = luminance(cheek);
        let distance = rgb_distance(cheek, neck);
        let luma_ratio = cheek_luma / neck_luma;
        let blue_residual_delta = ((cheek[2] - cheek[0]) - (neck[2] - neck[0])).abs();
        valid[index] =
            distance <= 40.0 && (0.65..=1.30).contains(&luma_ratio) && blue_residual_delta <= 30.0;
        diagnostics.push(format!(
            "{index}:rgb={:.0}/{:.0}/{:.0},d={distance:.1},y={luma_ratio:.2},blue={blue_residual_delta:.1},ok={}",
            cheek[0], cheek[1], cheek[2], valid[index]
        ));
        all_cheek_pixels.extend(pixels);
    }

    let valid_count = valid.into_iter().filter(|is_valid| *is_valid).count();
    let both_sides = valid[0..2].iter().any(|is_valid| *is_valid)
        && valid[2..4].iter().any(|is_valid| *is_valid);
    let near_black_ratio = all_cheek_pixels
        .iter()
        .filter(|color| luminance(**color) < 0.35 * neck_luma)
        .count() as f32
        / all_cheek_pixels.len() as f32;
    let cheek_blue_residual = median_scalar(
        all_cheek_pixels
            .iter()
            .map(|color| color[2] - color[0])
            .collect(),
    )
    .ok_or_else(|| "cheek union contains no pixels".to_owned())?;
    let blue_residual_delta = (cheek_blue_residual - (neck[2] - neck[0])).abs();
    if valid_count < 3 || !both_sides || near_black_ratio >= 0.35 || blue_residual_delta > 30.0 {
        return Err(format!(
            "neck={:.0}/{:.0}/{:.0}; valid={valid_count}/4 bothSides={both_sides}; \
             nearBlack={near_black_ratio:.3}; blueResidualDelta={blue_residual_delta:.1}; patches=[{}]",
            neck[0],
            neck[1],
            neck[2],
            diagnostics.join("; ")
        ));
    }
    Ok(format!(
        "faceRender=skin-visible neck={:.0}/{:.0}/{:.0} valid={valid_count}/4 \
         nearBlack={near_black_ratio:.3} blueResidualDelta={blue_residual_delta:.1}",
        neck[0], neck[1], neck[2]
    ))
}

fn pixels_in_box(
    image: &image::RgbaImage,
    min_x: u32,
    max_x: u32,
    min_y: u32,
    max_y: u32,
) -> Result<Vec<[f32; 3]>, String> {
    if min_x > max_x || min_y > max_y || max_x >= image.width() || max_y >= image.height() {
        return Err(format!(
            "face acceptance box {min_x}..={max_x},{min_y}..={max_y} is outside {}x{}",
            image.width(),
            image.height()
        ));
    }
    let mut pixels = Vec::with_capacity(((max_x - min_x + 1) * (max_y - min_y + 1)) as usize);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let pixel = image.get_pixel(x, y).0;
            pixels.push([
                f32::from(pixel[0]),
                f32::from(pixel[1]),
                f32::from(pixel[2]),
            ]);
        }
    }
    Ok(pixels)
}

fn median_rgb(colors: Vec<[f32; 3]>) -> Option<[f32; 3]> {
    if colors.is_empty() {
        return None;
    }
    let mut red = Vec::with_capacity(colors.len());
    let mut green = Vec::with_capacity(colors.len());
    let mut blue = Vec::with_capacity(colors.len());
    for color in colors {
        red.push(color[0]);
        green.push(color[1]);
        blue.push(color[2]);
    }
    Some([
        median_scalar(red)?,
        median_scalar(green)?,
        median_scalar(blue)?,
    ])
}

fn median_scalar(mut values: Vec<f32>) -> Option<f32> {
    if values.is_empty() {
        return None;
    }
    values.sort_by(f32::total_cmp);
    let middle = values.len() / 2;
    Some(if values.len().is_multiple_of(2) {
        (values[middle - 1] + values[middle]) * 0.5
    } else {
        values[middle]
    })
}

fn luminance(color: [f32; 3]) -> f32 {
    0.2126 * color[0] + 0.7152 * color[1] + 0.0722 * color[2]
}

fn rgb_distance(left: [f32; 3], right: [f32; 3]) -> f32 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| (left - right).powi(2))
        .sum::<f32>()
        .sqrt()
}

fn numbered_appearance_output(base: &Path, completed_captures: u32) -> PathBuf {
    if completed_captures == 0 {
        return base.to_path_buf();
    }
    let suffix = completed_captures.saturating_add(1);
    let stem = base
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("appearance");
    let mut output = base.to_path_buf();
    output.set_file_name(format!("{stem}.frame{suffix}.png"));
    output
}

#[cfg(test)]
mod tests;
