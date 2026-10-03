use super::*;
use ffone_client::world_map::WorldMapMarkerKind;

#[test]
fn cli_defaults_to_the_supplied_future_world_view() {
    let (mode, output, language) = parse_preview_args([]).unwrap();
    assert_eq!(mode, PreviewMode::Type3);
    assert_eq!(
        output,
        PathBuf::from("target/ui-parity/world-map-region-en-1252x667.png")
    );
    assert_eq!(language, "en");
}

#[test]
fn cli_accepts_all_four_views_and_rejects_bad_shapes() {
    for (name, expected) in [
        ("type1", PreviewMode::Type1),
        ("type2", PreviewMode::Type2),
        ("type3", PreviewMode::Type3),
        ("type4", PreviewMode::Type4),
    ] {
        let (actual, output, language) = parse_preview_args([OsString::from(name)]).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(output, expected.default_output("en"));
        assert_eq!(language, "en");
    }
    let (mode, output, language) = parse_preview_args([
        OsString::from("type4"),
        OsString::from("--language"),
        OsString::from("ru"),
    ])
    .unwrap();
    assert_eq!(mode, PreviewMode::Type4);
    assert_eq!(output, PreviewMode::Type4.default_output("ru"));
    assert_eq!(language, "ru");
    assert!(parse_preview_args([OsString::from("local")]).is_err());
    assert!(
        parse_preview_args([OsString::from("type4"), OsString::from("capture.jpg"),]).is_err()
    );
    assert!(
        parse_preview_args([
            OsString::from("type4"),
            OsString::from("capture.png"),
            OsString::from("extra"),
        ])
        .is_err()
    );
    assert!(
        parse_preview_args([
            OsString::from("type4"),
            OsString::from("--language"),
            OsString::from("de"),
        ])
        .is_err()
    );
}

#[test]
fn sample_views_are_pure_valid_and_cover_zoom_and_marker_roles() {
    for (mode, zoom) in [
        (PreviewMode::Type1, WorldMapZoom::Type1),
        (PreviewMode::Type2, WorldMapZoom::Type2),
        (PreviewMode::Type3, WorldMapZoom::Type3),
        (PreviewMode::Type4, WorldMapZoom::Type4),
    ] {
        let presentation = build_preview_presentation(mode).unwrap();
        assert_eq!(presentation.model.zoom(), zoom);
        assert_eq!(presentation.model.phase(), WorldMapPhase::Open);
        assert!(presentation.validate().is_ok());
        assert!(
            presentation
                .markers
                .iter()
                .any(|marker| matches!(marker.kind, WorldMapMarkerKind::Player { .. }))
        );
        assert!(presentation.markers.iter().any(|marker| matches!(
            marker.kind,
            WorldMapMarkerKind::MissionNew { .. } | WorldMapMarkerKind::MissionAdvance { .. }
        )));
    }
    let local = build_preview_presentation(PreviewMode::Type4).unwrap();
    assert_eq!(
        local.model.zone(),
        ffone_client::world_map::WorldMapZone::Future
    );
    assert!(
        local
            .markers
            .iter()
            .any(|marker| matches!(marker.kind, WorldMapMarkerKind::Npc { .. }))
    );
    assert!(local.markers.iter().any(|marker| matches!(
        marker.kind,
        WorldMapMarkerKind::WaypointOffscreenArrow { .. }
    )));
    let reference = build_preview_presentation(PreviewMode::Type3).unwrap();
    assert_eq!(
        reference.model.zone(),
        ffone_client::world_map::WorldMapZone::Future
    );
    assert_eq!(reference.model.zoom(), WorldMapZoom::Type3);
    assert!(reference.model.zoom().is_world_view());
}
