use super::*;

#[test]
fn success_reply_produces_separate_receipt_then_exact_post_state_commit() {
    let (mut snapshot, selection, catalog, recipes) = fixture();
    let projection =
        project_combi_mode_0104(&snapshot, selection, &catalog, &recipes).expect("projection");
    let before = snapshot.clone();
    let mut machine = awaiting_machine(&snapshot, selection, &projection);
    let style_after = expected_success_style_item(snapshot.inventory[4], snapshot.inventory[7]);
    let receipt = machine
        .receive_authoritative_reply(
            &snapshot,
            &projection,
            CombiSuccessReply0104 {
                new_item_slot: 4,
                new_item: style_after,
                stat_item_slot: 7,
                cash_item_slot_1: 0,
                cash_item_slot_2: 0,
                taros_after: 1,
                success_flag: 1,
            },
        )
        .expect("valid success");
    assert_eq!(snapshot, before, "receipt creation is not a mutation");
    assert!(matches!(machine.phase, CombiPhase0104::Success(_)));
    snapshot
        .apply_authoritative_receipt(&receipt)
        .expect("explicit commit");
    assert_eq!(snapshot.inventory[4], style_after);
    assert_eq!(snapshot.inventory[7], empty_item_0104());
    assert_eq!(snapshot.taros, 1);
}

#[test]
fn carried_item_drops_on_release_over_a_selection_slot_and_goes_back_elsewhere() {
    use CombiUiCommand0104::{BeginInventoryDrag, CancelInventoryDrag, DropOnSelection};
    let begin = BeginInventoryDrag { inventory_index: 4 };
    let mut app = pointer_app();
    let world = app.world_mut();
    let bag = world
        .spawn((
            CombiInteractiveControl0104::InventorySlot(4),
            Interaction::None,
        ))
        .id();
    let look = world
        .spawn((CombiInteractiveControl0104::LookDrop, Interaction::None))
        .id();
    let stats = world
        .spawn((
            CombiInteractiveControl0104::StatsSelection,
            Interaction::None,
        ))
        .id();

    // Hold-drag from bag cell 4 onto the STYLE detail area.
    assert_eq!(
        pointer_frame(&mut app, true, false, &[(bag, Interaction::Pressed)]),
        [begin]
    );
    assert!(pointer_frame(&mut app, false, false, &[(look, Interaction::Hovered)]).is_empty());
    assert_eq!(
        carried(&app),
        Some(CombiCarriedItem0104 {
            inventory_index: 4,
            held: true,
        })
    );
    assert_eq!(
        pointer_frame(&mut app, false, true, &[(bag, Interaction::None)]),
        [DropOnSelection {
            slot: CombiSelectionSlot0104::Style,
        }]
    );
    assert_eq!(carried(&app), None);

    // Released over nothing: the item goes back and the runtime forgets it.
    assert_eq!(
        pointer_frame(
            &mut app,
            true,
            false,
            &[(bag, Interaction::Pressed), (look, Interaction::None)],
        ),
        [begin]
    );
    assert_eq!(
        pointer_frame(&mut app, false, true, &[(bag, Interaction::None)]),
        [CancelInventoryDrag]
    );

    // A click picks the item up; the next click on the STATS slot places it.
    assert_eq!(
        pointer_frame(&mut app, true, false, &[(bag, Interaction::Pressed)]),
        [begin]
    );
    assert!(pointer_frame(&mut app, false, true, &[(bag, Interaction::Hovered)]).is_empty());
    assert_eq!(
        carried(&app),
        Some(CombiCarriedItem0104 {
            inventory_index: 4,
            held: false,
        })
    );
    assert_eq!(
        pointer_frame(
            &mut app,
            true,
            false,
            &[(bag, Interaction::None), (stats, Interaction::Pressed)],
        ),
        [DropOnSelection {
            slot: CombiSelectionSlot0104::Stats,
        }]
    );
    assert!(pointer_frame(&mut app, false, true, &[(stats, Interaction::Hovered)]).is_empty());

    // A click-picked item goes back on a click anywhere else.
    assert_eq!(
        pointer_frame(
            &mut app,
            true,
            false,
            &[(bag, Interaction::Pressed), (stats, Interaction::None)],
        ),
        [begin]
    );
    assert!(pointer_frame(&mut app, false, true, &[(bag, Interaction::Hovered)]).is_empty());
    assert_eq!(
        pointer_frame(&mut app, true, false, &[(bag, Interaction::None)]),
        [CancelInventoryDrag]
    );

    // A release lost outside the window, and a popup opening mid-drag,
    // both put the item back.
    assert!(pointer_frame(&mut app, false, true, &[]).is_empty());
    assert_eq!(
        pointer_frame(&mut app, true, false, &[(bag, Interaction::Pressed)]),
        [begin]
    );
    app.world_mut()
        .resource_mut::<ButtonInput<MouseButton>>()
        .reset_all();
    assert_eq!(
        pointer_frame(&mut app, false, false, &[]),
        [CancelInventoryDrag]
    );
    assert_eq!(
        pointer_frame(&mut app, true, false, &[(bag, Interaction::Pressed)]),
        [begin]
    );
    app.world_mut()
        .resource_mut::<CombiUiState0104>()
        .external_modal
        .system_popup = true;
    assert_eq!(
        pointer_frame(&mut app, false, false, &[]),
        [CancelInventoryDrag]
    );
    assert_eq!(carried(&app), None);
}
