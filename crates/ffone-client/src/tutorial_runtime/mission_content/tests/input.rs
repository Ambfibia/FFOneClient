use super::*;

#[test]
fn vendor_system_messages_resolve_exact_fixture_rows_without_fallbacks() {
    let content =
        TutorialMissionContent::from_project_assets(&Fixture::new(compact_document()).assets)
            .unwrap();
    let expected = [
        (
            VendorSystemMessageId0104::InventoryFull,
            "Inventory is full.",
            0,
            SystemMessageButtonType::None,
        ),
        (
            VendorSystemMessageId0104::StartFailed,
            "You cannot use this vendor currently.",
            10,
            SystemMessageButtonType::Ok,
        ),
        (
            VendorSystemMessageId0104::TableUpdateFailed,
            "Could not retrieve item information.",
            10,
            SystemMessageButtonType::Ok,
        ),
        (
            VendorSystemMessageId0104::BuyFailed,
            "This item could not be purchased.",
            1,
            SystemMessageButtonType::Ok,
        ),
        (
            VendorSystemMessageId0104::BatteryFailed,
            "This item could not be purchased.",
            1,
            SystemMessageButtonType::Ok,
        ),
        (
            VendorSystemMessageId0104::SellFailed,
            "This item could not be sold.",
            1,
            SystemMessageButtonType::Ok,
        ),
        (
            VendorSystemMessageId0104::CannotSellChest,
            "Unopened CRATEs cannot be sold to shopkeepers.",
            10,
            SystemMessageButtonType::Ok,
        ),
        (
            VendorSystemMessageId0104::CannotSellQuestItem,
            "You cannot sell quest items.",
            10,
            SystemMessageButtonType::Ok,
        ),
        (
            VendorSystemMessageId0104::ItemNotSellable,
            "This item cannot be sold.",
            10,
            SystemMessageButtonType::Ok,
        ),
        (
            VendorSystemMessageId0104::ConfirmDelete,
            "DELETE THIS ITEM?",
            12,
            SystemMessageButtonType::DeleteItem,
        ),
        (
            VendorSystemMessageId0104::ConfirmDisassemble,
            "Alert! One or more of your vehicle rentals has expired.",
            0,
            SystemMessageButtonType::None,
        ),
        (
            VendorSystemMessageId0104::PurchaseRestricted,
            "CANNOT PURCHASE: MISSION NOT COMPLETED. \nYou have not yet completed the mission that rewards this guide item. Please complete the associated mission, then try purchasing again.",
            1,
            SystemMessageButtonType::Ok,
        ),
    ];
    for (message_id, exact_text, raw_button_type, runtime_button_type) in expected {
        assert_eq!(
            content.system_message_definition(message_id.id()),
            Some(&SystemMessageDefinition {
                row_id: message_id.id(),
                exact_text: exact_text.to_owned(),
                raw_button_type,
                runtime_button_type,
            })
        );
    }

    assert_eq!(content.system_message_definition(256), None);
    let mut missing = compact_document();
    missing["tables"][0]["value"]["m_pMessageTable"]["m_pMessageData"][257] = Value::Null;
    let missing =
        TutorialMissionContent::from_project_assets(&Fixture::new(missing).assets).unwrap();
    assert_eq!(
        missing.system_message_definition(VendorSystemMessageId0104::ConfirmDisassemble.id()),
        None,
        "an absent source row must not invent fallback confirmation text"
    );
}
