use super::*;

#[test]
fn projection_reuses_exact_icon_boundary_and_unknown_equip_validation_is_disabled() {
    let open = open_with(
        1,
        &[
            (0, item(0, 42, 0x1234_0002_u32 as i32, 0)),
            (1, item(7, 77, 25, 0)),
            (2, item(8, 88, 0, 0)),
        ],
    );
    let projection = BankModeProjection0104::from_authoritative_open(
        OWNER_PC_ID,
        NPC_ID,
        &open,
        &runtime_with(&[]),
        &MissingCatalog,
        &BankFailClosedEquipEligibility,
    )
    .unwrap();
    assert!(projection.bank[0].show_combined_badge);
    assert_eq!(
        projection.bank[0].equip_validation,
        BankEquipValidation::Unverified
    );
    assert!(matches!(
        projection.bank[0].icon,
        UserEquipProjectedIcon::MissingChecker(UserEquipMissingIconReason::CatalogMiss { .. })
    ));
    assert_eq!(projection.bank[1].count_label.as_deref(), Some("25"));
    assert!(matches!(
        projection.bank[2].icon,
        UserEquipProjectedIcon::MissingChecker(
            UserEquipMissingIconReason::QuestLegacyNull { .. }
        )
    ));
}
