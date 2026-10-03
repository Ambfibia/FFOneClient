use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaceAbiScalar {
    I16,
    I32,
    U64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceAbiField {
    pub clean_name: &'static str,
    pub offset: usize,
    pub scalar: RaceAbiScalar,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RaceEcomType {
    Start = 0,
    End = 1,
    Fail = 2,
    Rank = 3,
}

impl RaceEcomType {
    /// The exact clean `OnGUI` control-flow result.
    #[must_use]
    pub const fn paints(self) -> bool {
        matches!(self, Self::End)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RaceTopRecord {
    pub rank: i32,
    pub rings: i32,
    pub score: i32,
    pub time_seconds: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RaceNpcContext {
    pub instance_id: i32,
    pub has_race_start_voice: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RaceRewardItem {
    /// `sItemBase.iType`, ABI offset 0, signed 16-bit.
    pub item_type: i16,
    /// `sItemBase.iID`, ABI offset 2, signed 16-bit.
    pub item_id: i16,
    /// `sItemBase.iOpt`, ABI offset 4.
    pub item_opt: i32,
    /// `sItemBase.iTimeLimit`, ABI offset 8.
    pub time_limit: i32,
    /// Inventory-location enum, ABI offset 12.
    pub e_il: i32,
    /// Inventory slot, ABI offset 16.
    pub slot: i32,
}

impl RaceRewardItem {
    #[must_use]
    pub const fn was_granted(self) -> bool {
        self.item_id > 0 && self.e_il != 4 && self.slot >= 0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RaceEndSuccess {
    pub race_mode: i32,
    pub race_time_seconds: i32,
    pub ring_count: i32,
    pub score: i32,
    pub rank: i32,
    pub reward_fusion_matter: i32,
    pub top_score: i32,
    pub top_rank: i32,
    pub top_time_seconds: i32,
    pub top_ring_count: i32,
    pub fusion_matter: i32,
    pub reward_item: RaceRewardItem,
    pub fatigue: i32,
    pub fatigue_level: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct RaceResult {
    pub race_time_seconds: i32,
    pub reward_fusion_matter: i32,
    pub ring_count: i32,
    pub score: i32,
    pub rank: i32,
    pub reward_item: RaceRewardItem,
    /// Kept for exact dormant UI semantics; normal clean routing never sets it.
    pub inventory_full: bool,
}

impl From<RaceEndSuccess> for RaceResult {
    fn from(value: RaceEndSuccess) -> Self {
        Self {
            race_time_seconds: value.race_time_seconds,
            reward_fusion_matter: value.reward_fusion_matter,
            ring_count: value.ring_count,
            score: value.score,
            rank: value.rank,
            reward_item: value.reward_item,
            inventory_full: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RaceEcomOperation {
    Delete(i32),
    Make(i32),
}

#[derive(Clone, Debug, Default, Resource, PartialEq, Eq)]
pub struct RaceRewardPresentation {
    pub item_type: i16,
    pub item_id: i16,
    pub item_opt: i32,
    pub name: String,
    pub name_key: Option<String>,
    pub level: i32,
    pub icon_path: Option<String>,
}

impl RaceRewardPresentation {
    #[must_use]
    pub fn matches(&self, item: RaceRewardItem) -> bool {
        self.item_type == item.item_type
            && self.item_id == item.item_id
            && self.item_opt == item.item_opt
    }
}
