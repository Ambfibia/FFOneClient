use super::*;

pub(in super::super) struct VendorItemPopupPlugin;

impl Plugin for VendorItemPopupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VendorItemPopupState>()
            .init_resource::<VendorChestOpenState>()
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn.after(spawn_vendor_ui),
            )
            .add_systems(
                Update,
                reconcile
                    .after(advance_vendor_lifecycle)
                    .in_set(VendorUiSet::Lifecycle),
            )
            .add_systems(
                Update,
                (input, rotate_try_on)
                    .chain()
                    .after(collect_vendor_ui_input)
                    .in_set(VendorUiSet::Interaction),
            )
            .add_systems(
                Update,
                bind.in_set(VendorUiSet::Bind)
                    .in_set(crate::ui_startup::NativeUiStartupSet)
                    .before(LocalizationSet::Apply),
            );
    }
}

#[derive(Component, Clone, Copy)]
pub(super) enum Part {
    TryOn,
    TryPanel,
    TryAvatar,
    TryClose,
    TryLeft,
    TryRight,
    AmountLabel,
    Description,
    Info,
    Detail(u8),
    Root,
    Back,
    Icon,
    CombinedBadge,
    Name,
    Level,
    Amount,
    Pad,
    Accept,
    Delete,
    Close,
    Digit(u8),
    Clear,
}
