//! Production protocol/TableData boundary for clean Nano Free Tuning mode 22.
//!
//! `NanoFreeTuningModel` owns the exact modal state machine and presentation
//! intents. This module owns the server-sized `sNano` bank (37-slot login prefix), strict
//! PC-Nano-create mutation, exact TableData projection, and the adapter from a
//! correlated protocol reply into that model. It never opens a socket and
//! never predicts server post-state.

use bevy::prelude::Resource;
use ffone_protocol::{
    NANO_TUNE_ITEM_SLOT_COUNT_0104, Nano0104, NanoTunePacket0104, PcLoadData0104,
    PcNanoCreateSuccess0104, WirePayload, packet,
};
use std::{error::Error, fmt};

use crate::{
    nano_free_tuning_ui::{
        NanoFreeTuningContent, NanoFreeTuningPower, NanoFreeTuningReplyBody,
        NanoFreeTuningReplyEnvelope, NanoTuneFailure, NanoTuneItemBase, NanoTuneSuccess,
    },
    network::CorrelatedNanoTuneReply0104,
    tutorial_mission_content::TutorialMissionContent,
};

pub const NANO_FREE_TUNING_SKILL_ICON_ROOT: &str = "ui/en/gameplay/nano/icons/skill";

const EMPTY_NANO_0104: Nano0104 = Nano0104 {
    id: 0,
    skill_id: 0,
    stamina: 0,
};

#[derive(Clone, Debug, PartialEq, Eq, Resource)]
pub struct NanoFreeTuningBank0104 {
    bank: Vec<Nano0104>,
}

impl Default for NanoFreeTuningBank0104 {
    fn default() -> Self {
        Self {
            bank: vec![EMPTY_NANO_0104; PcLoadData0104::NANO_BANK_COUNT],
        }
    }
}

impl NanoFreeTuningBank0104 {
    pub fn seed(&mut self, load: &PcLoadData0104) {
        self.bank = load.nano_bank().to_vec();
    }

    /// Replay pre-loading book pages before scanning for untuned Nanos or
    /// resolving equipped slots. These frames never reach live gameplay ingress.
    /// Commit atomically so a malformed later page cannot publish a partial bank.
    pub fn seed_with_bootstrap(
        &mut self,
        load: &PcLoadData0104,
        player_id: i32,
        bootstrap: &ffone_net::WorldBootstrap,
    ) -> Result<(), NanoFreeTuningBankError0104> {
        let mut next = Self::default();
        next.seed(load);
        for frame in bootstrap.packets.iter().map(|packet| packet.raw_frame()) {
            if frame.packet_type != packet::P_FE2CL_REP_NANO_BOOK_SUBSET {
                continue;
            }
            let page = ffone_protocol::wire_0104::NanoBookSubsetReply0104::decode(&frame.payload)
                .map_err(|_| NanoFreeTuningBankError0104::InvalidBookPage)?;
            next.apply_book_subset(&page, player_id)?;
        }
        *self = next;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.bank = vec![EMPTY_NANO_0104; PcLoadData0104::NANO_BANK_COUNT];
    }

    #[must_use]
    pub fn entries(&self) -> &[Nano0104] {
        &self.bank
    }

    /// The fixed login prefix is followed by bounded Nano Book pages. Size is
    /// server-authored (from its table set), never inferred from local XDT.
    pub fn apply_book_subset(
        &mut self,
        packet: &ffone_protocol::wire_0104::NanoBookSubsetReply0104,
        player_id: i32,
    ) -> Result<(), NanoFreeTuningBankError0104> {
        if packet.pcuid != i64::from(player_id)
            || !(1..=i32::from(i16::MAX) + 1).contains(&packet.book_size)
            || packet.element_offset < 0
            || packet.element_offset >= packet.book_size
        {
            return Err(NanoFreeTuningBankError0104::InvalidBookPage);
        }
        let size = packet.book_size as usize;
        let offset = packet.element_offset as usize;
        let count = (size - offset).min(packet.element.len());
        for (index, nano) in packet.element[..count].iter().enumerate() {
            if nano.id != 0 && usize::try_from(nano.id).ok() != Some(offset + index) {
                return Err(NanoFreeTuningBankError0104::InvalidBookPage);
            }
        }
        self.bank.resize(size, EMPTY_NANO_0104);
        // OpenFusion's first packet only announces capacity before login.
        if offset == 0 && packet.element.iter().all(|nano| nano.id == 0) {
            return Ok(());
        }
        for (index, nano) in packet.element[..count].iter().enumerate() {
            self.bank[offset + index] = Nano0104 {
                id: nano.id,
                skill_id: nano.skill_id,
                stamina: nano.stamina,
            };
        }
        Ok(())
    }

    /// Exact `cnMissionManager.CheckNanoFreeTuning` scan order: the first
    /// bank index whose stored Nano is non-zero and untuned.
    #[must_use]
    pub fn first_untuned_index(&self) -> Option<i16> {
        self.bank
            .iter()
            .position(|nano| nano.id != 0 && nano.skill_id == 0)
            .and_then(|index| i16::try_from(index).ok())
    }

    pub fn apply_create_success(
        &mut self,
        packet: PcNanoCreateSuccess0104,
    ) -> Result<(), NanoFreeTuningBankError0104> {
        let index = checked_nano_index(packet.nano.id, self.bank.len())?;
        self.bank[index] = packet.nano;
        Ok(())
    }

    pub fn apply_tune_success(
        &mut self,
        nano_id: i16,
        skill_id: i16,
    ) -> Result<(), NanoFreeTuningBankError0104> {
        let index = checked_nano_index(nano_id, self.bank.len())?;
        if skill_id <= 0 {
            return Err(NanoFreeTuningBankError0104::InvalidSkillId(skill_id));
        }
        let stored = self.bank[index];
        if stored.id != nano_id {
            return Err(NanoFreeTuningBankError0104::BankIdentityMismatch {
                index,
                expected: nano_id,
                stored: stored.id,
            });
        }
        self.bank[index].skill_id = skill_id;
        Ok(())
    }

    pub fn apply_stamina(
        &mut self,
        nano_id: i16,
        stamina: i16,
    ) -> Result<(), NanoFreeTuningBankError0104> {
        let index = checked_nano_index(nano_id, self.bank.len())?;
        if self.bank[index].id != nano_id {
            return Err(NanoFreeTuningBankError0104::BankIdentityMismatch {
                index,
                expected: nano_id,
                stored: self.bank[index].id,
            });
        }
        self.bank[index].stamina = stamina;
        Ok(())
    }
}

fn checked_nano_index(nano_id: i16, capacity: usize) -> Result<usize, NanoFreeTuningBankError0104> {
    let index = usize::try_from(nano_id)
        .map_err(|_| NanoFreeTuningBankError0104::InvalidNanoId(nano_id))?;
    if index == 0 || index >= capacity {
        return Err(NanoFreeTuningBankError0104::InvalidNanoId(nano_id));
    }
    Ok(index)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NanoFreeTuningBankError0104 {
    InvalidBookPage,
    InvalidNanoId(i16),
    InvalidSkillId(i16),
    BankIdentityMismatch {
        index: usize,
        expected: i16,
        stored: i16,
    },
}

impl fmt::Display for NanoFreeTuningBankError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "NanoFreeTuning bank rejected: {self:?}")
    }
}

impl Error for NanoFreeTuningBankError0104 {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NanoFreeTuningContentProjectionError {
    InvalidNanoId(i16),
    MissingTableData(i16),
    MissingGameplayNano(i16),
    InvalidTuneId { index: usize, value: i32 },
    InvalidSkillId { index: usize, value: i32 },
}

impl fmt::Display for NanoFreeTuningContentProjectionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "NanoFreeTuning TableData projection rejected: {self:?}"
        )
    }
}

impl Error for NanoFreeTuningContentProjectionError {}

pub fn project_nano_free_tuning_content(
    content: &TutorialMissionContent,
    nano_id: i16,
) -> Result<NanoFreeTuningContent, NanoFreeTuningContentProjectionError> {
    if nano_id <= 0 {
        return Err(NanoFreeTuningContentProjectionError::InvalidNanoId(nano_id));
    }
    let source = content.journal_nano(i32::from(nano_id)).ok_or(
        NanoFreeTuningContentProjectionError::MissingTableData(nano_id),
    )?;
    let gameplay_nano = content.gameplay_nano(nano_id).ok_or(
        NanoFreeTuningContentProjectionError::MissingGameplayNano(nano_id),
    )?;
    let mut powers = Vec::with_capacity(3);
    for (index, source) in source.skills.iter().enumerate() {
        let tune_id = i16::try_from(source.tune_id).map_err(|_| {
            NanoFreeTuningContentProjectionError::InvalidTuneId {
                index,
                value: source.tune_id,
            }
        })?;
        let skill_id = i16::try_from(source.skill_id).map_err(|_| {
            NanoFreeTuningContentProjectionError::InvalidSkillId {
                index,
                value: source.skill_id,
            }
        })?;
        if tune_id <= 0 {
            return Err(NanoFreeTuningContentProjectionError::InvalidTuneId {
                index,
                value: source.tune_id,
            });
        }
        if skill_id <= 0 {
            return Err(NanoFreeTuningContentProjectionError::InvalidSkillId {
                index,
                value: source.skill_id,
            });
        }
        powers.push(NanoFreeTuningPower {
            tune_id,
            skill_id,
            icon_path: format!(
                "{NANO_FREE_TUNING_SKILL_ICON_ROOT}/skillicon_{:02}.png",
                source.icon_number
            ),
            name: source.name.clone(),
            power_type: source.type_label.clone(),
            description: source.description.clone(),
        });
    }
    let powers: [NanoFreeTuningPower; 3] = powers.try_into().expect("source owns three powers");
    Ok(NanoFreeTuningContent {
        nano_id,
        nano_style: gameplay_nano.style,
        nano_name: source.name.clone(),
        powers,
    })
}

#[must_use]
pub fn project_correlated_nano_tune_reply(
    correlated: CorrelatedNanoTuneReply0104,
) -> NanoFreeTuningReplyEnvelope {
    let body = match correlated.packet {
        NanoTunePacket0104::Success(success) => NanoFreeTuningReplyBody::Success(NanoTuneSuccess {
            nano_id: success.nano_id,
            skill_id: success.skill_id,
            fusion_matter: success.fusion_matter,
            item_slots: success.item_slots,
            items: success.items.map(|item| NanoTuneItemBase {
                item_type: item.item_type,
                item_id: item.item_id,
                option: item.option,
                time_limit: item.time_limit,
            }),
        }),
        NanoTunePacket0104::Failure(failure) => NanoFreeTuningReplyBody::Failure(NanoTuneFailure {
            player_id: failure.pc_id,
            error_code: failure.error_code,
        }),
    };
    debug_assert_eq!(
        match &body {
            NanoFreeTuningReplyBody::Success(success) => success.item_slots.len(),
            NanoFreeTuningReplyBody::Failure(_) => NANO_TUNE_ITEM_SLOT_COUNT_0104,
        },
        NANO_TUNE_ITEM_SLOT_COUNT_0104
    );
    NanoFreeTuningReplyEnvelope {
        request_token: correlated.request_token,
        packet_id: correlated.packet_id(),
        payload_size: correlated.payload_size(),
        body,
    }
}

#[cfg(test)]
mod tests;
