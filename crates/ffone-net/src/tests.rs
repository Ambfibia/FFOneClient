use super::*;
use std::{net::TcpListener, thread};

use ffone_protocol::{
    BuddyBaseInfo0104, BuddyFindNameAcceptFailure0104, BuddyFindNameSuccess0104,
    BuddyListInfo0104, BuddyMakeSuccess0104, BuddyStateSuccess0104,
    BuddyWarpOtherShardSuccess0104, CountedPayloadError0104, EquipChangePacket0104,
    ItemBase0104, ItemMoveSuccessPacket0104, ItemUseSkillResults0104, ItemUseSuccessPrefix0104,
    ItemVendor0104, NANO_TUNE_ITEM_SLOT_COUNT_0104, NanoTuneFailure0104, NanoTuneSuccess0104,
    NpcAppearance0104, NpcMove0104, OnItem0104, OnItemIndex0104, PcAppearance0104,
    PcLoadingCompleteRequest, PcLoadingCompleteSuccess, PcStyle2Flags0104, PcStyle0104,
    PresentNpcTypesReply0104, QuickSlotEntry0104, QuickSlotInfo0104,
    TransportationAppearance0104, VENDOR_TABLE_ITEM_COUNT_0104, VendorStartSuccess0104,
    VendorTableUpdateSuccess0104, decode_client_frame, encode_client_frame,
    encode_server_frame,
};

mod constants;
mod codec;
mod input;
mod operations_fake_openfusion_login_to_loading_complete;
mod operations_fake_server_npc_combat_io_is_typed_and_lossl;
mod operations_fake_server_buddy_lifecycle_io_is_typed_stri;
mod output;
mod state;
mod commands;

use constants::{
    TEST_UID, TEST_SERIAL, TEST_PLAYER_ID, TEST_SERVER_TIME, TEST_FUSION_MATTER,
    PRE_ENTER_NANO, POST_ENTER_NANO
};
use codec::read_wire;
use input::read_client;
use operations_fake_openfusion_login_to_loading_complete::{
    send_server_bytes, send_server, put_utf16, serve_clean_login_roster
};
