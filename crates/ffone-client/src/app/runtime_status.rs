//! Runtime player/roster/chat status and authoritative runtime frame application.

use super::state::ClientState;
use super::world_nano::active_world_nano_slot;
use super::{gameplay_ui_actions, nano_recall};
use bevy::prelude::*;
use ffone_client::{
    coordinates::ProtocolPosition,
    gameplay_ui::ChatLineUi,
    inventory_runtime::InventoryRuntime0104,
    network::CharacterSummary,
    resurrect_ui::{ResurrectInventorySlot, clean_resurrection_item_slot},
    tutorial_mission_content::TutorialMissionContent,
    tutorial_nano_gameplay::TutorialNanoGameplayLoadout,
    world::NativeWorldScope,
};
use ffone_protocol::{
    DecodedFrame, NpcAttackChars0104, NpcCombatPacket0104, PcAttackChars0104, PcRegenData0104,
    PcRegenSuccess0104, PcTick0104, TimeBuffDotDamageTick0104, WirePayload,
    decode_npc_combat_packet_0104, packet,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Default)]
pub(super) struct RuntimeRosterStatus {
    pub(super) characters: Vec<CharacterSummary>,
    pub(super) selected_uid: Option<i64>,
    pub(super) pending_character_entry_uid: Option<i64>,
    /// UIDs that have entered the shard in this authenticated login session.
    pub(super) world_snapshot_uids: BTreeSet<i64>,
}

impl RuntimeRosterStatus {
    /// The worker retains the original login CHAR_INFO records across shard
    /// visits. Keep newer world snapshots by UID, including empty slots, when
    /// it republishes that roster after a return, creation or deletion.
    pub(super) fn replace_retained_login_characters(
        &mut self,
        mut characters: Vec<CharacterSummary>,
    ) {
        for character in &mut characters {
            if self.world_snapshot_uids.contains(&character.pc_uid)
                && let Some(current) = self
                    .characters
                    .iter()
                    .find(|old| old.pc_uid == character.pc_uid)
            {
                character.equipment = current.equipment;
                character.position = current.position;
            }
        }
        self.world_snapshot_uids
            .retain(|uid| characters.iter().any(|character| character.pc_uid == *uid));
        self.characters = characters;
    }
}

#[derive(Debug, Default)]
pub(super) struct RuntimeChatStatus {
    /// Server-accepted local chat echoes awaiting the audio collection pass.
    pub(super) outgoing_audio_pending: usize,
    /// Ordered Buddy result/message SFX; consumed once by the shared UI mixer.
    pub(super) pending_social_sfx: Vec<&'static str>,
    pub(super) gm: gameplay_ui_actions::gm_runtime::GmRuntime,
    /// Clean `CnGuiChat.AllChatStrings`.
    pub(super) world_chat_lines: Vec<ChatLineUi>,
    /// Clean `CnGuiChat.GroupChatStrings`.
    pub(super) world_group_chat_lines: Vec<ChatLineUi>,
    /// Clean `CnGuiChat.BuddyChatStrings`.
    pub(super) world_buddy_chat_lines: Vec<ChatLineUi>,
    /// Clean per-PCUID `BuddyChatArray`, retained independently from the
    /// aggregate FRIEND history.
    pub(super) world_buddy_chat_by_uid: BTreeMap<i64, Vec<ChatLineUi>>,
    pub(super) world_group_available: bool,
    pub(super) world_chat_alerts: [bool; 3],
}

#[derive(Debug, Default)]
pub(super) struct RuntimeDiagnostics {
    pub(super) tutorial_issue_history: VecDeque<String>,
    pub(super) gameplay_frames: u64,
    pub(super) bootstrap_packets: usize,
    pub(super) bootstrap_decode_errors: usize,
}

#[derive(Debug)]
pub(super) struct RuntimePlayerStatus {
    pub(super) message: String,
    pub(super) player_id: Option<i32>,
    pub(super) player_gender: Option<i32>,
    pub(super) player_name: String,
    pub(super) user_level: i16,
    pub(super) player_level: u16,
    pub(super) map_number: Option<i32>,
    pub(super) map_name: String,
    pub(super) fusion_matter: i32,
    pub(super) max_fusion_matter: i32,
    pub(super) candy: i32,
    pub(super) transportation_unlocks: ffone_client::transportation_ui::TransportationUnlocks,
    pub(super) hp: Option<i32>,
    pub(super) max_hp: i32,
    pub(super) special_state: i8,
    pub(super) free_chat: bool,
    pub(super) allow_player_interaction: bool,
    pub(super) tutorial_mouse_right: bool,
    pub(super) nano_slots: [RuntimeNanoSlot; 3],
    pub(super) pending_nano_activation: Option<usize>,
    pub(super) pending_passive_nano_voice: Option<TutorialNanoGameplayLoadout>,
    pub(super) nano_recall: nano_recall::NanoRecallState,
    pub(super) nano_battery: i32,
    pub(super) weapon_battery: i32,
    pub(super) resurrection_item_slot: Option<i32>,
    pub(super) tutorial_weapon_id: Option<i32>,
}

/// Compatibility façade for systems that still need more than one runtime
/// projection. The data itself is split by ownership so callers can migrate
/// toward the narrow roster/chat/diagnostic views without recreating another
/// flat god resource.
#[derive(Debug, Default, Resource)]
pub(super) struct RuntimeStatus {
    pub(super) core: RuntimePlayerStatus,
    pub(super) roster: RuntimeRosterStatus,
    pub(super) chat: RuntimeChatStatus,
    pub(super) diagnostics: RuntimeDiagnostics,
}

impl std::ops::Deref for RuntimeStatus {
    type Target = RuntimePlayerStatus;

    fn deref(&self) -> &Self::Target {
        &self.core
    }
}

impl std::ops::DerefMut for RuntimeStatus {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.core
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct RuntimeNanoSlot {
    pub(super) nano_id: Option<i16>,
    pub(super) skill_id: i16,
    pub(super) stamina: i16,
    pub(super) active: bool,
}

impl Default for RuntimePlayerStatus {
    fn default() -> Self {
        Self {
            message: "Native assets ready".to_owned(),
            player_id: None,
            player_gender: None,
            player_name: String::new(),
            user_level: 100,
            player_level: 1,
            map_number: None,
            map_name: String::new(),
            fusion_matter: 0,
            max_fusion_matter: 1,
            candy: 0,
            transportation_unlocks: default(),
            hp: None,
            max_hp: 1,
            special_state: 0,
            free_chat: false,
            allow_player_interaction: true,
            tutorial_mouse_right: false,
            nano_slots: [RuntimeNanoSlot::default(); 3],
            pending_nano_activation: None,
            pending_passive_nano_voice: None,
            nano_recall: default(),
            nano_battery: 0,
            weapon_battery: 0,
            resurrection_item_slot: None,
            tutorial_weapon_id: None,
        }
    }
}

impl RuntimeStatus {
    /// Called in network event order, before session teardown can discard the
    /// inventory. Every renderer returning to the roster then sees the same
    /// server-accepted equipment as the live world and inventory preview.
    pub(super) fn retain_local_equipment(&mut self, inventory: &InventoryRuntime0104) {
        if self.player_id != Some(inventory.owner_pc_id()) {
            return;
        }
        if let Some(character) = self
            .roster
            .characters
            .iter_mut()
            .find(|character| Some(character.pc_uid) == self.roster.selected_uid)
        {
            character.equipment = (*inventory.equipment())
                .map(super::world_equipment::equipped_item_from_inventory_item);
            self.roster.world_snapshot_uids.insert(character.pc_uid);
        }
    }

    pub(super) fn retain_local_position(&mut self, position: [i32; 3]) {
        if self.player_id.is_none() {
            return;
        }
        if let Some(character) = self
            .roster
            .characters
            .iter_mut()
            .find(|character| Some(character.pc_uid) == self.roster.selected_uid)
        {
            character.position = position;
        }
    }

    pub(super) fn clear_world(&mut self) {
        self.roster.pending_character_entry_uid = None;
        self.player_id = None;
        self.player_gender = None;
        self.player_name.clear();
        self.user_level = 100;
        self.player_level = 1;
        self.map_number = None;
        self.map_name.clear();
        self.fusion_matter = 0;
        self.max_fusion_matter = 1;
        self.candy = 0;
        self.transportation_unlocks = default();
        self.hp = None;
        self.max_hp = 1;
        self.special_state = 0;
        self.free_chat = false;
        self.allow_player_interaction = true;
        self.tutorial_mouse_right = false;
        self.nano_slots = [RuntimeNanoSlot::default(); 3];
        self.pending_nano_activation = None;
        self.pending_passive_nano_voice = None;
        self.nano_recall = default();
        self.nano_battery = 0;
        self.weapon_battery = 0;
        self.resurrection_item_slot = None;
        self.tutorial_weapon_id = None;
        self.chat.world_chat_lines.clear();
        self.chat.world_group_chat_lines.clear();
        self.chat.world_buddy_chat_lines.clear();
        self.chat.world_buddy_chat_by_uid.clear();
        self.chat.world_group_available = false;
        self.chat.world_chat_alerts = [false; 3];
        self.chat.outgoing_audio_pending = 0;
        self.chat.pending_social_sfx.clear();
    }
}

pub(super) fn legacy_avatar_max_hp(level: u16, class: i32, current_hp: i32) -> i32 {
    // Exact `m_pAvatarTable.m_pAvatarGrowData` values in Retrobution are
    // 1000 at level 1 and +75 for every level through 40.
    let base = if (1..=40).contains(&level) {
        925 + i32::from(level) * 75
    } else {
        return current_hp.max(1);
    };
    // `cnAvatarStatus.SetBaseAttrib` applies these m_iBonusHP percentages
    // only to post-Academy classes 2..=4.
    let bonus_percent = match class {
        2 => 50,
        3 => 20,
        _ => 0,
    };
    base * (100 + bonus_percent) / 100
}

pub(super) fn legacy_avatar_max_fusion_matter(level: u16, current_fusion_matter: i32) -> i32 {
    // Exact beta-20100104
    // `m_pAvatarTable.m_pAvatarGrowData.m_iReqBlob_NanoCreate` values.
    const REQUIRED_FUSION_MATTER: [i32; 41] = [
        0, 220, 700, 1_320, 2_510, 3_780, 5_940, 8_190, 11_610, 14_830, 19_760, 24_450, 28_220,
        34_020, 38_520, 45_550, 51_030, 59_400, 65_540, 75_350, 82_590, 93_960, 102_170, 115_200,
        124_430, 139_460, 149_760, 166_460, 177_880, 196_810, 209_410, 222_520, 235_870, 249_730,
        264_100, 278_710, 293_580, 320_000, 350_000, 400_000, 450_000,
    ];

    REQUIRED_FUSION_MATTER
        .get(usize::from(level))
        .copied()
        .filter(|maximum| *maximum > 0)
        .unwrap_or_else(|| current_fusion_matter.max(1))
}

pub(super) fn resolve_runtime_nano_slots(
    load: &ffone_protocol::PcLoadData0104,
    bank: &[ffone_protocol::Nano0104],
) -> [RuntimeNanoSlot; 3] {
    let slot_ids = load.nano_slots();
    let active_slot = load.active_nano_slot();
    std::array::from_fn(|index| {
        let nano_id = slot_ids[index];
        if nano_id <= 0 {
            return RuntimeNanoSlot::default();
        }
        let nano = usize::try_from(nano_id)
            .ok()
            .and_then(|bank_index| bank.get(bank_index))
            .filter(|nano| nano.id == nano_id)
            .or_else(|| bank.iter().find(|nano| nano.id == nano_id));
        let Some(nano) = nano else {
            return RuntimeNanoSlot {
                nano_id: Some(nano_id),
                active: active_slot == index as i16,
                ..default()
            };
        };
        RuntimeNanoSlot {
            nano_id: Some(nano.id),
            skill_id: nano.skill_id,
            stamina: nano.stamina,
            active: active_slot == index as i16,
        }
    })
}

pub(super) fn resolve_runtime_resurrection_item_slot(
    inventory: &InventoryRuntime0104,
    content: &TutorialMissionContent,
) -> Option<i32> {
    clean_resurrection_item_slot(inventory.inventory().iter().copied().enumerate().map(
        |(column, item)| {
            ResurrectInventorySlot {
                column: column as i32,
                empty: InventoryRuntime0104::item_is_empty(item),
                item_type: i32::from(item.item_type),
                general_item_type: (item.item_type == 7)
                    .then(|| content.general_item_type(item.item_id))
                    .flatten(),
            }
        },
    ))
}

pub(super) fn native_world_scope(login_tutorial_flag: i8) -> Result<NativeWorldScope, String> {
    match login_tutorial_flag {
        0 => Ok(NativeWorldScope::Tutorial),
        1 => Ok(NativeWorldScope::WorldMap),
        value => Err(format!(
            "unsupported protocol-0104 tutorial flag {value}; expected 0 or 1"
        )),
    }
}
pub(super) const fn client_state_for_world_scope(scope: NativeWorldScope) -> ClientState {
    match scope {
        NativeWorldScope::Tutorial => ClientState::Tutorial,
        NativeWorldScope::WorldMap => ClientState::World,
    }
}

pub(super) fn runtime_nano_slots_from_regen(regen: PcRegenData0104) -> [RuntimeNanoSlot; 3] {
    std::array::from_fn(|index| {
        let nano = regen.nanos[index];
        if nano.id <= 0 {
            return RuntimeNanoSlot::default();
        }
        RuntimeNanoSlot {
            nano_id: Some(nano.id),
            skill_id: nano.skill_id,
            stamina: nano.stamina,
            active: regen.active_nano_slot == index as i16,
        }
    })
}

pub(super) fn runtime_nano_slots_from_tick(
    tick: &PcTick0104,
    current: [RuntimeNanoSlot; 3],
) -> [RuntimeNanoSlot; 3] {
    let nanos = tick.nanos();
    std::array::from_fn(|index| {
        let nano = nanos[index];
        if nano.id <= 0 {
            return RuntimeNanoSlot::default();
        }
        RuntimeNanoSlot {
            nano_id: Some(nano.id),
            skill_id: nano.skill_id,
            stamina: nano.stamina,
            // PC_TICK does not serialize iActiveNanoSlotNum. Preserve the
            // last ACTIVE_SUCC/PC-load authority only while the same Nano
            // identity still occupies this exact equipped slot.
            active: current[index].active && current[index].nano_id == Some(nano.id),
        }
    })
}

pub(super) fn apply_pc_tick_to_runtime(tick: &PcTick0104, runtime: &mut RuntimeStatus) {
    runtime.hp = Some(tick.hp);
    runtime.nano_slots = runtime_nano_slots_from_tick(tick, runtime.nano_slots);
    runtime.nano_battery = tick.nano_battery();
}

pub(super) fn apply_pc_regen_success_to_runtime(
    success: PcRegenSuccess0104,
    runtime: &mut RuntimeStatus,
) -> Vec3 {
    let regen = success.regen_data;
    // Clean applies the authoritative avatar payload before ending
    // ResurrectMode. Keep this transactional ordering so every HUD observes
    // the revived state when the modal becomes hidden.
    runtime.hp = Some(regen.hp);
    runtime.map_number = Some(regen.map_number);
    runtime.nano_slots = runtime_nano_slots_from_regen(regen);
    runtime.pending_nano_activation = None;
    runtime.pending_passive_nano_voice = None;
    runtime.fusion_matter = success.fusion_matter;
    runtime.max_fusion_matter =
        legacy_avatar_max_fusion_matter(runtime.player_level, success.fusion_matter);
    ProtocolPosition::new(regen.position).to_native()
}

/// Applies authoritative local status packets and returns the HP value only
/// when this frame executed the clean client's local `SetHP` path.
pub(super) fn apply_runtime_frame(frame: &DecodedFrame, runtime: &mut RuntimeStatus) -> Option<i32> {
    let Some(player_id) = runtime.player_id else {
        return None;
    };
    let read_i32 = |offset: usize| {
        frame
            .payload
            .get(offset..offset + 4)
            .and_then(|bytes| bytes.try_into().ok())
            .map(i32::from_le_bytes)
    };
    let fusion_update = match frame.packet_type {
        packet::P_FE2CL_REP_PC_CHANGE_LEVEL => {
            let Ok(reply) =
                ffone_protocol::wire_0104::PcChangeLevelReply0104::decode(&frame.payload)
            else {
                return None;
            };
            // GM /level sends the same notification as nearby player updates.
            // It carries an owner ID and an i16 level, but no Fusion Matter.
            if reply.pc_id == player_id && reply.pc_level > 0 {
                runtime.player_level = reply.pc_level as u16;
                runtime.max_fusion_matter =
                    legacy_avatar_max_fusion_matter(runtime.player_level, runtime.fusion_matter);
            }
            return None;
        }
        // sP_FE2CL_REP_REWARD_ITEM.m_iFusionMatter
        packet::P_FE2CL_REP_REWARD_ITEM => read_i32(4),
        // Nano-create and Nano-tune replies are decoded and correlated by
        // their production mode before reaching this generic updater. Their
        // authoritative bank/inventory post-state must commit atomically with
        // Fusion Matter, so no offset-only fallback is allowed here.
        packet::P_FE2CL_REP_PC_CHANGE_LEVEL_SUCC => {
            if frame.payload.len() != 8 {
                return None;
            }
            if let Some(level) = read_i32(0)
                .and_then(|level| u16::try_from(level).ok())
                .filter(|level| *level > 0)
            {
                runtime.player_level = level;
            }
            read_i32(4)
        }
        _ => None,
    };
    if let Some(fusion_matter) = fusion_update {
        runtime.fusion_matter = fusion_matter;
        runtime.max_fusion_matter =
            legacy_avatar_max_fusion_matter(runtime.player_level, fusion_matter);
        return None;
    }
    if frame.packet_type == packet::P_FE2CL_REP_PC_TICK {
        if let Ok(tick) = PcTick0104::decode(&frame.payload) {
            apply_pc_tick_to_runtime(&tick, runtime);
            return Some(tick.hp);
        }
        return None;
    }
    if frame.packet_type == packet::P_FE2CL_CHAR_TIME_BUFF_TIME_TICK {
        if let Ok(tick) = ffone_protocol::TimeBuffHealTick0104::decode(&frame.payload)
            && tick.character_type == 1 && tick.character_id == player_id {
            runtime.hp = Some(tick.hp);
            return Some(tick.hp);
        }

        if let Ok(tick) = TimeBuffDotDamageTick0104::decode(&frame.payload)
            && tick.character_type == 1
            && tick.character_id == player_id
            && tick.time_buff_id == 17
        {
            runtime.hp = Some(tick.hp);
            if tick.protected {
                if let Some((active_index, _)) = active_world_nano_slot(runtime) {
                    runtime.nano_slots[active_index].stamina = tick.stamina;
                }
                if tick.nano_deactivated {
                    for slot in &mut runtime.nano_slots {
                        slot.active = false;
                    }
                    runtime.pending_nano_activation = None;
                }
            }
            return Some(tick.hp);
        }
        return None;
    }
    let Ok(Some(combat)) = decode_npc_combat_packet_0104(frame.packet_type, &frame.payload) else {
        return None;
    };
    match combat {
        NpcCombatPacket0104::PcAttackNpcsSuccess(success) => {
            runtime.weapon_battery = success.battery_w;
            None
        }
        NpcCombatPacket0104::PcAttackCharsSuccess(success) => {
            runtime.weapon_battery = success.battery_w;
            None
        }
        NpcCombatPacket0104::NpcAttackPcs(packet) => {
            if let Some(result) = packet.results.iter().find(|result| result.id == player_id) {
                runtime.hp = Some(result.hp);
                return Some(result.hp);
            }
            None
        }
        // Mixed PvP families carry NPC results under the same ID space, so a
        // local hit must also match the player entity type.
        NpcCombatPacket0104::NpcAttackChars(NpcAttackChars0104 { results, .. })
        | NpcCombatPacket0104::PcAttackChars(PcAttackChars0104 { results, .. }) => {
            if let Some(result) = results
                .iter()
                .find(|result| result.entity_type == 1 && result.id == player_id)
            {
                runtime.hp = Some(result.hp);
                return Some(result.hp);
            }
            None
        }
        _ => None,
    }
}

/// Applies shard-owned side effects from a live frame while preserving the
/// local virtual-server state owned by the tutorial. Retrobution does not let
/// an unrelated shard PC_TICK overwrite cntutorialscript's local damage or the
/// Buttercup Nano granted by InfectionC.
pub(super) fn apply_runtime_frame_with_authority(
    frame: &DecodedFrame,
    runtime: &mut RuntimeStatus,
    shard_owns_local_status: bool,
) -> Option<i32> {
    let previous_hp = runtime.hp;
    let previous_nano_slots = runtime.nano_slots;
    let updated_hp = apply_runtime_frame(frame, runtime);
    if shard_owns_local_status {
        return updated_hp;
    }
    runtime.hp = previous_hp;
    runtime.nano_slots = previous_nano_slots;
    None
}
