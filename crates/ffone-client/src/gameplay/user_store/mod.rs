//! Production boundary for street stalls. The paired shard registers all
//! seven existing 0104 packets for administrative /Store sessions. Ordinary
//! inventory/player-menu entries retain their separate ownership gates.
//! The application's GM bridge owns reply correlation and inventory commits.

use std::{error::Error, fmt};

use bevy::prelude::Resource;
use ffone_protocol::{
    DecodedFrame, RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104,
};

use crate::user_store_ui::{
    STREETSTALL_REP_CANCEL_FAIL, STREETSTALL_REP_CANCEL_SUCCESS, STREETSTALL_REP_ITEM_BUY_FAIL,
    STREETSTALL_REP_ITEM_BUY_SUCCESS_BUYER, STREETSTALL_REP_ITEM_BUY_SUCCESS_SELLER,
    STREETSTALL_REP_ITEM_LIST, STREETSTALL_REP_ITEM_LIST_FAIL, STREETSTALL_REP_READY_FAIL,
    STREETSTALL_REP_READY_SUCCESS, STREETSTALL_REP_REGISTER_ITEM_FAIL,
    STREETSTALL_REP_REGISTER_ITEM_SUCCESS, STREETSTALL_REP_SALE_START_FAIL,
    STREETSTALL_REP_SALE_START_SUCCESS, STREETSTALL_REP_UNREGISTER_ITEM_FAIL,
    STREETSTALL_REP_UNREGISTER_ITEM_SUCCESS, STREETSTALL_REQ_CANCEL, STREETSTALL_REQ_ITEM_BUY,
    STREETSTALL_REQ_ITEM_LIST, STREETSTALL_REQ_READY, STREETSTALL_REQ_REGISTER_ITEM,
    STREETSTALL_REQ_SALE_START, STREETSTALL_REQ_UNREGISTER_ITEM, UserStoreAuthority0104,
    UserStorePacket0104, UserStorePopupPresentation0104, UserStoreUiOutbox0104,
    UserStoreUiState0104,
};

pub const USER_STORE_GENERAL_ITEM_OUTER_TYPE_0104: i16 = 7;
pub const USER_STORE_STREET_STALL_ITEM_SUBTYPE_0104: i32 = 11;
/// Clean `MapInfo & 1` ground-use requirement.
pub const USER_STORE_REQUIRED_MAP_MASK_0104: u32 = 1;
/// Clean `MapInfo & 4` exclusion.
pub const USER_STORE_FORBIDDEN_MAP_MASK_0104: u32 = 4;
pub const USER_STORE_MAX_GROUND_DELTA_0104: f32 = 0.5;
pub const USER_STORE_NEARBY_SQUARED_DISTANCE_0104: f32 = 1.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserStoreRequestCapability0104 {
    pub packet_type: u32,
    pub clean_name: &'static str,
    pub payload_size: usize,
}

/// The complete clean request family, implemented by the paired GMStore backend.
pub const USER_STORE_REQUEST_CAPABILITIES_0104: [UserStoreRequestCapability0104; 7] = [
    UserStoreRequestCapability0104 {
        packet_type: STREETSTALL_REQ_READY,
        clean_name: "P_CL2FE_PC_STREETSTALL_REQ_READY",
        payload_size: 4,
    },
    UserStoreRequestCapability0104 {
        packet_type: STREETSTALL_REQ_CANCEL,
        clean_name: "P_CL2FE_PC_STREETSTALL_REQ_CANCEL",
        payload_size: 4,
    },
    UserStoreRequestCapability0104 {
        packet_type: STREETSTALL_REQ_REGISTER_ITEM,
        clean_name: "P_CL2FE_PC_STREETSTALL_REQ_REGIST_ITEM",
        payload_size: 24,
    },
    UserStoreRequestCapability0104 {
        packet_type: STREETSTALL_REQ_UNREGISTER_ITEM,
        clean_name: "P_CL2FE_PC_STREETSTALL_REQ_UNREGIST_ITEM",
        payload_size: 4,
    },
    UserStoreRequestCapability0104 {
        packet_type: STREETSTALL_REQ_SALE_START,
        clean_name: "P_CL2FE_PC_STREETSTALL_REQ_SALE_START",
        payload_size: 4,
    },
    UserStoreRequestCapability0104 {
        packet_type: STREETSTALL_REQ_ITEM_LIST,
        clean_name: "P_CL2FE_PC_STREETSTALL_REQ_ITEM_LIST",
        payload_size: 4,
    },
    UserStoreRequestCapability0104 {
        packet_type: STREETSTALL_REQ_ITEM_BUY,
        clean_name: "P_CL2FE_PC_STREETSTALL_REQ_ITEM_BUY",
        payload_size: 12,
    },
];

pub const USER_STORE_REPLY_PACKET_TYPES_0104: [u32; 15] = [
    STREETSTALL_REP_READY_SUCCESS,
    STREETSTALL_REP_READY_FAIL,
    STREETSTALL_REP_CANCEL_SUCCESS,
    STREETSTALL_REP_CANCEL_FAIL,
    STREETSTALL_REP_REGISTER_ITEM_SUCCESS,
    STREETSTALL_REP_REGISTER_ITEM_FAIL,
    STREETSTALL_REP_UNREGISTER_ITEM_SUCCESS,
    STREETSTALL_REP_UNREGISTER_ITEM_FAIL,
    STREETSTALL_REP_SALE_START_SUCCESS,
    STREETSTALL_REP_SALE_START_FAIL,
    STREETSTALL_REP_ITEM_LIST,
    STREETSTALL_REP_ITEM_LIST_FAIL,
    STREETSTALL_REP_ITEM_BUY_SUCCESS_BUYER,
    STREETSTALL_REP_ITEM_BUY_SUCCESS_SELLER,
    STREETSTALL_REP_ITEM_BUY_FAIL,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserStoreEntryOwner0104 {
    SelectablePlayerMenu,
    InventoryUse,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserStoreOwnerReachability0104 {
    Proven,
    Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserStoreOtherPlayerEntry0104 {
    pub owner_pc_id: i32,
    pub target_pc_id: i32,
    /// Authoritative clean `Status.storeState == eStoreOpen` feed.
    pub authoritative_store_open: bool,
    pub player_menu_owner: UserStoreOwnerReachability0104,
    pub previous_cursor_locked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UserStoreOwnEntry0104 {
    pub owner_pc_id: i32,
    pub inventory_slot: i32,
    pub item_type: i16,
    /// Exact GeneralItem table 27 `m_iItemType` value.
    pub general_item_type: Option<i32>,
    pub map_flags: u32,
    pub player_y: f32,
    pub map_height: f32,
    pub nearest_other_player_squared_distance: Option<f32>,
    pub nearest_store_squared_distance: Option<f32>,
    pub inventory_use_owner: UserStoreOwnerReachability0104,
    pub previous_cursor_locked: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UserStoreEntryGate0104 {
    OwnerUnavailable(UserStoreEntryOwner0104),
    InvalidOwnerPcId(i32),
    InvalidTargetPcId(i32),
    TargetIsOwner,
    TargetStoreClosed,
    InvalidInventorySlot(i32),
    WrongOuterItemType(i16),
    WrongGeneralItemType(Option<i32>),
    RequiredMapFlagMissing,
    ForbiddenMapFlagSet,
    NonFiniteGroundHeight,
    TooFarFromMapHeight { absolute_delta: f32 },
    InvalidNearbyDistance,
    NearbyPlayer { squared_distance: f32 },
    NearbyStore { squared_distance: f32 },
}

impl fmt::Display for UserStoreEntryGate0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "clean UserStore entry gate rejected: {self:?}")
    }
}

impl Error for UserStoreEntryGate0104 {}

#[derive(Clone, Debug, PartialEq)]
pub enum UserStoreProductionError0104 {
    Entry(UserStoreEntryGate0104),
    Registry(RegisteredGameplayRequestError0104),
    UnexpectedRequestPacket { packet_type: u32 },
    NoRegisteredProductionSession,
    ProductionOwnerBridgeUnavailable,
}

impl fmt::Display for UserStoreProductionError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Entry(error) => write!(formatter, "{error}"),
            Self::Registry(error) => write!(
                formatter,
                "UserStore request has no registered OpenFusion production handler: {error}"
            ),
            Self::UnexpectedRequestPacket { packet_type } => write!(
                formatter,
                "UserStore outbox produced non-street-stall packet {packet_type:#010x}"
            ),
            Self::NoRegisteredProductionSession => {
                formatter.write_str("UserStore UI has no registry-proven production session")
            }
            Self::ProductionOwnerBridgeUnavailable => {
                formatter.write_str("UserStore production entry owner/feed bridge is unavailable")
            }
        }
    }
}

impl Error for UserStoreProductionError0104 {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Entry(error) => Some(error),
            Self::Registry(error) => Some(error),
            _ => None,
        }
    }
}

impl From<UserStoreEntryGate0104> for UserStoreProductionError0104 {
    fn from(error: UserStoreEntryGate0104) -> Self {
        Self::Entry(error)
    }
}

impl From<RegisteredGameplayRequestError0104> for UserStoreProductionError0104 {
    fn from(error: RegisteredGameplayRequestError0104) -> Self {
        Self::Registry(error)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UserStoreGuardRejection0104 {
    pub dropped_commands: usize,
    pub error: UserStoreProductionError0104,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserStoreInboundPassthroughReason0104 {
    UnrelatedPacket,
    NoRegisteredRequestOwner,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserStoreInboundPassthrough0104 {
    pub frame: DecodedFrame,
    pub reason: UserStoreInboundPassthroughReason0104,
}

#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct UserStoreProductionRuntime0104 {
    /// Explicit GM chat entry; ordinary inventory/menu entries still require
    /// their independently validated map and consumable-owner bridges.
    pub gm_owner: Option<i32>,
    last_rejection: Option<UserStoreGuardRejection0104>,
    rejected_boundaries: u64,
}

impl UserStoreProductionRuntime0104 {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    #[must_use]
    pub const fn last_rejection(&self) -> Option<&UserStoreGuardRejection0104> {
        self.last_rejection.as_ref()
    }

    #[must_use]
    pub const fn rejected_boundaries(&self) -> u64 {
        self.rejected_boundaries
    }

    /// Registry-probes the clean other-player entry packet before consulting
    /// gates or touching any state. Ordinary player-menu ownership remains
    /// separate from the administrative chat session.
    pub fn attempt_other_player_entry(
        &mut self,
        state: &mut UserStoreUiState0104,
        authority: &mut UserStoreAuthority0104,
        outbox: &mut UserStoreUiOutbox0104,
        context: UserStoreOtherPlayerEntry0104,
    ) -> Result<RegisteredGameplayRequest0104, UserStoreProductionError0104> {
        let packet = UserStorePacket0104::item_list(context.target_pc_id);
        let registered = prove_user_store_packet_0104(&packet)?;
        validate_other_player_entry_0104(context)?;
        // The production player-menu/storeState owners are not yet present in
        // main.  Keep this explicit even if a future backend adds the packet.
        let _ = (state, authority, outbox, registered);
        Err(UserStoreProductionError0104::ProductionOwnerBridgeUnavailable)
    }

    /// Registry-probes READY before the inventory/table/map proximity gates
    /// and before any `begin_my_store` mutation.
    pub fn attempt_own_entry(
        &mut self,
        state: &mut UserStoreUiState0104,
        authority: &mut UserStoreAuthority0104,
        outbox: &mut UserStoreUiOutbox0104,
        context: UserStoreOwnEntry0104,
    ) -> Result<RegisteredGameplayRequest0104, UserStoreProductionError0104> {
        let packet = UserStorePacket0104::ready(context.inventory_slot);
        let registered = prove_user_store_packet_0104(&packet)?;
        validate_own_entry_0104(context)?;
        let _ = (state, authority, outbox, registered);
        Err(UserStoreProductionError0104::ProductionOwnerBridgeUnavailable)
    }

    /// Rejects a manually activated or otherwise unowned UI boundary.  Raw UI
    /// packets are registry-probed only to produce the exact typed blocker;
    /// they are never returned to transport.  The shell and popup are reset
    /// before the production presentation bind runs.
    pub fn guard_unowned_ui(
        &mut self,
        state: &mut UserStoreUiState0104,
        popup: &mut UserStorePopupPresentation0104,
        outbox: &mut UserStoreUiOutbox0104,
    ) -> Option<UserStoreGuardRejection0104> {
        if self.gm_owner.is_some() {
            return None;
        }
        if *state == UserStoreUiState0104::default()
            && *popup == UserStorePopupPresentation0104::default()
            && outbox.0.is_empty()
        {
            return None;
        }

        let dropped_commands = outbox.0.len();
        let error = outbox
            .0
            .iter()
            .find_map(|command| match command {
                crate::user_store_ui::UserStoreUiCommand0104::SendPacket(packet) => {
                    Some(match prove_user_store_packet_0104(packet) {
                        Ok(_) => UserStoreProductionError0104::NoRegisteredProductionSession,
                        Err(error) => error,
                    })
                }
                _ => None,
            })
            .unwrap_or(UserStoreProductionError0104::NoRegisteredProductionSession);

        *state = UserStoreUiState0104::default();
        *popup = UserStorePopupPresentation0104::default();
        outbox.clear();
        let rejection = UserStoreGuardRejection0104 {
            dropped_commands,
            error,
        };
        self.rejected_boundaries = self.rejected_boundaries.saturating_add(1);
        self.last_rejection = Some(rejection.clone());
        Some(rejection)
    }

    /// No street-stall reply can be owned without a registry-proven outgoing
    /// request.  Preserve all frame metadata and bytes for downstream owners.
    #[must_use]
    pub fn ingest(&self, frame: DecodedFrame) -> UserStoreInboundPassthrough0104 {
        let reason = if USER_STORE_REPLY_PACKET_TYPES_0104.contains(&frame.packet_type) {
            UserStoreInboundPassthroughReason0104::NoRegisteredRequestOwner
        } else {
            UserStoreInboundPassthroughReason0104::UnrelatedPacket
        };
        UserStoreInboundPassthrough0104 { frame, reason }
    }
}

pub fn prove_user_store_packet_0104(
    packet: &UserStorePacket0104,
) -> Result<RegisteredGameplayRequest0104, UserStoreProductionError0104> {
    if !USER_STORE_REQUEST_CAPABILITIES_0104
        .iter()
        .any(|capability| capability.packet_type == packet.packet_id)
    {
        return Err(UserStoreProductionError0104::UnexpectedRequestPacket {
            packet_type: packet.packet_id,
        });
    }
    Ok(RegisteredGameplayRequest0104::new(
        packet.packet_id,
        packet.payload.clone(),
    )?)
}

pub fn validate_other_player_entry_0104(
    context: UserStoreOtherPlayerEntry0104,
) -> Result<(), UserStoreEntryGate0104> {
    if context.player_menu_owner != UserStoreOwnerReachability0104::Proven {
        return Err(UserStoreEntryGate0104::OwnerUnavailable(
            UserStoreEntryOwner0104::SelectablePlayerMenu,
        ));
    }
    if context.owner_pc_id <= 0 {
        return Err(UserStoreEntryGate0104::InvalidOwnerPcId(
            context.owner_pc_id,
        ));
    }
    if context.target_pc_id <= 0 {
        return Err(UserStoreEntryGate0104::InvalidTargetPcId(
            context.target_pc_id,
        ));
    }
    if context.target_pc_id == context.owner_pc_id {
        return Err(UserStoreEntryGate0104::TargetIsOwner);
    }
    if !context.authoritative_store_open {
        return Err(UserStoreEntryGate0104::TargetStoreClosed);
    }
    Ok(())
}

pub fn validate_own_entry_0104(
    context: UserStoreOwnEntry0104,
) -> Result<(), UserStoreEntryGate0104> {
    if context.inventory_use_owner != UserStoreOwnerReachability0104::Proven {
        return Err(UserStoreEntryGate0104::OwnerUnavailable(
            UserStoreEntryOwner0104::InventoryUse,
        ));
    }
    if context.owner_pc_id <= 0 {
        return Err(UserStoreEntryGate0104::InvalidOwnerPcId(
            context.owner_pc_id,
        ));
    }
    if !(0..crate::user_store_ui::USER_STORE_INVENTORY_CAPACITY as i32)
        .contains(&context.inventory_slot)
    {
        return Err(UserStoreEntryGate0104::InvalidInventorySlot(
            context.inventory_slot,
        ));
    }
    if context.item_type != USER_STORE_GENERAL_ITEM_OUTER_TYPE_0104 {
        return Err(UserStoreEntryGate0104::WrongOuterItemType(
            context.item_type,
        ));
    }
    if context.general_item_type != Some(USER_STORE_STREET_STALL_ITEM_SUBTYPE_0104) {
        return Err(UserStoreEntryGate0104::WrongGeneralItemType(
            context.general_item_type,
        ));
    }
    if context.map_flags & USER_STORE_REQUIRED_MAP_MASK_0104 == 0 {
        return Err(UserStoreEntryGate0104::RequiredMapFlagMissing);
    }
    if context.map_flags & USER_STORE_FORBIDDEN_MAP_MASK_0104 != 0 {
        return Err(UserStoreEntryGate0104::ForbiddenMapFlagSet);
    }
    if !context.player_y.is_finite() || !context.map_height.is_finite() {
        return Err(UserStoreEntryGate0104::NonFiniteGroundHeight);
    }
    let absolute_delta = (context.player_y - context.map_height).abs();
    if absolute_delta >= USER_STORE_MAX_GROUND_DELTA_0104 {
        return Err(UserStoreEntryGate0104::TooFarFromMapHeight { absolute_delta });
    }
    validate_nearby_distance_0104(context.nearest_other_player_squared_distance, true)?;
    validate_nearby_distance_0104(context.nearest_store_squared_distance, false)?;
    Ok(())
}

fn validate_nearby_distance_0104(
    squared_distance: Option<f32>,
    player: bool,
) -> Result<(), UserStoreEntryGate0104> {
    let Some(squared_distance) = squared_distance else {
        return Ok(());
    };
    if !squared_distance.is_finite() || squared_distance < 0.0 {
        return Err(UserStoreEntryGate0104::InvalidNearbyDistance);
    }
    if squared_distance < USER_STORE_NEARBY_SQUARED_DISTANCE_0104 {
        return Err(if player {
            UserStoreEntryGate0104::NearbyPlayer { squared_distance }
        } else {
            UserStoreEntryGate0104::NearbyStore { squared_distance }
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
