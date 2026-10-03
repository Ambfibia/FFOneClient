
pub const NANO_FREE_TUNING_CREATION_SOUND: &str = "Nano_Creation";

pub const NANO_FREE_TUNING_SELECT_SOUND: &str = "Add_Attribute";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NanoFreeTuningSoundIntent {
    Play(&'static str),
}
