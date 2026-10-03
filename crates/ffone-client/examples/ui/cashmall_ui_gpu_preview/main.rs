//! Deterministic 1264x681 GPU acceptance frame for the clean-Retrobution
//! hidden `/cashmall` mode.
//!
//! The five named modes prove the retained tab visuals; `opening` captures the
//! midpoint of the exact one-second sine slide. The harness supplies local
//! authoritative projections and does not encode a purchase or inventory
//! mutation.

#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

#[path = "../../../tests/cashmall_ui_standalone/main.rs"]
mod cashmall_standalone;

pub use ffone_client::ui_startup;
pub mod inventory_runtime {
    pub use crate::cashmall_standalone::inventory_runtime::*;
}
pub mod localization {
    pub use crate::cashmall_standalone::localization::*;
}
pub mod user_equip_ui {
    pub use crate::cashmall_standalone::user_equip_ui::*;
}
pub mod vendor_ui {
    pub use crate::cashmall_standalone::vendor_ui::*;
}

use std::{
    env, fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use bevy::{
    asset::{AssetPlugin, LoadState},
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    text::{LineHeight, TextLayoutInfo},
    window::{PresentMode, WindowResolution},
};
use cashmall_standalone::{
    cashmall_ui::{
        CASHMALL_OPEN_SECONDS, CASHMALL_REPLACEMENT_FONT_Y_OFFSET, CASHMALL_SLOT_TYPE,
        CASHMALL_UI_DEFAULT_IMAGE_PATHS_0104, CashmallCloseGate0104, CashmallIconRef0104,
        CashmallModalState0104, CashmallModeProjection0104, CashmallOpenSource0104,
        CashmallPlayerInventoryItem0104, CashmallStaticAssetReadiness0104, CashmallTab0104,
        CashmallTextStyle0104, CashmallUiAssetStatus0104, CashmallUiElement0104,
        CashmallUiOutbox0104, CashmallUiPlugin0104, CashmallUiRoot0104, CashmallUiSet0104,
        CashmallUiState0104,
    },
    localization::{Language, Localization, LocalizationPlugin, LocalizedText},
    user_equip_ui::{USER_EQUIP_FONT_PATH, UserEquipItemModeProjection},
};
use ffone_protocol::ItemBase0104;

#[cfg(test)]
use cashmall_standalone::cashmall_ui::{
    CashmallLifecyclePhase0104, CashmallSlotFrameVisual0104, CashmallTabVisual0104,
    cashmall_mode_view_0104,
};

const CLIENT_AREA_WIDTH: u32 = 1_264;
const CLIENT_AREA_HEIGHT: u32 = 681;
const DEFAULT_OUTPUT: &str = "target/ui-parity/cashmall-new-en-1264x681.png";
const WARMUP_FRAMES_AFTER_LOAD: u32 = 30;
const GPU_UPLOAD_GRACE: Duration = Duration::from_secs(1);
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(25);
const MIN_VISIBLE_PIXELS: usize = 100_000;
const MIN_MISSING_CHECKER_PIXELS: usize = 500;

const ICON_GENERAL_00: &str = "icons/items/general/generalitemicon_00.png";
const ICON_GENERAL_01: &str = "icons/items/general/generalitemicon_01.png";
const ICON_WEAPON_01: &str = "icons/items/weapons/wpnicon_01.png";
const ICON_WEAPON_02: &str = "icons/items/weapons/wpnicon_02.png";
const ICON_COSMETIC_00: &str = "icons/items/cosmetics/cosicon_00.png";
const ICON_COSMETIC_01: &str = "icons/items/cosmetics/cosicon_01.png";
const ICON_COSMETIC_02: &str = "icons/items/cosmetics/cosicon_02.png";
const ICON_COSMETIC_03: &str = "icons/items/cosmetics/cosicon_03.png";
const ICON_COSMETIC_04: &str = "icons/items/cosmetics/cosicon_04.png";
const ICON_COSMETIC_05: &str = "icons/items/cosmetics/cosicon_05.png";
const ICON_VEHICLE_00: &str = "icons/items/vehicles/vehicle_00.png";
const PREVIEW_ICON_PATHS: [&str; 11] = [
    ICON_GENERAL_00,
    ICON_GENERAL_01,
    ICON_WEAPON_01,
    ICON_WEAPON_02,
    ICON_COSMETIC_00,
    ICON_COSMETIC_01,
    ICON_COSMETIC_02,
    ICON_COSMETIC_03,
    ICON_COSMETIC_04,
    ICON_COSMETIC_05,
    ICON_VEHICLE_00,
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
enum PreviewMode {
    #[default]
    New,
    Scroll,
    Potion,
    Equipment,
    Etc,
    Opening,
}

impl PreviewMode {
    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "new" => Self::New,
            "scroll" => Self::Scroll,
            "potion" => Self::Potion,
            "equipment" => Self::Equipment,
            "etc" => Self::Etc,
            "opening" => Self::Opening,
            _ => return None,
        })
    }

    const fn tab(self) -> CashmallTab0104 {
        match self {
            Self::New | Self::Opening => CashmallTab0104::New,
            Self::Scroll => CashmallTab0104::Scroll,
            Self::Potion => CashmallTab0104::Potion,
            Self::Equipment => CashmallTab0104::Equipment,
            Self::Etc => CashmallTab0104::Etc,
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Scroll => "scroll",
            Self::Potion => "potion",
            Self::Equipment => "equipment",
            Self::Etc => "etc",
            Self::Opening => "opening",
        }
    }

    const fn is_opening(self) -> bool {
        matches!(self, Self::Opening)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum PreviewLocale {
    #[default]
    En,
    Ru,
}

impl PreviewLocale {
    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "en" => Self::En,
            "ru" => Self::Ru,
            _ => return None,
        })
    }

    const fn slug(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ru => "ru",
        }
    }
}

#[derive(Resource)]
struct PreviewOutput(PathBuf);

#[derive(Resource)]
struct PreviewAssets {
    images: Vec<Handle<Image>>,
    font: Handle<Font>,
}

#[derive(Resource)]
struct PreviewState {
    frames: u32,
    ready_frame: Option<u32>,
    ready_at: Option<Instant>,
    capture_issued: bool,
    capture_saved: bool,
    capture_failed: bool,
    text_audited: bool,
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
            text_audited: false,
            started_at: Instant::now(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreviewCli {
    mode: PreviewMode,
    locale: PreviewLocale,
    output: PathBuf,
}

fn default_output(mode: PreviewMode, locale: PreviewLocale) -> PathBuf {
    PathBuf::from(format!(
        "target/ui-parity/cashmall-{}-{}-1264x681.png",
        mode.slug(),
        locale.slug(),
    ))
}

fn is_png(path: &Path) -> bool {
    path.extension().and_then(|value| value.to_str()) == Some("png")
}

fn parse_cli<I>(arguments: I) -> Result<PreviewCli, &'static str>
where
    I: IntoIterator<Item = std::ffi::OsString>,
{
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    let usage = "usage: cashmall_ui_gpu_preview \
        [new|scroll|potion|equipment|etc|opening] [en|ru] [OUTPUT.png]";
    match arguments.as_slice() {
        [] => Ok(PreviewCli {
            mode: PreviewMode::New,
            locale: PreviewLocale::En,
            output: PathBuf::from(DEFAULT_OUTPUT),
        }),
        [single] => {
            if let Some(mode) = single.to_str().and_then(PreviewMode::parse) {
                Ok(PreviewCli {
                    mode,
                    locale: PreviewLocale::En,
                    output: default_output(mode, PreviewLocale::En),
                })
            } else if let Some(locale) = single.to_str().and_then(PreviewLocale::parse) {
                Ok(PreviewCli {
                    mode: PreviewMode::New,
                    locale,
                    output: default_output(PreviewMode::New, locale),
                })
            } else {
                let output = PathBuf::from(single.as_os_str());
                is_png(&output)
                    .then_some(PreviewCli {
                        mode: PreviewMode::New,
                        locale: PreviewLocale::En,
                        output,
                    })
                    .ok_or(usage)
            }
        }
        [first, second] => {
            if let (Some(mode), Some(locale)) = (
                first.to_str().and_then(PreviewMode::parse),
                second.to_str().and_then(PreviewLocale::parse),
            ) {
                Ok(PreviewCli {
                    mode,
                    locale,
                    output: default_output(mode, locale),
                })
            } else if let Some(mode) = first.to_str().and_then(PreviewMode::parse) {
                let output = PathBuf::from(second.as_os_str());
                is_png(&output)
                    .then_some(PreviewCli {
                        mode,
                        locale: PreviewLocale::En,
                        output,
                    })
                    .ok_or(usage)
            } else if let Some(locale) = first.to_str().and_then(PreviewLocale::parse) {
                let output = PathBuf::from(second.as_os_str());
                is_png(&output)
                    .then_some(PreviewCli {
                        mode: PreviewMode::New,
                        locale,
                        output,
                    })
                    .ok_or(usage)
            } else {
                Err(usage)
            }
        }
        [mode, locale, output] => {
            let mode = mode.to_str().and_then(PreviewMode::parse).ok_or(usage)?;
            let locale = locale
                .to_str()
                .and_then(PreviewLocale::parse)
                .ok_or(usage)?;
            let output = PathBuf::from(output.as_os_str());
            is_png(&output)
                .then_some(PreviewCli {
                    mode,
                    locale,
                    output,
                })
                .ok_or(usage)
        }
        _ => Err(usage),
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
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .expect("assets/game must exist");
    let (localization, language) = Localization::open(&asset_root, cli.locale.slug())
        .expect("open production localization bundle");
    let mode = cli.mode;
    let locale = cli.locale;
    let output = cli.output;

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(localization)
        .insert_resource(language)
        .insert_resource(mode)
        .insert_resource(PreviewOutput(output))
        .insert_resource(PreviewState::default())
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: format!(
                            "FFOne clean Retrobution Cash Mall acceptance ({})",
                            locale.slug()
                        ),
                        resolution: WindowResolution::new(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT),
                        present_mode: PresentMode::AutoNoVsync,
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((LocalizationPlugin, CashmallUiPlugin0104))
        .add_systems(Startup, setup_preview)
        .add_systems(
            Update,
            hold_opening_preview
                .after(CashmallUiSet0104::Interaction)
                .before(CashmallUiSet0104::Bind),
        )
        .add_systems(Update, drive_capture.after(CashmallUiSet0104::Bind))
        .run();
}

fn preview_projection() -> CashmallModeProjection0104 {
    let mut shared_item_mode = UserEquipItemModeProjection::default();
    let equipment = [
        (0, item(4, 104, 0x0012_0000, 0), ICON_COSMETIC_03),
        (1, item(5, 105, 0, 0), ICON_COSMETIC_04),
        (2, item(6, 106, 0, 0), ICON_COSMETIC_05),
        (3, item(1, 101, 0, 0), ICON_COSMETIC_00),
        (5, item(3, 103, 0, 0), ICON_COSMETIC_02),
        (6, item(0, 100, 0, 0), ICON_WEAPON_01),
        (7, item(0, 108, 0, 0), ICON_WEAPON_02),
        (8, item(10, 107, 0, 0), ICON_VEHICLE_00),
    ];
    let inventory = [
        (0, item(0, 100, 0x0011_0002, 0), Some(ICON_WEAPON_01)),
        (1, item(7, 200, 25, 0), Some(ICON_GENERAL_00)),
        (2, item(9, 300, 0, 0), None),
        (4, item(1, 101, 0, 0), Some(ICON_COSMETIC_00)),
        (5, item(2, 102, 0, 0), Some(ICON_COSMETIC_01)),
        (6, item(3, 103, 0, 0), Some(ICON_COSMETIC_02)),
        (7, item(4, 104, 0, 0), Some(ICON_COSMETIC_03)),
        (8, item(5, 105, 0, 0), Some(ICON_COSMETIC_04)),
        (9, item(6, 106, 0, 0), Some(ICON_COSMETIC_05)),
        (10, item(10, 107, 0, 0), Some(ICON_VEHICLE_00)),
        (11, item(7, 201, 3, 0), Some(ICON_GENERAL_01)),
    ];
    for (slot, value, icon) in equipment {
        shared_item_mode.set_preview_equipment(slot, value, Some(icon));
    }
    for (slot, value, icon) in inventory {
        shared_item_mode.set_preview_inventory(slot, value, icon);
    }
    let rows = [
        preview_row(
            0,
            item(0, 100, 0, 0),
            "Retro Rocket",
            8,
            450,
            Some(ICON_WEAPON_01),
            true,
        ),
        preview_row(
            1,
            item(1, 101, 0, 0),
            "Dee Dee Jacket",
            9,
            550,
            Some(ICON_COSMETIC_00),
            true,
        ),
        preview_row(
            2,
            item(2, 102, 0, 0),
            "Samurai Pants",
            10,
            650,
            Some(ICON_COSMETIC_01),
            true,
        ),
        preview_row(
            3,
            item(3, 103, 0, 0),
            "Sector V Sneakers",
            11,
            750,
            Some(ICON_COSMETIC_02),
            true,
        ),
        preview_row(
            4,
            item(10, 107, 0, 0),
            "Monkey Skyboard",
            20,
            2_500,
            Some(ICON_VEHICLE_00),
            false,
        ),
        preview_row(
            5,
            item(0, 108, 0, 0),
            "Fusion Blaster",
            16,
            1_250,
            Some(ICON_WEAPON_02),
            true,
        ),
        preview_row(
            6,
            item(7, 200, 25, 0),
            "Health Pack",
            1,
            120,
            Some(ICON_GENERAL_00),
            true,
        ),
        // Deliberate missing native reference proves the checker fallback.
        preview_row(7, item(9, 300, 0, 0), "Unknown Chest", 7, 300, None, true),
    ];
    CashmallModeProjection0104 {
        player_inventory: rows.into(),
        shared_item_mode,
        taros: 1_500,
    }
}

#[allow(clippy::too_many_arguments)]
fn preview_row(
    slot_id: i32,
    item: ItemBase0104,
    name: &str,
    level: i32,
    item_price: i32,
    icon: Option<&str>,
    equip_eligible: bool,
) -> CashmallPlayerInventoryItem0104 {
    CashmallPlayerInventoryItem0104 {
        slot_type: CASHMALL_SLOT_TYPE,
        slot_id,
        item,
        name: name.to_owned(),
        level,
        item_price,
        icon: icon.map(|path| CashmallIconRef0104::new(path).unwrap()),
        equip_eligible: Some(equip_eligible),
    }
}

const fn item(item_type: i16, item_id: i16, option: i32, time_limit: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit,
    }
}

fn setup_preview(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mode: Res<PreviewMode>,
    mut state: ResMut<CashmallUiState0104>,
    mut modal: ResMut<CashmallModalState0104>,
    mut close_gate: ResMut<CashmallCloseGate0104>,
    mut outbox: ResMut<CashmallUiOutbox0104>,
) {
    commands.spawn((Camera2d, IsDefaultUiCamera));
    commands.insert_resource(preview_projection());
    state.open_from(
        CashmallOpenSource0104::HiddenChatCommand,
        false,
        &mut outbox,
    );
    state.tick(if mode.is_opening() {
        CASHMALL_OPEN_SECONDS * 0.5
    } else {
        CASHMALL_OPEN_SECONDS
    });
    if !mode.is_opening() {
        state
            .select_tab(*modal, mode.tab(), &mut outbox)
            .expect("preview tab must activate after the clean opening");
    }
    outbox.clear();
    *modal = CashmallModalState0104::default();
    *close_gate = CashmallCloseGate0104 {
        mode_accepts_escape: true,
        exit_arbitration_clear: true,
    };
    commands.insert_resource(PreviewAssets {
        images: CASHMALL_UI_DEFAULT_IMAGE_PATHS_0104
            .iter()
            .copied()
            .chain(PREVIEW_ICON_PATHS)
            .map(|path| asset_server.load(path))
            .collect(),
        font: asset_server.load(USER_EQUIP_FONT_PATH),
    });
}

fn hold_opening_preview(
    mode: Res<PreviewMode>,
    mut state: ResMut<CashmallUiState0104>,
    mut outbox: ResMut<CashmallUiOutbox0104>,
) {
    if !mode.is_opening() {
        return;
    }
    // The production plugin advances every frame. Re-entering and advancing to
    // exactly 0.5 s keeps this acceptance mode at the authoritative midpoint.
    state.open_from(
        CashmallOpenSource0104::HiddenChatCommand,
        false,
        &mut outbox,
    );
    state.tick(CASHMALL_OPEN_SECONDS * 0.5);
    outbox.clear();
}

#[allow(clippy::too_many_arguments)]
fn drive_capture(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    fonts: Res<Assets<Font>>,
    preview_assets: Res<PreviewAssets>,
    asset_status: Res<CashmallUiAssetStatus0104>,
    language: Res<Language>,
    mut state: ResMut<PreviewState>,
    roots: Query<(&ComputedNode, &Visibility), With<CashmallUiRoot0104>>,
    elements: Query<(&CashmallUiElement0104, &ComputedNode)>,
    all_texts: Query<&Text>,
    styled_texts: Query<(
        Entity,
        &Text,
        &LocalizedText,
        &CashmallTextStyle0104,
        (&TextFont, &LineHeight),
        &TextLayoutInfo,
        &ComputedNode,
        &UiTransform,
    )>,
    mut exit: MessageWriter<AppExit>,
) {
    state.frames = state.frames.saturating_add(1);
    let failed = preview_assets
        .images
        .iter()
        .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
        || matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Failed(_)
        )
        || asset_status.0 == CashmallStaticAssetReadiness0104::Failed;
    if failed {
        eprintln!("Cash Mall acceptance asset failed to load");
        exit.write(AppExit::error());
        return;
    }

    let assets_loaded = preview_assets
        .images
        .iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
        && matches!(
            asset_server.load_state(preview_assets.font.id()),
            LoadState::Loaded
        )
        && asset_status.0 == CashmallStaticAssetReadiness0104::Ready;
    let cpu_assets_present = preview_assets
        .images
        .iter()
        .all(|handle| images.get(handle).is_some())
        && fonts.get(&preview_assets.font).is_some();
    let root_exact = roots.iter().any(|(node, visibility)| {
        *visibility == Visibility::Visible
            && node.size() == Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32)
    });
    let panels_exact = element_has_size(
        &elements,
        CashmallUiElement0104::CashmallPanel,
        Vec2::new(498.0, 638.0),
    ) && element_has_size(
        &elements,
        CashmallUiElement0104::PcStuffPanel,
        Vec2::new(380.0, 632.0),
    ) && element_has_size(
        &elements,
        CashmallUiElement0104::EquipmentPanel,
        Vec2::new(66.0, 639.0),
    );
    let list_exact = element_has_size(
        &elements,
        CashmallUiElement0104::ListViewport,
        Vec2::new(464.0, 400.0),
    ) && element_has_size(
        &elements,
        CashmallUiElement0104::Row(0),
        Vec2::new(433.0, 75.0),
    );
    let text_audit = audit_cashmall_text_0104(
        &all_texts,
        &styled_texts,
        &preview_assets,
        &language.effective,
    );
    if assets_loaded && !state.text_audited {
        match &text_audit {
            Ok(report) => {
                println!("{report}");
                state.text_audited = true;
            }
            Err(error) if state.frames % 120 == 0 => {
                eprintln!("Cash Mall text audit waiting: {error}")
            }
            Err(_) => {}
        }
    }
    let ready = assets_loaded
        && cpu_assets_present
        && root_exact
        && panels_exact
        && list_exact
        && text_audit.is_ok();
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
        eprintln!(
            "Cash Mall acceptance capture timed out: elements={}",
            elements.iter().count()
        );
        exit.write(AppExit::error());
    }
    std::thread::sleep(Duration::from_millis(1));
}

fn audit_cashmall_text_0104(
    all_texts: &Query<'_, '_, &Text>,
    styled_texts: &Query<
        '_,
        '_,
        (
            Entity,
            &Text,
            &LocalizedText,
            &CashmallTextStyle0104,
            (&TextFont, &LineHeight),
            &TextLayoutInfo,
            &ComputedNode,
            &UiTransform,
        ),
    >,
    assets: &PreviewAssets,
    language: &str,
) -> Result<String, String> {
    let all_count = all_texts.iter().count();
    let styled_count = styled_texts.iter().count();
    if all_count == 0 || styled_count != all_count {
        return Err(format!(
            "every-Text ownership incomplete: styled={styled_count}, all={all_count}"
        ));
    }

    let mut visible_count = 0usize;
    let mut glyph_count = 0usize;
    let mut line_count = 0usize;
    let mut visible_styles = Vec::new();
    let mut saw_cyrillic = false;
    let mut glyph_min_y = f32::INFINITY;
    let mut glyph_max_y = f32::NEG_INFINITY;

    for (entity, text, localized, style, font, layout, computed, transform) in styled_texts {
        if localized.key.trim().is_empty() {
            return Err(format!("Text {entity:?} has an empty semantic key"));
        }
        if font.0.font != bevy::text::FontSource::Handle(assets.font.clone()) {
            return Err(format!(
                "{} bypassed the approved JEFFE replacement font",
                localized.key
            ));
        }
        if font.0.font_size.eval(Vec2::ZERO, 16.0) != style.font_size()
            || (*font.1) != LineHeight::Px(style.line_height())
        {
            return Err(format!(
                "{} metrics size={} line={:?} do not match {style:?}",
                localized.key,
                font.0.font_size.eval(Vec2::ZERO, 16.0),
                (*font.1)
            ));
        }
        if transform.translation.y != px(CASHMALL_REPLACEMENT_FONT_Y_OFFSET) {
            return Err(format!(
                "{} has unproved replacement baseline offset {:?}",
                localized.key, transform.translation.y
            ));
        }
        if computed.size().x <= 0.0
            || computed.size().y <= 0.0
            || text.0.is_empty()
            || layout.glyphs.is_empty()
        {
            continue;
        }

        visible_count += 1;
        glyph_count += layout.glyphs.len();
        line_count += layout.run_geometry.len();
        saw_cyrillic |= text.0.chars().any(|character| {
            ('\u{0400}'..='\u{052f}').contains(&character)
                || ('\u{2de0}'..='\u{2dff}').contains(&character)
        });
        if !visible_styles.contains(style) {
            visible_styles.push(*style);
        }

        if !layout.size.is_finite()
            || layout.size.x > computed.size().x + 2.0
            || layout.size.y > computed.size().y + 2.0
        {
            return Err(format!(
                "{} GPU text layout {:?} exceeds source node {:?}",
                localized.key,
                layout.size,
                computed.size()
            ));
        }
        let mut minimum = Vec2::splat(f32::INFINITY);
        let mut maximum = Vec2::splat(f32::NEG_INFINITY);
        for glyph in &layout.glyphs {
            minimum = minimum.min(glyph.position - glyph.atlas_info.rect.size() * 0.5);
            maximum = maximum.max(glyph.position + glyph.atlas_info.rect.size() * 0.5);
        }
        if minimum.x < -2.0
            || minimum.y < -2.0
            || maximum.x > computed.size().x + 2.0
            || maximum.y > computed.size().y + 2.0
        {
            return Err(format!(
                "{} glyph bounds {:?}..{:?} exceed source node {:?}",
                localized.key,
                minimum,
                maximum,
                computed.size()
            ));
        }
        glyph_min_y = glyph_min_y.min(minimum.y);
        glyph_max_y = glyph_max_y.max(maximum.y);

        for run in &layout.run_geometry {
            let line = run.bounds;
            if !line.min.is_finite()
                || !line.max.is_finite()
                || line.min.y < -0.5
                || line.max.y > layout.size.y + 0.5
                || (line.height() - style.line_height()).abs() > 0.55
            {
                return Err(format!(
                    "{} invalid GPU line box {:?} for line height {}",
                    localized.key,
                    line,
                    style.line_height()
                ));
            }
        }
    }

    if visible_count == 0 || glyph_count == 0 || line_count == 0 {
        return Err("visible GPU text has not been laid out yet".to_owned());
    }
    for required in [
        CashmallTextStyle0104::LabelUpperLeft,
        CashmallTextStyle0104::BlankBoxUpperLeft,
        CashmallTextStyle0104::BlankBoxMiddleRight,
        CashmallTextStyle0104::ButtonMiddleCenter,
        CashmallTextStyle0104::EquipBarMiddleCenter,
        CashmallTextStyle0104::EquipFontMiddleRight,
    ] {
        if !visible_styles.contains(&required) {
            return Err(format!("no visible {required:?} text"));
        }
    }
    if language == "ru" && !saw_cyrillic {
        return Err("RU production bundle produced no visible Cyrillic glyphs".to_owned());
    }

    visible_styles.sort_by_key(|style| format!("{style:?}"));
    Ok(format!(
        "Cash Mall GPU text audit: locale={language}, key-first={styled_count}/{all_count}, \
         visible={visible_count}, glyphs={glyph_count}, line-boxes={line_count}, \
         glyph-y={glyph_min_y:.1}..{glyph_max_y:.1}, styles={visible_styles:?}"
    ))
}

fn element_has_size(
    elements: &Query<(&CashmallUiElement0104, &ComputedNode)>,
    wanted: CashmallUiElement0104,
    size: Vec2,
) -> bool {
    elements
        .iter()
        .any(|(element, node)| *element == wanted && node.size() == size)
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
            "rejecting Cash Mall capture with unexpected size {:?}",
            rgba.dimensions()
        );
        state.capture_failed = true;
        return;
    }

    let mut visible_pixels = 0;
    let mut checker_pixels = 0;
    for pixel in rgba.pixels() {
        let [red, green, blue, _alpha] = pixel.0;
        if u16::from(red) + u16::from(green) + u16::from(blue) > 35 {
            visible_pixels += 1;
        }
        if red > 180 && blue > 180 && green < 80 {
            checker_pixels += 1;
        }
    }
    if visible_pixels < MIN_VISIBLE_PIXELS || checker_pixels < MIN_MISSING_CHECKER_PIXELS {
        eprintln!(
            "rejecting incomplete Cash Mall capture: visible={visible_pixels} \
             (min {MIN_VISIBLE_PIXELS}), checker={checker_pixels} \
             (min {MIN_MISSING_CHECKER_PIXELS})"
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
    println!("{}", absolute_display(&output.0));
    state.capture_saved = true;
}

fn absolute_display(path: &Path) -> String {
    path.canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .display()
        .to_string()
}

#[cfg(test)]
mod tests;
