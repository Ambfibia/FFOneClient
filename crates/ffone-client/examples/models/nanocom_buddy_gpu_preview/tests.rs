use super::*;

#[test]
fn preview_exposes_only_production_reached_message_types() {
    assert_eq!(
        PreviewStage::NpcCompact.expected_kind(),
        NanocomMessageKind::Npc
    );
    assert_eq!(
        PreviewStage::ComputressCompact.expected_kind(),
        NanocomMessageKind::Npc
    );
    assert_eq!(
        PreviewStage::BuddyModal.expected_kind(),
        NanocomMessageKind::BuddyInvite
    );
    assert_eq!(
        PreviewStage::GroupModal.expected_kind(),
        NanocomMessageKind::GroupInvite
    );
    assert!(PreviewStage::BuddyModal.is_modal());
    assert!(PreviewStage::GroupModal.is_modal());
    assert!(!PreviewStage::NpcCompact.is_modal());
    assert!(!PreviewStage::ComputressCompact.is_modal());
    assert!(PreviewStage::parse("trade-modal").is_none());
}

#[test]
fn acceptance_viewport_and_stage_slugs_are_stable() {
    assert_eq!((CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT), (1_264, 681));
    assert_eq!(PreviewStage::BuddyCompact.slug(), "buddy-compact");
    assert_eq!(PreviewStage::ComputressCompact.slug(), "computress-compact");
    assert_eq!(PreviewStage::GroupModal.slug(), "group-modal");
}
