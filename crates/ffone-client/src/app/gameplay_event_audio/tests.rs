use super::*;
#[test]
fn dialogue_audio_fires_once_per_open_and_chat_echoes_preserve_count() {
    let mut previous = None;
    assert_eq!(gameplay_event_audio(0, None, &mut previous).count(), 0);
    assert_eq!(
        gameplay_event_audio(2, Some(81), &mut previous).collect::<Vec<_>>(),
        vec!["Open_Screen", "Outgoing_chat", "Outgoing_chat"]
    );
    assert_eq!(gameplay_event_audio(0, Some(81), &mut previous).count(), 0);
    assert_eq!(gameplay_event_audio(0, None, &mut previous).count(), 0);
    assert_eq!(
        gameplay_event_audio(0, Some(81), &mut previous).collect::<Vec<_>>(),
        vec!["Open_Screen"]
    );
    assert_eq!(
        gameplay_event_audio(0, Some(82), &mut previous).collect::<Vec<_>>(),
        vec!["Open_Screen"]
    );
    assert_eq!(
        gameplay_event_audio(1, None, &mut previous).collect::<Vec<_>>(),
        vec!["Outgoing_chat"]
    );
    assert_eq!(gameplay_event_audio(0, None, &mut previous).count(), 0);
}
