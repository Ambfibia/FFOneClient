use super::*;

#[derive(Clone, Debug)]
pub(super) struct TutorialTableRoute {
    pub(super) source_path: String,
    pub(super) path: String,
    pub(super) bytes: u64,
    pub(super) blake3: String,
}

pub(super) trait TutorialAssetSource {
    fn table(&self) -> TutorialMissionContentResult<(TutorialTableRoute, Vec<u8>)>;
    fn installed_icon_paths(&self) -> TutorialMissionContentResult<BTreeSet<String>>;
}

/// Exact clean `NpcIconMode.WarpOK` inventory-search input.
///
/// `item_location` is `Quest` only when the row's *limit-item type* is 8;
/// clean code intentionally reuses that first type when it resolves the
/// separate use-item slot as well.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameplayWarpItemLookup {
    pub item_location: GameplayWarpInventoryLocation,
    pub start_slot: i32,
    pub item_id: i32,
    pub item_type: i32,
}

/// Mirrors `ResurrectMode.GetXCom`: index zero is never considered, rows with
/// a zero X coordinate are skipped, zone must match, and distance uses all
/// three protocol axes.
#[must_use]
pub fn clean_nearest_xcom_index(
    rows: &[GameplayXcomDefinition],
    zone: i32,
    position: [i32; 3],
) -> Option<i32> {
    rows.iter()
        .filter(|row| row.row_index > 0 && row.position[0] != 0 && row.zone == zone)
        .min_by_key(|row| {
            row.position
                .into_iter()
                .zip(position)
                .map(|(candidate, player)| {
                    let delta = i128::from(candidate) - i128::from(player);
                    delta * delta
                })
                .sum::<i128>()
        })
        .map(|row| row.row_index)
}

impl TutorialAssetSource for AssetLocator {
    fn table(&self) -> TutorialMissionContentResult<(TutorialTableRoute, Vec<u8>)> {
        let bytes =
            self.read(TABLE_SET_PATH)
                .map_err(|detail| TutorialMissionContentError::Io {
                    path: TABLE_SET_PATH.to_owned(),
                    detail,
                })?;
        Ok((
            TutorialTableRoute {
                source_path: TABLE_SET_PATH.to_owned(),
                path: TABLE_SET_PATH.to_owned(),
                bytes: bytes.len() as u64,
                blake3: blake3::hash(&bytes).to_hex().to_string(),
            },
            bytes,
        ))
    }

    fn installed_icon_paths(&self) -> TutorialMissionContentResult<BTreeSet<String>> {
        collect_relative_files(self.root(), "icons")
    }
}

pub(super) fn avatar_util_semantic_icon_path(icon_type: u8, icon_number: u32) -> Option<String> {
    let number = format!("{icon_number:02}");
    match icon_type {
        0 => Some(format!("icons/items/weapons/wpnicon_{number}.png")),
        1 => primary_nano_icon_slug(icon_number)
            .map(|slug| format!("icons/entities/nanos/nanoicon_{slug}.png")),
        2 => Some(format!("icons/skills/skillicon_{number}.png")),
        3 => Some(format!("icons/items/cosmetics/cosicon_{number}.png")),
        4 => Some(format!("icons/entities/npc/npcicon_{number}.png")),
        // Forced type 5 Nano-ready icons are projected by
        // `nano_ready_semantic_icon_path`; types 6 and 9 have no installed
        // native semantic catalog, so AvatarUtil's missing texture remains
        // their exact presentation.
        7 => Some(format!("icons/items/general/generalitemicon_{number}.png")),
        8 => Some(format!("icons/entities/mobs/mobicon_{number}.png")),
        10 => Some(format!("icons/entities/hnpc/hnpcicon_{number}.png")),
        11 => Some(format!("icons/transport/transport_{number}.png")),
        12 => Some(format!("icons/items/vehicles/vehicle_{number}.png")),
        _ => None,
    }
}

pub(super) fn nano_ready_semantic_icon_path(icon_number: u32) -> Option<String> {
    // InventoryManagerScript.GetNanoSlot preserves the m_iIconNumber resolved
    // through NanoElement.m_iIcon1 and only forces m_iIconType to 5. Zero is a
    // valid clean value: Nano 34 resolves icon row 27 / number 0, and primary
    // AvatarUtil therefore requests the real `Icons/nanoready_00.png` asset.
    primary_nano_icon_slug(icon_number)
        .map(|slug| format!("icons/entities/nanos/ready/nanoready_{slug}.png"))
}

pub(super) fn general_item_semantic_icon_path(icon_type: u8, icon_number: u32) -> Option<String> {
    // `AvatarUtil.GetEquipIconElement(7, id, 1)` resolves the row and then
    // dispatches by the icon row's own type. QuickSlot's clean table is type
    // 7; retaining the gate prevents an invented cross-category fallback.
    (icon_type == 7).then(|| format!("icons/items/general/generalitemicon_{icon_number:02}.png"))
}
