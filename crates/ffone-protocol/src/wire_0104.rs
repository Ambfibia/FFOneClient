//! Exact marshaled mirror of every `[StructLayout]` struct in the clean
//! Retrobution `Assembly-CSharp` (protocol 0104).
//!
//! GENERATED FILE. Do not edit by hand. Regenerate with
//! `../FusionForge/tools/legacy-sources/generate-wire-0104.py`.
//!
//! Every type below reproduces the original Mono marshaled layout byte for
//! byte: the generator recomputes each offset with the `#pragma pack` rules,
//! compares the result with the decompiled `Size = N` attribute and documents
//! the single struct where Retrobution left a stale attribute. Names are
//! derived mechanically from the C# identifiers; the original field name is
//! kept in each field's doc comment.
//!
//! These are *wire* types. Curated semantic types with validation live at the
//! crate root; where both exist, `tests::hand_written_sizes_agree_with_mirror`
//! proves the two agree on the fixed payload size.
//!
//! [`declared_struct_size`] reports the marshaled struct size of a packet ID.
//! Packets that carry a counted trailer (`iNPCCnt`, `iTargetCnt`, ...) are
//! longer on the wire than their declared struct; [`frame_kind`] tells the
//! three shapes apart using OpenFusion's own descriptor table
//! (`core/Packets.cpp`), and [`fixed_frame_size`] answers only for packets the
//! server validates as fixed-length. `fixed_payload_size` at the crate root
//! remains the authority for strict length classification; it falls back to
//! [`fixed_frame_size`] for packets without a hand-written codec.

#![allow(clippy::too_many_lines)]

use crate::{FixedUtf16, PayloadError, WirePayload};

fn require_size(bytes: &[u8], expected: usize) -> Result<(), PayloadError> {
    if bytes.len() == expected {
        Ok(())
    } else {
        Err(PayloadError::WrongSize {
            expected,
            actual: bytes.len(),
        })
    }
}

#[inline]
fn write_prim(out: &mut [u8], offset: usize, value: &[u8]) {
    out[offset..offset + value.len()].copy_from_slice(value);
}

#[inline]
fn read_array<const N: usize>(bytes: &[u8], offset: usize) -> [u8; N] {
    bytes[offset..offset + N].try_into().expect("fixed slice")
}

fn write_utf16<const N: usize>(out: &mut [u8], offset: usize, value: &FixedUtf16<N>) {
    for (index, unit) in value.as_units().iter().enumerate() {
        let start = offset + index * 2;
        out[start..start + 2].copy_from_slice(&unit.to_le_bytes());
    }
}

fn read_utf16<const N: usize>(bytes: &[u8], offset: usize) -> FixedUtf16<N> {
    FixedUtf16::from_units(std::array::from_fn(|index| {
        let start = offset + index * 2;
        u16::from_le_bytes([bytes[start], bytes[start + 1]])
    }))
}
mod declared_sizes;
pub use declared_sizes::declared_struct_size;

mod packet_names;
pub use packet_names::PACKET_IDS_0104;


/// Shape of one packet on the wire.
///
/// Kinds come from OpenFusion's descriptor table (`core/Packets.cpp`);
/// header sizes and count offsets come from the clean-client layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameKind0104 {
    /// The payload is exactly the declared struct.
    Fixed { size: usize },
    /// The declared struct is followed by `count` trailer records, where
    /// `count` is the little-endian signed integer of `count_width` bytes
    /// stored at `count_offset` inside the header.
    Counted {
        header_size: usize,
        count_offset: usize,
        count_width: usize,
        trailer_size: usize,
    },
    /// Variadic with a trailer whose type depends on the body; only a
    /// dedicated decoder can validate it.
    Manual { header_size: usize },
}
/// Wire shape from the original descriptor table.
#[must_use]
pub fn frame_kind(packet_id: u32) -> Option<FrameKind0104> {
    match packet_id >> 7 {
        0x240000 => frames_240000::frame_kind(packet_id),
        0x260000 => frames_260000::frame_kind(packet_id),
        0x260001 => frames_260001::frame_kind(packet_id),
        0x420000 => frames_420000::frame_kind(packet_id),
        0x620000 => frames_620000::frame_kind(packet_id),
        0x620001 => frames_620001::frame_kind(packet_id),
        0x620002 => frames_620002::frame_kind(packet_id),
        0x1060000 => frames_1060000::frame_kind(packet_id),
        _ => None,
    }
}

mod frames_240000;

mod frames_260000;

mod frames_260001;

mod frames_420000;

mod frames_620000;

mod frames_620001;

mod frames_620002;

mod frames_1060000;


/// Exact payload length for packets the server validates as fixed-length.
#[must_use]
pub fn fixed_frame_size(packet_id: u32) -> Option<usize> {
    match frame_kind(packet_id)? {
        FrameKind0104::Fixed { size } => Some(size),
        FrameKind0104::Counted { .. } | FrameKind0104::Manual { .. } => None,
    }
}
mod server_differences;
pub use server_differences::OPENFUSION_SIZE_DIVERGENCES_0104;

mod records_attack_result;
pub use records_attack_result::*;

mod records_item_trade;
pub use records_item_trade::*;

mod records_pc_group_member_info_record;
pub use records_pc_group_member_info_record::*;

mod records_running_quest;
pub use records_running_quest::*;

mod records_skill_result_stamina_self;
pub use records_skill_result_stamina_self::*;

mod packets_12_ls_login_request;
pub use packets_12_ls_login_request::*;

mod packets_13_pc_enter_request;
pub use packets_13_pc_enter_request::*;

mod packets_13_nano_skill_use_request;
pub use packets_13_nano_skill_use_request::*;

mod packets_13_pc_trade_offer_request;
pub use packets_13_pc_trade_offer_request::*;

mod packets_13_pc_combat_begin_request;
pub use packets_13_pc_combat_begin_request::*;

mod packets_13_pc_slope_request;
pub use packets_13_pc_slope_request::*;

mod packets_13_get_group_style_request;
pub use packets_13_get_group_style_request::*;

mod packets_13_barker_request;
pub use packets_13_barker_request::*;

mod packets_13_live_check_reply;
pub use packets_13_live_check_reply::*;

mod packets_13_pc_time_to_go_warp_request;
pub use packets_13_pc_time_to_go_warp_request::*;

mod packets_13_pc_skill_add_request;
pub use packets_13_pc_skill_add_request::*;

mod packets_21_ls_login_success;
pub use packets_21_ls_login_success::*;

mod packets_21_ls_check_name_list_failure;
pub use packets_21_ls_check_name_list_failure::*;

mod packets_31_error;
pub use packets_31_error::*;

mod packets_31_around_del_npc;
pub use packets_31_around_del_npc::*;

mod packets_31_npc_skill_hit;
pub use packets_31_npc_skill_hit::*;

mod packets_31_charge_nano_stamina_reply;
pub use packets_31_charge_nano_stamina_reply::*;

mod packets_31_pc_grenade_style_hit;
pub use packets_31_pc_grenade_style_hit::*;

mod packets_31_pc_trade_cash_register_failure;
pub use packets_31_pc_trade_cash_register_failure::*;

mod packets_31_pc_buddylist_info_success;
pub use packets_31_pc_buddylist_info_success::*;

mod packets_31_pc_jumppad;
pub use packets_31_pc_jumppad::*;

mod packets_31_npc_rocket_style_fire;
pub use packets_31_npc_rocket_style_fire::*;

mod packets_31_pc_change_mentor_success;
pub use packets_31_pc_change_mentor_success::*;

mod packets_31_ep_race_start_failure;
pub use packets_31_ep_race_start_failure::*;

mod packets_31_send_all_group_freechat_message_failure;
pub use packets_31_send_all_group_freechat_message_failure::*;

mod packets_31_gm_pc_change_value;
pub use packets_31_gm_pc_change_value::*;

mod packets_31_pc_riding_failure;
pub use packets_31_pc_riding_failure::*;

mod packets_31_pc_recv_email_item_success;
pub use packets_31_pc_recv_email_item_success::*;

mod packets_31_channel_info_reply;
pub use packets_31_channel_info_reply::*;

mod packets_31_pc_streetstall_unregist_item_success;
pub use packets_31_pc_streetstall_unregist_item_success::*;

mod packets_31_pc_skill_use_success;
pub use packets_31_pc_skill_use_success::*;

mod packets_31_gm_reward_rate_success;
pub use packets_31_gm_reward_rate_success::*;

mod packets_83_request_make_buddy_success;
pub use packets_83_request_make_buddy_success::*;

#[cfg(test)]
mod tests;
