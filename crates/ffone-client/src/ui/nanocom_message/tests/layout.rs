use super::*;

#[test]
fn exact_geometry_and_semantic_asset_routes_are_stable() {
    assert_eq!(NANOCOM_SOURCE_BUILD, "retrobution-20260613");
    assert_eq!(NANOCOM_SOURCE_MAIN_ARCHIVE_BYTES, 7_000_415);
    assert_eq!(NANOCOM_SOURCE_ASSEMBLY_BYTES, 1_517_568);
    assert_eq!(
        NANOCOM_SOURCE_ASSEMBLY_SHA256,
        "33D6F70216B1C7BA05BCC0F270FBA97E767B129159755AF4C8835922E60ACADB"
    );
    assert_eq!(NANOCOM_SOURCE_UI_CLASS, "cnGUINanocom");
    assert_eq!(
        NANOCOM_SOURCE_LOGIN_METHOD,
        "cnMissionManager.ReceiveStartGames"
    );
    assert_eq!(
        NANOCOM_SOURCE_MISSION_METHOD,
        "cnMissionManager.SetMissionMessage"
    );
    assert_eq!(NANOCOM_SOURCE_ICONS_ARCHIVE, "Icons.resourceFile");
    assert_eq!(NANOCOM_SOURCE_ICONS_ARCHIVE_BYTES, 5_800_411);
    assert_eq!(
        NANOCOM_SOURCE_ICONS_ARCHIVE_SHA256,
        "A05602D6E96E2E74ECAD207F42E519605259434210DE8DA19E422B30D642E544"
    );
    assert_eq!(NANOCOM_GAME_OBJECT_PATH_ID, 1_352);
    assert_eq!(NANOCOM_COMPONENT_PATH_ID, 1_562);
    assert_eq!(NANOCOM_SCRIPT_PATH_ID, 1_112);
    assert_eq!(NANOCOM_HUD_SKIN_PATH_ID, 1_372);
    assert_eq!(NANOCOM_GUI_DEPTH, 8);
    assert_eq!(
        NANOCOM_NETWORK_PREFAB_GUID,
        [2_679_906_017, 490_896_470, 355_223_436, 3_256_557_672]
    );
    assert_eq!(
        NANOCOM_COMPACT_FRAME_RECT,
        NanocomRect::new(51.0, 3.0, 321.0, 119.0)
    );
    assert_eq!(
        NANOCOM_NANO_FRAME_RECT,
        NanocomRect::new(0.0, 0.0, 372.0, 122.0)
    );
    assert_eq!(
        NANOCOM_NANO_ICON_RECT,
        NanocomRect::new(55.0, 10.0, 64.0, 64.0)
    );
    assert_eq!(
        NANOCOM_COMPACT_BODY_CONTENT_RECT,
        NanocomRect::new(130.0, 29.0, 164.0, 60.0)
    );
    assert_eq!(
        NANOCOM_EXPANDED_DIALOG_RECT,
        NanocomRect::new(0.0, 0.0, 520.0, 164.0)
    );
    assert_eq!(
        NANOCOM_EXPANDED_MESSAGE_AREA_RECT,
        NanocomRect::new(70.0, 30.0, 415.0, 87.0)
    );
    assert_eq!(
        NANOCOM_DECLINE_RECT,
        NanocomRect::new(44.0, 124.0, 150.0, 25.0)
    );
    assert_eq!(
        NANOCOM_ACCEPT_RECT,
        NanocomRect::new(334.0, 124.0, 150.0, 25.0)
    );
    for path in [
        NANOCOM_BUDDY_FRAME_PATH,
        NANOCOM_BUDDY_ICON_PATH,
        NANOCOM_GROUP_ICON_PATH,
        NANOCOM_TYPE_9_FRAME_PATH,
        NANOCOM_NANO_FRAME_PATH,
        NANOCOM_NUMBUH_TWO_ICON_PATH,
        NANOCOM_DIALOG_PATH,
        NANOCOM_MESSAGE_AREA_PATH,
        NANOCOM_BLUE_BUTTON_PATH,
        NANOCOM_BLUE_BUTTON_OVER_PATH,
        NANOCOM_RED_BUTTON_PATH,
        NANOCOM_RED_BUTTON_OVER_PATH,
        NANOCOM_JEFFE_FONT_PATH,
        NANOCOM_CHALET_FONT_PATH,
    ] {
        assert!(!path.contains("--"), "{path} exposes a content hash");
        assert!(path.contains('/'), "{path} has no semantic owner directory");
    }
    assert_eq!(NANOCOM_REACHED_TEXTURES.len(), 12);
    assert_eq!(NANOCOM_REACHED_TEXTURES[0].source_path_id, 290);
    assert_eq!(NANOCOM_REACHED_TEXTURES[5].source_path_id, 398);
    assert_eq!(NANOCOM_REACHED_TEXTURES[6].source_path_id, 233);
    assert_eq!(NANOCOM_REACHED_TEXTURES[11].source_path_id, 137);
}

#[test]
fn expanded_layout_scales_around_viewport_center() {
    let layout = nanocom_expanded_layout(Vec2::new(1_264.0, 681.0), 2.0);
    assert_eq!(layout.node, NanocomRect::new(372.0, 258.5, 520.0, 164.0));
    assert_eq!(
        layout.painted,
        NanocomRect::new(112.0, 176.5, 1_040.0, 328.0)
    );
    assert_eq!(clean_nanocom_ui_scale(681.0), 1.0);
    assert_close(clean_nanocom_ui_scale(1_536.0), 2.1);
}
