use super::*;

#[test]
fn cli_accepts_all_states_and_a_png_override() {
    assert_eq!(
        parse_cli(Vec::<std::ffi::OsString>::new()).unwrap(),
        PreviewCli {
            mode: PreviewMode::Ready,
            output: PathBuf::from(DEFAULT_OUTPUT),
        }
    );
    assert_eq!(
        parse_cli([std::ffi::OsString::from("waiting")])
            .unwrap()
            .mode,
        PreviewMode::Waiting
    );
    assert_eq!(
        parse_cli([
            std::ffi::OsString::from("orphan"),
            std::ffi::OsString::from("custom.png"),
        ])
        .unwrap(),
        PreviewCli {
            mode: PreviewMode::Orphan,
            output: PathBuf::from("custom.png"),
        }
    );
    assert!(parse_cli([std::ffi::OsString::from("bad.jpg")]).is_err());
}

#[test]
fn fixtures_cover_ready_waiting_success_cash_error_and_orphan_states() {
    let ready = preview_projection(PreviewMode::Ready);
    assert!(ready.capabilities.enchant_enabled);
    assert!(ready.requirements.is_some());
    let waiting = preview_projection(PreviewMode::Waiting);
    assert!(matches!(waiting.phase, EnchantPhase0104::Waiting { .. }));
    assert!(waiting.waiting_progress_width > 0.0);
    let success = preview_projection(PreviewMode::Success);
    assert!(matches!(success.phase, EnchantPhase0104::Success { .. }));
    assert!(success.success.is_some());
    let cash = preview_projection(PreviewMode::CashError);
    assert!(cash.target_error);
    assert!(!cash.dead_cash_warning_visible());
    let orphan = preview_projection(PreviewMode::Orphan);
    assert!(orphan.selection.any_orphaned());
    assert!(
        !orphan
            .selection
            .is_attached(EnchantAttachmentSlot0104::Target)
    );
}
