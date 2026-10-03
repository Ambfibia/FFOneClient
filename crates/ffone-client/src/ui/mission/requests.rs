//! Typed mission UI requests and their registered 0104 gameplay encoding.

use bevy::prelude::*;
use ffone_protocol::{RegisteredGameplayRequest0104, RegisteredGameplayRequestError0104};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PendingMissionUiRequest {
    TaskStart {
        task_id: i32,
        npc_id: i32,
    },
    QuestEnd {
        task_id: i32,
        npc_id: i32,
        box1_choice: i32,
        box2_choice: i32,
    },
    TaskStop {
        task_id: i32,
    },
}

impl PendingMissionUiRequest {
    pub const fn task_id(self) -> i32 {
        match self {
            Self::TaskStart { task_id, .. }
            | Self::QuestEnd { task_id, .. }
            | Self::TaskStop { task_id } => task_id,
        }
    }

    pub fn encode_registered(
        self,
    ) -> Result<RegisteredGameplayRequest0104, MissionRequestCodecError0104> {
        self.encode_registered_with_escort(0)
    }

    /// Bind the live escort owner at the world/network boundary. UI intents
    /// retain task identity without caching a potentially despawned NPC ID.
    pub fn encode_registered_with_escort(
        self,
        escort_npc_id: i32,
    ) -> Result<RegisteredGameplayRequest0104, MissionRequestCodecError0104> {
        match self {
            Self::TaskStart { task_id, npc_id } => {
                let mut payload = vec![0; 12];
                payload[0..4].copy_from_slice(&task_id.to_le_bytes());
                payload[4..8].copy_from_slice(&npc_id.to_le_bytes());
                payload[8..12].copy_from_slice(&escort_npc_id.to_le_bytes());
                Ok(RegisteredGameplayRequest0104::new(0x1300_000b, payload)?)
            }
            Self::QuestEnd {
                task_id,
                npc_id,
                box1_choice,
                box2_choice,
            } => {
                let box1_choice = i8::try_from(box1_choice).map_err(|_| {
                    MissionRequestCodecError0104::ChoiceOutOfRange {
                        field: "iBox1Choice",
                        value: box1_choice,
                    }
                })?;
                let box2_choice = i8::try_from(box2_choice).map_err(|_| {
                    MissionRequestCodecError0104::ChoiceOutOfRange {
                        field: "iBox2Choice",
                        value: box2_choice,
                    }
                })?;
                let mut payload = vec![0; 16];
                payload[0..4].copy_from_slice(&task_id.to_le_bytes());
                payload[4..8].copy_from_slice(&npc_id.to_le_bytes());
                payload[8] = box1_choice as u8;
                payload[9] = box2_choice as u8;
                // 10..12 remains pack(4) padding.
                payload[12..16].copy_from_slice(&escort_npc_id.to_le_bytes());
                Ok(RegisteredGameplayRequest0104::new(0x1300_000c, payload)?)
            }
            Self::TaskStop { task_id } => Ok(RegisteredGameplayRequest0104::new(
                0x1300_0012,
                task_id.to_le_bytes().to_vec(),
            )?),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MissionRequestCodecError0104 {
    ChoiceOutOfRange { field: &'static str, value: i32 },
    Registry(RegisteredGameplayRequestError0104),
}

impl std::fmt::Display for MissionRequestCodecError0104 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChoiceOutOfRange { field, value } => {
                write!(formatter, "mission {field} must fit i8, got {value}")
            }
            Self::Registry(error) => write!(formatter, "mission request ABI failed: {error}"),
        }
    }
}

impl std::error::Error for MissionRequestCodecError0104 {}

impl From<RegisteredGameplayRequestError0104> for MissionRequestCodecError0104 {
    fn from(value: RegisteredGameplayRequestError0104) -> Self {
        Self::Registry(value)
    }
}
