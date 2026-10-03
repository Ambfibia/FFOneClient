use super::*;

#[derive(Default)]
pub(super) struct FixtureCatalog {
    pub(super) rows: HashMap<(i16, i16), CombiItemMetadata0104>,
}

impl FixtureCatalog {
    pub(super) fn insert(&mut self, item_type: i16, item_id: i16, metadata: CombiItemMetadata0104) {
        self.rows.insert((item_type, item_id), metadata);
    }
}

impl CombiItemCatalog0104 for FixtureCatalog {
    fn resolve(&self, item_type: i16, item_id: i16) -> Option<CombiItemMetadata0104> {
        self.rows.get(&(item_type, item_id)).cloned()
    }

    fn enable_equip(&self, _item: ItemBase0104) -> Option<bool> {
        Some(true)
    }

    fn enable_equip_combi(&self, _item: ItemBase0104) -> Option<bool> {
        Some(true)
    }
}
