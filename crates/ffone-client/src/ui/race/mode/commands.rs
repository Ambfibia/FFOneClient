use super::*;

pub const RACE_START_REQUEST_SIZE: usize = 12;

pub const RACE_END_REQUEST_SIZE: usize = 8;

pub const RACE_CANCEL_REQUEST_SIZE: usize = 4;

pub const RACE_START_REQUEST_ABI: [RaceAbiField; 3] = [
    RaceAbiField {
        clean_name: "iStartEcomID",
        offset: 0,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iEPRaceMode",
        offset: 4,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iEPTicketItemSlotNum",
        offset: 8,
        scalar: RaceAbiScalar::I32,
    },
];

pub const RACE_END_REQUEST_ABI: [RaceAbiField; 2] = [
    RaceAbiField {
        clean_name: "iEndEcomID",
        offset: 0,
        scalar: RaceAbiScalar::I32,
    },
    RaceAbiField {
        clean_name: "iEPTicketItemSlotNum",
        offset: 4,
        scalar: RaceAbiScalar::I32,
    },
];

pub const RACE_CANCEL_REQUEST_ABI: [RaceAbiField; 1] = [RaceAbiField {
    clean_name: "iStartEcomID",
    offset: 0,
    scalar: RaceAbiScalar::I32,
}];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaceRequestKind {
    Start,
    End,
    Cancel,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaceRequestIntent {
    Start {
        request_id: u64,
        i_start_ecom_id: i32,
        i_ep_race_mode: i32,
        i_ep_ticket_item_slot_num: i32,
    },
    End {
        request_id: u64,
        i_end_ecom_id: i32,
        i_ep_ticket_item_slot_num: i32,
    },
    Cancel {
        request_id: u64,
        i_start_ecom_id: i32,
    },
}

impl RaceRequestIntent {
    #[must_use]
    pub const fn request_id(self) -> u64 {
        match self {
            Self::Start { request_id, .. }
            | Self::End { request_id, .. }
            | Self::Cancel { request_id, .. } => request_id,
        }
    }

    #[must_use]
    pub const fn kind(self) -> RaceRequestKind {
        match self {
            Self::Start { .. } => RaceRequestKind::Start,
            Self::End { .. } => RaceRequestKind::End,
            Self::Cancel { .. } => RaceRequestKind::Cancel,
        }
    }

    #[must_use]
    pub const fn packet_id(self) -> u32 {
        match self {
            Self::Start { .. } => RACE_START_REQUEST_PACKET_ID,
            Self::End { .. } => RACE_END_REQUEST_PACKET_ID,
            Self::Cancel { .. } => RACE_CANCEL_REQUEST_PACKET_ID,
        }
    }

    #[must_use]
    pub const fn abi_size(self) -> usize {
        match self {
            Self::Start { .. } => RACE_START_REQUEST_SIZE,
            Self::End { .. } => RACE_END_REQUEST_SIZE,
            Self::Cancel { .. } => RACE_CANCEL_REQUEST_SIZE,
        }
    }

    pub fn encode_registered(
        self,
    ) -> Result<RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104> {
        let mut payload = vec![0; self.abi_size()];
        match self {
            Self::Start {
                i_start_ecom_id,
                i_ep_race_mode,
                i_ep_ticket_item_slot_num,
                ..
            } => {
                write_i32(&mut payload, 0, i_start_ecom_id);
                write_i32(&mut payload, 4, i_ep_race_mode);
                write_i32(&mut payload, 8, i_ep_ticket_item_slot_num);
            }
            Self::End {
                i_end_ecom_id,
                i_ep_ticket_item_slot_num,
                ..
            } => {
                write_i32(&mut payload, 0, i_end_ecom_id);
                write_i32(&mut payload, 4, i_ep_ticket_item_slot_num);
            }
            Self::Cancel {
                i_start_ecom_id, ..
            } => write_i32(&mut payload, 0, i_start_ecom_id),
        }
        RegisteredGameplayRequest0104::new(self.packet_id(), payload)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceReplyEnvelope {
    pub request_id: u64,
    pub reply: RaceModeReply,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PendingRaceRequest {
    pub(super) request_id: u64,
    pub(super) kind: RaceRequestKind,
}
