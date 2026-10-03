use super::*;
use ffone_client::combi_ui::{CombiChance0104, CombiUiRect, combi_mode_layout_0104};

#[test]
fn cli_defaults_to_required_acceptance_path_and_validates_png_shape() {
    assert_eq!(
        parse_cli(Vec::<std::ffi::OsString>::new()).unwrap(),
        PreviewCli {
            output: PathBuf::from(DEFAULT_OUTPUT)
        }
    );
    assert!(parse_cli([std::ffi::OsString::from("wrong.jpg")]).is_err());
    assert!(
        parse_cli([
            std::ffi::OsString::from("one.png"),
            std::ffi::OsString::from("two.png")
        ])
        .is_err()
    );
}

#[test]
fn preview_is_a_ready_clean_combination_with_exact_geometry() {
    let projection = preview_projection();
    assert!(projection.combine_enabled);
    assert_eq!(projection.cost, 700);
    assert_eq!(
        projection.chance,
        CombiChance0104::Good { raw_percent: 87.5 }
    );
    let layout = combi_mode_layout_0104(CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT);
    assert_eq!(
        layout.main_group,
        CombiUiRect::new(114.0, 14.0, 585.0, 653.0)
    );
}
