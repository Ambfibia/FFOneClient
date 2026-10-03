use super::*;

#[test]
fn clean_archive_and_serialized_ownership_anchors_are_exact() {
    assert_eq!(CASHMALL_SOURCE_BUILD, "retrobution-20260613");
    assert_eq!(CASHMALL_SOURCE_MAIN_ARCHIVE, "main.unity3d");
    assert_eq!(
        CASHMALL_SOURCE_MAIN_ARCHIVE_SHA256,
        "59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F"
    );
    assert_eq!(CASHMALL_SOURCE_SERIALIZED_FILE, "sharedassets0.assets");
    assert_eq!(CASHMALL_GAME_MODE_0104, 27);
    assert_eq!(CASHMALL_HIDDEN_CHAT_COMMAND_0104, "/cashmall");
    assert_eq!(
        CASHMALL_HIDDEN_CHAT_BOUNDARY_0104,
        CashmallHiddenChatBoundary0104 {
            request_game_mode_event: [2, 0],
            requested_game_mode: 27,
            receive_init_event: [2, 3, 0],
        }
    );
    assert_eq!(CASHMALL_GAME_OBJECT_PATH_ID, 1_359);
    assert_eq!(CASHMALL_TRANSFORM_PATH_ID, 1_243);
    assert_eq!(CASHMALL_MODE_COMPONENT_PATH_ID, 1_573);
    assert_eq!(CASHMALL_PANEL_COMPONENT_PATH_ID, 1_574);
    assert_eq!(CASHMALL_PC_STUFF_COMPONENT_PATH_ID, 1_575);
    assert_eq!(CASHMALL_EQUIP_COMPONENT_PATH_ID, 1_576);
    assert_eq!(CASHMALL_MODE_SCRIPT_PATH_ID, 1_120);
    assert_eq!(CASHMALL_PANEL_SCRIPT_PATH_ID, 1_077);
    assert_eq!(CASHMALL_PC_STUFF_SCRIPT_PATH_ID, 1_045);
    assert_eq!(CASHMALL_EQUIP_SCRIPT_PATH_ID, 1_027);
    assert_eq!(CASHMALL_SKIN_PATH_ID, 1_376);
    assert_eq!(CASHMALL_CASH_TEXTURE_PATH_ID, 551);
    assert_eq!(CASHMALL_INFO_TEXTURE_PATH_ID, 493);
    assert_eq!(CASHMALL_ITEM_BAR_PATH_ID, 0);
}

#[test]
fn serialized_skin_font_and_depth_anchors_are_exact() {
    assert_eq!(CASHMALL_INVENTORY_SKIN_PATH_ID, 1_366);
    assert_eq!(CASHMALL_TAB_VISUAL_FONT_PATH_ID, 903);
    assert_eq!(CASHMALL_LABEL_FONT_PATH_ID, 977);
    assert_eq!(CASHMALL_BUTTON_FONT_PATH_ID, 933);
    assert_eq!(CASHMALL_SMALL_FONT_PATH_ID, 970);
    assert_eq!(CASHMALL_GUI_DEPTH_0104, 10);
    assert_eq!(CASHMALL_SHARED_GUI_DEPTH_0104, 9);
    assert!(CASHMALL_SHARED_NATIVE_Z_0104 > CASHMALL_PANEL_NATIVE_Z_0104);
}
