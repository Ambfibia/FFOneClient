use super::*;

#[test]
fn attach_move_and_clear_preserve_clean_detach_order_without_inventory_mutation() {
    let (snapshot, _, _, _) = fixture();
    let before = snapshot.inventory;
    let mut overlay = CombiSelectionOverlay0104::default();
    assert_eq!(
        overlay
            .attach(
                &snapshot,
                CombiSourceLocation0104::Inventory,
                4,
                CombiSelectionSlot0104::Style,
            )
            .expect("style"),
        vec![CombiSelectionChange0104::Attached {
            slot: CombiSelectionSlot0104::Style,
            inventory_index: 4,
        }]
    );
    overlay
        .attach(
            &snapshot,
            CombiSourceLocation0104::Inventory,
            7,
            CombiSelectionSlot0104::Stats,
        )
        .expect("stats");
    assert_eq!(
        overlay
            .attach(
                &snapshot,
                CombiSourceLocation0104::Inventory,
                4,
                CombiSelectionSlot0104::Stats,
            )
            .expect("move style to occupied stats"),
        vec![
            CombiSelectionChange0104::Detached {
                slot: CombiSelectionSlot0104::Style,
                inventory_index: 4,
            },
            CombiSelectionChange0104::Detached {
                slot: CombiSelectionSlot0104::Stats,
                inventory_index: 7,
            },
            CombiSelectionChange0104::Attached {
                slot: CombiSelectionSlot0104::Stats,
                inventory_index: 4,
            },
        ]
    );
    overlay
        .attach(
            &snapshot,
            CombiSourceLocation0104::Inventory,
            9,
            CombiSelectionSlot0104::Style,
        )
        .expect("new style");
    assert_eq!(
        overlay.clear_all(),
        vec![
            CombiSelectionChange0104::Detached {
                slot: CombiSelectionSlot0104::Style,
                inventory_index: 9,
            },
            CombiSelectionChange0104::Detached {
                slot: CombiSelectionSlot0104::Stats,
                inventory_index: 4,
            },
        ]
    );
    assert_eq!(snapshot.inventory, before);
}
