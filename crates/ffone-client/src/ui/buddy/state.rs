
pub const BUDDY_STATE_REFRESH_SECONDS: f32 = 10.0;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BuddyStateUpdate {
    pub slot: usize,
    pub runtime_pc_id: i32,
    pub legacy_state: i8,
}
