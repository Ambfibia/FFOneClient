use super::*;

pub(super) fn session() -> VendorSession0104 {
    VendorSession0104 {
        requested_npc_id: NPC_ID,
        table_vendor_id: TABLE_VENDOR_ID,
        accepted_npc_id: NPC_ID,
    }
}
