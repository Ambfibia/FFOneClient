use crate::entity_lifecycle::*;
use ffone_protocol::{
    FixedUtf16, NanoSkillUseSuccessPrefix0104, NpcSkillCorruptionHitPrefix0104,
    NpcSkillHitPrefix0104, PcMoveRequest0104, ShinyAppearance0104, packet,
};

mod constants;
mod assets;
mod operations_skill_movement_snaps_remote_entities_and_dia;
mod operations_live_transportation_and_shiny_packets_upsert;
mod codec;
mod output;
mod systems;
mod state;
mod commands;
mod validation;
mod animation;
mod equipment;

use constants::{EPOCH_ONE, EPOCH_TWO, LOCAL_PC_ID};
use operations_skill_movement_snaps_remote_entities_and_dia::{
    test_app, item, pc_appearance, npc_appearance, skill_target_record, pc_move, begin
};
use codec::{
    frame, npc_skill_payload, npc_skill_hit_payload, npc_corruption_hit_payload,
    nano_skill_payload, push_frame
};
use output::{write_i16, write_i32};
