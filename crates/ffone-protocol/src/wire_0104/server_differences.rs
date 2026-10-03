// Generated table; do not edit by hand.
/// Packets whose clean-client size differs from the pinned OpenFusion
/// `structs/0104.hpp`: `(name, id, retrobution_size, openfusion_size)`.
///
/// Hand-written codecs may deliberately follow the server side for these;
/// the mirror always follows the client. Source:
/// `../FusionForge/docs/reference/evidence/legacy/ffone/managed-code/wire-0104-openfusion-divergence.json`.
pub const OPENFUSION_SIZE_DIVERGENCES_0104: &[(&str, u32, usize, usize)] = &[
    ("P_CL2FE_REQ_PC_BANK_OPEN", 0x1300002e, 8, 4),
    ("P_CL2FE_REQ_PC_WARP_USE_TRANSPORTATION", 0x13000069, 20, 16),
    ("P_FE2CL_REP_PC_ENTER_SUCC", 0x31000002, 2564, 2700),
    ("P_FE2CL_NPC_SKILL_READY", 0x31000020, 24, 20),
    ("P_FE2CL_REP_PC_BANK_OPEN_SUCC", 0x31000056, 2404, 1432),
    (
        "P_FE2CL_REP_PC_VENDOR_TABLE_UPDATE_SUCC",
        0x3100005c,
        1200,
        480,
    ),
];
