use super::*;

#[derive(Clone, Copy)]
pub(super) struct TestCatalog;

impl EmailItemCatalog0104 for TestCatalog {
    fn resolve(&self, item: ItemBase0104) -> Option<EmailItemCatalogMetadata0104> {
        if item.item_id == 900 {
            return None;
        }
        Some(EmailItemCatalogMetadata0104 {
            icon_path: Some(if item.item_id == 901 {
                "../unowned.png".to_owned()
            } else {
                format!("icons/test-{}.png", item.item_id)
            }),
            tradeable: (item.item_id != 902).then_some(item.item_id != 903),
            general_item_type: (item.item_id != 904).then_some(if item.item_id == 905 {
                3
            } else {
                2
            }),
        })
    }
}

#[test]
fn open_keeps_inventory_and_mail_access_when_catalog_icon_is_missing_or_unsafe() {
    for item_id in [900, 901] {
        let inventory = authority_with(Some(base_item(7, item_id, 1)));
        let mut production = EmailProductionRuntime0104::default();
        let mut model = EmailUiModel::default();
        let mut actions = EmailUiOutbox::default();
        let mut audio = EmailUiAudioOutbox::default();
        let network = EmailNetworkRuntime0104::default();
        let inbox = EmailNetworkInbox0104::default();
        let transport = EmailTransportOutbox::default();
        let result = production.open(
            EmailOpenContext0104 {
                player: EmailPlayerAuthority0104 {
                    owner_pc_id: 42,
                    taros: 1,
                    current_local_time: EmailSystemTime::default(),
                },
                cursor_was_locked: false,
                item_policy: EmailItemFeaturePolicy0104::default(),
            },
            Vec::new(),
            Vec::new(),
            &inventory,
            &TestCatalog,
            &mut model,
            &mut actions,
            &mut audio,
            &network,
            &inbox,
            &transport,
        );
        result.unwrap();
        assert!(model.visible);
        assert!(production.modal_active());
        let slot = model.inventory[0].as_ref().unwrap();
        assert_eq!(slot.item, email_item_from_base(base_item(7, item_id, 1)));
        assert!(slot.icon_path.is_none());
        assert_eq!(production.session().unwrap().inventory(), &inventory);

    }
}
