use super::*;

pub(super) fn runtime_with(
    equipment: &[(usize, ItemBase0104)],
    inventory: &[(usize, ItemBase0104)],
) -> InventoryRuntime0104 {
    let mut load = PcLoadData0104::zeroed();
    for &(index, value) in equipment {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::EQUIPMENT_OFFSET + index * ItemBase0104::SIZE,
            value,
        );
    }
    for &(index, value) in inventory {
        write_item(
            load.as_bytes_mut(),
            PcLoadData0104::INVENTORY_OFFSET + index * ItemBase0104::SIZE,
            value,
        );
    }
    InventoryRuntime0104::from_pc_load(OWNER_PC_ID, &load)
}

#[test]
fn status_guide_icons_are_exact_panel_user_clothes_assets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("assets/game");
    assert_eq!(
        USER_EQUIP_GUIDE_ICON_EVIDENCE.map(|(_, path_id, _)| path_id),
        [248, 234, 75, 475, 226]
    );
    for (path, _, expected_hash) in USER_EQUIP_GUIDE_ICON_EVIDENCE {
        let bytes = fs::read(root.join(path)).unwrap();
        assert_eq!(format!("{:X}", Sha256::digest(&bytes)), expected_hash);
        let image = image::load_from_memory(&bytes).unwrap();
        assert_eq!((image.width(), image.height()), (36, 36));
    }
}

#[test]
fn nano_viewer_selection_opens_and_closes_without_mutating_authority() {
    let mut viewer = UserEquipNanoViewerState::default();
    assert_eq!(viewer.selected_visual_index(), None);
    viewer.open(7);
    assert_eq!(viewer.selected_visual_index(), Some(7));
    viewer.close();
    assert_eq!(viewer.selected_visual_index(), None);
}
