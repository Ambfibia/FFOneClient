use super::*;

impl VendorUiAssets {
    pub(super) fn load(
        asset_server: &AssetServer,
        images: &mut Assets<Image>,
        contract: &VendorUiAssetContract,
    ) -> Self {
        Self {
            images: array::from_fn(|index| asset_server.load(contract.image_paths[index].clone())),
            font: asset_server.load(contract.font_path.clone()),
            service_font: asset_server.load(contract.service_font_path.clone()),
            missing_checker: images.add(Image::new(
                Extent3d {
                    width: USER_EQUIP_MISSING_CHECKER_SIZE,
                    height: USER_EQUIP_MISSING_CHECKER_SIZE,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                user_equip_missing_checker_rgba(),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
            )),
        }
    }

    pub(super) fn image(&self, role: VendorStaticAssetRole) -> Handle<Image> {
        self.images[role.index()].clone()
    }

    pub(super) fn readiness(&self, asset_server: &AssetServer) -> VendorStaticAssetReadiness {
        if self
            .images
            .iter()
            .any(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Failed(_)))
            || matches!(
                asset_server.load_state(self.font.id()),
                LoadState::Failed(_)
            )
            || matches!(
                asset_server.load_state(self.service_font.id()),
                LoadState::Failed(_)
            )
        {
            VendorStaticAssetReadiness::Failed
        } else if self
            .images
            .iter()
            .all(|handle| matches!(asset_server.load_state(handle.id()), LoadState::Loaded))
            && matches!(asset_server.load_state(self.font.id()), LoadState::Loaded)
            && matches!(
                asset_server.load_state(self.service_font.id()),
                LoadState::Loaded
            )
        {
            VendorStaticAssetReadiness::Ready
        } else {
            VendorStaticAssetReadiness::Loading
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct VendorUiRoot;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum VendorUiElement {
    Backdrop,
    VendorBackplate,
    RightBackplate,
    VendorPanel,
    NpcPreviewBoundary,
    VendorInfo,
    VendorTitle,
    VendorService,
    VendorDialog,
    ListBack,
    ListDivider,
    BuyTab,
    BuyTabSelected,
    BuyTabLabel,
    BuybackTab,
    BuybackTabSelected,
    BuybackTabLabel,
    Table,
    ListViewport,
    ListContent,
    Row(usize),
    RowFrameUnder(usize),
    RowIcon(usize),
    RowFrameOver(usize),
    RowCount(usize),
    RowName(usize),
    RowLevel(usize),
    RowVehicleSpeed(usize),
    RowPrice(usize),
    RowTaros(usize),
    TableShadow,
    ScrollTrack,
    ScrollThumb,
    ScrollUp,
    ScrollDown,
    GoToStuff,
    GoToStuffLabel,
    PcStuffPanel,
    ItemTabLabel,
    InventoryViewport,
    InventoryContent,
    InventoryShadow,
    InventorySlotFrame(usize),
    InventorySlotIcon(usize),
    InventorySlotBadge(usize),
    InventorySlotCount(usize),
    DexlabsBanner,
    RedeemCode,
    RedeemCodeLabel,
    TarosCounter,
    TarosDigit(usize),
    Close,
    Trash,
    Help,
    EquipmentPanel,
    EquipmentTitle,
    EquipmentTitleLabel,
    EquipmentSlotFrame(usize),
    EquipmentSlotIcon(usize),
    EquipmentSlotBadge(usize),
    EquipmentSlotLabel(usize),
    BatterySlotFrame(usize),
    BatteryIcon(usize),
    BatteryLabel(usize),
    BatteryCount(usize),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) enum VendorInteractiveControl {
    BuyTab,
    BuybackTab,
    Row(usize),
    InventorySlot(usize),
    GoToStuff,
    RedeemCode,
    Close,
    Help,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum VendorUiSet {
    Lifecycle,
    Interaction,
    Bind,
}

#[derive(Default)]
pub struct VendorUiPlugin;

impl Plugin for VendorUiPlugin {
    fn build(&self, app: &mut App) {
        crate::item_card::install(app);
        crate::service_scroll::install(app);
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<VendorUiState>()
            .add_plugins(item_popup::VendorItemPopupPlugin)
            .init_resource::<VendorModalState>()
            .init_resource::<VendorModeProjection0104>()
            .init_resource::<VendorUiOutbox0104>()
            .init_resource::<VendorUiAudioOutbox0104>()
            .init_resource::<VendorUiAssetContract>()
            .init_resource::<VendorUiAssetStatus>()
            .init_resource::<VendorHoverState>()
            .init_resource::<VendorCloseGate0104>()
            .configure_sets(
                Update,
                (
                    VendorUiSet::Lifecycle,
                    VendorUiSet::Interaction,
                    VendorUiSet::Bind,
                )
                    .chain(),
            )
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_vendor_ui)
            .add_systems(
                Update,
                (advance_vendor_lifecycle.in_set(VendorUiSet::Lifecycle))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (collect_vendor_ui_input.in_set(VendorUiSet::Interaction))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                (bind_vendor_ui
                    .in_set(VendorUiSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
