use super::*;

/// Short authored blend used when entering FFR-only custom emotes.
pub const FFR_CUSTOM_EMOTE_CROSS_FADE_SECONDS: f32 = 0.2;

/// Exact `cnAvatarAnimation.AvatarEmote(int)` cross-fade duration.
pub const AVATAR_EMOTE_CODE_CROSS_FADE_SECONDS: f32 = 0.15;

/// Exact `cnAvatarAnimation.AvatarStandForce` `0.15f` fade length.
pub const TUTORIAL_STAND_FORCE_CROSS_FADE_SECONDS: f32 = 0.15;

/// The only direct string `AvatarEmote` clips called by `cntutorialscript`.
pub const TUTORIAL_DIRECT_AVATAR_EMOTE_NAMES: [&str; 3] = ["staying", "standup", "run"];

pub const FFR_CUSTOM_AVATAR_EMOTE_NAMES: [&str; 14] = [
    "ffr_dance_02",
    "ffr_dance_04",
    "ffr_dance_07",
    "ffr_dance_08",
    "ffr_dance_10",
    "ffr_dance_13",
    "ffr_dance_15",
    "ffr_dance_18",
    "ffr_dance_19",
    "ffr_dance_20",
    "ffr_dance_bully",
    "ffr_dance_tellme",
    "ffr_emote_catpose",
    "ffr_emote_idolpose",
];

/// Original tutorial weapon event slot and `sItemBase` type.
pub const TUTORIAL_WEAPON_SLOT: CharacterEquipSlot0104 = CharacterEquipSlot0104::Hand;

pub const TUTORIAL_WEAPON_ITEM_TYPE: i16 = 0;
