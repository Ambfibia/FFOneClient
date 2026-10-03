use super::*;
use server_selection_ui::{
    SERVER_SELECTION_ASSET_CONTRACTS, ServerSelectionUiLayout, ServerSelectionUiRect,
};

#[test]
fn cli_defaults_to_exact_acceptance_output_and_rejects_non_pngs() {
    assert_eq!(
        parse_cli(Vec::<std::ffi::OsString>::new()).unwrap(),
        PreviewCli {
            output: PathBuf::from("target/ui-parity/server-selection-expanded-1264x681.png"),
        }
    );
    assert!(parse_cli(["bad.jpg".into()]).is_err());
    assert!(parse_cli(["one.png".into(), "two.png".into()]).is_err());
}

#[test]
fn preview_uses_exact_viewport_and_panel_geometry() {
    let layout = ServerSelectionUiLayout::from_viewport(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        36.0,
        true,
    );
    assert_eq!(
        layout.panel,
        ServerSelectionUiRect::new(705.0, 180.0, 375.0, 330.0)
    );
    assert!(layout.scroll_thumb.top > layout.scroll_track.top);
}

#[test]
fn every_preview_asset_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for relative in SERVER_SELECTION_IMAGE_PATHS
        .into_iter()
        .chain([SERVER_SELECTION_FONT_PATH])
    {
        assert!(
            root.join(relative).is_file(),
            "missing {}",
            root.join(relative).display()
        );
    }
    assert_eq!(SERVER_SELECTION_ASSET_CONTRACTS.len(), 14);
}
