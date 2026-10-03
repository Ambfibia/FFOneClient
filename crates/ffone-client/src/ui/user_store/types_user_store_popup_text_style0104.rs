use super::*;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UserStorePreviewMode0104 {
    #[default]
    MyReady,
    MyItems,
    MyOpen,
    UserList,
    UserSold,
    PopupQuantity,
    PopupPrice,
    PopupUnregister,
    PopupBuy,
    Busy,
    Error,
}

impl UserStorePreviewMode0104 {
    pub const ALL: [Self; 11] = [
        Self::MyReady,
        Self::MyItems,
        Self::MyOpen,
        Self::UserList,
        Self::UserSold,
        Self::PopupQuantity,
        Self::PopupPrice,
        Self::PopupUnregister,
        Self::PopupBuy,
        Self::Busy,
        Self::Error,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MyReady => "my-ready",
            Self::MyItems => "my-items",
            Self::MyOpen => "my-open",
            Self::UserList => "user-list",
            Self::UserSold => "user-sold",
            Self::PopupQuantity => "popup-quantity",
            Self::PopupPrice => "popup-price",
            Self::PopupUnregister => "popup-unregister",
            Self::PopupBuy => "popup-buy",
            Self::Busy => "busy",
            Self::Error => "error",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|mode| mode.as_str() == value)
    }
}

#[derive(Clone, Resource)]
pub(super) struct UserStoreUiAssets0104 {
    pub(super) images: [Handle<Image>; UserStoreAssetRole0104::COUNT],
    pub(super) font: Handle<Font>,
    pub(super) body_font: Handle<Font>,
    pub(super) missing_checker: Handle<Image>,
}

impl UserStoreUiAssets0104 {
    pub(super) fn load(asset_server: &AssetServer, images: &mut Assets<Image>) -> Self {
        Self {
            images: std::array::from_fn(|index| {
                asset_server.load(USER_STORE_IMAGE_ASSET_PATHS[index])
            }),
            font: asset_server.load(USER_STORE_FONT_PATH),
            body_font: asset_server.load(USER_STORE_BODY_FONT_PATH),
            missing_checker: images.add(user_store_missing_checker_image()),
        }
    }

    pub(super) fn image(&self, role: UserStoreAssetRole0104) -> Handle<Image> {
        self.images[role.index()].clone()
    }

    pub(super) fn readiness(&self, asset_server: &AssetServer) -> UserStoreStaticAssetReadiness0104 {
        if self
            .images
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
            || matches!(
                asset_server.load_state(self.font.id()),
                LoadState::Failed(_)
            )
            || matches!(
                asset_server.load_state(self.body_font.id()),
                LoadState::Failed(_)
            )
        {
            UserStoreStaticAssetReadiness0104::Failed
        } else if self
            .images
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
            && matches!(asset_server.load_state(self.font.id()), LoadState::Loaded)
            && matches!(
                asset_server.load_state(self.body_font.id()),
                LoadState::Loaded
            )
        {
            UserStoreStaticAssetReadiness0104::Ready
        } else {
            UserStoreStaticAssetReadiness0104::Loading
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct UserStoreUiRoot0104;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum UserStoreUiElement0104 {
    Root,
    Backdrop,
    StoreBackplate,
    RightBackplate,
    StorePanel,
    Info,
    Title,
    Dialog,
    ListBack,
    ListDivider,
    ItemTab,
    ItemTabLabel,
    Table,
    ListViewport,
    ListContent,
    Row(usize),
    RowIconFrame(usize),
    RowIcon(usize),
    RowCount(usize),
    RowName(usize),
    RowLevel(usize),
    RowPrice(usize),
    RowSellerNet(usize),
    TableShadow,
    PrimaryButton,
    GoToGameButton,
    PcStuffPanel,
    ItemTabPcStuff,
    InventoryViewport,
    InventoryContent,
    InventorySlotFrame(usize),
    InventorySlotIcon(usize),
    InventorySlotCount(usize),
    Close,
    Trash,
    Help,
    EquipmentPanel,
    EquipmentTitle,
    EquipmentSlot(usize),
    Popup,
    PopupIcon,
    PopupName,
    PopupCost,
    PopupDescription,
    PopupClose,
    PopupValueType,
    PopupCalculator,
    PopupValue,
    PopupDigit(u8),
    PopupClear,
    PopupNoOp,
    PopupAction,
    BusyLabel,
    ErrorLabel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum UserStoreUiTextRole0104 {
    Title,
    ItemTab,
    RowCount(usize),
    RowName(usize),
    RowLevel(usize),
    RowPrice(usize),
    RowSellerNet(usize),
    PrimaryButton,
    GoToGameButton,
    InventoryCount(usize),
    PopupName,
    PopupCost,
    PopupDescription,
    PopupValueType,
    PopupValue,
    PopupDigit(u8),
    PopupClear,
    PopupAction,
    Busy,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) enum UserStoreInteractiveControl0104 {
    Row(usize),
    InventorySlot(usize),
    PrimaryButton,
    GoToGame,
    Close,
    Help,
    PopupClose,
    PopupDigit(u8),
    PopupClear,
    PopupAction,
}

/// Exact clean `FusionFallInvenSkin` GUIStyle role for GumPopup text. The
/// replacement font changes glyph metrics, while the source size, line
/// spacing, padding, alignment and Rect stay authoritative.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Component)]
pub enum UserStorePopupTextStyle0104 {
    LabelUpperLeft,
    LabelMiddleRight,
    CenterLabel,
    CalculatorButton,
    Button,
}

impl UserStorePopupTextStyle0104 {
    pub(super) fn font(self, assets: &UserStoreUiAssets0104) -> (TextFont, LineHeight) {
        let (font, font_size, line_height) = match self {
            Self::LabelUpperLeft | Self::LabelMiddleRight => (
                assets.font.clone(),
                USER_STORE_JEFFE_12_FONT_SIZE,
                USER_STORE_JEFFE_12_LINE_HEIGHT,
            ),
            Self::CenterLabel => (
                assets.body_font.clone(),
                USER_STORE_CHALET_SMALL_FONT_SIZE,
                USER_STORE_CHALET_SMALL_LINE_HEIGHT,
            ),
            Self::CalculatorButton => (
                assets.font.clone(),
                USER_STORE_JEFFE_16_FONT_SIZE,
                USER_STORE_JEFFE_16_LINE_HEIGHT,
            ),
            Self::Button => (
                assets.font.clone(),
                USER_STORE_JEFFE_14_FONT_SIZE,
                USER_STORE_JEFFE_14_LINE_HEIGHT,
            ),
        };
        (
            TextFont {
                font: (font).into(),
                font_size: (font_size).into(),
                ..default()
            },
            LineHeight::Px(line_height),
        )
    }

    pub(super) fn apply_to_container(self, node: &mut Node) {
        node.justify_content = match self {
            Self::LabelUpperLeft => JustifyContent::FlexStart,
            Self::LabelMiddleRight => JustifyContent::FlexEnd,
            Self::CenterLabel | Self::CalculatorButton | Self::Button => JustifyContent::Center,
        };
        node.align_items = match self {
            Self::LabelUpperLeft | Self::CenterLabel => AlignItems::FlexStart,
            Self::LabelMiddleRight | Self::CalculatorButton | Self::Button => AlignItems::Center,
        };
        node.padding = match self {
            Self::LabelUpperLeft | Self::LabelMiddleRight => {
                UiRect::new(px(0), px(0), px(3), px(3))
            }
            Self::Button => UiRect::new(px(6), px(6), px(3), px(3)),
            Self::CenterLabel | Self::CalculatorButton => UiRect::ZERO,
        };
        node.overflow = Overflow::clip();
    }

    pub(super) fn layout(self) -> TextLayout {
        let justify = match self {
            Self::LabelUpperLeft => Justify::Left,
            Self::LabelMiddleRight => Justify::Right,
            Self::CenterLabel | Self::CalculatorButton | Self::Button => Justify::Center,
        };
        let linebreak = match self {
            Self::LabelUpperLeft | Self::CenterLabel => LineBreak::WordBoundary,
            Self::LabelMiddleRight | Self::CalculatorButton | Self::Button => LineBreak::NoWrap,
        };
        TextLayout::new(justify, linebreak)
    }

    pub(super) fn content_bounds(self, rect: UserStoreUiRect) -> Vec2 {
        let (horizontal, vertical) = match self {
            Self::LabelUpperLeft | Self::LabelMiddleRight => (0.0, 6.0),
            Self::Button => (12.0, 6.0),
            Self::CenterLabel | Self::CalculatorButton => (0.0, 0.0),
        };
        Vec2::new(
            (rect.width - horizontal).max(1.0),
            (rect.height - vertical).max(1.0),
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum UserStoreUiSet0104 {
    Lifecycle,
    Interaction,
    Bind,
}

#[derive(Default)]
pub struct UserStoreUiPlugin0104;

impl Plugin for UserStoreUiPlugin0104 {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<UserStoreUiState0104>()
            .init_resource::<UserStoreAuthority0104>()
            .init_resource::<UserStoreItemCatalog0104>()
            .init_resource::<UserStoreUiProjection0104>()
            .init_resource::<UserStoreUiOutbox0104>()
            .init_resource::<UserStorePopupPresentation0104>()
            .init_resource::<UserStoreUiAssetStatus0104>()
            .configure_sets(
                Update,
                (
                    UserStoreUiSet0104::Lifecycle,
                    UserStoreUiSet0104::Interaction,
                    UserStoreUiSet0104::Bind,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_user_store_ui_0104,
            )
            .add_systems(
                Update,
                (advance_user_store_ui_0104.in_set(UserStoreUiSet0104::Lifecycle))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (collect_user_store_input_0104.in_set(UserStoreUiSet0104::Interaction))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (bind_user_store_ui_0104
                    .in_set(UserStoreUiSet0104::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
