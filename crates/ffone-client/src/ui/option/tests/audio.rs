use super::*;

#[test]
fn sound_edits_use_only_the_clean_immediate_commit_exception() {
    let mut model = OptionUiModel::default();
    let mut outbox = OptionUiOutbox::default();
    model.open(
        OptionSettings::default(),
        InputSettings::default(),
        OptionOpenAudioRoute::default(),
        &mut outbox,
    );
    outbox.clear();
    assert!(model.set_sound_volume_step(OptionSoundChannel::Master, 8, &mut outbox));
    assert_eq!(model.draft_options.sound.master.volume, 0.8);
    assert_eq!(model.persisted_options.sound.master.volume, 0.8);
    assert_eq!(model.persisted_options.graphics.width, 1_024);
    assert!(!model.clean);
    assert_eq!(
        outbox.pop_front(),
        Some(OptionUiEvent::Action(
            OptionUiAction::ApplySoundImmediately(model.draft_options.sound.clone())
        ))
    );
}
