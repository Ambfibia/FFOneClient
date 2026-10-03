use super::*;
use launcher_ui::{LAUNCHER_UI_ASSET_CONTRACTS, LauncherUiLayout};

#[test]
fn cli_defaults_to_exact_acceptance_output_and_rejects_non_pngs() {
    assert_eq!(
        parse_cli(Vec::<std::ffi::OsString>::new()).unwrap(),
        PreviewCli {
            output: PathBuf::from("target/ui-parity/launcher-aiming-1264x681.png"),
        }
    );
    assert_eq!(
        parse_cli(["target/custom-launcher.png".into()]).unwrap(),
        PreviewCli {
            output: PathBuf::from("target/custom-launcher.png"),
        }
    );
    assert!(parse_cli(["bad.jpg".into()]).is_err());
    assert!(parse_cli(["one.png".into(), "two.png".into()]).is_err());
}

#[test]
fn preview_uses_exact_viewport_geometry_and_power() {
    assert_eq!((CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT), (1_264, 681));
    assert_eq!(PREVIEW_POWER, 0.64);
    let layout = LauncherUiLayout::from_viewport(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        PREVIEW_POWER,
    );
    assert_eq!(layout.crosshair.x, 302.0);
    assert_eq!(layout.crosshair.y, 10.0);
    assert_eq!(layout.gauge.x, 123.0);
    assert!((layout.gauge_bar.y - 238.84).abs() <= 0.001);
}

#[test]
fn every_preview_asset_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for relative in LAUNCHER_UI_IMAGE_PATHS
        .into_iter()
        .chain([LAUNCHER_UI_FONT_PATH])
    {
        assert!(
            root.join(relative).is_file(),
            "missing preview asset {}",
            root.join(relative).display()
        );
    }
    assert_eq!(LAUNCHER_UI_ASSET_CONTRACTS.len(), 4);
}
