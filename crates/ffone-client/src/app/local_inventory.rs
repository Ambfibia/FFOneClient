//! Local inventory runtime and world player equipment projection.

use bevy::prelude::*;
use ffone_client::inventory_runtime::InventoryRuntime0104;
use ffone_protocol::{ItemBase0104, ItemReward0104, PcNanoCreateSuccess0104};

#[derive(Clone, Debug, Default, Resource)]
pub(super) struct LocalInventoryRuntime {
    pub(super) snapshot: Option<InventoryRuntime0104>,
    pub(super) quest_inventory: Option<[ItemBase0104; ffone_protocol::PcLoadData0104::QUEST_INVENTORY_COUNT]>,
}

pub(super) type WorldPlayerApparel0104 = [ItemBase0104; 6];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WorldPlayerApparelCandidate0104 {
    pub(super) entity: Entity,
    pub(super) apparel: WorldPlayerApparel0104,
}

/// Last authoritative equipment projected onto the shared local-player rig.
///
/// The inventory snapshot is server-owned. Hand changes can be applied to the
/// active rig in place, while apparel slots 1..=6 require a complete hidden
/// replacement rig. This includes the three skinned body parts plus rigid
/// head/face/back attachments, so every visible clothes change shares one
/// atomic readiness boundary. `candidate` is committed only after that rig
/// passes the same animation/material/texture gates as initial world entry.
#[derive(Clone, Debug, Default, PartialEq, Eq, Resource)]
pub(super) struct WorldPlayerEquipmentProjection {
    pub(super) force_refresh: bool,
    pub(super) controller_root: Option<Entity>,
    pub(super) rig_root: Option<Entity>,
    pub(super) hand: Option<ItemBase0104>,
    pub(super) hand_rig_root: Option<Entity>,
    pub(super) apparel: Option<WorldPlayerApparel0104>,
    pub(super) candidate: Option<WorldPlayerApparelCandidate0104>,
    pub(super) rejected_apparel: Option<WorldPlayerApparel0104>,
    pub(super) next_generation: u64,
}

impl LocalInventoryRuntime {
    pub(super) fn reset(&mut self) {
        self.snapshot = None;
        self.quest_inventory = None;
    }

    pub(super) fn seed(&mut self, owner_pc_id: i32, load: &ffone_protocol::PcLoadData0104) {
        self.snapshot = Some(InventoryRuntime0104::from_pc_load(owner_pc_id, load));
        self.quest_inventory = Some(load.quest_inventory());
    }

    #[must_use]
    pub(super) fn snapshot(&self) -> Option<&InventoryRuntime0104> {
        self.snapshot.as_ref()
    }

    pub(super) fn snapshot_mut(&mut self) -> Option<&mut InventoryRuntime0104> {
        self.snapshot.as_mut()
    }

    pub(super) fn apply_nano_create_quest_post_state(
        &mut self,
        packet: PcNanoCreateSuccess0104,
    ) -> Result<(), String> {
        if packet.quest_item_slot == -1 && packet.quest_item.item_id <= 0 {
            // OpenFusion's item-triggered Nano award intentionally carries the
            // clean no-quest-item sentinel and a zeroed authoritative item.
            return Ok(());
        }
        let slot = usize::try_from(packet.quest_item_slot)
            .map_err(|_| format!("negative quest-inventory slot {}", packet.quest_item_slot))?;
        if slot >= ffone_protocol::PcLoadData0104::QUEST_INVENTORY_COUNT {
            return Err(format!(
                "quest-inventory slot {} exceeds capacity {}",
                packet.quest_item_slot,
                ffone_protocol::PcLoadData0104::QUEST_INVENTORY_COUNT
            ));
        }
        if packet.quest_item.item_id > 0 && packet.quest_item.item_type < 0 {
            return Err(format!(
                "quest-inventory post-state has malformed type={} id={}",
                packet.quest_item.item_type, packet.quest_item.item_id
            ));
        }
        let inventory = self
            .quest_inventory
            .as_mut()
            .ok_or_else(|| "quest inventory is not seeded".to_owned())?;
        inventory[slot] = packet.quest_item;
        Ok(())
    }

    pub(super) fn apply_reward_item_post_state(
        &mut self,
        reply: &ffone_protocol::RewardItemReply0104,
    ) -> Result<(), String> {
        let mut next = self.clone();
        for reward in &reply.items {
            match reward.inventory_location {
                0 | 1 => {
                    next.snapshot
                        .as_mut()
                        .ok_or_else(|| {
                            "reward arrived before authoritative inventory load".to_owned()
                        })?
                        .apply_reward_item_post_state(*reward)
                        .map_err(|error| error.to_string())?;
                }
                2 => {
                    let slot = usize::try_from(reward.slot).map_err(|_| {
                        format!(
                            "reward references negative quest-inventory slot {}",
                            reward.slot
                        )
                    })?;
                    if slot >= ffone_protocol::PcLoadData0104::QUEST_INVENTORY_COUNT {
                        return Err(format!(
                            "reward quest-inventory slot {} exceeds capacity {}",
                            reward.slot,
                            ffone_protocol::PcLoadData0104::QUEST_INVENTORY_COUNT
                        ));
                    }
                    if reward.item.item_id > 0 && reward.item.item_type < 0 {
                        return Err(format!(
                            "reward quest-inventory post-state has malformed type={} id={}",
                            reward.item.item_type, reward.item.item_id
                        ));
                    }
                    next.quest_inventory.as_mut().ok_or_else(|| {
                        "reward arrived before authoritative quest-inventory load".to_owned()
                    })?[slot] = reward.item;
                }
                location => {
                    return Err(format!(
                        "reward references unsupported item location {location}"
                    ));
                }
            }
        }
        *self = next;
        Ok(())
    }

    pub(super) fn apply_single_race_reward_item_post_state(
        &mut self,
        reward: ItemReward0104,
    ) -> Result<(), String> {
        let mut next = self.clone();
        match reward.inventory_location {
            0 | 1 => {
                next.snapshot
                    .as_mut()
                    .ok_or_else(|| {
                        "race reward arrived before authoritative inventory load".to_owned()
                    })?
                    .apply_reward_item_post_state(reward)
                    .map_err(|error| error.to_string())?;
            }
            2 => {
                let slot = usize::try_from(reward.slot).map_err(|_| {
                    format!(
                        "race reward references negative quest-inventory slot {}",
                        reward.slot
                    )
                })?;
                if slot >= ffone_protocol::PcLoadData0104::QUEST_INVENTORY_COUNT {
                    return Err(format!(
                        "race reward quest-inventory slot {} exceeds capacity {}",
                        reward.slot,
                        ffone_protocol::PcLoadData0104::QUEST_INVENTORY_COUNT
                    ));
                }
                if reward.item.item_id > 0 && reward.item.item_type < 0 {
                    return Err(format!(
                        "race reward quest-inventory post-state has malformed type={} id={}",
                        reward.item.item_type, reward.item.item_id
                    ));
                }
                next.quest_inventory.as_mut().ok_or_else(|| {
                    "race reward arrived before authoritative quest-inventory load".to_owned()
                })?[slot] = reward.item;
            }
            location => {
                return Err(format!(
                    "race reward references unsupported item location {location}"
                ));
            }
        }
        *self = next;
        Ok(())
    }
}
