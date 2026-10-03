use bevy::{asset::AssetPlugin, image::Image, text::Font};
use sha2::{Digest, Sha256};
use tempfile::tempdir;

use super::*;
use ffone_client::resurrect_ui::{
    RESURRECT_BODY_FONT_SHA256, RESURRECT_BUTTON_FONT_SHA256,
    RESURRECT_UNUSED_ITEM_BUTTON_RECT, RESURRECT_WINDOW_RECT,
};

#[test]
fn cli_defaults_to_self_and_accepts_group_item() {
    assert_eq!(
        parse_preview_args(Vec::<OsString>::new()).unwrap(),
        (
            PreviewMode::PhoenixSelf,
            "en".to_owned(),
            PreviewMode::PhoenixSelf.default_output("en")
        )
    );
    assert_eq!(
        parse_preview_args([OsString::from("self.png")]).unwrap(),
        (
            PreviewMode::PhoenixSelf,
            "en".to_owned(),
            PathBuf::from("self.png")
        )
    );
    assert_eq!(
        parse_preview_args([
            OsString::from("group-item"),
            OsString::from("group-item.png"),
            OsString::from("--language"),
            OsString::from("ru"),
        ])
        .unwrap(),
        (
            PreviewMode::GroupItem,
            "ru".to_owned(),
            PathBuf::from("group-item.png")
        )
    );
    assert!(parse_preview_args([OsString::from("bad.jpg")]).is_err());
    assert!(parse_preview_args([OsString::from("--language"), OsString::from("de")]).is_err());
    assert!(
        parse_preview_args([
            OsString::from("self"),
            OsString::from("a.png"),
            OsString::from("b.png")
        ])
        .is_err()
    );
}

#[test]
fn authority_frame_geometry_priority_and_input_boundary_are_exact() {
    assert_eq!((CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT), (1_264, 681));
    assert_eq!(clean_resurrect_ui_scale(CLIENT_AREA_HEIGHT as f32), 1.0);
    let layout = resurrect_ui_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        1.0,
    );
    assert_eq!(layout.dialog_left, 372.0);
    assert_eq!(layout.dialog_top, 258.0);
    assert_eq!(
        layout.visual_rect,
        ResurrectUiRect::new(372.0, 258.0, 520.0, 164.0)
    );
    assert_eq!(
        layout.source_size,
        Vec2::new(RESURRECT_WINDOW_RECT.width, RESURRECT_WINDOW_RECT.height)
    );
    assert_eq!(
        ResurrectChoice::UseItem.rect(),
        RESURRECT_PHOENIX_BUTTON_RECT
    );
    assert_ne!(
        ResurrectChoice::UseItem.rect(),
        RESURRECT_UNUSED_ITEM_BUTTON_RECT
    );

    let self_context = PreviewMode::PhoenixSelf.context();
    assert_eq!(
        self_context.choice_draw_order(),
        [
            Some(ResurrectChoice::NearestResurrectEm),
            Some(ResurrectChoice::PhoenixSelf),
            None,
        ]
    );
    let group_item = PreviewMode::GroupItem.context();
    assert_eq!(
        group_item.choice_draw_order(),
        [
            Some(ResurrectChoice::NearestResurrectEm),
            Some(ResurrectChoice::PhoenixGroup),
            Some(ResurrectChoice::UseItem),
        ]
    );
    assert!(!group_item.choice_is_visible(ResurrectChoice::PhoenixSelf));
    assert_eq!(
        group_item.topmost_choice_at(Vec2::new(60.0, 130.0)),
        Some(ResurrectChoice::UseItem)
    );

    let mut model = ResurrectUiModel::default();
    model.visible = true;
    assert_eq!(model.input_boundary(self_context), EXPECTED_INPUT_BOUNDARY);
}

#[test]
fn every_source_image_and_font_exists_with_exact_hash() {
    assert_eq!(RESURRECT_TEXTURE_CONTRACTS.len(), 9);
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for contract in RESURRECT_TEXTURE_CONTRACTS {
        assert_file_sha256(&root.join(contract.runtime_path), contract.sha256);
    }
    for (relative, sha256) in [
        (RESURRECT_BUTTON_FONT_PATH, RESURRECT_BUTTON_FONT_SHA256),
        (RESURRECT_BODY_FONT_PATH, RESURRECT_BODY_FONT_SHA256),
    ] {
        assert_file_sha256(&root.join(relative), sha256);
    }
}

#[test]
fn plugin_renders_only_clean_priority_choices_for_both_modes() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>()
        .add_plugins(ResurrectUiPlugin);
    app.update();

    for (mode, expected_visible) in [
        (PreviewMode::PhoenixSelf, 2usize),
        (PreviewMode::GroupItem, 3),
    ] {
        *app.world_mut().resource_mut::<ResurrectUiContext>() = mode.context();
        {
            let mut model = app.world_mut().resource_mut::<ResurrectUiModel>();
            model.visible = true;
            model.request_sent = false;
            model.elapsed_seconds = 0.0;
        }
        app.update();

        let world = app.world_mut();
        let mut buttons = world.query_filtered::<&Node, With<Button>>();
        let nodes = buttons.iter(world).collect::<Vec<_>>();
        assert_eq!(nodes.len(), 4);
        assert_eq!(
            nodes
                .iter()
                .filter(|node| node.display == Display::Flex)
                .count(),
            expected_visible
        );
    }
}

fn assert_file_sha256(path: &Path, expected: &str) {
    let bytes = fs::read(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    let actual = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<String>();
    assert_eq!(actual, expected, "hash mismatch for {}", path.display());
}
