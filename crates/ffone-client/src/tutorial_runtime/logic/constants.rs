use super::*;

pub const NUMBUH_TWO_ID: i32 = 1005;

pub const BUTTERCUP_ID: i32 = 1007;

pub const TECH_SQUARE_ATTENDANT_ID: i32 = 1009;

pub const FUSION_PORTAL_ID: i32 = 1012;

pub const LAIR_DEXTER_ID: i32 = 3000;

pub const FUSION_BUTTERCUP_ID: i32 = 4000;

pub const LAIR_EXIT_ID: i32 = 4001;

pub const DEMO_MONSTER_ID: i32 = 5000;

pub const COLLAPSE_NUMBUH_TWO_ID: i32 = 5100;

/// `CheckNpcDistance(1100, 30)`: the objective's Oil Ogre attacks once the
/// avatar is this close or has damaged it.
pub const MISSION_TARGET_ENGAGE_RADIUS: f32 = 30.0;

pub(super) const ALL_TUTORIAL_EVENTS: [TutorialEvent; TUTORIAL_EVENT_COUNT] = [
    TutorialEvent::CameraAngle,
    TutorialEvent::Move,
    TutorialEvent::Turn,
    TutorialEvent::Jump,
    TutorialEvent::UseSkill,
    TutorialEvent::DeadNpc,
    TutorialEvent::DamageNpc,
    TutorialEvent::DamageUser,
    TutorialEvent::EquipWeapon,
    TutorialEvent::ChangeWeapon,
    TutorialEvent::FreeChat,
    TutorialEvent::NanoEquip,
    TutorialEvent::NanoUnequip,
    TutorialEvent::NanoActive,
    TutorialEvent::NanoEquipCount,
    TutorialEvent::NanoCharge,
    TutorialEvent::VendorStart,
    TutorialEvent::VendorBuy,
    TutorialEvent::UseJumpPad,
    TutorialEvent::UseLauncher,
    TutorialEvent::UseZipLine,
    TutorialEvent::UseMovePlatform,
    TutorialEvent::UseSlide,
    TutorialEvent::QuestEnd,
    TutorialEvent::EquipHat,
    TutorialEvent::ItemChange,
    TutorialEvent::OpenInventory,
    TutorialEvent::BuffShiny,
    TutorialEvent::TaskStart,
    TutorialEvent::ActiveNanoId,
    TutorialEvent::OpenNanoManager,
    TutorialEvent::EquipWeapon2,
    TutorialEvent::NpcWarp,
    TutorialEvent::NpcIconClose,
];
