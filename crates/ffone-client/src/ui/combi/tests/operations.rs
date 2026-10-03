use super::*;

pub(super) fn item(item_type: i16, item_id: i16, option: i32, time_limit: i32) -> ItemBase0104 {
    ItemBase0104 {
        item_type,
        item_id,
        option,
        time_limit,
    }
}

pub(super) fn metadata(
    name: &str,
    minimum_level: i32,
    rarity: i32,
    item_price: i32,
    target_mode: i32,
    icon_path: &str,
) -> CombiItemMetadata0104 {
    CombiItemMetadata0104 {
        name: name.to_owned(),
        description: format!("{name} description"),
        minimum_level,
        required_gender: 1,
        rarity,
        rarity_label: format!("Rarity {rarity}"),
        mentor: 0,
        cashable: 0,
        item_price,
        point_rating: minimum_level * 10,
        group_rating: minimum_level * 11,
        defense_rating: minimum_level * 12,
        delay_time: 10,
        equip_type: 4,
        target_mode,
        type_label: "Weapon".to_owned(),
        trade_label: "Tradable".to_owned(),
        icon_path: Some(icon_path.to_owned()),
    }
}

pub(super) fn fixture() -> (
    CombiAuthoritativeSnapshot0104,
    CombiSelectionOverlay0104,
    FixtureCatalog,
    CombiRecipeTable0104,
) {
    let mut inventory = [empty_item_0104(); INVENTORY_SLOT_COUNT_0104];
    inventory[4] = item(0, 10, 11 << 16, 55);
    inventory[7] = item(0, 20, 0, 66);
    inventory[9] = item(0, 30, 0, 77);
    let mut equipment = [empty_item_0104(); EQUIPMENT_SLOT_COUNT_0104];
    equipment[0] = item(0, 30, 0, 0);
    let snapshot = CombiAuthoritativeSnapshot0104 {
        owner_pc_id: 4_242,
        gender: 1,
        level: 8,
        guide: 2,
        taros: 701,
        equipment,
        inventory,
    };
    let mut selection = CombiSelectionOverlay0104::default();
    selection
        .attach(
            &snapshot,
            CombiSourceLocation0104::Inventory,
            4,
            CombiSelectionSlot0104::Style,
        )
        .expect("style attach");
    selection
        .attach(
            &snapshot,
            CombiSourceLocation0104::Inventory,
            7,
            CombiSelectionSlot0104::Stats,
        )
        .expect("stats attach");
    let mut catalog = FixtureCatalog::default();
    catalog.insert(
        0,
        10,
        metadata(
            "Base Style",
            3,
            0,
            90,
            5,
            "icons/items/weapons/wpnicon_01.png",
        ),
    );
    catalog.insert(
        0,
        11,
        metadata(
            "Visible Style",
            6,
            1,
            100,
            5,
            "icons/items/weapons/wpnicon_01.png",
        ),
    );
    catalog.insert(
        0,
        20,
        metadata(
            "Stats Item",
            4,
            2,
            200,
            5,
            "icons/items/weapons/wpnicon_02.png",
        ),
    );
    catalog.insert(
        0,
        30,
        metadata(
            "Equipped",
            2,
            1,
            50,
            5,
            "icons/items/weapons/wpnicon_02.png",
        ),
    );
    let recipes =
        CombiRecipeTable0104::from_table_set_bytes(CLEAN_TABLE_SET).expect("clean recipes");
    (snapshot, selection, catalog, recipes)
}

#[test]
fn published_clean_recipe_table_is_strict_and_indexed_after_sentinel() {
    assert_eq!(
        format!("{:x}", Sha256::digest(CLEAN_TABLE_SET)),
        COMBI_RECIPE_TABLE_SHA256
    );
    let table =
        CombiRecipeTable0104::from_table_set_bytes(CLEAN_TABLE_SET).expect("clean table");
    assert_eq!(table.rows().len(), 38);
    assert_eq!(table.rows()[0].level_gap_standard, 0.0);
    let gap_zero = table.row_for_level_gap(0).expect("gap zero");
    assert_eq!(gap_zero.level_gap, 0);
    assert_eq!(gap_zero.same_grade, 95.0);
    assert_eq!(gap_zero.look_constant, 1);
    assert_eq!(gap_zero.stat_constant, 3);
    assert_eq!(
        table
            .row_for_level_gap(COMBI_RECIPE_MAX_LEVEL_GAP_0104)
            .expect("last clean gap")
            .level_gap,
        COMBI_RECIPE_MAX_LEVEL_GAP_0104 as i32
    );
    assert!(
        table
            .row_for_level_gap(COMBI_RECIPE_MAX_LEVEL_GAP_0104 + 1)
            .is_none()
    );
}

#[test]
fn equipped_item_is_rejected_with_exact_clean_message_260() {
    let (snapshot, _, _, _) = fixture();
    let mut overlay = CombiSelectionOverlay0104::default();
    assert_eq!(
        overlay.attach(
            &snapshot,
            CombiSourceLocation0104::Equipment,
            0,
            CombiSelectionSlot0104::Style,
        ),
        Err(CombiSelectionError0104::EquippedItem {
            equipment_index: 0,
            system_message: COMBI_MESSAGE_260,
        })
    );
}

#[test]
fn projection_uses_combined_look_row_but_displays_base_level() {
    let (snapshot, selection, catalog, recipes) = fixture();
    let projection =
        project_combi_mode_0104(&snapshot, selection, &catalog, &recipes).expect("projection");
    let look = projection.look.as_ref().expect("look");
    assert_eq!(look.base_metadata.as_ref().expect("base").minimum_level, 3);
    assert_eq!(
        look.appearance_metadata
            .as_ref()
            .expect("appearance")
            .minimum_level,
        6
    );
    assert_eq!(look.appearance_item_id, 11);
    assert_eq!(projection.cost, 700);
    assert_eq!(
        projection.chance,
        CombiChance0104::Good { raw_percent: 87.5 }
    );
    assert!(projection.combine_enabled);
    assert!(projection.inventory[4].hidden_by_selection_overlay);
    assert!(projection.inventory[7].hidden_by_selection_overlay);
}

#[test]
fn equal_taros_is_not_enough_by_clean_strict_greater_than_rule() {
    let (mut snapshot, selection, catalog, recipes) = fixture();
    snapshot.taros = 700;
    let projection =
        project_combi_mode_0104(&snapshot, selection, &catalog, &recipes).expect("projection");
    let mut machine = CombiMachine0104 {
        phase: CombiPhase0104::Ready,
        selection,
        pending: None,
    };
    machine
        .begin_combine(&snapshot, &projection)
        .expect("handled modal");
    assert_eq!(
        machine.phase,
        CombiPhase0104::Modal(CombiSystemModal0104::NotEnoughTaros)
    );
}

#[test]
fn wait_is_strictly_greater_than_four_and_never_mutates_snapshot() {
    let (snapshot, selection, catalog, recipes) = fixture();
    let projection =
        project_combi_mode_0104(&snapshot, selection, &catalog, &recipes).expect("projection");
    let before = snapshot.clone();
    let mut machine = CombiMachine0104 {
        phase: CombiPhase0104::Ready,
        selection,
        pending: None,
    };
    machine
        .begin_combine(&snapshot, &projection)
        .expect("confirm");
    machine
        .resolve_modal(CombiModalChoice0104::Continue)
        .expect("continue");
    assert_eq!(machine.tick_waiting(4.0).expect("tick"), None);
    assert_eq!(snapshot, before);
    assert_eq!(
        machine.tick_waiting(0.001).expect("strict tick"),
        Some(CombiRequest0104 {
            costume_item_slot: 4,
            stat_item_slot: 7,
            cash_item_slot_1: 0,
            cash_item_slot_2: 0,
        })
    );
    assert_eq!(snapshot, before);
}

#[test]
fn inconsistent_authoritative_item_is_rejected_and_machine_stays_locked() {
    let (snapshot, selection, catalog, recipes) = fixture();
    let projection =
        project_combi_mode_0104(&snapshot, selection, &catalog, &recipes).expect("projection");
    let mut machine = awaiting_machine(&snapshot, selection, &projection);
    let error = machine
        .receive_authoritative_reply(
            &snapshot,
            &projection,
            CombiSuccessReply0104 {
                new_item_slot: 4,
                new_item: item(0, 999, 0, 0),
                stat_item_slot: 7,
                cash_item_slot_1: 0,
                cash_item_slot_2: 0,
                taros_after: 1,
                success_flag: 1,
            },
        )
        .expect_err("invented post-state must fail");
    assert!(matches!(
        error,
        CombiReplyError0104::UnexpectedSuccessItem { .. }
    ));
    assert_eq!(machine.phase, CombiPhase0104::AwaitingAuthoritativeReply);
    assert!(machine.phase.sending_locked());
}

pub(super) fn awaiting_machine(
    snapshot: &CombiAuthoritativeSnapshot0104,
    selection: CombiSelectionOverlay0104,
    projection: &CombiModeProjection0104,
) -> CombiMachine0104 {
    let mut machine = CombiMachine0104 {
        phase: CombiPhase0104::Ready,
        selection,
        pending: None,
    };
    machine.begin_combine(snapshot, projection).expect("begin");
    machine
        .resolve_modal(CombiModalChoice0104::Continue)
        .expect("continue");
    assert!(machine.tick_waiting(4.001).expect("tick").is_some());
    machine
}

#[test]
fn abi_constants_match_clean_pack4_structs() {
    assert_eq!(COMBI_REQUEST_PACKET_ID_0104, 0x1300_0098);
    assert_eq!(COMBI_SUCCESS_PACKET_ID_0104, 0x3100_0116);
    assert_eq!(COMBI_FAILURE_PACKET_ID_0104, 0x3100_0117);
    assert_eq!(COMBI_REQUEST_PACKET_SIZE_0104, 16);
    assert_eq!(COMBI_SUCCESS_PACKET_SIZE_0104, 36);
    assert_eq!(COMBI_FAILURE_PACKET_SIZE_0104, 20);
    assert_eq!(COMBI_ITEM_BASE_SIZE_0104, 12);
    assert_eq!(
        [
            COMBI_REQUEST_COSTUME_SLOT_OFFSET_0104,
            COMBI_REQUEST_STAT_SLOT_OFFSET_0104,
            COMBI_REQUEST_CASH_SLOT_1_OFFSET_0104,
            COMBI_REQUEST_CASH_SLOT_2_OFFSET_0104,
        ],
        [0, 4, 8, 12]
    );
    assert_eq!(
        [
            COMBI_SUCCESS_NEW_ITEM_SLOT_OFFSET_0104,
            COMBI_SUCCESS_NEW_ITEM_OFFSET_0104,
            COMBI_SUCCESS_STAT_SLOT_OFFSET_0104,
            COMBI_SUCCESS_CASH_SLOT_1_OFFSET_0104,
            COMBI_SUCCESS_CASH_SLOT_2_OFFSET_0104,
            COMBI_SUCCESS_TAROS_OFFSET_0104,
            COMBI_SUCCESS_FLAG_OFFSET_0104,
        ],
        [0, 4, 16, 20, 24, 28, 32]
    );
    assert_eq!(
        [
            COMBI_FAILURE_ERROR_OFFSET_0104,
            COMBI_FAILURE_COSTUME_SLOT_OFFSET_0104,
            COMBI_FAILURE_STAT_SLOT_OFFSET_0104,
            COMBI_FAILURE_CASH_SLOT_1_OFFSET_0104,
            COMBI_FAILURE_CASH_SLOT_2_OFFSET_0104,
        ],
        [0, 4, 8, 12, 16]
    );
}

pub(super) fn carried(app: &App) -> Option<CombiCarriedItem0104> {
    app.world().resource::<CombiPointerState0104>().carried()
}
