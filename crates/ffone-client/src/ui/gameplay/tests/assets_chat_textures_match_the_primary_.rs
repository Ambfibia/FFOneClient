use super::*;

#[test]
fn chat_textures_match_the_primary_path_id_contracts() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for (path, expected_bytes, expected_hash) in [
        (
            QUICK_CHAT_BOX_PATH,
            QUICK_CHAT_BOX_BYTES,
            QUICK_CHAT_BOX_SHA256,
        ),
        (
            QUICK_CHAT_HOVER_PATH,
            QUICK_CHAT_HOVER_BYTES,
            QUICK_CHAT_HOVER_SHA256,
        ),
        (
            QUICK_CHAT_NEXT_HOVER_PATH,
            QUICK_CHAT_NEXT_HOVER_BYTES,
            QUICK_CHAT_NEXT_HOVER_SHA256,
        ),
        (
            QUICK_CHAT_NEXT_ARROW_PATH,
            QUICK_CHAT_NEXT_ARROW_BYTES,
            QUICK_CHAT_NEXT_ARROW_SHA256,
        ),
        (
            CHAT_RESIZE_NORMAL_PATH,
            CHAT_RESIZE_NORMAL_BYTES,
            CHAT_RESIZE_NORMAL_SHA256,
        ),
        (
            CHAT_RESIZE_HOVER_PATH,
            CHAT_RESIZE_HOVER_BYTES,
            CHAT_RESIZE_HOVER_SHA256,
        ),
        (
            CHAT_ACTIVE_TEXT_FIELD_PATH,
            CHAT_ACTIVE_TEXT_FIELD_BYTES,
            CHAT_ACTIVE_TEXT_FIELD_SHA256,
        ),
        (
            CHAT_EMPTY_STATE_BACKGROUND_PATH,
            CHAT_EMPTY_STATE_BACKGROUND_BYTES,
            CHAT_EMPTY_STATE_BACKGROUND_SHA256,
        ),
        (
            CHAT_BUDDY_ICON_PATH,
            CHAT_BUDDY_ICON_BYTES,
            CHAT_BUDDY_ICON_SHA256,
        ),
        (
            CHAT_GROUP_ICON_PATH,
            CHAT_GROUP_ICON_BYTES,
            CHAT_GROUP_ICON_SHA256,
        ),
        (
            CHAT_SCROLLBAR_TRACK_PATH,
            CHAT_SCROLLBAR_TRACK_BYTES,
            CHAT_SCROLLBAR_TRACK_SHA256,
        ),
        (
            CHAT_SCROLLBAR_THUMB_PATH,
            CHAT_SCROLLBAR_THUMB_BYTES,
            CHAT_SCROLLBAR_THUMB_SHA256,
        ),
        (
            CHAT_SCROLLBAR_UP_PATH,
            CHAT_SCROLLBAR_UP_BYTES,
            CHAT_SCROLLBAR_UP_SHA256,
        ),
        (
            CHAT_SCROLLBAR_DOWN_PATH,
            CHAT_SCROLLBAR_DOWN_BYTES,
            CHAT_SCROLLBAR_DOWN_SHA256,
        ),
    ] {
        let path = asset_root.join(path);
        let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
        assert_eq!(bytes.len() as u64, expected_bytes);
        assert_eq!(format!("{:x}", sha2::Sha256::digest(bytes)), expected_hash);
    }
    for index in 0..QUICK_CHAT_EMOTE_ICON_PATHS.len() {
        let path = QUICK_CHAT_EMOTE_ICON_PATHS[index];
        let bytes = std::fs::read(asset_root.join(path)).expect("read exact emote icon");
        assert_eq!(
            bytes.len() as u64,
            QUICK_CHAT_EMOTE_ICON_BYTES[index],
            "{path}"
        );
        assert_eq!(
            format!("{:x}", sha2::Sha256::digest(bytes)),
            QUICK_CHAT_EMOTE_ICON_SHA256[index],
            "{path}"
        );
    }
}

#[test]
fn npc_barker_textures_match_the_primary_path_id_contracts() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for (path, expected_bytes, expected_hash) in [
        (
            NPC_BARKER_BOX_PATH,
            NPC_BARKER_BOX_BYTES,
            NPC_BARKER_BOX_SHA256,
        ),
        (
            NPC_BARKER_TAIL_PATH,
            NPC_BARKER_TAIL_BYTES,
            NPC_BARKER_TAIL_SHA256,
        ),
    ] {
        let path = asset_root.join(path);
        let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
        assert_eq!(bytes.len() as u64, expected_bytes);
        assert_eq!(format!("{:x}", sha2::Sha256::digest(bytes)), expected_hash);
    }
}

#[test]
fn player_freechat_textures_match_the_primary_path_id_contracts() {
    let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    for (path, expected_bytes, expected_hash) in [
        (
            PLAYER_FREECHAT_BOX_PATH,
            PLAYER_FREECHAT_BOX_BYTES,
            PLAYER_FREECHAT_BOX_SHA256,
        ),
        (
            PLAYER_FREECHAT_TAIL_PATH,
            PLAYER_FREECHAT_TAIL_BYTES,
            PLAYER_FREECHAT_TAIL_SHA256,
        ),
    ] {
        let path = asset_root.join(path);
        let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
        assert_eq!(bytes.len() as u64, expected_bytes);
        assert_eq!(format!("{:x}", sha2::Sha256::digest(bytes)), expected_hash);
    }
}

#[test]
fn normal_world_minimap_uses_exact_table_gate_catalog_and_mission_priority() {
    let definition = |npc_class, sound, map_icon| GameplayNpcMinimapDefinition {
        npc_type: 700,
        npc_class,
        sound,
        map_icon,
    };

    let npc = normal_world_minimap_marker_style(definition(3, 1, 31), false, false, false).unwrap();
    let MinimapMarkerIcon::TableData(npc_icon) = npc.icon else {
        panic!("ordinary NPC must retain its TableData marker")
    };
    assert_eq!(npc_icon.index(), 31);
    assert_eq!(npc.source_dimensions, Vec2::splat(16.0));
    let fusion =
        normal_world_minimap_marker_style(definition(0, 2, 19), false, false, false).unwrap();
    assert!(matches!(fusion.icon, MinimapMarkerIcon::TableData(_)));

    assert_eq!(
        normal_world_minimap_marker_style(definition(0, 1, 20), false, false, false),
        None
    );
    assert!(normal_world_minimap_marker_style(definition(0, 1, 20), true, false, false).is_some());

    assert_eq!(
        normal_world_minimap_marker_style(definition(3, 1, 0), false, false, false),
        None
    );
    let advance =
        normal_world_minimap_marker_style(definition(3, 1, 0), false, true, true).unwrap();
    assert_eq!(advance.icon, MinimapMarkerIcon::Advance);
    assert_eq!(advance.source_dimensions, Vec2::splat(18.0));
    let new = normal_world_minimap_marker_style(definition(3, 1, 0), false, false, true).unwrap();
    assert_eq!(new.icon, MinimapMarkerIcon::New);

    for map_icon in 1..35 {
        let style =
            normal_world_minimap_marker_style(definition(3, 1, map_icon), false, false, false)
                .unwrap();
        assert_eq!(
            style.icon.asset_path(),
            WORLD_MAP_MARKER_PATHS[map_icon as usize]
        );
    }
    for map_icon in [-1, 35, i32::MAX] {
        assert_eq!(
            normal_world_minimap_marker_style(definition(3, 1, map_icon), false, true, true,),
            None
        );
    }
}
