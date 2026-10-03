use super::*;
use ffone_client::transportation_ui::TransportationService;

#[test]
fn cli_defaults_to_the_required_capture_and_rejects_bad_shapes() {
    assert_eq!(
        parse_cli([]).unwrap(),
        PreviewCli {
            language: "en".to_owned(),
            output: default_output("en"),
        }
    );
    assert_eq!(
        parse_cli([OsString::from("ru")]).unwrap(),
        PreviewCli {
            language: "ru".to_owned(),
            output: default_output("ru"),
        }
    );
    assert_eq!(
        parse_cli([OsString::from("en"), OsString::from("custom.png")]).unwrap(),
        PreviewCli {
            language: "en".to_owned(),
            output: PathBuf::from("custom.png"),
        }
    );
    assert!(parse_cli([OsString::from("de")]).is_err());
    assert!(parse_cli([OsString::from("en"), OsString::from("custom.jpg")]).is_err());
    assert!(
        parse_cli([
            OsString::from("en"),
            OsString::from("one.png"),
            OsString::from("two.png"),
        ])
        .is_err()
    );
}

#[test]
fn preview_uses_the_clean_wyvern_branch_with_scroll_and_selection() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game")
        .canonicalize()
        .unwrap();
    let model = build_preview_model(&root).unwrap();
    assert_eq!(model.phase(), TransportationPhase::Browsing);
    assert_eq!(model.service(), TransportationService::Wyvern);
    assert_eq!(model.routes().len(), 7);
    assert_eq!(model.selected_route(), Some(3));
    assert!(model.turbo());
    assert!(model.maximum_scroll_y() > 0.0);
}
