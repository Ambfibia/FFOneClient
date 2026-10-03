use super::*;

#[derive(Default)]
pub(super) struct TestCatalog {
    pub(super) entries: BTreeMap<i32, WorldMapCatalogLookup<WorldMapNpcCatalogEntry>>,
}

impl WorldMapCatalog for TestCatalog {
    fn npc(&self, npc_type: i32) -> WorldMapCatalogLookup<WorldMapNpcCatalogEntry> {
        self.entries
            .get(&npc_type)
            .cloned()
            .unwrap_or(WorldMapCatalogLookup::Missing)
    }
}

pub(super) fn catalog_entry(
    name: &str,
    map_icon: i32,
    mission: WorldMapMissionAvailability,
) -> WorldMapCatalogLookup<WorldMapNpcCatalogEntry> {
    WorldMapCatalogLookup::Unique(WorldMapNpcCatalogEntry {
        display_name: name.to_owned(),
        map_icon,
        mission,
    })
}

#[test]
fn marker_projection_uses_present_catalog_mission_priority_and_clean_order() {
    let mut model = open_other_local();
    model
        .apply_present_npc_types(true, 1, &[10, 11, 12, 13])
        .unwrap();
    model
        .set_waypoint(Some(WorldMapPoint::new(7000.0, 100.0, 4096.0)))
        .unwrap();
    let catalog = TestCatalog {
        entries: BTreeMap::from([
            (
                10,
                catalog_entry("Vendor", 4, WorldMapMissionAvailability::None),
            ),
            (
                11,
                catalog_entry("Starter", 5, WorldMapMissionAvailability::New),
            ),
            (
                12,
                catalog_entry("Finisher", 6, WorldMapMissionAvailability::NewAndAdvance),
            ),
            (
                13,
                catalog_entry("Hidden", 7, WorldMapMissionAvailability::None),
            ),
        ]),
    };
    let npcs = [
        WorldMapNpcSource {
            npc_type: 10,
            position: WorldMapPoint::new(4050.0, 0.0, 4096.0),
        },
        WorldMapNpcSource {
            npc_type: 11,
            position: WorldMapPoint::new(4096.0, 0.0, 4050.0),
        },
        WorldMapNpcSource {
            npc_type: 12,
            position: WorldMapPoint::new(4140.0, 0.0, 4050.0),
        },
        // A Future NPC is excluded while the current zone is Other.
        WorldMapNpcSource {
            npc_type: 13,
            position: WorldMapPoint::new(6000.0, 0.0, 1000.0),
        },
    ];
    let markers = model.project_markers(&catalog, &npcs).unwrap();
    assert!(matches!(
        markers[0].kind,
        WorldMapMarkerKind::Npc {
            npc_type: 10,
            map_icon: 4,
            ..
        }
    ));
    assert!(matches!(
        markers[1].kind,
        WorldMapMarkerKind::MissionNew { npc_type: 11, .. }
    ));
    assert!(matches!(
        markers[2].kind,
        WorldMapMarkerKind::MissionAdvance { npc_type: 12, .. }
    ));
    let WorldMapMarkerKind::WaypointOffscreenArrow { rotation_degrees } = markers[3].kind
    else {
        panic!("expected clipped waypoint");
    };
    assert_close(rotation_degrees, 90.0);
    assert!(matches!(
        markers[4].kind,
        WorldMapMarkerKind::Player { yaw_degrees: 37.0 }
    ));
    assert_eq!(markers.len(), 5);
    assert!(
        markers
            .iter()
            .all(|marker| marker.rect.width == 16.0 && marker.rect.height == 16.0)
    );
}

#[test]
fn ambiguous_catalog_invalid_icons_and_nonfinite_sources_fail_closed() {
    let mut model = open_other_local();
    model.apply_present_npc_types(true, 1, &[10]).unwrap();
    let source = [WorldMapNpcSource {
        npc_type: 10,
        position: WorldMapPoint::new(4096.0, 0.0, 4050.0),
    }];
    let ambiguous = TestCatalog {
        entries: BTreeMap::from([(10, WorldMapCatalogLookup::Ambiguous)]),
    };
    assert_eq!(
        model.project_markers(&ambiguous, &source),
        Err(WorldMapError::AmbiguousNpcCatalog(10))
    );

    let invalid_icon = TestCatalog {
        entries: BTreeMap::from([(
            10,
            catalog_entry("Broken", 35, WorldMapMissionAvailability::None),
        )]),
    };
    assert_eq!(
        model.project_markers(&invalid_icon, &source),
        Err(WorldMapError::InvalidNpcMapIcon {
            npc_type: 10,
            map_icon: 35,
        })
    );

    let nonfinite = [WorldMapNpcSource {
        npc_type: 10,
        position: WorldMapPoint::new(f32::NAN, 0.0, 0.0),
    }];
    assert_eq!(
        model.project_markers(&TestCatalog::default(), &nonfinite),
        Err(WorldMapError::NonFiniteInput)
    );
}

#[test]
fn converted_asset_set_matches_verified_dimensions_bytes_and_hash() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let mut paths = world_map_presentation_asset_paths();
    paths.sort_unstable();
    assert_eq!(paths.len(), WORLD_MAP_SEMANTIC_ASSET_FILES);
    assert_eq!(
        paths.iter().copied().collect::<BTreeSet<_>>().len(),
        paths.len()
    );

    let mut total_bytes = 0_u64;
    let mut digest = Sha256::new();
    for path in paths {
        let bytes = fs::read(asset_root.join(path))
            .unwrap_or_else(|error| panic!("read {path}: {error}"));
        let dimensions = image::image_dimensions(asset_root.join(path))
            .unwrap_or_else(|error| panic!("decode {path}: {error}"));
        assert_eq!(
            Some(dimensions),
            world_map_texture_dimensions(path),
            "{path}"
        );
        let length = u64::try_from(bytes.len()).unwrap();
        total_bytes += length;
        digest.update(path.as_bytes());
        digest.update([0]);
        digest.update(length.to_le_bytes());
        digest.update(&bytes);
    }
    assert_eq!(total_bytes, WORLD_MAP_SEMANTIC_ASSET_BYTES);
    assert_eq!(
        format!("{:x}", digest.finalize()),
        WORLD_MAP_SEMANTIC_ASSET_SET_SHA256
    );

    for (path, expected_hash) in [
        (WORLD_MAP_CHALET_FONT_PATH, WORLD_MAP_CHALET_FONT_SHA256),
        (WORLD_MAP_JEFFE_FONT_PATH, WORLD_MAP_JEFFE_FONT_SHA256),
    ] {
        let bytes = fs::read(asset_root.join(path))
            .unwrap_or_else(|error| panic!("read {path}: {error}"));
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), expected_hash);
    }
    assert!(
        WORLD_MAP_TEXTURE_PROOFS
            .iter()
            .all(|proof| proof.source_path_id > 0
                && proof.source.archive_bytes() > 0
                && proof.source.archive_sha256().len() == 64)
    );
}
