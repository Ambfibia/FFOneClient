use super::*;

/// Clean `Panel_PCStuffScript.ActiveInventoryTab` value on Combi open.
pub const COMBI_ACTIVE_INVENTORY_TAB_0104: i32 = 0;

/// Clean My Stuff mode opened by `cnCombiMode.GoToMyStuff`.
pub const COMBI_MY_STUFF_GAME_MODE_0104: i32 = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CombiModeLease0104 {
    pub source: CombiServiceSource0104,
    pub game_mode: i32,
    pub active_inventory_tab: i32,
    pub cursor_locked_during_mode: bool,
    /// Both clean render cameras resolve this fixed NPC table ID, independent
    /// of which type-26 service NPC opened the mode.
    pub camera_npc_id: i32,
    pub first_use_condition_checked_by_service_menu: i32,
    pub normal_exit_sends_packet: bool,
}

impl From<CombiOpenContext0104> for CombiModeLease0104 {
    fn from(context: CombiOpenContext0104) -> Self {
        Self {
            source: context.source,
            game_mode: COMBI_GAME_MODE_0104,
            active_inventory_tab: COMBI_ACTIVE_INVENTORY_TAB_0104,
            cursor_locked_during_mode: false,
            camera_npc_id: COMBI_NPC_ID_0104,
            first_use_condition_checked_by_service_menu: COMBI_FIRST_USE_CONDITION_0104,
            normal_exit_sends_packet: COMBI_NORMAL_EXIT_SENDS_PACKET_0104,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CombiInventoryWrite0104 {
    pub inventory_index: usize,
    pub item: ItemBase0104,
}
