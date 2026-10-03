use super::*;

#[test]
fn active_nano_info_texture_matches_the_primary_path_id_53_contract() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../assets/game")
        .join(ACTIVE_NANO_INFO_TEXTURE_PATH);
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("read {path:?}: {error}"));
    assert_eq!(bytes.len() as u64, ACTIVE_NANO_INFO_TEXTURE_BYTES);
    assert_eq!(
        format!("{:x}", sha2::Sha256::digest(bytes)),
        ACTIVE_NANO_INFO_TEXTURE_SHA256
    );
}

#[test]
fn tutorial_talk_icon_uses_exact_print_name_range_and_texture_route() {
    let npc = Entity::from_bits(42);
    let far = LegacyFocusedTarget {
        entity: npc,
        kind: LegacyTargetKind::Npc { team: 1 },
        distance: 6.0,
        talk_enabled: false,
    };
    assert!(!friendly_target_ui_visible(far));
    assert_eq!(
        primary_target_icon(&LegacyTargetSelection {
            focused_npc: Some(far),
            check_attack_target: false,
            ..default()
        }),
        None
    );

    let near = LegacyFocusedTarget {
        distance: 5.99,
        talk_enabled: true,
        ..far
    };
    assert!(friendly_target_ui_visible(near));
    assert_eq!(
        primary_target_icon(&LegacyTargetSelection {
            focused_npc: Some(near),
            check_attack_target: false,
            ..default()
        }),
        Some((npc, PrimaryTargetIconKind::Talk))
    );
    assert_eq!(
        NPC_TALK_TARGET_ICON_PATH,
        "ui/en/gameplay/shared/target_icon2.png"
    );
    assert_eq!(NPC_TALK_TARGET_ICON_SIZE, Vec2::new(42.0, 41.0));
}
