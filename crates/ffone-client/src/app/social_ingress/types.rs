use super::*;

pub(in super::super) struct SocialIngressPlugin;

impl Plugin for SocialIngressPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BuddyRuntimeIntegration>()
            .init_resource::<GroupProductionRuntime0104>()
            .init_resource::<GroupRuntimeIntegration0104>()
            .init_resource::<WorldPc2pcOfferPrompt>()
            .init_resource::<PlayerFreeChatBubbleRuntime>()
            .init_resource::<ffone_client::player_emote::PlayerEmoteContinuation>()
            .add_systems(
                Update,
                send_player_emote_continuation
                    .after(ffone_client::player_emote::PlayerEmoteAdvance),
            )
            .add_systems(Update, receive_npc_chat.after(GameplayUiSet::NpcSpeech))
            .add_systems(Update, receive_nanocom_chat.after(receive_npc_chat))
            .add_systems(OnExit(ClientState::World), reset_group_session_0104)
            .add_systems(OnExit(ClientState::World), reset_world_pc2pc_session)
            .add_systems(
                Update,
                sync_buddy_nanocom_context
                    .after(TutorialPresentationSet::GameplayHud)
                    .after(sync_guide_ui_context)
                    .after(sync_vendor_ui_context)
                    .after(sync_upsell_ui_context)
                    .after(sync_nanocom_foreign_modal_suppression)
                    .after(consume_world_gameplay_ui_outbox)
                    .before(TutorialNanocomMessageSet::Tick)
                    .before(NanocomMessageUiSet::Tick),
            )
            .add_systems(
                Update,
                sync_buddy_ui_context
                    .after(TutorialPresentationSet::GameplayHud)
                    .after(sync_guide_ui_context)
                    .after(sync_vendor_ui_context)
                    .after(sync_upsell_ui_context)
                    .before(BuddyUiSet::Interaction)
                    .before(consume_group_system_message_outbox_0104)
                    .before(consume_group_nanocom_outbox_0104)
                    .before(consume_buddy_system_message_outbox)
                    .before(consume_buddy_nanocom_outbox)
                    .before(consume_buddy_ui_outbox)
                    .before(BuddyUiSet::Bind),
            )
            .add_systems(
                Update,
                consume_buddy_ui_outbox
                    .after(BuddyUiSet::Interaction)
                    .before(BuddyUiSet::Bind),
            )
            .add_systems(
                Update,
                consume_world_pc2pc_offer_prompt
                    .after(SystemMessageUiSet::Interaction)
                    .before(consume_buddy_system_message_outbox)
                    .run_if(in_state(ClientState::World)),
            )
            .add_systems(
                Update,
                consume_group_system_message_outbox_0104
                    .after(SystemMessageUiSet::Interaction)
                    .before(consume_buddy_system_message_outbox)
                    .run_if(in_state(ClientState::World)),
            )
            .add_systems(
                Update,
                consume_buddy_system_message_outbox
                    .after(SystemMessageUiSet::Interaction)
                    .after(consume_group_system_message_outbox_0104)
                    .before(consume_buddy_ui_outbox)
                    .before(BuddyUiSet::Bind),
            )
            .add_systems(
                Update,
                consume_group_nanocom_outbox_0104
                    .after(NanocomMessageUiSet::Input)
                    .after(consume_group_system_message_outbox_0104)
                    .before(consume_buddy_nanocom_outbox)
                    .run_if(in_state(ClientState::World)),
            )
            .add_systems(
                Update,
                consume_buddy_nanocom_outbox
                    .after(NanocomMessageUiSet::Input)
                    .after(consume_buddy_system_message_outbox)
                    .after(consume_group_nanocom_outbox_0104)
                    .before(consume_buddy_ui_outbox)
                    .before(BuddyUiSet::Bind),
            );
    }
}

#[derive(SystemParam)]
pub(in super::super) struct SocialIngress<'w> {
    pub(in super::super) email_catalog: Res<'w, EmailProductionCatalog0104>,
    pub(in super::super) email_inbox: ResMut<'w, EmailNetworkInbox0104>,
    pub(in super::super) buddy_ui: ResMut<'w, BuddyUiModel>,
    pub(in super::super) buddy_outbox: ResMut<'w, BuddyUiOutbox>,
    pub(in super::super) buddy_runtime: ResMut<'w, BuddyRuntimeIntegration>,
    pub(in super::super) nanocom_messages: ResMut<'w, NanocomMessageUiModel>,
    pub(in super::super) nanocom_outbox: ResMut<'w, NanocomMessageUiOutbox>,
    pub(in super::super) group_runtime: ResMut<'w, GroupProductionRuntime0104>,
    pub(in super::super) group_integration: ResMut<'w, GroupRuntimeIntegration0104>,
    pub(in super::super) group_ui: ResMut<'w, GroupUiModel>,
    pub(in super::super) pc2pc_offers: ResMut<'w, Pc2pcOfferRuntime0104>,
    pub(in super::super) pc2pc_ui: ResMut<'w, Pc2pcUiModel0104>,
    pub(in super::super) pc2pc_prompt: ResMut<'w, WorldPc2pcOfferPrompt>,
    pub(in super::super) freechat_bubbles: ResMut<'w, PlayerFreeChatBubbleRuntime>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in super::super) struct PendingBuddyWarp {
    pub(in super::super) target: BuddyTarget,
    pub(in super::super) target_was_group_member: bool,
}
