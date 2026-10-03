use super::*;

pub const USER_STORE_GAME_MODE_SLOT: usize = 28;

pub const USER_STORE_INVENTORY_CAPACITY: usize = 50;

pub const USER_STORE_INVENTORY_COLUMNS: usize = 5;

pub const USER_STORE_INVENTORY_ROWS: usize = 10;

pub const USER_STORE_INVENTORY_SLOT_SIZE: f32 = 67.0;

pub const USER_STORE_INVENTORY_SLOT_STRIDE: f32 = 69.0;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserStoreInventoryProjection0104 {
    pub slot: usize,
    pub slot_type: i32,
    pub item: UserStoreProjectedItem0104,
}
