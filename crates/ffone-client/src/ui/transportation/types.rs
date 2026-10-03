use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransportationUiPoint {
    pub x: f32,
    pub y: f32,
}

impl TransportationUiPoint {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub(super) fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

/// Exact clean `FusionFallTransportSkin` text role carried by every reached
/// `cnTrans` `Text`. The replacement font changes glyph coverage only; source
/// font ownership, alignment, wrapping, padding, content offset and Rect stay
/// typed per serialized GUIStyle.
#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub enum TransportationUiTextStyle {
    BigFont14UpperLeft,
    BigFont16UpperLeft,
    RightTextUpperRight,
    ButtonMiddleCenter,
}

impl TransportationUiTextStyle {
    #[must_use]
    pub const fn source_style_name(self) -> &'static str {
        match self {
            Self::BigFont14UpperLeft => "bigfont14",
            Self::BigFont16UpperLeft => "bigfont16",
            Self::RightTextUpperRight => "rightText",
            Self::ButtonMiddleCenter => "button",
        }
    }

    #[must_use]
    pub const fn source_skin_path_id(self) -> i64 {
        RETROBUTION_TRANSPORT_SKIN_PATH_ID
    }

    #[must_use]
    pub const fn source_font_path_id(self) -> i64 {
        match self {
            Self::BigFont14UpperLeft | Self::RightTextUpperRight | Self::ButtonMiddleCenter => {
                RETROBUTION_TRANSPORT_BIGFONT14_PATH_ID
            }
            Self::BigFont16UpperLeft => RETROBUTION_TRANSPORT_BIGFONT16_PATH_ID,
        }
    }

    /// Unity `TextAnchor`: 0 UpperLeft, 2 UpperRight, 4 MiddleCenter.
    #[must_use]
    pub const fn legacy_alignment(self) -> i32 {
        match self {
            Self::BigFont14UpperLeft | Self::BigFont16UpperLeft => 0,
            Self::RightTextUpperRight => 2,
            Self::ButtonMiddleCenter => 4,
        }
    }

    /// Serialized `[left, right, top, bottom]` `RectOffset`.
    #[must_use]
    pub const fn padding(self) -> [f32; 4] {
        match self {
            Self::BigFont14UpperLeft | Self::BigFont16UpperLeft => [0.0; 4],
            Self::RightTextUpperRight => [2.0; 4],
            Self::ButtonMiddleCenter => [6.0, 6.0, 3.0, 6.0],
        }
    }

    #[must_use]
    pub const fn content_offset(self) -> [f32; 2] {
        [0.0, 0.0]
    }

    #[must_use]
    pub const fn word_wrap(self) -> bool {
        matches!(self, Self::BigFont16UpperLeft | Self::RightTextUpperRight)
    }

    #[must_use]
    pub const fn font_size(self) -> f32 {
        match self {
            Self::BigFont14UpperLeft | Self::RightTextUpperRight | Self::ButtonMiddleCenter => {
                TRANSPORTATION_JEFFE_14_FONT_SIZE
            }
            Self::BigFont16UpperLeft => TRANSPORTATION_JEFFE_16_FONT_SIZE,
        }
    }

    #[must_use]
    pub const fn line_height(self) -> f32 {
        match self {
            Self::BigFont14UpperLeft | Self::RightTextUpperRight | Self::ButtonMiddleCenter => {
                TRANSPORTATION_JEFFE_14_LINE_HEIGHT
            }
            Self::BigFont16UpperLeft => TRANSPORTATION_JEFFE_16_LINE_HEIGHT,
        }
    }

    #[must_use]
    pub const fn replacement_y_offset(self) -> f32 {
        match self {
            Self::BigFont14UpperLeft => TRANSPORTATION_BIGFONT14_REPLACEMENT_Y_OFFSET,
            Self::BigFont16UpperLeft => TRANSPORTATION_BIGFONT16_REPLACEMENT_Y_OFFSET,
            Self::RightTextUpperRight => TRANSPORTATION_RIGHT_TEXT_REPLACEMENT_Y_OFFSET,
            Self::ButtonMiddleCenter => TRANSPORTATION_BUTTON_REPLACEMENT_Y_OFFSET,
        }
    }

    pub(super) fn font(self, font: &Handle<Font>) -> (TextFont, LineHeight) {
        (
            TextFont {
                font: (font.clone()).into(),
                font_size: (self.font_size()).into(),
                ..default()
            },
            LineHeight::Px(self.line_height()),
        )
    }

    pub(super) fn layout(self) -> TextLayout {
        let justify = match self {
            Self::BigFont14UpperLeft | Self::BigFont16UpperLeft => Justify::Left,
            Self::RightTextUpperRight => Justify::Right,
            Self::ButtonMiddleCenter => Justify::Center,
        };
        TextLayout::new(
            justify,
            if self.word_wrap() {
                LineBreak::WordBoundary
            } else {
                LineBreak::NoWrap
            },
        )
    }

    pub(super) fn apply_to_container(self, node: &mut Node) {
        node.justify_content = match self {
            Self::BigFont14UpperLeft | Self::BigFont16UpperLeft => JustifyContent::FlexStart,
            Self::RightTextUpperRight => JustifyContent::FlexEnd,
            Self::ButtonMiddleCenter => JustifyContent::Center,
        };
        node.align_items = match self {
            Self::BigFont14UpperLeft | Self::BigFont16UpperLeft | Self::RightTextUpperRight => {
                AlignItems::FlexStart
            }
            Self::ButtonMiddleCenter => AlignItems::Center,
        };
        let [left, right, top, bottom] = self.padding();
        node.padding = UiRect::new(px(left), px(right), px(top), px(bottom));
        node.overflow = Overflow::clip();
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportationRegionProof {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub dong_name: &'static str,
    pub zone_name: &'static str,
}

impl TransportationRegionProof {
    pub(super) const fn new(
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        dong_name: &'static str,
        zone_name: &'static str,
    ) -> Self {
        Self {
            x,
            y,
            width,
            height,
            dong_name,
            zone_name,
        }
    }

    pub(super) fn contains(self, point: TransportationUiPoint) -> bool {
        self.width > 0
            && self.height > 0
            && point.x >= self.x as f32
            && point.x < (self.x + self.width) as f32
            && point.y >= self.y as f32
            && point.y < (self.y + self.height) as f32
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransportationLocation {
    pub row_index: usize,
    pub location_id: i32,
    pub npc_id: i32,
    pub server_x: i32,
    pub server_y: i32,
    pub server_z: i32,
    pub position: TransportationUiPoint,
    pub icon_index: i32,
    pub icon_number: i32,
    pub table_zone: i32,
    pub name: String,
    pub information: String,
    pub region: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TransportationWorldPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl TransportationWorldPoint {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub(super) fn is_finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TransportationUnlocks {
    pub warp_location_flags: u32,
    pub wyvern_location_flags: [u64; 2],
}

impl TransportationUnlocks {
    #[must_use]
    pub fn warp_registered(self, end_location: i32) -> bool {
        end_location > 0
            && self.warp_location_flags & 1_u32.wrapping_shl((end_location - 1) as u32) != 0
    }

    #[must_use]
    pub fn wyvern_registered(self, end_location: i32) -> bool {
        if end_location > 63 {
            self.wyvern_location_flags[1] & 1_u64.wrapping_shl((end_location - 64) as u32) != 0
        } else if end_location > 0 {
            self.wyvern_location_flags[0] & 1_u64.wrapping_shl((end_location - 1) as u32) != 0
        } else {
            false
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransportationPlayerSnapshot {
    pub position: TransportationWorldPoint,
    pub taros: i32,
    pub unlocks: TransportationUnlocks,
    pub cursor_was_locked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransportationTarget {
    Npc {
        npc_instance_id: i32,
        npc_table_id: i32,
        npc_position: TransportationWorldPoint,
        has_move_ok_voice: bool,
    },
    ItemUse {
        e_il: i32,
        slot_number: i32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransportationOpenContext {
    pub player: TransportationPlayerSnapshot,
    pub target: TransportationTarget,
}

impl TransportationOpenContext {
    pub(super) fn validate(self) -> Result<(), TransportationModelError> {
        if !self.player.position.is_finite() {
            return Err(TransportationModelError::NonFiniteInput);
        }
        if let TransportationTarget::Npc { npc_position, .. } = self.target
            && !npc_position.is_finite()
        {
            return Err(TransportationModelError::NonFiniteInput);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportationService {
    Warp = 1,
    Wyvern = 2,
    ItemUse = 3,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportationMap {
    Future,
    Darklands,
    PayZone,
}

impl TransportationMap {
    #[must_use]
    pub const fn asset_path(self) -> &'static str {
        match self {
            Self::Future => WORLD_MAP_FREEZONE_PATHS[1],
            Self::Darklands => WORLD_MAP_DARKLAND_PATHS[1],
            Self::PayZone => WORLD_MAP_PAYZONE_PATHS[3],
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct TransportationProjection {
    pub(super) service: TransportationService,
    pub(super) routes: Vec<TransportationRoute>,
    pub(super) start_position: TransportationUiPoint,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportationPhase {
    Hidden,
    Browsing,
    PendingWarp,
    AwaitingServer,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportationFade {
    WindowIn,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportationWarpReply0104 {
    Success {
        transportation_type: i32,
        position: [i32; 3],
        candy: i32,
    },
    Failure {
        transportation_id: i32,
        error_code: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportationRegistrationReply0104 {
    Success {
        transportation_type: i32,
        location_id: i32,
        unlocks: TransportationUnlocks,
    },
    Failure {
        transportation_type: i32,
        location_id: i32,
        error_code: i32,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportationMarkerKind {
    Route,
    Start,
    StartLabel,
    Selected,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransportationMarkerProjection {
    pub route_index: Option<usize>,
    pub rect: TransportationUiRect,
    pub asset_path: &'static str,
    pub kind: TransportationMarkerKind,
}

#[derive(Resource, Clone)]
pub(super) struct TransportationPresentationAssets {
    pub(super) images: BTreeMap<&'static str, Handle<Image>>,
    pub(super) jeffe: Handle<Font>,
}

impl TransportationPresentationAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            images: transportation_presentation_asset_paths()
                .into_iter()
                .map(|path| (path, asset_server.load(path)))
                .collect(),
            jeffe: asset_server.load(WORLD_MAP_JEFFE_FONT_PATH),
        }
    }

    pub(super) fn image(&self, path: &'static str) -> Handle<Image> {
        self.images.get(path).cloned().unwrap_or_default()
    }

    pub(super) fn failed_path(&self, asset_server: &AssetServer) -> Option<&'static str> {
        self.images
            .iter()
            .find_map(|(path, handle)| {
                matches!(asset_server.load_state(handle.id()), LoadState::Failed(_))
                    .then_some(*path)
            })
            .or_else(|| {
                matches!(
                    asset_server.load_state(self.jeffe.id()),
                    LoadState::Failed(_)
                )
                .then_some(WORLD_MAP_JEFFE_FONT_PATH)
            })
    }

    pub(super) fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.images
            .values()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
            && matches!(asset_server.load_state(self.jeffe.id()), LoadState::Loaded)
    }
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportationPresentationRoot;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportationPresentationWindow;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportationPresentationMap;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportationPresentationNpcCameraSlot;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportationPresentationMarker(pub TransportationMarkerKind);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportationPresentationControlNode(pub TransportationPresentationControl);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TransportationPresentationControl {
    Close,
    GoNow,
    Turbo,
    Route(usize),
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationBackdrop;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationMonkey;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationTurboLayer;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationTitle;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationSubtitle;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationWhereTo;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationGoLabel;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationTurboLabel;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationMarkerLayer;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationDynamicMarker;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct TransportationPresentationLineEffect(pub(super) u8);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum TransportationPresentationSet {
    Preload,
    Input,
    Bind,
}

/// Passive presentation plugin. It starts hidden, renders only the pure model,
/// and queues typed commands; it never mutates player currency or position.
pub struct TransportationUiPlugin;

impl Plugin for TransportationUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<TransportationModel>()
            .init_resource::<TransportationUiCopy>()
            .init_resource::<TransportationUiCommandOutbox>()
            .init_resource::<TransportationPresentationInput>()
            .init_resource::<TransportationPresentationAssetStatus>()
            .init_resource::<TransportationPresentationAnimation>()
            .configure_sets(
                Update,
                (
                    TransportationPresentationSet::Preload,
                    TransportationPresentationSet::Input,
                    TransportationPresentationSet::Bind,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_transportation_presentation,
            )
            .add_systems(
                Update,
                (update_transportation_asset_status.in_set(TransportationPresentationSet::Preload))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    queue_transportation_controls,
                    queue_transportation_escape,
                    queue_transportation_scroll,
                    advance_transportation_animation,
                )
                    .in_set(TransportationPresentationSet::Input))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    sync_transportation_shell,
                    sync_transportation_controls,
                    sync_transportation_text,
                    sync_transportation_routes,
                    sync_transportation_markers,
                    sync_transportation_animation,
                )
                    .in_set(TransportationPresentationSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
