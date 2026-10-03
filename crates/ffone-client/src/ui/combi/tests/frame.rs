use super::*;

#[test]
fn wire_failure_releases_send_lock_without_mutating_selection_or_inventory() {
    let (snapshot, selection, catalog, recipes) = fixture();
    let projection =
        project_combi_mode_0104(&snapshot, selection, &catalog, &recipes).expect("projection");
    let mut machine = awaiting_machine(&snapshot, selection, &projection);
    machine
        .receive_wire_failure(CombiFailureReply0104 {
            error_code: 42,
            costume_item_slot: 4,
            stat_item_slot: 7,
            cash_item_slot_1: 0,
            cash_item_slot_2: 0,
        })
        .expect("matched fail envelope");
    assert_eq!(machine.phase,CombiPhase0104::Ready);
    assert!(!machine.phase.sending_locked());
    assert_eq!(machine.selection,selection);
    machine.begin_combine(&snapshot,&projection).expect("can retry rejected request");
}

/// One frame as `ui_focus_system` would leave it: `interactions` are the
/// Interaction values that changed since the previous frame.
pub(super) fn pointer_frame(
    app: &mut App,
    press: bool,
    release: bool,
    interactions: &[(Entity, Interaction)],
) -> Vec<CombiUiCommand0104> {
    {
        let mut mouse = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
        mouse.clear();
        if press {
            mouse.press(MouseButton::Left);
        }
        if release {
            mouse.release(MouseButton::Left);
        }
    }
    for (entity, interaction) in interactions {
        *app.world_mut().get_mut::<Interaction>(*entity).unwrap() = *interaction;
    }
    app.update();
    let mut outbox = app.world_mut().resource_mut::<CombiUiOutbox0104>();
    std::iter::from_fn(|| outbox.pop_front()).collect()
}
