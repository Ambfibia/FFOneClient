use super::*;

/// Mirrors `CNSocketEncryption::createNewKey`, including unsigned wrapping math.
pub const fn derive_key(time_or_seed: u64, iv1: i32, iv2: i32) -> u64 {
    let factor1 = iv1.wrapping_add(1) as u64;
    let factor2 = iv2.wrapping_add(1) as u64;
    DEFAULT_KEY.wrapping_mul(time_or_seed.wrapping_mul(factor1).wrapping_mul(factor2))
}

/// E key installed after `P_LS2CL_REP_LOGIN_SUCC`.
pub const fn derive_login_e_key(server_time: u64, character_count: i8, selected_slot: i8) -> u64 {
    derive_key(
        server_time,
        (character_count as i32).wrapping_add(1),
        (selected_slot as i32).wrapping_add(1),
    )
}

/// FE key derived from `iClientVerC` and transferred to the shard with the enter serial.
pub const fn derive_frontend_key(client_version_c: i32) -> u64 {
    derive_key(DEFAULT_KEY, client_version_c, 1)
}

/// E key installed after `P_FE2CL_REP_PC_ENTER_SUCC`.
pub const fn derive_shard_e_key(server_time: u64, player_id: i32, fusion_matter: i32) -> u64 {
    derive_key(
        server_time,
        player_id.wrapping_add(1),
        fusion_matter.wrapping_add(1),
    )
}

/// Encrypt one body in place: XOR followed by the legacy byte swap.
pub fn encrypt_in_place(data: &mut [u8], key: u64) {
    xor_in_place(data, key);
    byte_swap_in_place(data);
}

/// Decrypt one body in place: byte swap followed by XOR.
pub fn decrypt_in_place(data: &mut [u8], key: u64) {
    byte_swap_in_place(data);
    xor_in_place(data, key);
}

pub(super) fn xor_in_place(data: &mut [u8], key: u64) {
    let key = key.to_le_bytes();
    for (index, byte) in data.iter_mut().enumerate() {
        *byte ^= key[index % key.len()];
    }
}

pub(super) fn byte_swap_in_place(data: &mut [u8]) {
    let block_size = data.len() % 5 * 2 + 8;
    let mut block_start = 0usize;
    let mut swap_offset = 0usize;
    while block_start + block_size <= data.len() {
        data.swap(
            block_start + swap_offset,
            block_start + block_size - 1 - swap_offset,
        );
        block_start += block_size;
        swap_offset += 1;
        if swap_offset > block_size / 2 {
            swap_offset = 0;
        }
    }
}

/// Exact OpenFusion result-record width selected by `Abilities::handleSkill`.
/// Skill types which cannot produce a Nano-skill success packet return None.
#[must_use]
pub const fn nano_skill_result_size_0104(skill_type: i32) -> Option<usize> {
    match skill_type {
        26 => Some(12),
        2 | 7 | 10 | 11 | 12 | 14 | 15 | 16 | 17 | 18 | 19 | 20 | 25 | 31 | 32 | 33 | 34 | 35 => {
            Some(16)
        }
        1 | 22 | 38 | 39 => Some(20),
        27 | 28 => Some(24),
        3 | 4 | 5 | 8 => Some(32),
        30 => Some(36),
        21 => Some(40),
        _ => None,
    }
}

/// Exact OpenFusion result width for `sP_FE2CL_NPC_SKILL_HIT`. Its ordinary
/// result families are the same `Abilities::handleSkill` records used by Nano
/// skills. `RETROROCKET_SELF` (eST 29) is an explicit server no-op and may
/// only produce a header with zero results.
#[must_use]
pub const fn npc_skill_result_size_0104(skill_type: i32) -> Option<usize> {
    match skill_type {
        29 => Some(0),
        _ => nano_skill_result_size_0104(skill_type),
    }
}

pub(super) fn checked_npc_skill_target_count(
    packet: &'static str,
    target_count: i32,
    maximum: usize,
    allow_empty: bool,
) -> Result<usize, NpcSkillAuthorityDecodeError0104> {
    if target_count < 0 {
        return Err(NpcSkillAuthorityDecodeError0104::NegativeTargetCount {
            packet,
            target_count,
        });
    }
    if !allow_empty && target_count == 0 {
        return Err(NpcSkillAuthorityDecodeError0104::EmptyTargetResults { packet });
    }
    let count = usize::try_from(target_count).expect("non-negative i32 target count fits usize");
    if count > maximum {
        return Err(NpcSkillAuthorityDecodeError0104::TargetCountTooLarge {
            packet,
            target_count,
            maximum,
        });
    }
    Ok(count)
}

/// `CL2LS` (`0x12..`) and `CL2FE` (`0x13..`) packet families.
#[must_use]
pub const fn is_client_to_server_0104(packet_id: u32) -> bool {
    matches!(packet_id & 0xff00_0000, 0x1200_0000 | 0x1300_0000)
}
