//! Production authority boundary for clean Retrobution player groups.
//!
//! The clean owner is `cnGroupManager` from the primary Retrobution
//! `Assembly-CSharp.dll`.  The pinned unmodified OpenFusion shard registers
//! exactly invite, refuse, join, and self-leave requests.  It does not expose
//! player kick, leader transfer, or group-owned block-list mutation packets.
//! This controller consequently keeps those actions typed but fail-closed.
//!
//! Roster packets are committed only after their local identity and transition
//! invariants have been checked.  Fixed replies without a wire request token
//! are correlated against the controller's single pending clean operation.
//! Unknown, malformed, and valid-but-unowned frames retain their complete raw
//! [`DecodedFrame`] for lossless downstream routing.

use std::{collections::BTreeMap, error::Error, fmt};

use bevy::prelude::Resource;
use ffone_protocol::{
    CountedPayloadError0104, DecodedFrame, GroupPacket0104, GroupPcMemberInfo0104, GroupRoster0104,
    PayloadError, RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104,
    decode_group_packet_0104,
};

pub const P_CL2FE_REQ_PC_GROUP_INVITE_0104: u32 = 0x1300_004c;
pub const P_CL2FE_REQ_PC_GROUP_INVITE_REFUSE_0104: u32 = 0x1300_004d;
pub const P_CL2FE_REQ_PC_GROUP_JOIN_0104: u32 = 0x1300_004e;
pub const P_CL2FE_REQ_PC_GROUP_LEAVE_0104: u32 = 0x1300_004f;

pub const P_FE2CL_PC_GROUP_INVITE_0104: u32 = 0x3100_0086;
pub const P_FE2CL_PC_GROUP_INVITE_FAIL_0104: u32 = 0x3100_0087;
pub const P_FE2CL_PC_GROUP_INVITE_REFUSE_0104: u32 = 0x3100_0088;
pub const P_FE2CL_PC_GROUP_JOIN_0104: u32 = 0x3100_0089;
pub const P_FE2CL_PC_GROUP_JOIN_FAIL_0104: u32 = 0x3100_008a;
pub const P_FE2CL_PC_GROUP_JOIN_SUCC_0104: u32 = 0x3100_008b;
pub const P_FE2CL_PC_GROUP_LEAVE_0104: u32 = 0x3100_008c;
pub const P_FE2CL_PC_GROUP_LEAVE_FAIL_0104: u32 = 0x3100_008d;
pub const P_FE2CL_PC_GROUP_LEAVE_SUCC_0104: u32 = 0x3100_008e;
pub const P_FE2CL_PC_GROUP_MEMBER_INFO_0104: u32 = 0x3100_008f;

pub const GROUP_INVITE_FAILURE_MESSAGE_ID_0104: i32 = 59;
pub const GROUP_JOIN_FAILURE_MESSAGE_ID_0104: i32 = 60;
pub const GROUP_INVITATION_MESSAGE_ID_0104: i32 = 61;
pub const GROUP_INVITE_DECLINED_MESSAGE_ID_0104: i32 = 62;
pub const GROUP_MEMBER_LEFT_MESSAGE_ID_0104: i32 = 63;
pub const GROUP_LOCAL_LEFT_MESSAGE_ID_0104: i32 = 147;
pub const GROUP_LEAVE_CONFIRMATION_MESSAGE_ID_0104: i32 = 214;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupRemotePlayer0104 {
    pub pc_id: i32,
    pub pc_uid: i64,
    pub display_name: String,
}

impl GroupRemotePlayer0104 {
    #[must_use]
    pub fn new(pc_id: i32, pc_uid: i64, display_name: impl Into<String>) -> Self {
        Self {
            pc_id,
            pc_uid,
            display_name: display_name.into(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GroupInboundAuthority0104 {
    /// Authoritative `cnOption.socialOption.bGroup` equivalent.
    pub allow_group_invites: bool,
    /// Authoritative Buddy-list block flag for the packet's host ID.
    pub host_blocked: bool,
    /// Live `UserContainer.SearchPlayer(host)` equivalent.  `None` preserves
    /// the clean silent branch for an out-of-scope or already-despawned host.
    pub visible_host: Option<GroupRemotePlayer0104>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupInviteResolution0104 {
    Accepted,
    Declined,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupInviteOwner0104 {
    PlayerMenu,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupLeaveOwner0104 {
    MenuChat,
    BuddyWarp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GroupOwnerReachability0104 {
    Proven,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnsupportedGroupAction0104 {
    KickMember,
    TransferLeader,
    MutateBlockList,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupActionError0104 {
    MissingLocalIdentity,
    InvalidRemotePlayer { pc_id: i32, pc_uid: i64 },
    ProductionOwnerUnavailable { owner: &'static str },
    OperationPending { operation: &'static str },
    StaleInvitation { host_pc_id: i32 },
    UnsupportedAction(UnsupportedGroupAction0104),
    Registry(RegisteredGameplayRequestError0104),
}

impl fmt::Display for GroupActionError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingLocalIdentity => write!(formatter, "group session has no local identity"),
            Self::InvalidRemotePlayer { pc_id, pc_uid } => write!(
                formatter,
                "group remote player identity is invalid: PC ID {pc_id}, PC UID {pc_uid}"
            ),
            Self::ProductionOwnerUnavailable { owner } => {
                write!(
                    formatter,
                    "clean group production owner {owner} is unavailable"
                )
            }
            Self::OperationPending { operation } => {
                write!(formatter, "group {operation} already has a pending request")
            }
            Self::StaleInvitation { host_pc_id } => {
                write!(
                    formatter,
                    "group invitation from PC {host_pc_id} is no longer pending"
                )
            }
            Self::UnsupportedAction(action) => {
                write!(
                    formatter,
                    "group action {action:?} has no supported 0104 route"
                )
            }
            Self::Registry(error) => {
                write!(formatter, "group request registry rejected body: {error}")
            }
        }
    }
}

impl Error for GroupActionError0104 {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Registry(error) => Some(error),
            _ => None,
        }
    }
}

impl From<RegisteredGameplayRequestError0104> for GroupActionError0104 {
    fn from(error: RegisteredGameplayRequestError0104) -> Self {
        Self::Registry(error)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupEffect0104 {
    IncomingInvitation {
        host: GroupRemotePlayer0104,
        message_id: i32,
    },
    SystemMessage {
        message_id: i32,
    },
    PassiveNotice {
        message_id: i32,
        arguments: Vec<String>,
    },
    PlayActionFailure,
    RosterCommitted(GroupRoster0104),
    RosterCleared,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GroupFrameOutput0104 {
    pub requests: Vec<RegisteredGameplayRequest0104>,
    pub effects: Vec<GroupEffect0104>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupFrameDecodeError0104 {
    Fixed(PayloadError),
    Roster(CountedPayloadError0104),
}

impl fmt::Display for GroupFrameDecodeError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed(error) => write!(formatter, "fixed group payload: {error}"),
            Self::Roster(error) => write!(formatter, "group roster payload: {error}"),
        }
    }
}

impl Error for GroupFrameDecodeError0104 {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Fixed(error) => Some(error),
            Self::Roster(error) => Some(error),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupFrameDisposition0104 {
    Passthrough(DecodedFrame),
    Malformed {
        frame: DecodedFrame,
        error: GroupFrameDecodeError0104,
    },
    Owned {
        frame: DecodedFrame,
        output: GroupFrameOutput0104,
    },
}

impl GroupFrameDisposition0104 {
    #[must_use]
    pub const fn owned(&self) -> bool {
        matches!(self, Self::Owned { .. })
    }

    #[must_use]
    pub const fn frame(&self) -> &DecodedFrame {
        match self {
            Self::Passthrough(frame)
            | Self::Malformed { frame, .. }
            | Self::Owned { frame, .. } => frame,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct GroupProductionRuntime0104 {
    local_pc_id: Option<i32>,
    local_pc_uid: Option<i64>,
    roster: Option<GroupRoster0104>,
    pending_incoming: BTreeMap<i32, GroupRemotePlayer0104>,
    pending_outgoing_invite: Option<GroupRemotePlayer0104>,
    pending_join: Option<GroupRemotePlayer0104>,
    pending_leave: Option<GroupLeaveOwner0104>,
}

impl GroupProductionRuntime0104 {
    pub fn begin_session(&mut self, local_pc_id: i32, local_pc_uid: i64) {
        self.reset();
        if local_pc_id > 0 && local_pc_uid > 0 {
            self.local_pc_id = Some(local_pc_id);
            self.local_pc_uid = Some(local_pc_uid);
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    #[must_use]
    pub const fn local_pc_id(&self) -> Option<i32> {
        self.local_pc_id
    }

    #[must_use]
    pub const fn local_pc_uid(&self) -> Option<i64> {
        self.local_pc_uid
    }

    #[must_use]
    pub const fn roster(&self) -> Option<&GroupRoster0104> {
        self.roster.as_ref()
    }

    #[must_use]
    pub fn pending_invitation(&self, host_pc_id: i32) -> Option<&GroupRemotePlayer0104> {
        self.pending_incoming.get(&host_pc_id)
    }

    #[must_use]
    pub const fn pending_leave_owner(&self) -> Option<GroupLeaveOwner0104> {
        self.pending_leave
    }

    pub fn request_invite(
        &mut self,
        target: GroupRemotePlayer0104,
        owner: GroupInviteOwner0104,
        reachability: GroupOwnerReachability0104,
    ) -> Result<RegisteredGameplayRequest0104, GroupActionError0104> {
        self.require_local_identity()?;
        if reachability != GroupOwnerReachability0104::Proven {
            return Err(GroupActionError0104::ProductionOwnerUnavailable {
                owner: match owner {
                    GroupInviteOwner0104::PlayerMenu => "NpcIcon player menu",
                },
            });
        }
        validate_remote(&target)?;
        if self.pending_outgoing_invite.is_some() {
            return Err(GroupActionError0104::OperationPending {
                operation: "invite",
            });
        }
        let request = registered_i32(P_CL2FE_REQ_PC_GROUP_INVITE_0104, target.pc_id)?;
        self.pending_outgoing_invite = Some(target);
        Ok(request)
    }

    pub fn resolve_invitation(
        &mut self,
        host_pc_id: i32,
        resolution: GroupInviteResolution0104,
    ) -> Result<RegisteredGameplayRequest0104, GroupActionError0104> {
        self.require_local_identity()?;
        let Some(host) = self.pending_incoming.get(&host_pc_id).cloned() else {
            return Err(GroupActionError0104::StaleInvitation { host_pc_id });
        };
        let (packet_type, pending_join) = match resolution {
            GroupInviteResolution0104::Accepted => {
                if self.pending_join.is_some() {
                    return Err(GroupActionError0104::OperationPending { operation: "join" });
                }
                (P_CL2FE_REQ_PC_GROUP_JOIN_0104, true)
            }
            GroupInviteResolution0104::Declined | GroupInviteResolution0104::TimedOut => {
                (P_CL2FE_REQ_PC_GROUP_INVITE_REFUSE_0104, false)
            }
        };
        let request = registered_i32(packet_type, host.pc_id)?;
        self.pending_incoming.remove(&host_pc_id);
        if pending_join {
            self.pending_join = Some(host);
        }
        Ok(request)
    }

    pub fn request_leave(
        &mut self,
        owner: GroupLeaveOwner0104,
        reachability: GroupOwnerReachability0104,
    ) -> Result<RegisteredGameplayRequest0104, GroupActionError0104> {
        self.require_local_identity()?;
        if reachability != GroupOwnerReachability0104::Proven {
            return Err(GroupActionError0104::ProductionOwnerUnavailable {
                owner: match owner {
                    GroupLeaveOwner0104::MenuChat => "MenuChat leave confirmation",
                    GroupLeaveOwner0104::BuddyWarp => "Buddy warp group-leave chain",
                },
            });
        }
        if self.pending_leave.is_some() {
            return Err(GroupActionError0104::OperationPending { operation: "leave" });
        }
        let request = RegisteredGameplayRequest0104::new(P_CL2FE_REQ_PC_GROUP_LEAVE_0104, vec![0])?;
        self.pending_leave = Some(owner);
        Ok(request)
    }

    pub fn unsupported_action(
        &self,
        action: UnsupportedGroupAction0104,
    ) -> Result<RegisteredGameplayRequest0104, GroupActionError0104> {
        Err(GroupActionError0104::UnsupportedAction(action))
    }

    /// Releases only the pending operation whose exact registered envelope
    /// could not be handed to the network worker.
    pub fn release_transport_failure(&mut self, request: &RegisteredGameplayRequest0104) {
        match request.packet_type() {
            P_CL2FE_REQ_PC_GROUP_INVITE_0104 => {
                if self
                    .pending_outgoing_invite
                    .as_ref()
                    .is_some_and(|target| request_i32(request) == Some(target.pc_id))
                {
                    self.pending_outgoing_invite = None;
                }
            }
            P_CL2FE_REQ_PC_GROUP_JOIN_0104 => {
                if self
                    .pending_join
                    .as_ref()
                    .is_some_and(|host| request_i32(request) == Some(host.pc_id))
                {
                    self.pending_join = None;
                }
            }
            P_CL2FE_REQ_PC_GROUP_LEAVE_0104 if request.payload() == [0] => {
                self.pending_leave = None;
            }
            _ => {}
        }
    }

    pub fn ingest(
        &mut self,
        frame: DecodedFrame,
        authority: GroupInboundAuthority0104,
    ) -> GroupFrameDisposition0104 {
        match decode_group_packet_0104(&frame) {
            Err(error) => {
                return GroupFrameDisposition0104::Malformed {
                    frame,
                    error: GroupFrameDecodeError0104::Roster(error),
                };
            }
            Ok(Some(packet)) => return self.ingest_roster_packet(frame, packet),
            Ok(None) => {}
        }

        match frame.packet_type {
            P_FE2CL_PC_GROUP_INVITE_0104 => {
                let host_pc_id = match decode_i32_exact(&frame.payload) {
                    Ok(value) => value,
                    Err(error) => return malformed_fixed(frame, error),
                };
                let Some(local_pc_id) = self.local_pc_id else {
                    return GroupFrameDisposition0104::Passthrough(frame);
                };
                if host_pc_id <= 0 || host_pc_id == local_pc_id {
                    return GroupFrameDisposition0104::Passthrough(frame);
                }
                let mut output = GroupFrameOutput0104::default();
                if !authority.allow_group_invites {
                    match registered_i32(P_CL2FE_REQ_PC_GROUP_INVITE_REFUSE_0104, host_pc_id) {
                        Ok(request) => output.requests.push(request),
                        Err(_) => return GroupFrameDisposition0104::Passthrough(frame),
                    }
                } else if !authority.host_blocked
                    && let Some(host) = authority
                        .visible_host
                        .filter(|host| host.pc_id == host_pc_id && validate_remote(host).is_ok())
                {
                    self.pending_incoming.insert(host_pc_id, host.clone());
                    output.effects.push(GroupEffect0104::IncomingInvitation {
                        host,
                        message_id: GROUP_INVITATION_MESSAGE_ID_0104,
                    });
                }
                owned(frame, output)
            }
            P_FE2CL_PC_GROUP_INVITE_FAIL_0104 => {
                if let Err(error) = decode_i32_exact(&frame.payload) {
                    return malformed_fixed(frame, error);
                }
                if self.pending_outgoing_invite.take().is_none() {
                    return GroupFrameDisposition0104::Passthrough(frame);
                }
                owned(
                    frame,
                    GroupFrameOutput0104 {
                        effects: vec![
                            GroupEffect0104::SystemMessage {
                                message_id: GROUP_INVITE_FAILURE_MESSAGE_ID_0104,
                            },
                            GroupEffect0104::PlayActionFailure,
                        ],
                        ..Default::default()
                    },
                )
            }
            P_FE2CL_PC_GROUP_INVITE_REFUSE_0104 => {
                let target_pc_id = match decode_i32_exact(&frame.payload) {
                    Ok(value) => value,
                    Err(error) => return malformed_fixed(frame, error),
                };
                let Some(target) = self
                    .pending_outgoing_invite
                    .as_ref()
                    .filter(|target| target.pc_id == target_pc_id)
                    .cloned()
                else {
                    return GroupFrameDisposition0104::Passthrough(frame);
                };
                self.pending_outgoing_invite = None;
                owned(
                    frame,
                    GroupFrameOutput0104 {
                        effects: vec![
                            GroupEffect0104::PassiveNotice {
                                message_id: GROUP_INVITE_DECLINED_MESSAGE_ID_0104,
                                arguments: vec![target.display_name],
                            },
                            GroupEffect0104::PlayActionFailure,
                        ],
                        ..Default::default()
                    },
                )
            }
            P_FE2CL_PC_GROUP_JOIN_FAIL_0104 => {
                let (reported_pc_id, _) = match decode_two_i32_exact(&frame.payload) {
                    Ok(values) => values,
                    Err(error) => return malformed_fixed(frame, error),
                };
                let Some(host) = self.pending_join.as_ref() else {
                    return GroupFrameDisposition0104::Passthrough(frame);
                };
                if reported_pc_id != 0
                    && Some(reported_pc_id) != self.local_pc_id
                    && reported_pc_id != host.pc_id
                {
                    return GroupFrameDisposition0104::Passthrough(frame);
                }
                self.pending_join = None;
                owned(
                    frame,
                    GroupFrameOutput0104 {
                        effects: vec![
                            GroupEffect0104::SystemMessage {
                                message_id: GROUP_JOIN_FAILURE_MESSAGE_ID_0104,
                            },
                            GroupEffect0104::PlayActionFailure,
                        ],
                        ..Default::default()
                    },
                )
            }
            P_FE2CL_PC_GROUP_LEAVE_FAIL_0104 => {
                let (reported_pc_id, _) = match decode_two_i32_exact(&frame.payload) {
                    Ok(values) => values,
                    Err(error) => return malformed_fixed(frame, error),
                };
                if self.pending_leave.is_none()
                    || (reported_pc_id != 0 && Some(reported_pc_id) != self.local_pc_id)
                {
                    return GroupFrameDisposition0104::Passthrough(frame);
                }
                self.pending_leave = None;
                owned(
                    frame,
                    GroupFrameOutput0104 {
                        effects: vec![GroupEffect0104::PlayActionFailure],
                        ..Default::default()
                    },
                )
            }
            _ => GroupFrameDisposition0104::Passthrough(frame),
        }
    }

    fn ingest_roster_packet(
        &mut self,
        frame: DecodedFrame,
        packet: GroupPacket0104,
    ) -> GroupFrameDisposition0104 {
        let Some(local_pc_id) = self.local_pc_id else {
            return GroupFrameDisposition0104::Passthrough(frame);
        };
        match packet {
            GroupPacket0104::LeaveSuccess => {
                let suppress_notice = self.pending_leave == Some(GroupLeaveOwner0104::BuddyWarp);
                self.roster = None;
                self.pending_leave = None;
                self.pending_join = None;
                self.pending_outgoing_invite = None;
                self.pending_incoming.clear();
                let mut effects = vec![GroupEffect0104::RosterCleared];
                if !suppress_notice {
                    effects.push(GroupEffect0104::PassiveNotice {
                        message_id: GROUP_LOCAL_LEFT_MESSAGE_ID_0104,
                        arguments: Vec::new(),
                    });
                }
                owned(
                    frame,
                    GroupFrameOutput0104 {
                        effects,
                        ..Default::default()
                    },
                )
            }
            GroupPacket0104::Roster(roster) => {
                let local_present = roster
                    .pc_members
                    .iter()
                    .any(|member| member.pc_id == local_pc_id);
                if !local_present {
                    return GroupFrameDisposition0104::Passthrough(frame);
                }
                let mut effects = Vec::new();
                match frame.packet_type {
                    P_FE2CL_PC_GROUP_JOIN_SUCC_0104 => {
                        let Some(host) = self.pending_join.as_ref() else {
                            return GroupFrameDisposition0104::Passthrough(frame);
                        };
                        if roster.context_id != local_pc_id
                            || !roster
                                .pc_members
                                .iter()
                                .any(|member| member.pc_id == host.pc_id)
                        {
                            return GroupFrameDisposition0104::Passthrough(frame);
                        }
                        self.pending_join = None;
                        self.pending_incoming.clear();
                    }
                    P_FE2CL_PC_GROUP_JOIN_0104 => {
                        if roster.context_id == local_pc_id
                            || !roster
                                .pc_members
                                .iter()
                                .any(|member| member.pc_id == roster.context_id)
                        {
                            return GroupFrameDisposition0104::Passthrough(frame);
                        }
                        if self
                            .pending_outgoing_invite
                            .as_ref()
                            .is_some_and(|target| target.pc_id == roster.context_id)
                        {
                            self.pending_outgoing_invite = None;
                        }
                    }
                    P_FE2CL_PC_GROUP_LEAVE_0104 => {
                        if roster
                            .pc_members
                            .iter()
                            .any(|member| member.pc_id == roster.context_id)
                        {
                            return GroupFrameDisposition0104::Passthrough(frame);
                        }
                        if let Some(name) = self
                            .roster
                            .as_ref()
                            .and_then(|previous| find_member(previous, roster.context_id))
                            .map(group_member_display_name)
                        {
                            effects.push(GroupEffect0104::PassiveNotice {
                                message_id: GROUP_MEMBER_LEFT_MESSAGE_ID_0104,
                                arguments: vec![name],
                            });
                        }
                    }
                    P_FE2CL_PC_GROUP_MEMBER_INFO_0104 => {
                        if roster.context_id != local_pc_id {
                            return GroupFrameDisposition0104::Passthrough(frame);
                        }
                    }
                    // NPC invite/kick success packets are already exact-decoded
                    // by `ffone-protocol`; local membership is their authority.
                    _ if roster.npc_id.is_some() => {}
                    _ => return GroupFrameDisposition0104::Passthrough(frame),
                }
                self.roster = Some(roster.clone());
                effects.push(GroupEffect0104::RosterCommitted(roster));
                owned(
                    frame,
                    GroupFrameOutput0104 {
                        effects,
                        ..Default::default()
                    },
                )
            }
        }
    }

    fn require_local_identity(&self) -> Result<(i32, i64), GroupActionError0104> {
        self.local_pc_id
            .zip(self.local_pc_uid)
            .ok_or(GroupActionError0104::MissingLocalIdentity)
    }
}

fn validate_remote(remote: &GroupRemotePlayer0104) -> Result<(), GroupActionError0104> {
    if remote.pc_id <= 0 || remote.pc_uid <= 0 {
        return Err(GroupActionError0104::InvalidRemotePlayer {
            pc_id: remote.pc_id,
            pc_uid: remote.pc_uid,
        });
    }
    Ok(())
}

fn registered_i32(
    packet_type: u32,
    value: i32,
) -> Result<RegisteredGameplayRequest0104, GroupActionError0104> {
    Ok(RegisteredGameplayRequest0104::new(
        packet_type,
        value.to_le_bytes().to_vec(),
    )?)
}

fn request_i32(request: &RegisteredGameplayRequest0104) -> Option<i32> {
    (request.payload().len() == 4).then(|| {
        i32::from_le_bytes(
            request
                .payload()
                .try_into()
                .expect("four-byte request payload was checked"),
        )
    })
}

fn decode_i32_exact(payload: &[u8]) -> Result<i32, PayloadError> {
    if payload.len() != 4 {
        return Err(PayloadError::WrongSize {
            expected: 4,
            actual: payload.len(),
        });
    }
    Ok(i32::from_le_bytes(
        payload.try_into().expect("four-byte payload was checked"),
    ))
}

fn decode_two_i32_exact(payload: &[u8]) -> Result<(i32, i32), PayloadError> {
    if payload.len() != 8 {
        return Err(PayloadError::WrongSize {
            expected: 8,
            actual: payload.len(),
        });
    }
    Ok((
        i32::from_le_bytes(payload[0..4].try_into().expect("first i32 is exact")),
        i32::from_le_bytes(payload[4..8].try_into().expect("second i32 is exact")),
    ))
}

fn malformed_fixed(frame: DecodedFrame, error: PayloadError) -> GroupFrameDisposition0104 {
    GroupFrameDisposition0104::Malformed {
        frame,
        error: GroupFrameDecodeError0104::Fixed(error),
    }
}

fn owned(frame: DecodedFrame, output: GroupFrameOutput0104) -> GroupFrameDisposition0104 {
    GroupFrameDisposition0104::Owned { frame, output }
}

fn find_member(roster: &GroupRoster0104, pc_id: i32) -> Option<&GroupPcMemberInfo0104> {
    roster
        .pc_members
        .iter()
        .find(|member| member.pc_id == pc_id)
}

fn group_member_display_name(member: &GroupPcMemberInfo0104) -> String {
    if member.name_check == 1 {
        let first = member.first_name.to_string_lossy();
        let last = member.last_name.to_string_lossy();
        let display_name = format!("{} {}", first.trim(), last.trim())
            .trim()
            .to_owned();
        if !display_name.is_empty() {
            return display_name;
        }
    }
    format!("Player {}", member.pc_uid)
}

#[cfg(test)]
mod tests;
