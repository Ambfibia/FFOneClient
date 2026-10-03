use super::*;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SocialRequestSettings {
    pub allow_group_invites: bool,
    pub allow_buddy_requests: bool,
    pub allow_trade_requests: bool,
}

impl Default for SocialRequestSettings {
    fn default() -> Self {
        Self {
            allow_group_invites: true,
            allow_buddy_requests: true,
            allow_trade_requests: true,
        }
    }
}

impl SocialRequestSettings {
    pub fn restore_defaults(&mut self) {
        *self = Self::default();
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum LegacyOptionAction {
    Up,
    Down,
    Left,
    Right,
    Escape,
    Jump,
    Fire1,
    Fire2,
    Nano1,
    Nano2,
    Nano3,
    WeaponChange,
    WeaponCharge,
    NanoCharge,
    Inventory,
    NanoBook,
    Journal,
    Email,
    Contacts,
    Help,
    AutoRun,
    Menu,
    WorldMap,
    Option,
    LeftTurn,
    RightTurn,
    FreeCamera,
    SendChat,
    ZoomIn,
    ZoomOut,
    CameraUp,
    CameraDown,
    CameraLeft,
    CameraRight,
    Skill1,
    Skill2,
    Skill3,
    VehicleToggle,
    None,
}

impl LegacyOptionAction {
    pub const ALL: [Self; OPTION_INPUT_ACTION_COUNT] = [
        Self::Up,
        Self::Down,
        Self::Left,
        Self::Right,
        Self::Escape,
        Self::Jump,
        Self::Fire1,
        Self::Fire2,
        Self::Nano1,
        Self::Nano2,
        Self::Nano3,
        Self::WeaponChange,
        Self::WeaponCharge,
        Self::NanoCharge,
        Self::Inventory,
        Self::NanoBook,
        Self::Journal,
        Self::Email,
        Self::Contacts,
        Self::Help,
        Self::AutoRun,
        Self::Menu,
        Self::WorldMap,
        Self::Option,
        Self::LeftTurn,
        Self::RightTurn,
        Self::FreeCamera,
        Self::SendChat,
        Self::ZoomIn,
        Self::ZoomOut,
        Self::CameraUp,
        Self::CameraDown,
        Self::CameraLeft,
        Self::CameraRight,
        Self::Skill1,
        Self::Skill2,
        Self::Skill3,
        Self::VehicleToggle,
        Self::None,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Up => "FORWARD",
            Self::Down => "BACKWARD",
            Self::Left => "SIDESTEP LEFT",
            Self::Right => "SIDESTEP RIGHT",
            Self::Escape => "ESCAPE",
            Self::Jump => "JUMP",
            Self::Fire1 => "ATTACK/USE TARGET",
            Self::Fire2 => "NANO POWER",
            Self::Nano1 => "SUMMON NANO #1",
            Self::Nano2 => "SUMMON NANO #2",
            Self::Nano3 => "SUMMON NANO #3",
            Self::WeaponChange => "SWITCH WEAPON",
            Self::WeaponCharge => "CHARGE WEAPON",
            Self::NanoCharge => "NANO BOOST",
            Self::Inventory => "MY STUFF",
            Self::NanoBook => "NANO COM",
            Self::Journal => "JOURNAL",
            Self::Email => "E-MAIL",
            Self::Contacts => "CONTACTS",
            Self::Help => "HELP",
            Self::AutoRun => "AUTORUN",
            Self::Menu => "MENU",
            Self::WorldMap => "MAP",
            Self::Option => "OPTIONS",
            Self::LeftTurn => "TURN LEFT",
            Self::RightTurn => "TURN RIGHT",
            Self::FreeCamera => "FREE CAMERA",
            Self::SendChat => "SEND CHAT",
            Self::ZoomIn => "ZOOM IN",
            Self::ZoomOut => "ZOOM OUT",
            Self::CameraUp => "CAMERA UP",
            Self::CameraDown => "CAMERA DOWN",
            Self::CameraLeft => "CAMERA LEFT",
            Self::CameraRight => "CAMERA RIGHT",
            Self::Skill1 => "SKILL #1",
            Self::Skill2 => "SKILL #2",
            Self::Skill3 => "SKILL #3",
            Self::VehicleToggle => "VEHICLE",
            Self::None => "NONE",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum OptionUiAction {
    PersistOptions(OptionSettings),
    PersistInput(InputSettings),
    /// Clean `cnSoundOption.Save(); cnSoundOption.Apply();` eager path.
    ApplySoundImmediately(SoundSettings),
    RemoveBuddy {
        slot: usize,
        pc_uid: i64,
    },
    Closed,
}

#[derive(Clone, Debug, PartialEq)]
pub enum OptionUiEvent {
    Action(OptionUiAction),
    Audio(OptionUiAudioCue),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SocialRequestKind {
    Group,
    Buddy,
    Trade,
}
