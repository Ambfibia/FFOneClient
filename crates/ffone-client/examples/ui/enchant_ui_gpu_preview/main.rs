//! Deterministic 1280x720 GPU acceptance frames for clean-Retrobution
//! `eGameMode.Enchant`.
//!
//! Usage: `enchant_ui_gpu_preview [ready|empty|waiting|success|cash-error|orphan]
//! [OUTPUT.png]`. The preview owns no socket, audio, legacy camera, or mutable
//! inventory authority; those boundaries are represented by the standalone
//! module's typed contracts.

#![allow(dead_code)]

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    enchant_ui::*,
    localization::{Localization, LocalizationPlugin},
};
use ffone_protocol::ItemBase0104;

const CLIENT_AREA_WIDTH: u32 = 1_280;
const CLIENT_AREA_HEIGHT: u32 = 720;
const DEFAULT_OUTPUT: &str = "target/ui-parity/enchant-ready-1280x720.png";
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_VISIBLE_PIXELS: usize = 90_000;
const ICON_TARGET: &str = "icons/items/weapons/wpnicon_01.png";
const ICON_WEAPON_MATERIAL: &str = "icons/items/general/generalitemicon_07.png";
const ICON_ARMOR_MATERIAL: &str = "icons/items/general/generalitemicon_08.png";
const ICON_HELPER_1: &str = "icons/items/general/generalitemicon_57.png";
const ICON_HELPER_2: &str = "icons/items/general/generalitemicon_58.png";
const ICON_COSTUME: &str = "icons/items/cosmetics/cosicon_01.png";
const ICON_VEHICLE: &str = "icons/items/vehicles/vehicle_00.png";
const PREVIEW_ICON_PATHS: [&str; 7] = [
    ICON_TARGET,
    ICON_WEAPON_MATERIAL,
    ICON_ARMOR_MATERIAL,
    ICON_HELPER_1,
    ICON_HELPER_2,
    ICON_COSTUME,
    ICON_VEHICLE,
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
enum PreviewMode {
    #[default]
    Ready,
    Empty,
    Waiting,
    Success,
    CashError,
    Orphan,
}

impl PreviewMode {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "ready" => Some(Self::Ready),
            "empty" => Some(Self::Empty),
            "waiting" => Some(Self::Waiting),
            "success" => Some(Self::Success),
            "cash-error" => Some(Self::CashError),
            "orphan" => Some(Self::Orphan),
            _ => None,
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Empty => "empty",
            Self::Waiting => "waiting",
            Self::Success => "success",
            Self::CashError => "cash-error",
            Self::Orphan => "orphan",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreviewCli {
    mode: PreviewMode,
    output: PathBuf,
}

fn parse_cli<I>(arguments: I) -> Result<PreviewCli, &'static str>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let mut arguments = arguments.into_iter();
    let first = arguments.next();
    let (mode, output) = match first {
        None => (PreviewMode::Ready, PathBuf::from(DEFAULT_OUTPUT)),
        Some(value) => {
            let value_text = value
                .to_str()
                .ok_or("usage: enchant_ui_gpu_preview [STATE] [OUTPUT.png]")?;
            if let Some(mode) = PreviewMode::parse(value_text) {
                let output = arguments.next().map(PathBuf::from).unwrap_or_else(|| {
                    PathBuf::from(format!(
                        "target/ui-parity/enchant-{}-1280x720.png",
                        mode.slug()
                    ))
                });
                (mode, output)
            } else {
                (PreviewMode::Ready, PathBuf::from(value))
            }
        }
    };
    if arguments.next().is_some()
        || output.extension().and_then(|value| value.to_str()) != Some("png")
    {
        return Err("usage: enchant_ui_gpu_preview [STATE] [OUTPUT.png]");
    }
    Ok(PreviewCli { mode, output })
}

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    fonts: Vec<Handle<Font>>,
}

#[derive(Resource)]
struct PreviewState {
    frames: u32,
    ready_frame: Option<u32>,
    ready_at: Option<Instant>,
    capture_issued: bool,
    capture_saved: bool,
    capture_failed: bool,
    started_at: Instant,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            frames: 0,
            ready_frame: None,
            ready_at: None,
            capture_issued: false,
            capture_saved: false,
            capture_failed: false,
            started_at: Instant::now(),
        }
    }
}

fn main() {
    let cli = match parse_cli(std::env::args_os().skip(1)) {
        Ok(cli) => cli,
        Err(usage) => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    };
    let asset_root = env::var_os("FFONE_ENCHANT_ASSET_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("assets/game"))
        .canonicalize()
        .expect("assets/game must exist");
    let (localization, language) = Localization::open(
        &asset_root,
        &env::var("FFONE_ENCHANT_LOCALE").unwrap_or_else(|_| "en".into()),
    )
    .expect("open production localization bundles");

    let content = ffone_client::tutorial_mission_content::TutorialMissionContent::open(
        &ffone_client::assets::AssetLocator::open(&asset_root).expect("native assets"),
    )
    .expect("production item tables");
    App::new()
        .insert_resource(content)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(cli.mode)
        .insert_resource(PreviewOutput(cli.output))
        .insert_resource(PreviewState::default())
        .init_resource::<TextGeometryAudit>()
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!("FFOne Retrobution Enchant {} acceptance", cli.mode.slug()),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LocalizationPlugin, EnchantUiPlugin0104))
        .add_systems(Startup, setup_preview)
        .add_systems(
            PostUpdate,
            audit_text_geometry.after(bevy::ui::UiSystems::PostLayout),
        )
        .add_systems(Update, drive_capture.after(EnchantUiSet0104::Bind))
        .add_systems(
            PreUpdate,
            exercise_button_label
                .after(bevy::input::InputSystems)
                .before(bevy::ui::UiSystems::Focus),
        )
        .run();
}

fn exercise_button_label(
    mut frame: Local<u32>,
    preview: Res<PreviewState>,
    mut windows: Query<&mut Window>,
    labels: Query<(
        &ffone_client::localization::LocalizedText,
        &UiGlobalTransform,
        &ComputedNode,
    )>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut outbox: ResMut<EnchantUiOutbox0104>,
    elements: Query<(&EnchantUiElement0104, &UiGlobalTransform)>,
) {
    let Ok(mode) = env::var("FFONE_ENCHANT_POINTER") else {
        return;
    };
    if preview.ready_frame.is_none() {
        return;
    }
    *frame += 1;
    if mode == "drag" || mode == "close" {
        let element = if mode == "close" {
            EnchantUiElement0104::Close
        } else if *frame < 15 {
            EnchantUiElement0104::InventorySlotFrame(0)
        } else {
            EnchantUiElement0104::AttachmentFrame(EnchantAttachmentSlot0104::Target)
        };
        let (_, transform) = elements.iter().find(|(candidate, _)| **candidate == element).unwrap();
        let mut window = windows.single_mut().unwrap();
        window.focused = true;
        window.set_physical_cursor_position(Some(transform.translation.as_dvec2()));
        if *frame == 10 { mouse.press(MouseButton::Left); }
        if *frame == 20 { mouse.release(MouseButton::Left); }
        if *frame == 25 {
            let expected = if mode == "close" {
                vec![EnchantUiCommand0104::Close]
            } else {
                vec![EnchantUiCommand0104::BeginInventoryDrag(0), EnchantUiCommand0104::DropOnAttachment(EnchantAttachmentSlot0104::Target)]
            };
            for command in expected { assert_eq!(outbox.pop_front(), Some(command), "real pointer {mode}"); }
            assert_eq!(outbox.pop_front(), None, "duplicate pointer command");
            println!("PASS enchant {mode} pointer");
        }
        return;
    }
    let (key, expected) = match mode.as_str() {
        "preview" => ("ui.enchant.preview", Some(EnchantUiCommand0104::Preview)),
        "clear" => ("ui.enchant.clear", Some(EnchantUiCommand0104::Clear)),
        "enchant" => ("ui.enchant.action", Some(EnchantUiCommand0104::Enchant)),
        "redeem" => (
            "ui.enchant.redeem_code",
            Some(EnchantUiCommand0104::OpenRedeemCode),
        ),
        "more" => (
            "ui.enchant.success.more_items",
            Some(EnchantUiCommand0104::EnchantMoreItems),
        ),
        "disabled" => ("ui.enchant.action", None),
        unknown => panic!("unknown enchant pointer fixture {unknown}"),
    };
    if *frame <= 25 {
        let (_, transform, _) = labels
            .iter()
            .find(|(text, _, node)| text.key == key && node.size().min_element() > 0.0)
            .expect("visible button label");
        let mut window = windows.single_mut().unwrap();
        window.focused = true;
        window.set_physical_cursor_position(Some(transform.translation.as_dvec2()));
    }
    if *frame == 10 {
        mouse.press(MouseButton::Left);
    }
    if *frame == 15 {
        mouse.release(MouseButton::Left);
    }
    if *frame == 25 {
        assert_eq!(outbox.pop_front(), expected, "real click on {key}");
        assert_eq!(
            outbox.pop_front(),
            None,
            "one click must produce at most one command"
        );
        println!("PASS enchant {mode} label pointer");
    }
}

fn base_item(item_type: i16, item_id: i16, option: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit: 0,
    }
}

fn item_presentation(name: &str, icon: &str) -> EnchantItemPresentation0104 {
    EnchantItemPresentation0104 {
        name: name.to_owned(),
        description: format!("A clean-Retrobution {name} used by the Enchant preview."),
        icon_path: Some(icon.to_owned()),
        minimum_level: 7,
        cashable: 0,
        can_equip: true,
        point_rating: 72,
        group_rating: 61,
        defense_rating: 18,
        type_label: "Weapon".to_owned(),
        range_label: "Ranged".to_owned(),
        rarity_label: "Uncommon".to_owned(),
        trade_label: "Tradable".to_owned(),
    }
}

fn selectable(
    source_slot: i32,
    item_type: i16,
    item_id: i16,
    option: i32,
    name: &str,
    icon: &str,
) -> EnchantSelectableItem0104 {
    EnchantSelectableItem0104 {
        source_slot,
        item: base_item(item_type, item_id, option),
        presentation: item_presentation(name, icon),
    }
}

fn support_presentations() -> EnchantSupportPresentation0104 {
    EnchantSupportPresentation0104 {
        weapon_material: item_presentation("Weapon Matter", ICON_WEAPON_MATERIAL),
        armor_material: item_presentation("Armor Matter", ICON_ARMOR_MATERIAL),
        helper_1: item_presentation("Lucky Helper", ICON_HELPER_1),
        helper_2: item_presentation("Extra Helper", ICON_HELPER_2),
    }
}

fn attach_ready_items(model: &mut EnchantModeModel0104) {
    model
        .attach(
            EnchantAttachmentSlot0104::Target,
            selectable(0, 1, 301, 3, "Croc Pot Jacket", ICON_COSTUME),
        )
        .expect("target attach");
    model
        .attach(
            EnchantAttachmentSlot0104::WeaponMaterial,
            selectable(1, 7, 101, 22, "Weapon Matter", ICON_WEAPON_MATERIAL),
        )
        .expect("weapon material attach");
    model
        .attach(
            EnchantAttachmentSlot0104::ArmorMaterial,
            selectable(2, 7, 102, 18, "Armor Matter", ICON_ARMOR_MATERIAL),
        )
        .expect("armor material attach");
}

fn preview_model(mode: PreviewMode) -> EnchantModeModel0104 {
    let mut model = EnchantModeModel0104::default();
    model.open(8_150, false);
    model.clear_intents();
    if mode == PreviewMode::Empty {
        return model;
    }
    if mode == PreviewMode::CashError {
        let mut target = selectable(0, 0, 100, 2, "Cash Blaster", ICON_TARGET);
        target.presentation.cashable = 1;
        model
            .attach(EnchantAttachmentSlot0104::Target, target)
            .expect("cash target attach");
        return model;
    }
    attach_ready_items(&mut model);
    if mode == PreviewMode::Ready {
        return model;
    }
    model.activate_enchant().expect("enchant modal");
    model.accept_system_message().expect("confirm enchant");
    if mode == PreviewMode::Waiting {
        model.advance(2.35).expect("waiting progress");
        return model;
    }
    model.advance(4.001).expect("request boundary");
    model
        .receive_success(EnchantSuccess0104 {
            enchant_item_slot: 0,
            enchant_item: base_item(1, 301, 4),
            weapon_material_item_slot: 1,
            weapon_material_item: base_item(7, 101, 12),
            defence_material_item_slot: 2,
            defence_material_item: base_item(7, 102, 8),
            cash_item_slot_1: -1,
            cash_item_slot_2: -1,
            taros: 7_650,
            success_flag: 1,
        })
        .expect("success reply");
    if mode == PreviewMode::Orphan {
        model.enchant_more_items().expect("enchant more");
    }
    model
}

fn preview_projection(mode: PreviewMode) -> EnchantModeProjection0104 {
    let model = preview_model(mode);
    let mut inventory = std::array::from_fn(|_| EnchantInventorySlotProjection0104::default());
    for (slot, item, icon, count) in [
        (0, base_item(1, 301, 3), ICON_COSTUME, None),
        (1, base_item(7, 101, 22), ICON_WEAPON_MATERIAL, Some("22")),
        (2, base_item(7, 102, 18), ICON_ARMOR_MATERIAL, Some("18")),
        (4, base_item(0, 100, 0), ICON_TARGET, None),
        (5, base_item(10, 400, 0), ICON_VEHICLE, None),
        (7, base_item(7, 141, 2), ICON_HELPER_1, Some("2")),
        (8, base_item(7, 142, 1), ICON_HELPER_2, Some("1")),
    ] {
        inventory[slot] = EnchantInventorySlotProjection0104 {
            item: Some(item),
            icon_path: Some(icon.to_owned()),
            restricted: slot == 5,
            show_combined_badge: slot == 4,
            quantity_label: count.map(str::to_owned),
            hidden_by_selection_overlay: mode != PreviewMode::Empty
                && mode != PreviewMode::CashError
                && slot <= 2,
        };
    }
    let mut equipment = std::array::from_fn(|_| EnchantInventorySlotProjection0104::default());
    equipment[0] = EnchantInventorySlotProjection0104 {
        item: Some(base_item(0, 100, 0)),
        icon_path: Some(ICON_TARGET.to_owned()),
        show_combined_badge: true,
        ..EnchantInventorySlotProjection0104::default()
    };
    equipment[1] = EnchantInventorySlotProjection0104 {
        item: Some(base_item(1, 301, 3)),
        icon_path: Some(ICON_COSTUME.to_owned()),
        ..EnchantInventorySlotProjection0104::default()
    };
    equipment[8] = EnchantInventorySlotProjection0104 {
        item: Some(base_item(10, 400, 0)),
        icon_path: Some(ICON_VEHICLE.to_owned()),
        ..EnchantInventorySlotProjection0104::default()
    };
    EnchantModeProjection0104::from_model(
        &model,
        support_presentations(),
        inventory,
        equipment,
        468,
        392,
    )
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mode: Res<PreviewMode>,
    content: Res<ffone_client::tutorial_mission_content::TutorialMissionContent>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    let mut projection = preview_projection(*mode);
    for slot in EnchantAttachmentSlot0104::ALL {
        if let Some(selected) = projection.selection.visual_item_mut(slot)
            && let Some(mut data) = enchant_item_presentation_from_content(&content, selected.item)
        {
            data.cashable = selected.presentation.cashable;
            selected.presentation = data;
        }
    }
    if let Some(success) = &mut projection.success
        && let Some(data) = enchant_item_presentation_from_content(&content, success.item)
    {
        success.presentation = data;
    }
    commands.insert_resource(projection);
    commands.insert_resource(PreviewAssets {
        images: ENCHANT_UI_DEFAULT_IMAGE_PATHS_0104
            .iter()
            .copied()
            .chain(PREVIEW_ICON_PATHS)
            .map(|path| asset_server.load(path))
            .collect(),
        fonts: vec![
            asset_server.load(ENCHANT_JEFFE_FONT_PATH),
            asset_server.load(ENCHANT_CHALET_FONT_PATH),
        ],
    });
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    mut state: ResMut<PreviewState>,
    text_audit: Res<TextGeometryAudit>,
    inventory_ui: Res<EnchantInventoryUiState0104>,
    roots: Query<(&ComputedNode, &Visibility), With<EnchantUiRoot0104>>,
    elements: Query<(&EnchantUiElement0104, &ComputedNode)>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || preview_assets
            .fonts
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)));
    if failed {
        eprintln!("Enchant acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }
    let assets_loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && preview_assets
            .fonts
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded));
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && preview_assets
            .fonts
            .iter()
            .all(|handle| fonts.get(handle).is_some());
    let root_exact = roots.iter().any(|(node, visibility)| {
        *visibility == Visibility::Visible
            && node.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let main_exact = elements.iter().any(|(element, node)| {
        *element == EnchantUiElement0104::MainGroup && node.size() == Vec2::new(585.0, 653.0)
    });
    let inventory_exact = elements.iter().any(|(element, node)| {
        *element == EnchantUiElement0104::PcStuffPanel && node.size() == Vec2::new(380.0, 632.0)
    });
    let attachment_count_exact = elements
        .iter()
        .filter(|(element, node)| {
            matches!(**element, EnchantUiElement0104::AttachmentFrame(_))
                && node.size() == Vec2::new(64.0, 64.0)
        })
        .count()
        == 5;
    let ready = assets_loaded
        && cpu_assets_present
        && root_exact
        && main_exact
        && inventory_exact
        && attachment_count_exact
        && inventory_ui.panel_controls_enabled()
        && text_audit.ready;
    if ready && state.ready_frame.is_none() {
        state.ready_frame = Some(state.frames);
        state.ready_at = Some(Instant::now());
    }
    let warmed = state
        .ready_frame
        .is_some_and(|frame| state.frames.saturating_sub(frame) >= WARMUP_FRAMES_AFTER_LOAD)
        && state
            .ready_at
            .is_some_and(|ready_at| ready_at.elapsed() >= GPU_UPLOAD_GRACE);
    if ready && warmed && !state.capture_issued {
        state.capture_issued = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_screenshot);
    }
    if state.capture_saved {
        exit.write(AppExit::Success);
    } else if state.capture_failed {
        exit.write(AppExit::error());
    } else if state.started_at.elapsed() >= CAPTURE_TIMEOUT {
        eprintln!("Enchant acceptance capture timed out");
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn save_screenshot(
    event: On<ScreenshotCaptured>,
    output: Res<PreviewOutput>,
    mut state: ResMut<PreviewState>,
) {
    let image = event
        .image
        .clone()
        .try_into_dynamic()
        .expect("GPU screenshot must convert to a dynamic image");
    let rgba = image.to_rgba8();
    if rgba.dimensions() != (CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT) {
        eprintln!(
            "rejecting Enchant capture with size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }
    let visible_pixels = rgba
        .pixels()
        .filter(|pixel| {
            let [red, green, blue, _alpha] = pixel.0;
            u16::from(red) + u16::from(green) + u16::from(blue) > 35
        })
        .count();
    if visible_pixels < MIN_VISIBLE_PIXELS {
        eprintln!(
            "rejecting incomplete Enchant capture: visible={visible_pixels}, min={MIN_VISIBLE_PIXELS}"
        );
        state.capture_failed = true;
        return;
    }
    if let Some(parent) = output
        .0
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .unwrap_or_else(|error| panic!("cannot create {}: {error}", parent.display()));
    }
    image
        .save(&output.0)
        .unwrap_or_else(|error| panic!("cannot save {}: {error}", output.0.display()));
    println!(
        "{}",
        output
            .0
            .canonicalize()
            .unwrap_or_else(|_| output.0.clone())
            .display()
    );
    state.capture_saved = true;
}

#[cfg(test)]
mod tests;

#[derive(Resource, Default)]
struct TextGeometryAudit {
    ready: bool,
    reported: bool,
    error: Option<String>,
}

fn audit_text_geometry(
    mut audit: ResMut<TextGeometryAudit>,
    texts: Query<(
        &ffone_client::localization::LocalizedText,
        &EnchantUiTextStyle0104,
        &ComputedNode,
        &UiGlobalTransform,
        &ChildOf,
    )>,
    parents: Query<(&ComputedNode, &UiGlobalTransform)>,
) {
    let mut visible = 0;
    let mut error = None;
    for (label, style, node, transform, parent) in &texts {
        if node.size().min_element() <= 0.0 {
            continue;
        }
        let Ok((container, container_transform)) = parents.get(parent.parent()) else {
            continue;
        };
        if container.size().min_element() <= 0.0 {
            continue;
        }
        visible += 1;
        let scale = container.inverse_scale_factor();
        let size = node.size() * scale;
        let bounds = container.size() * scale;
        let actual = (transform.translation - container_transform.translation
            + container.size() * 0.5)
            * scale
            - size * 0.5;
        let [left, right, top, bottom] = style.padding();
        let expected_x = match style.legacy_alignment() {
            _ if label.key == "ui.enchant.counter_digit" => (bounds.x - size.x) * 0.5,
            4 => (bounds.x - size.x + left - right) * 0.5,
            5 => bounds.x - right - size.x,
            _ => left,
        };
        let expected_y = if style.legacy_alignment() == 0 {
            top
        } else {
            (bounds.y - size.y + top - bottom) * 0.5
        };
        if (actual - Vec2::new(expected_x, expected_y))
            .abs()
            .max_element()
            > 0.8
        {
            error = Some(format!(
                "{} alignment {:?} expected ({expected_x}, {expected_y}), text={size:?}, container={bounds:?}",
                label.key, actual
            ));
            break;
        }
        if matches!(
            style,
            EnchantUiTextStyle0104::EnchantDefaultButtonMiddleCenter
                | EnchantUiTextStyle0104::InventoryButtonMiddleCenter
        ) && (size.y - style.line_height()).abs() > 0.8
        {
            error = Some(format!(
                "{} button glyph box height {} expected {}",
                label.key,
                size.y,
                style.line_height()
            ));
            break;
        }
    }
    audit.ready = visible >= 5 && error.is_none();
    if audit.error != error {
        if let Some(message) = &error {
            eprintln!("Enchant text audit waiting: {message}");
        }
        audit.error = error;
    }
    if audit.ready && !audit.reported {
        println!(
            "PASS Enchant GPU text geometry: {visible} visible labels, native anchors and button line boxes"
        );
        audit.reported = true;
    }
}
