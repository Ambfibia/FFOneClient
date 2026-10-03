use super::*;

#[test]
fn preview_frame_uses_exact_authority_geometry_and_hover_target() {
    assert_eq!((CLIENT_AREA_WIDTH, CLIENT_AREA_HEIGHT), (1_264, 681));
    assert_eq!(PREVIEW_HOVER, QuitMenuButtonKind::QuitGame);
    let layout = quit_menu_ui_layout(
        Vec2::new(CLIENT_AREA_WIDTH as f32, CLIENT_AREA_HEIGHT as f32),
        clean_quit_menu_ui_scale(CLIENT_AREA_HEIGHT as f32),
    );
    assert_eq!(layout.scale, 1.0);
    assert_eq!(layout.dialog.node_left, 529.0);
    assert_eq!(layout.dialog.node_top, 246.0);
    assert_eq!(layout.dialog.visual.width, 206.0);
    assert_eq!(layout.dialog.visual.height, 188.0);
}

#[test]
fn every_preview_image_font_and_audio_asset_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    for relative in [
        QUIT_MENU_BACKDROP_PATH,
        QUIT_MENU_DIALOG_PATH,
        QUIT_MENU_BUTTON_NORMAL_PATH,
        QUIT_MENU_BUTTON_HOVER_PATH,
        QUIT_MENU_CANCEL_NORMAL_PATH,
        QUIT_MENU_CANCEL_HOVER_PATH,
        QUIT_MENU_FONT_PATH,
        QUIT_MENU_OPEN_SOUND_PATH,
        QUIT_MENU_CLOSE_SOUND_PATH,
    ]
    .into_iter()
    .chain(QUIT_MENU_BUTTON_SOUND_PATHS)
    {
        assert!(
            root.join(relative).is_file(),
            "missing preview asset {}",
            root.join(relative).display()
        );
    }
}
