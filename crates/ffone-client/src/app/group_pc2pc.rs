//! Buddy/PC2PC offer prompts and group runtime integration.

use super::WorldSliceEntity;
use super::local_inventory::LocalInventoryRuntime;
use super::option_runtime::{OptionProductionRuntime, option_channel_gain};
use super::runtime_status::RuntimeStatus;
use bevy::{audio::Volume, prelude::*};
use ffone_client::{
    buddy_ui::BuddyUiModel,
    entity_lifecycle::NetworkPcAppearance0104,
    gameplay_ui::ChatChannel,
    group_runtime::{
        GROUP_INVITATION_MESSAGE_ID_0104, GroupEffect0104, GroupFrameDisposition0104,
        GroupInboundAuthority0104, GroupInviteResolution0104, GroupProductionRuntime0104,
        GroupRemotePlayer0104, P_FE2CL_PC_GROUP_INVITE_0104,
    },
    group_ui::{GroupNanoUi, GroupNpcMemberUi, GroupPcMemberUi, GroupUiModel},
    nanocom_message_ui::{
        NANOCOM_BUDDY_FRAME_PATH, NANOCOM_BUDDY_LIFETIME_SECONDS, NanocomMessageKind,
        NanocomMessageRequest, NanocomMessageResolution, NanocomMessageUiModel,
        NanocomMessageUiOutbox,
    },
    network::{NetworkBridge, NetworkCommand},
    pc2pc_ui::{
        Pc2pcAuthoritativeSnapshot0104, Pc2pcFailClosedEquipEligibility, Pc2pcOfferRuntime0104,
        Pc2pcParticipantNames0104, Pc2pcPendingOffer0104, Pc2pcSessionIdentity0104,
        Pc2pcUiModel0104,
    },
    system_message_ui::{
        SystemMessageButtonType, SystemMessageChoice, SystemMessageRequest, SystemMessageUiAction,
        SystemMessageUiModel, SystemMessageUiOutbox,
    },
    tutorial_mission_content::TutorialMissionContent,
};
use ffone_protocol::{DecodedFrame, GroupRoster0104};
use std::collections::{BTreeMap, BTreeSet};

pub(super) const BUDDY_SYSTEM_MESSAGE_ID_BASE: u64 = 0x4255_4444_0000_0000;
pub(super) const BUDDY_NANOCOM_MESSAGE_ID_BASE: u64 = 0x4E41_4E4F_0000_0000;
pub(super) const PC2PC_OFFER_SYSTEM_MESSAGE_ID_BASE: u64 = 0x5032_5043_0000_0000;

#[derive(Debug, Resource)]
pub(super) struct WorldPc2pcOfferPrompt {
    pub(super) pending: Option<(u64, Pc2pcSessionIdentity0104)>,
    pub(super) next_request_id: u64,
    pub(super) selected: Option<(u64, i32, i32, Entity, Entity)>,
    pub(super) selected_name: String,
}

impl Default for WorldPc2pcOfferPrompt {
    fn default() -> Self {
        Self {
            pending: None,
            selected: None,
            selected_name: String::new(),
            next_request_id: PC2PC_OFFER_SYSTEM_MESSAGE_ID_BASE,
        }
    }
}

impl WorldPc2pcOfferPrompt {
    pub(super) fn open(
        &mut self,
        identity: Pc2pcSessionIdentity0104,
        text: String,
        system_messages: &mut SystemMessageUiModel,
    ) {
        if self.pending.is_some_and(|(_, pending)| pending == identity) {
            return;
        }
        self.dismiss(system_messages);
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1);
        if self.next_request_id < PC2PC_OFFER_SYSTEM_MESSAGE_ID_BASE {
            self.next_request_id = PC2PC_OFFER_SYSTEM_MESSAGE_ID_BASE;
        }
        self.pending = Some((request_id, identity));
        system_messages.push(SystemMessageRequest::new(
            request_id,
            text,
            SystemMessageButtonType::YesNo,
        ));
    }

    pub(super) fn select(&mut self, local: i32, remote: i32, actor: Entity, entity: Entity, name: String, _messages: &mut SystemMessageUiModel) {
        if self.pending.is_some() || self.selected.is_some() { return; }
        let id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1);
        self.selected = Some((id, local, remote, actor, entity));
        self.selected_name = name;
    }

    pub(super) fn dismiss(&mut self, system_messages: &mut SystemMessageUiModel) {
        if let Some((id, ..)) = self.selected.take() { system_messages.remove(id); }
        if let Some((request_id, _)) = self.pending.take() {
            system_messages.remove(request_id);
        }
    }

    pub(super) fn reset(&mut self, system_messages: &mut SystemMessageUiModel) {
        self.dismiss(system_messages);
        self.next_request_id = PC2PC_OFFER_SYSTEM_MESSAGE_ID_BASE;
    }
}

pub(super) const GROUP_SYSTEM_MESSAGE_ID_BASE_0104: u64 = 0x4752_4F55_5053_0000;
pub(super) const GROUP_NANOCOM_MESSAGE_ID_BASE_0104: u64 = 0x4752_4F55_504E_0000;
pub(super) const GROUP_NANOCOM_ICON_PATH_0104: &str = "ui/en/gameplay/system/messicon_group.png";
pub(super) const GROUP_ACTION_FAILURE_AUDIO_PATH_0104: &str = "audio/sfx/ui/action_failure01.ogg";

#[derive(Debug, Resource)]
pub(super) struct GroupRuntimeIntegration0104 {
    pub(super) pending_system_messages: BTreeSet<u64>,
    pub(super) pending_nanocom_invites: BTreeMap<u64, i32>,
    pub(super) next_system_message_id: u64,
    pub(super) next_nanocom_message_id: u64,
}

impl Default for GroupRuntimeIntegration0104 {
    fn default() -> Self {
        Self {
            pending_system_messages: BTreeSet::new(),
            pending_nanocom_invites: BTreeMap::new(),
            next_system_message_id: GROUP_SYSTEM_MESSAGE_ID_BASE_0104,
            next_nanocom_message_id: GROUP_NANOCOM_MESSAGE_ID_BASE_0104,
        }
    }
}

impl GroupRuntimeIntegration0104 {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn next_system_message_id(&mut self) -> u64 {
        let request_id = self.next_system_message_id;
        self.next_system_message_id = self.next_system_message_id.wrapping_add(1);
        if self.next_system_message_id < GROUP_SYSTEM_MESSAGE_ID_BASE_0104 {
            self.next_system_message_id = GROUP_SYSTEM_MESSAGE_ID_BASE_0104;
        }
        request_id
    }

    pub(super) fn next_nanocom_message_id(&mut self) -> u64 {
        let request_id = self.next_nanocom_message_id;
        self.next_nanocom_message_id = self.next_nanocom_message_id.wrapping_add(1);
        if self.next_nanocom_message_id < GROUP_NANOCOM_MESSAGE_ID_BASE_0104 {
            self.next_nanocom_message_id = GROUP_NANOCOM_MESSAGE_ID_BASE_0104;
        }
        request_id
    }

    pub(super) fn push_system_message(
        &mut self,
        messages: &mut SystemMessageUiModel,
        text: impl Into<String>,
        button_type: SystemMessageButtonType,
    ) -> u64 {
        let request_id = self.next_system_message_id();
        self.pending_system_messages.insert(request_id);
        messages.push(SystemMessageRequest::new(request_id, text, button_type));
        request_id
    }

    pub(super) fn push_invitation(
        &mut self,
        messages: &mut NanocomMessageUiModel,
        host_pc_id: i32,
        body: impl Into<String>,
    ) -> u64 {
        let request_id = self.next_nanocom_message_id();
        self.pending_nanocom_invites.insert(request_id, host_pc_id);
        messages.enqueue(NanocomMessageRequest {
            request_id,
            kind: NanocomMessageKind::GroupInvite,
            // Clean type 14 owns no separate authored title string.  Keeping
            // this empty avoids borrowing BuddyInvite copy for Group.
            title: String::new(),
            body: body.into(),
            compact_frame_path: NANOCOM_BUDDY_FRAME_PATH.to_owned(),
            compact_icon_path: Some(GROUP_NANOCOM_ICON_PATH_0104.to_owned()),
            lifetime_seconds: NANOCOM_BUDDY_LIFETIME_SECONDS,
            localized_title: None,
            localized_body: None,
            voice_true_name: None,
            buddy_name: None,
        });
        request_id
    }
}

pub(super) fn reset_group_session_0104(
    mut production: ResMut<GroupProductionRuntime0104>,
    mut integration: ResMut<GroupRuntimeIntegration0104>,
    mut model: ResMut<GroupUiModel>,
    mut system_messages: ResMut<SystemMessageUiModel>,
) {
    for request_id in &integration.pending_system_messages {
        system_messages.remove(*request_id);
    }
    production.reset();
    integration.reset();
    *model = GroupUiModel::default();
}

pub(super) fn drain_world_pc2pc_offer_outbox(
    bridge: Res<NetworkBridge>,
    mut offers: ResMut<Pc2pcOfferRuntime0104>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    while let Some(request) = offers.pop_request() {
        let registered = match request.encode_registered() {
            Ok(request) => request,
            Err(error) => {
                offers.reset();
                runtime.message = format!("Pc2pcMode offer encoding failed: {error}");
                return;
            }
        };
        if let Err(error) = bridge.send(NetworkCommand::SendRegisteredGameplay0104(registered)) {
            // The request never reached the transport. Drop the optimistic
            // pending identity so a later TalkPlayer action can retry.
            offers.reset();
            runtime.message = format!("Pc2pcMode offer send failed: {error}");
            return;
        }
    }
}

pub(super) fn consume_world_pc2pc_offer_prompt(
    mut system_outbox: ResMut<SystemMessageUiOutbox>,
    mut prompt: ResMut<WorldPc2pcOfferPrompt>,
    mut offers: ResMut<Pc2pcOfferRuntime0104>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    let Some((request_id, expected_identity)) = prompt.pending else {
        return;
    };
    let actions = system_outbox.drain_matching(|action| {
        matches!(
            action,
            SystemMessageUiAction::Chosen {
                request_id: chosen_id,
                ..
            } if *chosen_id == request_id
        )
    });
    for action in actions {
        let SystemMessageUiAction::Chosen { choice, .. } = action;
        prompt.pending = None;
        if offers.pending() != Some(Pc2pcPendingOffer0104::Incoming(expected_identity)) {
            runtime.message = "Pc2pcMode ignored a stale trade-offer confirmation".to_owned();
            continue;
        }
        let result = if choice == SystemMessageChoice::Primary {
            offers.accept_incoming()
        } else {
            offers.refuse_incoming()
        };
        if let Err(error) = result {
            runtime.message = format!("Pc2pcMode response rejected: {error}");
        }
    }
}

pub(super) fn world_remote_pc_display_name(appearance: &NetworkPcAppearance0104) -> String {
    let first = appearance.0.style.first_name.to_string_lossy();
    let last = appearance.0.style.last_name.to_string_lossy();
    if appearance.0.style.name_check != 0 {
        format!("{first} {last}").trim().to_owned()
    } else {
        format!("Player {}", appearance.0.id)
    }
}

pub(super) fn open_world_pc2pc_accepted_session(
    identity: Pc2pcSessionIdentity0104,
    runtime: &RuntimeStatus,
    inventory: &LocalInventoryRuntime,
    content: &TutorialMissionContent,
    remote_appearances: &Query<&NetworkPcAppearance0104>,
    model: &mut Pc2pcUiModel0104,
) -> Result<(), String> {
    let inventory = inventory
        .snapshot()
        .ok_or_else(|| "trade accepted before authoritative inventory load".to_owned())?;
    let remote = remote_appearances
        .iter()
        .find(|appearance| appearance.0.id == identity.remote_pc_id)
        .ok_or_else(|| {
            format!(
                "trade accepted for remote PC {} without a live appearance",
                identity.remote_pc_id
            )
        })?;
    let remote_name = world_remote_pc_display_name(remote);
    let local_name = if runtime.player_name.trim().is_empty() {
        format!("Player {}", identity.local_pc_id)
    } else {
        runtime.player_name.clone()
    };
    let names = Pc2pcParticipantNames0104::new(local_name, remote_name)
        .map_err(|error| format!("trade participant names are invalid: {error:?}"))?;
    let snapshot = Pc2pcAuthoritativeSnapshot0104::from_accepted_trade(
        identity,
        names,
        runtime.candy,
        inventory,
    )
    .map_err(|error| format!("trade snapshot rejected: {error:?}"))?;
    model.begin_session(snapshot, content, &Pc2pcFailClosedEquipEligibility);
    Ok(())
}

pub(super) fn reset_world_pc2pc_session(
    mut offers: ResMut<Pc2pcOfferRuntime0104>,
    mut model: ResMut<Pc2pcUiModel0104>,
    mut prompt: ResMut<WorldPc2pcOfferPrompt>,
    mut system_messages: ResMut<SystemMessageUiModel>,
) {
    offers.reset();
    model.reset();
    prompt.reset(&mut system_messages);
}

pub(super) fn consume_group_system_message_outbox_0104(
    mut system_outbox: ResMut<SystemMessageUiOutbox>,
    mut integration: ResMut<GroupRuntimeIntegration0104>,
) {
    if integration.pending_system_messages.is_empty() {
        return;
    }
    let mut unrelated = Vec::new();
    for action in system_outbox.drain().collect::<Vec<_>>() {
        let SystemMessageUiAction::Chosen { request_id, .. } = action;
        if !integration.pending_system_messages.remove(&request_id) {
            unrelated.push(action);
        }
    }
    for action in unrelated {
        system_outbox.push(action);
    }
}

pub(super) fn consume_group_nanocom_outbox_0104(
    bridge: Res<NetworkBridge>,
    mut nanocom_outbox: ResMut<NanocomMessageUiOutbox>,
    mut production: ResMut<GroupProductionRuntime0104>,
    mut integration: ResMut<GroupRuntimeIntegration0104>,
    mut runtime: ResMut<RuntimeStatus>,
) {
    if integration.pending_nanocom_invites.is_empty() {
        return;
    }

    let mut unrelated = Vec::new();
    while let Some(action) = nanocom_outbox.pop_front() {
        if action.kind != NanocomMessageKind::GroupInvite {
            unrelated.push(action);
            continue;
        }
        let Some(host_pc_id) = integration
            .pending_nanocom_invites
            .remove(&action.request_id)
        else {
            unrelated.push(action);
            continue;
        };
        let resolution = match action.resolution {
            NanocomMessageResolution::Accepted => GroupInviteResolution0104::Accepted,
            NanocomMessageResolution::Declined => GroupInviteResolution0104::Declined,
            NanocomMessageResolution::TimedOut => GroupInviteResolution0104::TimedOut,
        };
        let request = match production.resolve_invitation(host_pc_id, resolution) {
            Ok(request) => request,
            Err(error) => {
                runtime.message = format!("Group invitation response ignored: {error}");
                continue;
            }
        };
        if let Err(error) = bridge.send(NetworkCommand::SendRegisteredGameplay0104(request.clone()))
        {
            production.release_transport_failure(&request);
            runtime.message =
                format!("Group invitation response for PC {host_pc_id} could not be sent: {error}");
        } else {
            runtime.message = match resolution {
                GroupInviteResolution0104::Accepted => {
                    format!("Joining the group invited by PC {host_pc_id}...")
                }
                GroupInviteResolution0104::Declined | GroupInviteResolution0104::TimedOut => {
                    format!("Group invitation from PC {host_pc_id} declined")
                }
            };
        }
    }
    for action in unrelated {
        nanocom_outbox.push(action);
    }
}

pub(super) fn group_ui_model_from_roster(
    local_pc_uid: Option<i64>,
    roster: GroupRoster0104,
    content: &TutorialMissionContent,
) -> GroupUiModel {
    let pc_members = roster
        .pc_members
        .into_iter()
        .map(|member| {
            let (first_name, last_name) = if member.name_check == 1 {
                (
                    member.first_name.to_string_lossy(),
                    member.last_name.to_string_lossy(),
                )
            } else {
                (String::new(), String::new())
            };
            let nano = (member.nano_active == 1 && member.nano.id > 0)
                .then(|| {
                    let definition = content.gameplay_nano(member.nano.id)?;
                    let journal = content.journal_nano(i32::from(member.nano.id))?;
                    let skill_icon_path =
                        content.gameplay_skill(member.nano.skill_id).map(|skill| {
                            format!(
                                "ui/en/gameplay/nano/icons/skill/skillicon_{:02}.png",
                                skill.icon_number
                            )
                        });
                    Some(GroupNanoUi {
                        name: journal.name.clone(),
                        stamina: i32::from(member.nano.stamina),
                        max_stamina: i32::from(definition.max_stamina),
                        skill_icon_path,
                    })
                })
                .flatten();
            GroupPcMemberUi {
                pc_id: member.pc_id,
                pc_uid: member.pc_uid,
                map_type: member.map_type,
                map_number: member.map_number,
                position: member.position,
                first_name,
                last_name,
                level: member.level,
                hp: member.hp,
                max_hp: member.max_hp,
                free_chat: member.special_state & 64 == 0,
                nano,
            }
        })
        .collect();
    let npc_members = roster
        .npc_members
        .into_iter()
        .filter_map(|member| {
            let definition = content.gameplay_npc(member.npc_type)?;
            (definition.max_hp > 0).then(|| GroupNpcMemberUi {
                npc_type: definition.npc_type,
                name: definition.name.clone(),
                hp: member.hp,
                max_hp: definition.max_hp,
            })
        })
        .collect();
    GroupUiModel {
        local_pc_uid,
        pc_members,
        npc_members,
    }
}

pub(super) fn format_clean_group_message_0104(template: &str, arguments: &[String]) -> String {
    let mut rendered = template.to_owned();
    for argument in arguments {
        let Some(offset) = rendered.find("%s") else {
            break;
        };
        rendered.replace_range(offset..offset + 2, argument);
    }
    rendered
}

pub(super) fn group_remote_player_0104(
    pc_id: i32,
    remote_appearances: &Query<&NetworkPcAppearance0104>,
) -> Option<GroupRemotePlayer0104> {
    remote_appearances
        .iter()
        .find(|appearance| appearance.0.id == pc_id)
        .map(|appearance| {
            GroupRemotePlayer0104::new(
                pc_id,
                appearance.0.style.pc_uid,
                world_remote_pc_display_name(appearance),
            )
        })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn apply_group_production_frame_0104(
    frame: &DecodedFrame,
    commands: &mut Commands,
    asset_server: &AssetServer,
    bridge: &NetworkBridge,
    content: &TutorialMissionContent,
    production: &mut GroupProductionRuntime0104,
    integration: &mut GroupRuntimeIntegration0104,
    model: &mut GroupUiModel,
    buddy_ui: &BuddyUiModel,
    nanocom_messages: &mut NanocomMessageUiModel,
    system_messages: &mut SystemMessageUiModel,
    remote_appearances: &Query<&NetworkPcAppearance0104>,
    option_runtime: &OptionProductionRuntime,
    runtime: &mut RuntimeStatus,
) -> Result<bool, String> {
    let authority = if frame.packet_type == P_FE2CL_PC_GROUP_INVITE_0104 && frame.payload.len() == 4
    {
        let host_pc_id = i32::from_le_bytes(
            frame
                .payload
                .as_slice()
                .try_into()
                .expect("four-byte group invitation was checked"),
        );
        GroupInboundAuthority0104 {
            allow_group_invites: option_runtime.options.social.allow_group_invites,
            host_blocked: buddy_ui.is_blocked_runtime_pc_id(host_pc_id),
            visible_host: group_remote_player_0104(host_pc_id, remote_appearances),
        }
    } else {
        GroupInboundAuthority0104::default()
    };

    let output = match production.ingest(frame.clone(), authority) {
        GroupFrameDisposition0104::Passthrough(_) => return Ok(false),
        GroupFrameDisposition0104::Malformed { error, .. } => {
            return Err(format!(
                "malformed protocol-0104 group packet 0x{:08x}: {error}",
                frame.packet_type
            ));
        }
        GroupFrameDisposition0104::Owned { output, .. } => output,
    };

    let mut transport_error = None;
    for request in output.requests {
        if let Err(error) = bridge.send(NetworkCommand::SendRegisteredGameplay0104(request.clone()))
        {
            production.release_transport_failure(&request);
            transport_error = Some(format!(
                "group request 0x{:08x} could not be sent: {error}",
                request.packet_type()
            ));
        }
    }

    for effect in output.effects {
        match effect {
            GroupEffect0104::IncomingInvitation { host, message_id } => {
                debug_assert_eq!(message_id, GROUP_INVITATION_MESSAGE_ID_0104);
                let definition =
                    content
                        .system_message_definition(message_id)
                        .ok_or_else(|| {
                            format!("Group invitation SystemMessage row {message_id} is missing")
                        })?;
                let body = format_clean_group_message_0104(
                    &definition.exact_text,
                    std::slice::from_ref(&host.display_name),
                );
                integration.push_invitation(nanocom_messages, host.pc_id, body);
            }
            GroupEffect0104::SystemMessage { message_id } => {
                let definition = content
                    .system_message_definition(message_id)
                    .ok_or_else(|| format!("Group SystemMessage row {message_id} is missing"))?;
                integration.push_system_message(
                    system_messages,
                    definition.exact_text.clone(),
                    definition.runtime_button_type,
                );
            }
            GroupEffect0104::PassiveNotice {
                message_id,
                arguments,
            } => {
                let definition = content
                    .system_message_definition(message_id)
                    .ok_or_else(|| format!("Group NanoCom notice row {message_id} is missing"))?;
                // Native Nanocom currently models interactive type 14 but not
                // clean passive button types 3/7.  Preserve the exact text in
                // the production status owner instead of fabricating a modal.
                runtime.message =
                    format_clean_group_message_0104(&definition.exact_text, &arguments);
            }
            GroupEffect0104::PlayActionFailure => {
                let gain = option_channel_gain(option_runtime.options.sound.effects);
                commands.spawn((
                    Name::new("Group Action_Failure01"),
                    WorldSliceEntity,
                    ffone_client::audio_channel::GameplayAudioChannel::ui_sfx(),
                    AudioPlayer::new(asset_server.load(GROUP_ACTION_FAILURE_AUDIO_PATH_0104)),
                    PlaybackSettings::DESPAWN.with_volume(Volume::Linear(0.7 * gain)),
                ));
            }
            GroupEffect0104::RosterCommitted(roster) => {
                let local_pc_uid = model.local_pc_uid.or(production.local_pc_uid());
                *model = group_ui_model_from_roster(local_pc_uid, roster, content);
            }
            GroupEffect0104::RosterCleared => {
                let local_pc_uid = model.local_pc_uid.or(production.local_pc_uid());
                *model = GroupUiModel {
                    local_pc_uid,
                    ..default()
                };
            }
        }
    }
    runtime.chat.world_group_available = model.pc_members.len() > 1;
    if !runtime.chat.world_group_available {
        runtime.chat.world_chat_alerts[ChatChannel::Group.index()] = false;
    }
    if let Some(error) = transport_error {
        return Err(error);
    }
    Ok(true)
}

pub(super) fn drain_world_pc2pc_session_outbox(
    bridge: Res<NetworkBridge>, mut model: ResMut<Pc2pcUiModel0104>, mut runtime: ResMut<RuntimeStatus>,
    messages: Res<SystemMessageUiModel>, mut modal: ResMut<ffone_client::pc2pc_ui::Pc2pcModalState>,
) {
    modal.system_popup_active = messages.is_popup();
    while let Some(intent) = model.outbox.pop_front() {
        let cancelled = matches!(intent, ffone_client::pc2pc_ui::Pc2pcIntent0104::Cancel(_));
        match intent.encode_registered() {
            Ok(request) => match bridge.send(NetworkCommand::SendRegisteredGameplay0104(request)) {
                Ok(()) if cancelled => { if let Some(identity) = model.identity() { let _ = model.acknowledge_local_cancel_sent(ffone_client::pc2pc_ui::Pc2pcEnvelope0104::local(identity)); } }
                Ok(()) => {}
                Err(error) => { model.state.pending = None; runtime.message = format!("Trade send failed: {error}"); }
            },
            Err(error) => { model.state.pending = None; runtime.message = format!("Trade encoding failed: {error}"); }
        }
    }
}
