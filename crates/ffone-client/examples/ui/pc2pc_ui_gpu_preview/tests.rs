use super::*;

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
fn fixture_is_correlated_ready_authority_with_fail_closed_backends() {
    let model = preview_model();
    let projection: &Pc2pcModeProjection0104 = &model.projection;
    assert_eq!(projection.local_offer_taros, 12_500);
    assert_eq!(projection.remote_offer_taros, 8_750);
    assert!(projection.local_offer[0].item.is_some());
    assert!(projection.remote_offer[0].item.is_some());
    assert!(projection.remote_offer[2].item.is_some());
    assert!(model.state.local_ready);
    assert!(model.state.remote_ready);
    assert!(model.chat.is_empty());
    assert_eq!(Pc2pcBackendCapabilities::default().portrait_backend, false);
    assert_eq!(Pc2pcBackendCapabilities::default().free_chat_backend, false);
}

#[test]
fn pixel_guard_rejects_large_white_placeholders_but_not_small_ui_marks() {
    let mut clean = RgbaImage::from_pixel(80, 80, image::Rgba([0, 12, 20, 255]));
    for y in 5..35 {
        for x in 5..35 {
            clean.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
        }
    }
    assert!(!contains_large_near_white_placeholder(&clean));

    for y in 20..60 {
        for x in 20..60 {
            clean.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
        }
    }
    assert!(contains_large_near_white_placeholder(&clean));
}
