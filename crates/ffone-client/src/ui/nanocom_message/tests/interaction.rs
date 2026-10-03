use super::*;

#[test]
fn button_visual_states_match_hud_skin_active_and_hover_states() {
    let asset_root = tempdir().unwrap();
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(AssetPlugin {
            file_path: asset_root.path().to_string_lossy().into_owned(),
            ..default()
        })
        .init_asset::<Image>()
        .init_asset::<Font>();
    let assets = NanocomMessageUiAssets::load(app.world().resource::<AssetServer>());
    assert_eq!(
        assets.button_image(NanocomMessageChoice::Accept, Interaction::Hovered),
        assets.blue_over
    );
    assert_eq!(
        assets.button_image(NanocomMessageChoice::Accept, Interaction::Pressed),
        assets.blue
    );
    assert_eq!(
        assets.button_image(NanocomMessageChoice::Decline, Interaction::Hovered),
        assets.red_over
    );
    assert_eq!(
        assets.button_image(NanocomMessageChoice::Decline, Interaction::Pressed),
        assets.red
    );
    assert_eq!(
        nanocom_button_text_color(NanocomMessageChoice::Accept, Interaction::Hovered),
        Color::srgb(0.0, 0.278_431_4, 0.478_431_37)
    );
}

#[test]
fn stale_button_press_cannot_resolve_a_new_head_after_tick() {
    assert!(!nanocom_button_targets_presented_head(None, Some(1)));
    assert!(nanocom_button_targets_presented_head(Some(1), Some(1)));
    assert!(
        !nanocom_button_targets_presented_head(Some(1), Some(2)),
        "a button painted for the timed-out head must not choose its successor"
    );
    assert!(!nanocom_button_targets_presented_head(Some(1), None));
}
