use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum EmailUiSet {
    Interaction,
    Bind,
    Visuals,
}

#[derive(Default)]
pub struct EmailUiPlugin;

impl Plugin for EmailUiPlugin {
    fn build(&self, app: &mut App) {
        body_scroll::install(app);
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<EmailUiModel>()
            .init_resource::<EmailUiOutbox>()
            .init_resource::<EmailTransportOutbox>()
            .init_resource::<EmailNetworkRuntime0104>()
            .init_resource::<EmailNetworkInbox0104>()
            .init_resource::<EmailUiAudioOutbox>()
            .init_resource::<EmailUiItemDragState>()
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_email_ui)
            .configure_sets(
                Update,
                (
                    EmailUiSet::Interaction,
                    EmailUiSet::Bind,
                    EmailUiSet::Visuals,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                ((
                    consume_email_network_inbox_0104,
                    flush_email_transport_outbox_0104,
                    handle_email_keyboard,
                    handle_email_buddy_scroll,
                    handle_email_item_interactions,
                    handle_email_interactions,
                )
                    .chain()
                    .in_set(EmailUiSet::Interaction))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((
                    tick_email_opening,
                    sync_email_ui_layout,
                    rebuild_email_buddy_rows,
                    sync_email_ui_model,
                    sync_email_guide_sender_icon,
                    sync_email_buddy_list_content,
                )
                    .chain()
                    .in_set(EmailUiSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((sync_email_ui_button_visuals, sync_email_buddy_row_visuals)
                    .in_set(EmailUiSet::Visuals))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
