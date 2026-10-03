use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportationOutboxEvent {
    SetCursorLocked(bool),
    SetTransportCameraTarget(i32),
    SetGameConditionCooldown(i32),
    EndCameraSubTarget,
    InstantiatePlayerEffect(i32),
    PlayNpcMoveOkVoice(i32),
    PlaySound(TransportationSound),
    SystemMessage(i32),
    SetMovementPacketEmission(bool),
    Fade(TransportationFade),
    RestoreCursorLocked(bool),
    ExitMode { reason: TransportationCloseReason },
    MarkPacketHandled(u32),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportationCloseReason {
    UserClose,
    Departure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportationTravelIntent {
    pub npc_id: i32,
    /// Preserves the clean packet field spelling `iTransporationID`.
    pub transporation_id: i32,
    pub e_il: i32,
    pub slot_number: i32,
    pub turbo: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TransportationRegistrationIntent {
    pub transportation_type: i32,
    pub npc_id: i32,
    pub location_id: i32,
}

impl TransportationRegistrationIntent {
    #[must_use]
    pub fn is_registered(self, unlocks: TransportationUnlocks) -> bool {
        match self.transportation_type {
            1 => unlocks.warp_registered(self.location_id),
            2 => unlocks.wyvern_registered(self.location_id),
            _ => false,
        }
    }

    #[must_use]
    pub fn became_registered(
        self,
        previous: TransportationUnlocks,
        current: TransportationUnlocks,
    ) -> bool {
        !self.is_registered(previous) && self.is_registered(current)
    }

    pub fn encode_registered(
        self,
    ) -> Result<RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104> {
        let mut payload = vec![0; 12];
        payload[0..4].copy_from_slice(&self.transportation_type.to_le_bytes());
        payload[4..8].copy_from_slice(&self.npc_id.to_le_bytes());
        payload[8..12].copy_from_slice(&self.location_id.to_le_bytes());
        RegisteredGameplayRequest0104::new(TRANSPORTATION_REGISTRATION_REQUEST_PACKET_ID, payload)
    }
}

impl TransportationTravelIntent {
    pub fn encode_registered(
        self,
    ) -> Result<RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104> {
        let mut payload = vec![0; 16];
        payload[0..4].copy_from_slice(&self.npc_id.to_le_bytes());
        payload[4..8].copy_from_slice(&self.transporation_id.to_le_bytes());
        payload[8..12].copy_from_slice(&self.e_il.to_le_bytes());
        payload[12..16].copy_from_slice(&self.slot_number.to_le_bytes());
        RegisteredGameplayRequest0104::new(TRANSPORTATION_REQUEST_PACKET_ID, payload)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TransportationUiCommand {
    SelectRoute(usize),
    GoNow,
    ToggleTurbo,
    CloseButton(TransportationInputGates),
    Escape(TransportationInputGates),
    ScrollAxis {
        axis: f32,
        legacy_scroll_velocity: f32,
    },
}

#[derive(Debug, Default, Resource)]
pub struct TransportationUiCommandOutbox(pub(super) VecDeque<TransportationUiCommand>);

impl TransportationUiCommandOutbox {
    pub fn push(&mut self, command: TransportationUiCommand) {
        self.0.push_back(command);
    }

    pub fn pop(&mut self) -> Option<TransportationUiCommand> {
        self.0.pop_front()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
