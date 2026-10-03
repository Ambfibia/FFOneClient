
/// TableData-owned metadata needed by the clean email inventory and
/// `InventoryManagerScript.PutItemInEmailSlot` branches.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmailItemCatalogMetadata0104 {
    /// Optional installed artwork, independent of authoritative trade metadata.
    /// Unsafe paths are never loaded. Missing artwork does not block the mailbox.
    pub icon_path: Option<String>,
    /// Exact clean `m_iTradeAble`. `None` means the catalog did not prove it.
    pub tradeable: Option<bool>,
    /// Exact `GeneralItemElement.m_iItemType`, required only for outer type 7.
    pub general_item_type: Option<i32>,
}

pub(super) fn is_safe_email_icon_path(path: &str) -> bool {
    !path.is_empty()
        && !path.contains('\\')
        && !path.starts_with('/')
        && path.starts_with("icons/")
        && path.ends_with(".png")
        && !path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "." || segment == "..")
}
