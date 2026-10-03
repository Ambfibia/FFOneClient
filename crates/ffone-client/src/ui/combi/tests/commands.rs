use super::*;

#[test]
fn failure_reply_keeps_both_items_and_commits_only_authoritative_taros() {
    let (mut snapshot, selection, catalog, recipes) = fixture();
    let projection =
        project_combi_mode_0104(&snapshot, selection, &catalog, &recipes).expect("projection");
    let style_before = snapshot.inventory[4];
    let stats_before = snapshot.inventory[7];
    let before = snapshot.clone();
    let mut machine = awaiting_machine(&snapshot, selection, &projection);
    let receipt = machine
        .receive_authoritative_reply(
            &snapshot,
            &projection,
            CombiSuccessReply0104 {
                new_item_slot: 4,
                new_item: style_before,
                stat_item_slot: 7,
                cash_item_slot_1: 0,
                cash_item_slot_2: 0,
                taros_after: 1,
                success_flag: 0,
            },
        )
        .expect("valid failure result");
    assert_eq!(snapshot, before);
    assert_eq!(
        machine.phase,
        CombiPhase0104::Modal(CombiSystemModal0104::CombinationFailed)
    );
    snapshot
        .apply_authoritative_receipt(&receipt)
        .expect("explicit failure commit");
    assert_eq!(snapshot.inventory[4], style_before);
    assert_eq!(snapshot.inventory[7], stats_before);
    assert_eq!(snapshot.taros, 1);
}
