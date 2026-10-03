use super::*;

#[test]
fn cli_defaults_and_supports_all_five_tabs_plus_opening() {
    assert_eq!(
        parse_cli(Vec::<std::ffi::OsString>::new()).unwrap(),
        PreviewCli {
            mode: PreviewMode::New,
            locale: PreviewLocale::En,
            output: PathBuf::from(DEFAULT_OUTPUT),
        }
    );
    for name in ["new", "scroll", "potion", "equipment", "etc", "opening"] {
        let cli = parse_cli([std::ffi::OsString::from(name)]).unwrap();
        assert_eq!(cli.mode, PreviewMode::parse(name).unwrap());
        assert_eq!(cli.locale, PreviewLocale::En);
        assert!(is_png(&cli.output));
    }
    for locale in [PreviewLocale::En, PreviewLocale::Ru] {
        let cli = parse_cli([
            std::ffi::OsString::from("new"),
            std::ffi::OsString::from(locale.slug()),
        ])
        .unwrap();
        assert_eq!(cli.locale, locale);
        assert_eq!(cli.output, default_output(PreviewMode::New, locale));
    }
    assert!(parse_cli([std::ffi::OsString::from("wrong.jpg")]).is_err());
    assert!(
        parse_cli([
            std::ffi::OsString::from("new"),
            std::ffi::OsString::from("wrong.jpg")
        ])
        .is_err()
    );
}

#[test]
fn preview_projection_stays_identical_for_every_tab() {
    let projection = preview_projection();
    let baseline = projection.rows_for_tab(CashmallTab0104::New);
    assert_eq!(baseline.len(), 8);
    for tab in CashmallTab0104::ALL {
        assert_eq!(projection.rows_for_tab(tab), baseline);
    }
    assert_eq!(baseline[4].frame_alpha_percent, 40);
    assert_eq!(
        baseline[4].frame_visual,
        CashmallSlotFrameVisual0104::Restricted
    );
}

#[test]
fn preview_modes_produce_the_expected_final_or_mid_open_view() {
    for mode in [
        PreviewMode::New,
        PreviewMode::Scroll,
        PreviewMode::Potion,
        PreviewMode::Equipment,
        PreviewMode::Etc,
        PreviewMode::Opening,
    ] {
        let mut state = CashmallUiState0104::default();
        let mut outbox = CashmallUiOutbox0104::default();
        state.open_from(
            CashmallOpenSource0104::HiddenChatCommand,
            false,
            &mut outbox,
        );
        state.tick(if mode.is_opening() { 0.5 } else { 1.0 });
        if !mode.is_opening() {
            state
                .select_tab(Default::default(), mode.tab(), &mut outbox)
                .unwrap();
        }
        let projection = preview_projection();
        let view = cashmall_mode_view_0104(
            CLIENT_AREA_WIDTH,
            CLIENT_AREA_HEIGHT,
            &state,
            Default::default(),
            &projection,
            [false; 5],
            [false; 5],
            true,
        )
        .unwrap();
        assert_eq!(
            view.tabs[mode.tab().index()].visual,
            CashmallTabVisual0104::Selected
        );
        assert_eq!(
            state.phase(),
            if mode.is_opening() {
                CashmallLifecyclePhase0104::Opening
            } else {
                CashmallLifecyclePhase0104::Visible
            }
        );
    }
}
