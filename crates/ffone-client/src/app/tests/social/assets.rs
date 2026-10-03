use super::*;

#[test]
fn ordinary_nanocom_routes_world_map_reuses_asset_and_authoritative_player_readiness() {
    let player = WorldMapPlayer::new(WorldMapPoint::new(12.0, 3.0, 45.0), 271.5);
    let catalog = RaceRankCatalog::default();

    let mut loading = WorldMapPresentation::default();
    assert_eq!(
        open_world_map_for_ready_player(
            &mut loading,
            &WorldMapPresentationAssetStatus::Loading,
            Some(player),
            Some(0),
            &catalog,
        ),
        Err("WorldMap blocked while its exact native assets are still loading".to_owned())
    );
    assert_eq!(loading.model.phase(), WorldMapPhase::Closed);

    let mut failed = WorldMapPresentation::default();
    assert_eq!(
        open_world_map_for_ready_player(
            &mut failed,
            &WorldMapPresentationAssetStatus::Failed {
                asset_path: "ui/worldmap/exact.png",
            },
            Some(player),
            Some(0),
            &catalog,
        ),
        Err("WorldMap blocked by missing exact asset ui/worldmap/exact.png".to_owned())
    );
    assert_eq!(failed.model.phase(), WorldMapPhase::Closed);

    let mut missing_player = WorldMapPresentation::default();
    assert_eq!(
        open_world_map_for_ready_player(
            &mut missing_player,
            &WorldMapPresentationAssetStatus::Ready,
            None,
            Some(0),
            &catalog,
        ),
        Err("WorldMap blocked before the authoritative local player is ready".to_owned())
    );
    assert_eq!(missing_player.model.phase(), WorldMapPhase::Closed);

    let mut ready = WorldMapPresentation::default();
    assert_eq!(
        open_world_map_for_ready_player(
            &mut ready,
            &WorldMapPresentationAssetStatus::Ready,
            Some(player),
            Some(0),
            &catalog,
        ),
        Ok(())
    );
    assert_ne!(ready.model.phase(), WorldMapPhase::Closed);
}

#[test]
fn asset_residency_releases_groups_outside_their_state_scope() {
    let mut residency = AssetResidency::default();
    residency.groups.insert(
        AssetResidencyGroupId::DexterShip,
        ResidentAssetGroup::default(),
    );
    residency.groups.insert(
        AssetResidencyGroupId::Tutorial,
        ResidentAssetGroup::default(),
    );
    residency.release_except(&[AssetResidencyGroupId::Tutorial]);
    assert!(residency.group(AssetResidencyGroupId::DexterShip).is_none());
    assert!(residency.group(AssetResidencyGroupId::Tutorial).is_some());
}

#[test]
fn asset_residency_groups_follow_transition_lookahead_and_release_boundaries() {
    assert_eq!(
        asset_residency_groups_for_state(ClientState::CharacterSelect),
        &[AssetResidencyGroupId::DexterShip]
    );
    assert_eq!(
        asset_residency_groups_for_state(ClientState::TutorialIntro),
        &[
            AssetResidencyGroupId::DexterShip,
            AssetResidencyGroupId::Tutorial,
        ]
    );
    assert_eq!(
        asset_residency_groups_for_state(ClientState::Tutorial),
        &[AssetResidencyGroupId::Tutorial]
    );
    assert!(asset_residency_groups_for_state(ClientState::World).is_empty());
}
