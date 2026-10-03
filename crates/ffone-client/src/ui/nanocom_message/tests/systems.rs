use super::*;

#[test]
fn accept_and_decline_emit_immediately_but_remove_on_next_tick() {
    for (choice, resolution, sound) in [
        (
            NanocomMessageChoice::Accept,
            NanocomMessageResolution::Accepted,
            NanocomMessageSound::Yes,
        ),
        (
            NanocomMessageChoice::Decline,
            NanocomMessageResolution::Declined,
            NanocomMessageSound::No,
        ),
    ] {
        let mut model = NanocomMessageUiModel::default();
        model.enqueue(buddy(17));
        assert_eq!(model.pop_sound(), Some(NanocomMessageSound::SlideIn));

        let action = model.choose(choice).expect("interactive action");
        assert_eq!(action.resolution, resolution);
        assert_eq!(model.len(), 1, "legacy bPopflag defers removal");
        assert_eq!(model.pop_sound(), Some(sound));

        assert_eq!(model.tick(0.0), None);
        assert!(model.is_empty());
        assert_eq!(model.pop_sound(), Some(NanocomMessageSound::SlideOut));
    }
}

#[test]
fn resolving_head_reveals_next_in_same_tick() {
    let mut model = NanocomMessageUiModel::default();
    model.enqueue(buddy(1));
    model.enqueue(buddy(2));
    model.pop_sound();

    let action = model
        .choose(NanocomMessageChoice::Accept)
        .expect("accept head");
    assert_eq!(action.request_id, 1);
    assert_eq!(model.pop_sound(), Some(NanocomMessageSound::Yes));
    assert_eq!(model.tick(0.016), None);
    assert_eq!(
        model.active().map(|active| active.request.request_id),
        Some(2)
    );
    assert_eq!(model.reveal_parameter(), 1.0);
    assert_eq!(model.pop_sound(), Some(NanocomMessageSound::SlideIn));
}
