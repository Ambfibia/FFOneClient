use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[repr(i32)]
pub enum LegacyChatWindowStyle {
    /// Clean `cnDisplayOption.bNewChat` defaults to true and `Apply` maps it
    /// to the large chat style.
    #[default]
    Large = 0,
    Small = 1,
}

impl LegacyChatWindowStyle {
    #[must_use]
    pub const fn quick_slot_group_left(self) -> f32 {
        match self {
            Self::Large => QUICK_SLOT_LARGE_CHAT_GROUP_X,
            Self::Small => QUICK_SLOT_SMALL_CHAT_GROUP_X,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Resource)]
pub struct QuickSlotUiConfig {
    pub localized_mode: LegacyQuickSlotMode,
    pub macro_chat_mode: LegacyMacroChatMode,
    pub chat_window_style: LegacyChatWindowStyle,
    pub scale_ui: bool,
    pub ui_scale_factor: f32,
    pub ui_scale_override: Option<f32>,
}

impl Default for QuickSlotUiConfig {
    fn default() -> Self {
        Self {
            localized_mode: LegacyQuickSlotMode::None,
            macro_chat_mode: LegacyMacroChatMode::Use,
            chat_window_style: LegacyChatWindowStyle::Large,
            scale_ui: true,
            ui_scale_factor: QUICK_SLOT_SCALE_FACTOR,
            ui_scale_override: None,
        }
    }
}

impl QuickSlotUiConfig {
    #[must_use]
    pub fn effective_ui_scale(self, viewport_height: f32) -> f32 {
        if let Some(scale) = self.ui_scale_override {
            return valid_ui_scale(scale);
        }
        if !self.scale_ui {
            return 1.0;
        }
        clean_quick_slot_ui_scale(viewport_height, self.ui_scale_factor)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Resource)]
pub struct QuickSlotParityPreview {
    pub enabled: bool,
}

impl QuickSlotParityPreview {
    #[must_use]
    pub const fn enabled() -> Self {
        Self { enabled: true }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LegacyQuickSlotEntry {
    /// Exact `sQuickSlot.iType`. Clean production entries are GeneralItem
    /// (`7`); retaining the field is required for the inventory lookup rather
    /// than guessing from the table ID.
    pub item_type: i32,
    /// Legacy `InventoryManagerScript.QuickItem[index]`: an item table ID,
    /// not a stack count.
    pub item_id: i32,
    /// Native semantic icon route corresponding to legacy `QuickInfo.t`.
    pub icon_path: Option<String>,
    /// Exact legacy `QuickInfo.bEmpty` meaning: the quick-slot points at an
    /// item ID that the inventory lookup can no longer resolve.
    pub inventory_empty: bool,
    /// Remaining fraction returned by `IsItemCoolTimePercent`.
    pub cooldown_remaining_fraction: f32,
}

impl LegacyQuickSlotEntry {
    #[must_use]
    pub fn occupied(item_id: i32, icon_path: impl Into<String>) -> Self {
        Self {
            item_type: 7,
            item_id,
            icon_path: Some(icon_path.into()),
            ..default()
        }
    }

    #[must_use]
    pub fn has_item_id(&self) -> bool {
        self.item_id > 0
    }

    #[must_use]
    pub fn has_semantic_icon(&self) -> bool {
        self.icon_path.as_deref().is_some_and(is_semantic_png_path)
    }

    #[must_use]
    pub fn cooldown_fraction(&self) -> f32 {
        if !self.cooldown_remaining_fraction.is_finite() || self.cooldown_remaining_fraction <= 0.0
        {
            0.0
        } else {
            self.cooldown_remaining_fraction.min(1.0)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum QuickSlotActivationSource {
    Hotkey,
    Pointer,
}

#[derive(Default, Resource)]
pub struct QuickSlotUiOutbox {
    pub(super) actions: VecDeque<QuickSlotUiAction>,
}

impl QuickSlotUiOutbox {
    pub fn drain(&mut self) -> impl Iterator<Item = QuickSlotUiAction> + '_ {
        self.actions.drain(..)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum QuickSlotVisual {
    #[default]
    Empty,
    Occupied,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct QuickSlotView {
    pub logical_slot: usize,
    pub frame: QuickSlotUiRect,
    pub visual: QuickSlotVisual,
    pub pointer_enabled: bool,
    pub icon_visible: bool,
    pub cooldown: Option<QuickSlotUiRect>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct QuickSlotUiView {
    pub visible: bool,
    pub scale: f32,
    pub background: Option<QuickSlotUiRect>,
    pub slots: [QuickSlotView; QUICK_SLOT_COUNT],
}

impl Default for QuickSlotUiView {
    fn default() -> Self {
        Self {
            visible: false,
            scale: 1.0,
            background: None,
            slots: array::from_fn(|logical_slot| QuickSlotView {
                logical_slot,
                ..default()
            }),
        }
    }
}

#[derive(Clone, Resource)]
pub(super) struct QuickSlotUiAssets {
    pub(super) background: Handle<Image>,
    pub(super) occupied_style: Handle<Image>,
    pub(super) empty_style: Handle<Image>,
    pub(super) cooldown: Handle<Image>,
}

impl QuickSlotUiAssets {
    pub(super) fn load(asset_server: &AssetServer, paths: [&str; 4]) -> Self {
        Self {
            background: asset_server.load(paths[0].to_owned()),
            occupied_style: asset_server.load(paths[1].to_owned()),
            empty_style: asset_server.load(paths[2].to_owned()),
            cooldown: asset_server.load(paths[3].to_owned()),
        }
    }

    pub(super) fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        [
            &self.background,
            &self.occupied_style,
            &self.empty_style,
            &self.cooldown,
        ]
        .into_iter()
        .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct QuickSlotUiRoot;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum QuickSlotUiElement {
    Background,
    SlotFrame(usize),
    SlotIcon(usize),
    Cooldown(usize),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum QuickSlotUiSet {
    Input,
    Interaction,
    Bind,
}

#[derive(Default)]
pub struct QuickSlotUiPlugin;

impl Plugin for QuickSlotUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<QuickSlotUiConfig>()
            .init_resource::<QuickSlotParityPreview>()
            .init_resource::<QuickSlotUiAssetContract>()
            .init_resource::<QuickSlotUiModel>()
            .init_resource::<QuickSlotUiOutbox>()
            .configure_sets(
                Update,
                (
                    QuickSlotUiSet::Input,
                    QuickSlotUiSet::Interaction,
                    QuickSlotUiSet::Bind,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_quick_slot_ui,
            )
            .add_systems(
                Update,
                (read_quick_slot_hotkeys.in_set(QuickSlotUiSet::Input))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (handle_quick_slot_buttons.in_set(QuickSlotUiSet::Interaction))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (bind_quick_slot_ui.in_set(QuickSlotUiSet::Bind))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
