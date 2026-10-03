use super::*;

#[test]
fn skyway_inventory_stays_plain_while_special_traversals_block_the_pose() {
    assert!(traversal_allows_inventory_pose(
        LegacyAvatarTraversalPresentation::None
    ));
    assert!(traversal_allows_inventory_pose(
        LegacyAvatarTraversalPresentation::Slope
    ));
    for traversal in [
        LegacyAvatarTraversalPresentation::Zipline,
        LegacyAvatarTraversalPresentation::RopeDrop,
        LegacyAvatarTraversalPresentation::RopeLeft,
        LegacyAvatarTraversalPresentation::RopeRight,
        LegacyAvatarTraversalPresentation::RopeStand1,
        LegacyAvatarTraversalPresentation::RopeStand2,
        LegacyAvatarTraversalPresentation::RopeTurn,
        LegacyAvatarTraversalPresentation::RopeUp,
    ] {
        assert!(!traversal_allows_inventory_pose(traversal));
    }
}
