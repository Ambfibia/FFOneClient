use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum BankUiElement {
    Backdrop,
    BankBackplate,
    RightBackplate,
    BankPanel,
    BankInfo,
    BankTitle,
    BankDialog,
    BankTab,
    BankViewport,
    BankContent,
    BankSlotFrame(usize),
    BankSlotIcon(usize),
    BankSlotBadge(usize),
    BankSlotCount(usize),
    BankScrollTrack,
    BankScrollThumb,
    BankScrollUp,
    BankScrollDown,
    BankShadow,
    PcStuffPanel,
    ItemTabLabel,
    InventoryViewport,
    InventoryContent,
    InventorySlotFrame(usize),
    InventorySlotIcon(usize),
    InventorySlotBadge(usize),
    InventorySlotCount(usize),
    DexlabsBanner,
    TarosCounter,
    TarosDigit(usize),
    RedeemCode,
    RedeemCodeLabel,
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct BankTarosDigitText0104(pub(super) usize);

/// Clean-visible controls that do not yet have a production drag/popup owner.
/// They stay `Pickable::IGNORE`; typed methods on [`BankUiState`] remain the
/// only mutation-request boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum BankDisabledControl {
    BankSlot(usize),
    InventorySlot(usize),
    EquipmentSlot(usize),
    ActiveItemTab,
    RedeemCode,
    Trash,
    Help,
    ScrollUp,
    ScrollDown,
    ScrollThumb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct BankCloseControl;

#[derive(Component)]
pub(super) struct BankHelpControl;

#[derive(Component)]
pub(super) struct BankRedeemControl;

#[derive(Default, Resource)]
pub(super) struct BankLocalRequests {
    pub(super) full: Option<BankSlotLocation0104>,
}

#[derive(Clone, Copy, Debug, Component)]
pub(super) struct BankSlotControl(pub(super) BankSlotRef0104);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum BankUiSet {
    Lifecycle,
    Interaction,
    Bind,
}

#[derive(Default)]
pub struct BankUiPlugin;

impl Plugin for BankUiPlugin {
    fn build(&self, app: &mut App) {
        crate::item_card::install(app);
        crate::service_scroll::install(app);
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<BankUiState>()
            .init_resource::<BankSearch>()
            .init_resource::<BankLocalRequests>()
            .init_resource::<drag::DragVisual>()
            .init_resource::<BankItemPopupState>()
            .init_resource::<BankItemDeleteState>()
            .init_resource::<BankModalState>()
            .init_resource::<BankModeProjection0104>()
            .init_resource::<BankPcStuffAuthority0104>()
            .init_resource::<BankUiOutbox0104>()
            .init_resource::<BankUiAssetContract>()
            .configure_sets(
                Update,
                (
                    BankUiSet::Lifecycle,
                    BankUiSet::Interaction,
                    BankUiSet::Bind,
                )
                    .chain(),
            )
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                (
                    spawn_bank_ui,
                    drag::spawn,
                    item_popup::spawn.after(spawn_bank_ui),
                ),
            )
            .add_systems(
                Update,
                (advance_bank_lifecycle.in_set(BankUiSet::Lifecycle))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    item_delete::consume,
                    search::input,
                    item_popup::reconcile,
                    item_popup::input,
                    collect_bank_ui_input,
                    collect_bank_slot_input,
                    collect_bank_local_controls,
                )
                    .chain()
                    .in_set(BankUiSet::Interaction))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    bind_bank_ui,
                    bind_bank_button_labels.after(bind_bank_ui),
                    item_popup::bind,
                    drag::bind,
                )
                    .in_set(BankUiSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
