use crate::app::*;
use ffone_client::gameplay_ui::{GameplayMenuTransition, NpcChatEvent};

mod types;
mod operations;
mod codec;
mod localization;
mod command_text;
mod systems;
mod state;
mod commands;

pub(super) use types::{SocialIngressPlugin, SocialIngress, PendingBuddyWarp};
use operations::send_player_emote_continuation;
pub(super) use operations::{
    legacy_chat_sender_name, push_world_chat_history, push_world_chat_line, receive_npc_chat,
    receive_nanocom_chat, local_freechat_line, group_freechat_line, buddy_freechat_line,
    buddy_entry_from_protocol, buddy_target_by_uid, push_buddy_presence_notice, queue_buddy_invite,
    consume_buddy_ui_outbox, consume_buddy_system_message_outbox, consume_buddy_nanocom_outbox,
    complete_pending_buddy_warp_after_authoritative_teleport
};
#[cfg(test)]
pub(super) use operations::buddy_nanocom_resolution_accepted;
#[cfg(test)]
pub(super) use operations::world_chat_line;
pub(super) use codec::{
    apply_group_and_freechat_frame, apply_buddy_chat_frame, apply_music_frame_0104,
    apply_server_message_frame_0104
};
#[cfg(test)]
pub(super) use codec::{
    apply_freechat_frame, apply_menu_chat_frame, apply_avatar_emote_frame,
    apply_all_group_freechat_frame, apply_all_group_menu_chat_frame, apply_buddy_menu_chat_frame
};
use localization::localized_player_chat_line;
pub(super) use systems::{
    apply_freechat_success, apply_all_group_freechat_success, apply_buddy_list_info,
    apply_buddy_freechat_success, sync_buddy_nanocom_context, sync_buddy_ui_context
};
use systems::{apply_avatar_emote_code, apply_buddy_menu_chat_success};
pub(super) use state::{apply_buddy_state_success, BuddyRuntimeIntegration};
pub(super) use commands::{PendingBuddySystemAction, PendingBuddyNanocomAction};
