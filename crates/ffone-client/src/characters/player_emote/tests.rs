use super::*;

#[test]
fn emote_events_follow_playback_once_and_reset_on_reentry() {
    let events = emote_events(PlayerRigGender::Male, TutorialPlayerClip::Dance1).unwrap();
    let mut cursor = PlayerEmoteCursor::default();
    assert_eq!(cursor.sounds(&events, 0.24).count(), 0);
    assert_eq!(cursor.sounds(&events, 0.25).count(), 2);
    assert_eq!(cursor.sounds(&events, 0.25).count(), 0);
    assert_eq!(cursor.sounds(&events, 8.0).count(), 0);
    assert_eq!(PlayerEmoteCursor::default().sounds(&events, 8.0).count(), 2);
}

#[test]
fn persistent_emotes_keep_their_server_contract() {
    let mut continuation = PlayerEmoteContinuation::default();
    for (clip, code) in [
        (TutorialPlayerClip::Beach1, 22),
        (TutorialPlayerClip::Beach2, 23),
        (TutorialPlayerClip::Beach3, 24),
    ] {
        assert_eq!(continuation.next_code(clip), Some(code));
    }
    let mut seen = std::collections::BTreeSet::new();
    for _ in 0..1000 {
        let code = continuation.next_code(TutorialPlayerClip::Dance1).unwrap();
        assert!([6, 17, 18, 19, 20].contains(&code));
        seen.insert(code);
    }
    assert_eq!(seen.len(), 5);
    assert_eq!(continuation.next_code(TutorialPlayerClip::Laugh), None);
    assert_eq!(continuation.next_code(TutorialPlayerClip::Standup), None);
}
