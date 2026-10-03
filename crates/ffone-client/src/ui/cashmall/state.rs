use super::*;

pub const CASHMALL_MANAGED_MODE_SHA256: &str =
    "74B6E6C515D9D91AA3E232199B15998B016696F0E38DA968D6DFE01E622241C4";

pub const CASHMALL_GAME_MODE_0104: i32 = 27;

pub const CASHMALL_INVENTORY_MODE_INITIALIZATION_REACHABLE: bool = false;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CashmallPlayerInventoryItem0104 {
    pub slot_type: i32,
    pub slot_id: i32,
    pub item: ItemBase0104,
    pub name: String,
    pub level: i32,
    pub item_price: i32,
    pub icon: Option<CashmallIconRef0104>,
    /// Result of the existing `InventoryManagerScript.EnableEquip` boundary.
    pub equip_eligible: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Resource)]
pub struct CashmallModeProjection0104 {
    pub player_inventory: Vec<CashmallPlayerInventoryItem0104>,
    pub shared_item_mode: UserEquipItemModeProjection,
    /// Clean compares item prices with Taros even though the Cash Mall display
    /// is the unrelated, permanently-zero `iUserCash` field.
    pub taros: i32,
}

impl Default for CashmallModeProjection0104 {
    fn default() -> Self {
        Self {
            player_inventory: Vec::new(),
            shared_item_mode: UserEquipItemModeProjection::default(),
            taros: 0,
        }
    }
}

impl CashmallModeProjection0104 {
    #[must_use]
    pub fn rows_for_tab(&self, _tab: CashmallTab0104) -> Vec<CashmallRowProjection0104> {
        // Exact retained bug: `eTabMode` is never consulted by `DoSlot`.
        cashmall_scan_slot9_0104(&self.player_inventory, self.taros)
    }

    #[must_use]
    pub fn rows(&self) -> Vec<CashmallRowProjection0104> {
        self.rows_for_tab(CashmallTab0104::New)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CashmallModeView0104 {
    pub layout: CashmallModeLayout0104,
    pub controls: CashmallInputCapabilities0104,
    pub tabs: [CashmallTabView0104; CASHMALL_TAB_COUNT],
    pub cash_digits: [u8; CASHMALL_CASH_DIGIT_COUNT],
    pub npc_name: String,
    pub rows: Vec<CashmallRowProjection0104>,
}

#[must_use]
pub fn cashmall_mode_view_0104(
    viewport_width: u32,
    viewport_height: u32,
    state: &CashmallUiState0104,
    modal: CashmallModalState0104,
    projection: &CashmallModeProjection0104,
    hovered_tabs: [bool; CASHMALL_TAB_COUNT],
    pressed_tabs: [bool; CASHMALL_TAB_COUNT],
    static_assets_ready: bool,
) -> Option<CashmallModeView0104> {
    let controls = state.input_capabilities(modal);
    if viewport_width == 0 || viewport_height == 0 || !controls.draw || !static_assets_ready {
        return None;
    }
    Some(CashmallModeView0104 {
        layout: cashmall_mode_layout_0104(
            viewport_width,
            viewport_height,
            state.opening_elapsed_seconds,
            state.inventory_scroll_y,
        ),
        controls,
        tabs: array::from_fn(|index| {
            cashmall_tab_view_0104(
                state.tab,
                CashmallTab0104::ALL[index],
                hovered_tabs[index],
                pressed_tabs[index],
            )
        }),
        cash_digits: cashmall_cash_digits_0104(),
        // `/cashmall` never invokes Panel_Cashmall.SetNpc.
        npc_name: String::new(),
        rows: projection.rows_for_tab(state.tab),
    })
}
