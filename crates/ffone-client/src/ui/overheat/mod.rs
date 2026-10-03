//! Dormant clean-Retrobution weapon-overheat model and HUD.
//!
//! The primary build serializes `CnGuiWpnOverheat` on `GameHUD`, but
//! `localized.WpnOverheat` is initialized to `eNone`. Consequently this
//! plugin is fail-closed by default. `OverheatParityPreview` is the explicit
//! opt-in used by isolated parity captures and tests; installing the plugin
//! alone never enables a UI that the clean client kept disabled.
//!
//! The module owns no camera, input, protocol, equipment, or table loader.
//! Those systems can populate the typed model and call its deterministic
//! state-transition methods without coupling the dormant slice to the main
//! client runtime.

use bevy::{
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};

use crate::mission_ui::{MissionUiModel, gameplay_chrome_visible};

pub const OVERHEAT_SOURCE_BUILD: &str = "retrobution-20260613";
pub const OVERHEAT_SOURCE_ARCHIVE: &str = "main.unity3d";
pub const OVERHEAT_SOURCE_ARCHIVE_SHA256: &str =
    "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F";
pub const OVERHEAT_SOURCE_SERIALIZED_FILE: &str = "sharedassets0.assets";
pub const OVERHEAT_TABLE_SOURCE_FILE: &str = "TableData.resourceFile";
pub const OVERHEAT_TABLE_SOURCE_SHA256: &str =
    "6D4CEA151152E2AB75B7D16590BDA00FBA172600A163E0B3A5318564DF0D7E3B";
pub const OVERHEAT_GAME_HUD_PATH_ID: i64 = 1_352;
pub const OVERHEAT_COMPONENT_PATH_ID: i64 = 1_569;
pub const OVERHEAT_HUD_SKIN_PATH_ID: i64 = 1_372;

pub const OVERHEAT_BACKGROUND_PATH: &str = "ui/en/gameplay/overheat/background.png";
pub const OVERHEAT_MAXIMUM_PATH: &str = "ui/en/gameplay/overheat/maximum.png";
pub const OVERHEAT_NORMAL_PATH: &str = "ui/en/gameplay/overheat/fill.png";

pub const OVERHEAT_BACKGROUND_PATH_ID: i64 = 162;
pub const OVERHEAT_MAXIMUM_PATH_ID: i64 = 206;
pub const OVERHEAT_NORMAL_PATH_ID: i64 = 669;

pub const OVERHEAT_BACKGROUND_SHA256: &str =
    "FC5E4960463A6243A4B81D23582E77E12DA474132BB6C41A2CD3C74031700332";
pub const OVERHEAT_MAXIMUM_SHA256: &str =
    "F50B7DF20A113E9B31F0B2393EF6C778AFF385F414A9279EF6DA15CE31CC234D";
pub const OVERHEAT_NORMAL_SHA256: &str =
    "BF921B23E18E4E30DB9DEFBD44214EEE597D422189443894B0950FCD68ABC7A2";

pub const OVERHEAT_BACKGROUND_WIDTH: u32 = 17;
pub const OVERHEAT_BACKGROUND_HEIGHT: u32 = 86;
pub const OVERHEAT_MAXIMUM_WIDTH: u32 = 22;
/// Native source texture height. The clean IMGUI call deliberately draws it
/// into a 95-pixel-high rectangle.
pub const OVERHEAT_MAXIMUM_TEXTURE_HEIGHT: u32 = 94;
pub const OVERHEAT_NORMAL_WIDTH: u32 = 6;
pub const OVERHEAT_NORMAL_HEIGHT: u32 = 80;

pub const OVERHEAT_UI_Z_INDEX: i32 = 10;
pub const OVERHEAT_GUI_STYLE_BORDER: BorderRect = BorderRect {
    min_inset: Vec2::new(8.0, 2.0),
    max_inset: Vec2::new(8.0, 4.0),
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OverheatTextureRole {
    Background,
    Maximum,
    NormalFill,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OverheatTextureContract {
    pub role: OverheatTextureRole,
    pub runtime_path: &'static str,
    pub sha256: &'static str,
    pub source_path_id: i64,
    pub source_width: u32,
    pub source_height: u32,
}

pub const OVERHEAT_TEXTURE_CONTRACTS: [OverheatTextureContract; 3] = [
    OverheatTextureContract {
        role: OverheatTextureRole::Background,
        runtime_path: OVERHEAT_BACKGROUND_PATH,
        sha256: OVERHEAT_BACKGROUND_SHA256,
        source_path_id: OVERHEAT_BACKGROUND_PATH_ID,
        source_width: OVERHEAT_BACKGROUND_WIDTH,
        source_height: OVERHEAT_BACKGROUND_HEIGHT,
    },
    OverheatTextureContract {
        role: OverheatTextureRole::Maximum,
        runtime_path: OVERHEAT_MAXIMUM_PATH,
        sha256: OVERHEAT_MAXIMUM_SHA256,
        source_path_id: OVERHEAT_MAXIMUM_PATH_ID,
        source_width: OVERHEAT_MAXIMUM_WIDTH,
        source_height: OVERHEAT_MAXIMUM_TEXTURE_HEIGHT,
    },
    OverheatTextureContract {
        role: OverheatTextureRole::NormalFill,
        runtime_path: OVERHEAT_NORMAL_PATH,
        sha256: OVERHEAT_NORMAL_SHA256,
        source_path_id: OVERHEAT_NORMAL_PATH_ID,
        source_width: OVERHEAT_NORMAL_WIDTH,
        source_height: OVERHEAT_NORMAL_HEIGHT,
    },
];

/// Exact clean `eWpnOverheat` values.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum LegacyWpnOverheatMode {
    #[default]
    None = 0,
    Use = 1,
    UseOriginal = 2,
}

impl LegacyWpnOverheatMode {
    /// All live `CnGuiWpnOverheat` initialization, update, and paint gates in
    /// the clean build compare the value with `eUse` (`1`) exactly.
    #[must_use]
    pub const fn enables_clean_component(self) -> bool {
        matches!(self, Self::Use)
    }
}

/// Runtime feature configuration. Its default mirrors the clean
/// `localized` static constructor and therefore keeps the component dormant.
#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct OverheatUiConfig {
    pub localized_mode: LegacyWpnOverheatMode,
    /// Legacy `FFGUIUtility.ScaleAroundPivot(ScreenPivot.Center)` scale.
    /// Clean display-option defaults yield `1.0`.
    pub ui_scale: f32,
}

impl Default for OverheatUiConfig {
    fn default() -> Self {
        Self {
            localized_mode: LegacyWpnOverheatMode::None,
            ui_scale: 1.0,
        }
    }
}

/// Explicit isolated-preview override. This does not mutate
/// `OverheatUiConfig::localized_mode`, so production state remains visibly
/// distinguishable from a parity/test opt-in.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct OverheatParityPreview {
    pub enabled: bool,
}

impl OverheatParityPreview {
    #[must_use]
    pub const fn enabled() -> Self {
        Self { enabled: true }
    }
}

#[must_use]
pub const fn overheat_component_enabled(
    config: OverheatUiConfig,
    preview: OverheatParityPreview,
) -> bool {
    config.localized_mode.enables_clean_component() || preview.enabled
}

/// Typed `m_pClassWpnTypeData` row used by the overheat state machine.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LegacyClassWeaponOverheatRow {
    pub weapon_type: i32,
    pub rate_of_fire: i32,
    pub overheat_max: i32,
    pub overheat_use: i32,
    pub unuse_cool_per_second: i32,
    pub use_cool_per_second: i32,
    pub cooldown_tenths: i32,
    pub overheat_effect: i32,
}

impl LegacyClassWeaponOverheatRow {
    #[must_use]
    pub fn cooldown_seconds(self) -> f32 {
        self.cooldown_tenths.max(0) as f32 * 0.1
    }

    #[must_use]
    pub const fn has_gauge(self) -> bool {
        self.overheat_max > 0
    }
}

pub const CLEAN_RETROBUTION_CLASS_WPN_TYPE_ROWS: [LegacyClassWeaponOverheatRow; 7] = [
    class_weapon_row(0, 0, 0, 0, 0, 0, 0, 0),
    class_weapon_row(1, 75, 100, 10, -35, -5, 20, 1),
    class_weapon_row(2, 75, 100, 12, -35, -5, 20, 1),
    class_weapon_row(3, 75, 100, 13, -35, -5, 30, 1),
    class_weapon_row(4, 75, 100, 16, -35, -5, 30, 1),
    class_weapon_row(5, 75, 100, 20, -35, -5, 40, 1),
    class_weapon_row(6, 75, 100, 35, -35, -5, 40, 1),
];

const fn class_weapon_row(
    weapon_type: i32,
    rate_of_fire: i32,
    overheat_max: i32,
    overheat_use: i32,
    unuse_cool_per_second: i32,
    use_cool_per_second: i32,
    cooldown_tenths: i32,
    overheat_effect: i32,
) -> LegacyClassWeaponOverheatRow {
    LegacyClassWeaponOverheatRow {
        weapon_type,
        rate_of_fire,
        overheat_max,
        overheat_use,
        unuse_cool_per_second,
        use_cool_per_second,
        cooldown_tenths,
        overheat_effect,
    }
}

/// Replaceable native table boundary, initialized with the seven clean rows.
#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct LegacyClassWeaponOverheatTable {
    pub rows: [LegacyClassWeaponOverheatRow; 7],
}

impl Default for LegacyClassWeaponOverheatTable {
    fn default() -> Self {
        Self {
            rows: CLEAN_RETROBUTION_CLASS_WPN_TYPE_ROWS,
        }
    }
}

impl LegacyClassWeaponOverheatTable {
    #[must_use]
    pub fn row(&self, weapon_type: i32) -> Option<&LegacyClassWeaponOverheatRow> {
        self.rows.iter().find(|row| row.weapon_type == weapon_type)
    }

    fn gauge_row(&self, slot: LegacyOverheatWeaponSlot) -> Option<&LegacyClassWeaponOverheatRow> {
        (slot.item_id > 0)
            .then_some(slot.weapon_type)
            .flatten()
            .and_then(|weapon_type| self.row(weapon_type))
            .filter(|row| row.has_gauge())
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LegacyOverheatWeaponSlot {
    /// Legacy equipment item ID. Values `<= 0` mean no equipped weapon.
    pub item_id: i16,
    /// `ItemElement.m_iTargetMode`, used as the ClassWpnType lookup key.
    pub weapon_type: Option<i32>,
    pub heat: f32,
}

impl LegacyOverheatWeaponSlot {
    #[must_use]
    pub const fn equipped(item_id: i16, weapon_type: i32, heat: f32) -> Self {
        Self {
            item_id,
            weapon_type: Some(weapon_type),
            heat,
        }
    }
}

/// Heat-transfer codes set by the legacy equipment-change path.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum LegacyOverheatWeaponSwitch {
    SwapMainAndBackup = 1,
    MoveMainToBackup = 2,
    MoveBackupToMain = 3,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Resource)]
pub struct OverheatUiModel {
    pub ready_for_play: bool,
    pub main: LegacyOverheatWeaponSlot,
    pub backup: LegacyOverheatWeaponSlot,
    /// Remaining time in `GameCondition` cooldown channel 17.
    pub main_use_cooldown_remaining: f32,
}

impl OverheatUiModel {
    #[must_use]
    pub fn main_fraction(self, table: &LegacyClassWeaponOverheatTable) -> f32 {
        let Some(row) = table.gauge_row(self.main) else {
            return 0.0;
        };
        if !self.main.heat.is_finite() {
            return 0.0;
        }
        (self.main.heat / row.overheat_max as f32).clamp(0.0, 1.0)
    }

    #[must_use]
    pub fn allows_attack(self, table: &LegacyClassWeaponOverheatTable) -> bool {
        let Some(row) = table.gauge_row(self.main) else {
            return true;
        };
        self.main.heat < row.overheat_max as f32
    }

    /// Mirrors one enabled `CnGuiWpnOverheat::Update`: the main weapon cools
    /// at `m_iUseCool` while channel 17 is active and at `m_iUnuseCool`
    /// otherwise; the backup always uses `m_iUnuseCool`.
    pub fn advance(&mut self, delta_seconds: f32, table: &LegacyClassWeaponOverheatTable) {
        if !delta_seconds.is_finite() || delta_seconds <= 0.0 {
            return;
        }

        let main_in_use_cooldown = self.main_use_cooldown_remaining > 0.0;
        self.main_use_cooldown_remaining =
            (self.main_use_cooldown_remaining - delta_seconds).max(0.0);

        if self.main.item_id > 0 {
            if let Some(row) = table.gauge_row(self.main) {
                let rate = if main_in_use_cooldown {
                    row.use_cool_per_second
                } else {
                    row.unuse_cool_per_second
                };
                self.main.heat = (self.main.heat + delta_seconds * rate as f32).max(0.0);
            } else {
                self.main.heat = 0.0;
                self.main_use_cooldown_remaining = 0.0;
            }
        }
        if self.backup.item_id > 0 {
            if let Some(row) = table.gauge_row(self.backup) {
                self.backup.heat =
                    (self.backup.heat + delta_seconds * row.unuse_cool_per_second as f32).max(0.0);
            } else {
                self.backup.heat = 0.0;
            }
        }
    }

    /// Records the overheat part of an already accepted primary attack.
    ///
    /// The enabled clean branch adds `Time.deltaTime` itself. It does not use
    /// the row's `m_iOverheatUse`, though that serialized field remains part
    /// of the source-owned table contract.
    pub fn record_accepted_attack(
        &mut self,
        frame_delta_seconds: f32,
        table: &LegacyClassWeaponOverheatTable,
    ) {
        let Some(row) = table.gauge_row(self.main).copied() else {
            self.main.heat = 0.0;
            self.main_use_cooldown_remaining = 0.0;
            return;
        };

        self.main_use_cooldown_remaining = row.cooldown_seconds();
        if frame_delta_seconds.is_finite() && frame_delta_seconds > 0.0 {
            self.main.heat += frame_delta_seconds;
        }
    }

    pub fn apply_weapon_switch(&mut self, transition: LegacyOverheatWeaponSwitch) {
        match transition {
            LegacyOverheatWeaponSwitch::SwapMainAndBackup => {
                std::mem::swap(&mut self.main.heat, &mut self.backup.heat);
            }
            LegacyOverheatWeaponSwitch::MoveMainToBackup => {
                self.backup.heat = self.main.heat;
                self.main.heat = 0.0;
            }
            LegacyOverheatWeaponSwitch::MoveBackupToMain => {
                self.main.heat = self.backup.heat;
                self.backup.heat = 0.0;
            }
        }
    }

    pub fn reset_main_for_replacement(&mut self) {
        self.main.heat = 0.0;
        self.main_use_cooldown_remaining = 0.0;
    }

    pub fn reset_backup_for_replacement(&mut self) {
        self.backup.heat = 0.0;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverheatUiRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl OverheatUiRect {
    #[must_use]
    pub const fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }

    fn node(self) -> Node {
        Node {
            position_type: PositionType::Absolute,
            left: px(self.left),
            top: px(self.top),
            width: px(self.width),
            height: px(self.height),
            ..default()
        }
    }

    fn scaled_around(self, pivot_x: f32, pivot_y: f32, scale: f32) -> Self {
        Self {
            left: pivot_x + (self.left - pivot_x) * scale,
            top: pivot_y + (self.top - pivot_y) * scale,
            width: self.width * scale,
            height: self.height * scale,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct OverheatUiView {
    pub visible: bool,
    pub fraction: f32,
    pub background: Option<OverheatUiRect>,
    pub normal_fill: Option<OverheatUiRect>,
    pub maximum: Option<OverheatUiRect>,
}

/// Produces the exact integer-half `Screen.width` / `Screen.height`
/// rectangles used by the clean IMGUI component.
#[must_use]
pub fn overheat_ui_view(
    viewport_width: u32,
    viewport_height: u32,
    config: OverheatUiConfig,
    preview: OverheatParityPreview,
    model: OverheatUiModel,
    table: &LegacyClassWeaponOverheatTable,
) -> OverheatUiView {
    let fraction = model.main_fraction(table);
    if !overheat_component_enabled(config, preview) || !model.ready_for_play || fraction <= 0.0 {
        return OverheatUiView::default();
    }

    // `Screen.width` and `Screen.height` are ints in the source, so odd
    // dimensions truncate before conversion to float.
    let center_x = (viewport_width / 2) as f32;
    let center_y = (viewport_height / 2) as f32;
    let scale = if config.ui_scale.is_finite() && config.ui_scale > 0.0 {
        config.ui_scale
    } else {
        1.0
    };
    let scale_rect = |rect: OverheatUiRect| rect.scaled_around(center_x, center_y, scale);
    let background = scale_rect(OverheatUiRect::new(center_x - 60.0, center_y, 17.0, 86.0));

    if fraction == 1.0 {
        return OverheatUiView {
            visible: true,
            fraction,
            background: Some(background),
            normal_fill: None,
            maximum: Some(scale_rect(OverheatUiRect::new(
                center_x - 61.0,
                center_y - 4.0,
                22.0,
                95.0,
            ))),
        };
    }

    let fill_height = 80.0 * fraction;
    OverheatUiView {
        visible: true,
        fraction,
        background: Some(background),
        normal_fill: Some(scale_rect(OverheatUiRect::new(
            center_x - 53.0,
            center_y + 4.0 + (80.0 - fill_height),
            6.0,
            fill_height,
        ))),
        maximum: None,
    }
}

#[derive(Clone, Resource)]
struct OverheatUiAssets {
    background: Handle<Image>,
    maximum: Handle<Image>,
    normal: Handle<Image>,
}

impl OverheatUiAssets {
    fn load(asset_server: &AssetServer) -> Self {
        Self {
            background: asset_server.load(OVERHEAT_BACKGROUND_PATH),
            maximum: asset_server.load(OVERHEAT_MAXIMUM_PATH),
            normal: asset_server.load(OVERHEAT_NORMAL_PATH),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OverheatUiRoot;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum OverheatUiElement {
    Background,
    NormalFill,
    Maximum,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum OverheatUiSet {
    Bind,
}

pub struct OverheatUiPlugin;

impl Plugin for OverheatUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<OverheatUiConfig>()
            .init_resource::<OverheatParityPreview>()
            .init_resource::<LegacyClassWeaponOverheatTable>()
            .init_resource::<OverheatUiModel>()
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_overheat_ui,
            )
            .add_systems(
                Update,
                (bind_overheat_ui.in_set(OverheatUiSet::Bind))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}

fn spawn_overheat_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = OverheatUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            OverheatUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
            ZIndex(OVERHEAT_UI_Z_INDEX),
        ))
        .with_children(|root| {
            root.spawn((
                OverheatUiElement::Background,
                OverheatUiRect::default().node(),
                styled_image(assets.background),
                Pickable::IGNORE,
            ));
            root.spawn((
                OverheatUiElement::NormalFill,
                OverheatUiRect::default().node(),
                stretched_image(assets.normal),
                Pickable::IGNORE,
            ));
            root.spawn((
                OverheatUiElement::Maximum,
                OverheatUiRect::default().node(),
                styled_image(assets.maximum),
                Pickable::IGNORE,
            ));
        });
}

fn styled_image(image: Handle<Image>) -> ImageNode {
    ImageNode {
        image,
        visual_box: bevy::ui::VisualBox::BorderBox,
        image_mode: NodeImageMode::Sliced(TextureSlicer {
            border: OVERHEAT_GUI_STYLE_BORDER,
            center_scale_mode: SliceScaleMode::Stretch,
            sides_scale_mode: SliceScaleMode::Stretch,
            max_corner_scale: 1.0,
        }),
        ..default()
    }
}

use crate::ui_support::stretched_image;

fn bind_overheat_ui(
    config: Res<OverheatUiConfig>,
    preview: Res<OverheatParityPreview>,
    model: Res<OverheatUiModel>,
    mission_ui: Option<Res<MissionUiModel>>,
    table: Res<LegacyClassWeaponOverheatTable>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<(&mut Node, &mut Visibility), With<OverheatUiRoot>>,
    mut elements: Query<(&OverheatUiElement, &mut Node), Without<OverheatUiRoot>>,
) {
    let Ok(window) = windows.single() else {
        for (_, mut visibility) in &mut roots {
            *visibility = Visibility::Hidden;
        }
        return;
    };
    let viewport_width = window.width().max(0.0) as u32;
    let viewport_height = window.height().max(0.0) as u32;
    let view = overheat_ui_view(
        viewport_width,
        viewport_height,
        *config,
        *preview,
        *model,
        &table,
    );

    for (mut node, mut visibility) in &mut roots {
        node.width = px(viewport_width);
        node.height = px(viewport_height);
        *visibility = if gameplay_chrome_visible(view.visible, mission_ui.as_deref()) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (element, mut node) in &mut elements {
        let rect = match element {
            OverheatUiElement::Background => view.background,
            OverheatUiElement::NormalFill => view.normal_fill,
            OverheatUiElement::Maximum => view.maximum,
        };
        bind_rect(&mut node, rect);
    }
}

fn bind_rect(node: &mut Node, rect: Option<OverheatUiRect>) {
    let Some(rect) = rect else {
        node.display = Display::None;
        return;
    };
    node.display = Display::Flex;
    node.left = px(rect.left);
    node.top = px(rect.top);
    node.width = px(rect.width);
    node.height = px(rect.height);
}

#[cfg(test)]
mod tests;
