use super::*;

#[test]
fn outbox_and_projection_reset_discard_stale_session_state() {
    let runtime = runtime_with(&[(4, item(4, 77, 0, 0))], &[(49, item(7, 40, 2, 0))]);
    let mut projection = UserEquipItemModeProjection::from_authoritative(&runtime, &AllCatalog);
    let mut outbox = UserEquipUiOutbox::default();
    outbox.push(UserEquipUiAction::RequestClose {
        source: UserEquipCloseSource::Escape,
    });

    assert_eq!(projection.owner_pc_id, OWNER_PC_ID);
    assert_eq!(outbox.len(), 1);
    projection.reset();
    outbox.clear();
    assert_eq!(projection, UserEquipItemModeProjection::default());
    assert!(projection.inventory.iter().all(|slot| slot.item.empty));
    assert!(projection.equipment.iter().all(|slot| slot.item.empty));
    assert!(outbox.is_empty());
}
