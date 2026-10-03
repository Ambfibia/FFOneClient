//! Engine-independent, reference-grounded Retrobution tutorial state.
//!
//! The original `cntutorialscript` stores its current chapter in `pFlag[2]`,
//! the current step in `pFlag[1]`, and 34 gameplay observations in
//! `pEventFlag`.  These types keep those numeric contracts visible while the
//! Bevy systems attach rendering, input, UI, combat, mission and cutscene
//! behaviour to them.

pub const TUTORIAL_EVENT_COUNT: usize = 34;
pub const TUTORIAL_INPUT_LOCK_COUNT: usize = 19;
pub const TUTORIAL_CHAPTER_COUNT: u8 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TutorialEvent {
    CameraAngle = 0,
    Move = 1,
    Turn = 2,
    Jump = 3,
    UseSkill = 4,
    DeadNpc = 5,
    DamageNpc = 6,
    DamageUser = 7,
    EquipWeapon = 8,
    ChangeWeapon = 9,
    FreeChat = 10,
    NanoEquip = 11,
    NanoUnequip = 12,
    NanoActive = 13,
    NanoEquipCount = 14,
    NanoCharge = 15,
    VendorStart = 16,
    VendorBuy = 17,
    UseJumpPad = 18,
    UseLauncher = 19,
    UseZipLine = 20,
    UseMovePlatform = 21,
    UseSlide = 22,
    QuestEnd = 23,
    EquipHat = 24,
    ItemChange = 25,
    OpenInventory = 26,
    BuffShiny = 27,
    TaskStart = 28,
    ActiveNanoId = 29,
    OpenNanoManager = 30,
    EquipWeapon2 = 31,
    NpcWarp = 32,
    NpcIconClose = 33,
}

impl TutorialEvent {
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TutorialInputLock {
    Front = 0,
    Back = 1,
    Side = 2,
    Turn = 3,
    Jump = 4,
    Attack = 5,
    NanoPower = 6,
    WeaponChange = 7,
    Menu = 8,
    Inventory = 9,
    Option = 10,
    Journal = 11,
    Social = 12,
    WorldMap = 13,
    Email = 14,
    Quit = 15,
    ModeChange = 16,
    Mouse = 17,
    NanoActive = 18,
}

impl TutorialInputLock {
    pub const fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TutorialScene {
    None = 0,
    BasicMove = 1,
    BasicCombatA = 2,
    BasicCombatB = 3,
    BasicCombatC = 4,
    InfectionA = 5,
    InfectionB = 6,
    InfectionC = 7,
    InfectionD = 8,
    NanoPowerA = 9,
    NanoPowerA2 = 10,
    NanoPowerB = 11,
}

impl Default for TutorialScene {
    fn default() -> Self {
        Self::None
    }
}

/// Inputs which the original `cntutorialscript` allows at a stable tutorial
/// stage. A set bit is the inverse of the matching `bInputLock` entry.
///
/// Scene and delay stages use [`Self::NONE`], because the original
/// `bBackground` makes every `IsLock` check succeed while those presentations
/// are active even if the underlying `bInputLock` array still has open entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TutorialInputPermissions {
    allowed_bits: u32,
}

impl TutorialInputPermissions {
    pub const NONE: Self = Self { allowed_bits: 0 };

    const fn from_bits(allowed_bits: u32) -> Self {
        Self { allowed_bits }
    }

    pub const fn allows(self, input: TutorialInputLock) -> bool {
        self.allowed_bits & input_bit(input) != 0
    }

    pub const fn allowed_bits(self) -> u32 {
        self.allowed_bits
    }
}

const fn input_bit(input: TutorialInputLock) -> u32 {
    1_u32 << input as u8
}

const INPUT_FRONT: u32 = input_bit(TutorialInputLock::Front);
const INPUT_BACK: u32 = input_bit(TutorialInputLock::Back);
const INPUT_SIDE: u32 = input_bit(TutorialInputLock::Side);
const INPUT_TURN: u32 = input_bit(TutorialInputLock::Turn);
const INPUT_JUMP: u32 = input_bit(TutorialInputLock::Jump);
const INPUT_ATTACK: u32 = input_bit(TutorialInputLock::Attack);
const INPUT_NANO_POWER: u32 = input_bit(TutorialInputLock::NanoPower);
const INPUT_MENU: u32 = input_bit(TutorialInputLock::Menu);
const INPUT_JOURNAL: u32 = input_bit(TutorialInputLock::Journal);
const INPUT_MODE_CHANGE: u32 = input_bit(TutorialInputLock::ModeChange);
const INPUT_MOUSE: u32 = input_bit(TutorialInputLock::Mouse);
const INPUT_NANO_ACTIVE: u32 = input_bit(TutorialInputLock::NanoActive);

const LOOK_INPUT: TutorialInputPermissions = TutorialInputPermissions::from_bits(INPUT_MOUSE);
const FORWARD_INPUT: TutorialInputPermissions =
    TutorialInputPermissions::from_bits(INPUT_FRONT | INPUT_MOUSE);
const BACKWARD_INPUT: TutorialInputPermissions =
    TutorialInputPermissions::from_bits(INPUT_BACK | INPUT_MOUSE);
const STEER_INPUT: TutorialInputPermissions =
    TutorialInputPermissions::from_bits(INPUT_FRONT | INPUT_TURN | INPUT_MOUSE);
const WALK_INPUT: TutorialInputPermissions = TutorialInputPermissions::from_bits(
    INPUT_FRONT | INPUT_BACK | INPUT_SIDE | INPUT_TURN | INPUT_MOUSE,
);
const MOVE_INPUT: TutorialInputPermissions =
    TutorialInputPermissions::from_bits(WALK_INPUT.allowed_bits | INPUT_JUMP);
const COMBAT_INPUT: TutorialInputPermissions =
    TutorialInputPermissions::from_bits(MOVE_INPUT.allowed_bits | INPUT_ATTACK);
const TALK_INPUT: TutorialInputPermissions =
    TutorialInputPermissions::from_bits(INPUT_TURN | INPUT_ATTACK | INPUT_MOUSE);
const TALK_CLOSE_INPUT: TutorialInputPermissions =
    TutorialInputPermissions::from_bits(TALK_INPUT.allowed_bits | INPUT_MENU | INPUT_MODE_CHANGE);
const COMBAT_MENU_INPUT: TutorialInputPermissions =
    TutorialInputPermissions::from_bits(COMBAT_INPUT.allowed_bits | INPUT_MENU | INPUT_MODE_CHANGE);
const MENU_JOURNAL_INPUT: TutorialInputPermissions =
    TutorialInputPermissions::from_bits(INPUT_MENU | INPUT_JOURNAL);
const GENERAL_INPUT: TutorialInputPermissions =
    TutorialInputPermissions::from_bits(COMBAT_MENU_INPUT.allowed_bits | INPUT_JOURNAL);
const NPC_MENU_INPUT: TutorialInputPermissions =
    TutorialInputPermissions::from_bits(COMBAT_INPUT.allowed_bits | INPUT_JOURNAL);
const NANO_SUMMON_INPUT: TutorialInputPermissions =
    TutorialInputPermissions::from_bits(INPUT_NANO_ACTIVE);
const NANO_COMBAT_INPUT: TutorialInputPermissions = TutorialInputPermissions::from_bits(
    GENERAL_INPUT.allowed_bits | INPUT_NANO_POWER | INPUT_NANO_ACTIVE,
);

/// Reference presentation and input facts attached to one legacy stage.
///
/// `instruction` is the exact hard-coded `SubText`/`SubText2` string displayed
/// by that stage. Localized voice subtitles come from `TextManager` and are not
/// duplicated here. `entry_voice_cue` is the first `Tut Sound/<name>.wav` cue
/// started on entry; longer coroutines can play more cues afterwards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialStageMetadata {
    pub instruction: Option<&'static str>,
    pub entry_voice_cue: Option<&'static str>,
    pub scene: TutorialScene,
    pub input: TutorialInputPermissions,
}

const fn stage_metadata(
    instruction: Option<&'static str>,
    entry_voice_cue: Option<&'static str>,
    scene: TutorialScene,
    input: TutorialInputPermissions,
) -> TutorialStageMetadata {
    TutorialStageMetadata {
        instruction,
        entry_voice_cue,
        scene,
        input,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialStage {
    Movement(MovementStage),
    Combat(CombatStage),
    Minimap(MinimapStage),
    Mission(MissionStage),
    Infection(InfectionStage),
    NanoPower(NanoPowerStage),
}

impl TutorialStage {
    pub fn from_legacy(chapter: u8, step: i16) -> Option<Self> {
        match chapter {
            0 => MovementStage::from_step(step).map(Self::Movement),
            1 => CombatStage::from_step(step).map(Self::Combat),
            2 => MinimapStage::from_step(step).map(Self::Minimap),
            3 => MissionStage::from_step(step).map(Self::Mission),
            4 => InfectionStage::from_step(step).map(Self::Infection),
            5 => NanoPowerStage::from_step(step).map(Self::NanoPower),
            _ => None,
        }
    }

    pub const fn legacy(self) -> (u8, i16) {
        match self {
            Self::Movement(stage) => (0, stage as i16),
            Self::Combat(stage) => (1, stage as i16),
            Self::Minimap(stage) => (2, stage as i16),
            Self::Mission(stage) => (3, stage as i16),
            Self::Infection(stage) => (4, stage as i16),
            Self::NanoPower(stage) => (5, stage as i16),
        }
    }
}

macro_rules! legacy_stage_enum {
    (
        $name:ident {
            $($variant:ident = $value:literal),+ $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        #[repr(i16)]
        pub enum $name {
            $($variant = $value),+
        }

        impl $name {
            pub const fn from_step(step: i16) -> Option<Self> {
                match step {
                    $($value => Some(Self::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

legacy_stage_enum!(MovementStage {
    AwaitIntro = 0,
    IntroCutscene = 1,
    PostIntro = 2,
    LookRight = 3,
    LookLeft = 4,
    LookUp = 5,
    LookDown = 6,
    MoveForward = 7,
    MoveBackward = 8,
    MoveAndSteer = 9,
    ReachLedge = 10,
    JumpAndLand = 11,
});

legacy_stage_enum!(CombatStage {
    AwaitIntro = 0,
    CombatIntro = 1,
    TargetHostile = 2,
    FirstAttack = 3,
    KillThree = 4,
    CyberusIntro = 5,
    FightCyberus = 6,
    PlayerWasHit = 7,
    PlanetFusionOutro = 8,
});

legacy_stage_enum!(MinimapStage {
    AwaitSequence = 0,
    MinimapSequence = 5,
    ApproachNumbuhTwo = 6,
});

legacy_stage_enum!(MissionStage {
    Approach = 0,
    TargetAndTalk = 1,
    MissionMenu = 3,
    ObjectiveCombat = 4,
    NumbuhTwoResponse = 5,
    OpenMenu = 6,
    SelectJournal = 7,
    JournalIntro = 8,
    JournalMissionTab = 9,
    JournalObjective = 10,
    JournalDetail = 11,
    CloseJournal = 12,
    ReturnToNumbuhTwo = 13,
    NearNumbuhTwo = 14,
    RewardMenu = 16,
    ClaimReward = 17,
    CloseReward = 18,
    Handoff = 19,
    NumbuhTwoDelay = 100,
    AcceptMission = 101,
    CloseAccept = 102,
});

legacy_stage_enum!(InfectionStage {
    ApproachButtercup = 0,
    TalkButtercup = 1,
    SelectMission = 2,
    InfectedFlythrough = 3,
    TravelToGate = 4,
    SelectAttendant = 5,
    WarpFromTechSquare = 6,
    OutsidePortal = 8,
    TalkDexter = 9,
    ApproachFusion = 10,
    FightFusion = 11,
    NanoCreation = 13,
    ExitPrompt = 15,
    WarpOut = 16,
    ButtercupDelay = 101,
    CloseMission = 102,
    SelectDexterMission = 109,
    SelectTentacles = 200,
    WarpIntoLair = 201,
    DexterCutscene = 210,
});

legacy_stage_enum!(NanoPowerStage {
    AwaitCollapse = 0,
    CollapseCutscene = 1,
    CrossBridge = 2,
    DemoMonsterCutscene = 3,
    SummonNano = 4,
    UseNanoPower = 6,
    ReturnToNumbuhTwo = 7,
    FinaleCutscene = 8,
    Exit = 9,
});

impl TutorialStage {
    /// Exact stage-local presentation and effective input permissions recovered
    /// from `cntutorialscript.Chapter_02` through `Chapter_07`.
    ///
    /// A few raw steps exist only between two statements in one legacy update
    /// (`Handoff`, for example). They conservatively expose no input rather than
    /// inventing a player-interactive state the original never had.
    pub const fn metadata(self) -> TutorialStageMetadata {
        match self {
            Self::Movement(stage) => match stage {
                MovementStage::AwaitIntro => stage_metadata(
                    None,
                    None,
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                MovementStage::IntroCutscene => stage_metadata(
                    None,
                    None,
                    TutorialScene::BasicMove,
                    TutorialInputPermissions::NONE,
                ),
                MovementStage::PostIntro => stage_metadata(
                    None,
                    Some("Computress_Tut02"),
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                MovementStage::LookRight => stage_metadata(
                    Some("Move your mouse to the right and find the marker."),
                    Some("Computress_Tut03"),
                    TutorialScene::None,
                    LOOK_INPUT,
                ),
                MovementStage::LookLeft => stage_metadata(
                    Some("Move your mouse to the left and find the marker."),
                    Some("Computress_Tut04"),
                    TutorialScene::None,
                    LOOK_INPUT,
                ),
                MovementStage::LookUp => stage_metadata(
                    Some("Push your mouse forward and find the marker."),
                    Some("Computress_Tut05"),
                    TutorialScene::None,
                    LOOK_INPUT,
                ),
                MovementStage::LookDown => stage_metadata(
                    Some("Pull your mouse back and find the marker."),
                    Some("Computress_Tut06"),
                    TutorialScene::None,
                    LOOK_INPUT,
                ),
                MovementStage::MoveForward => stage_metadata(
                    Some("Press and hold the \"W\" key to move forward."),
                    Some("Computress_Tut07"),
                    TutorialScene::None,
                    FORWARD_INPUT,
                ),
                MovementStage::MoveBackward => stage_metadata(
                    Some("Press the \"S\" key to move backwards."),
                    Some("Computress_Tut09"),
                    TutorialScene::None,
                    BACKWARD_INPUT,
                ),
                MovementStage::MoveAndSteer => stage_metadata(
                    Some(
                        "Press and hold the \"W\" key to move forward while steering with the mouse.",
                    ),
                    Some("Computress_Tut10"),
                    TutorialScene::None,
                    STEER_INPUT,
                ),
                MovementStage::ReachLedge => stage_metadata(
                    Some("Move to the marker near the ledge."),
                    Some("Computress_Tut11"),
                    TutorialScene::None,
                    WALK_INPUT,
                ),
                MovementStage::JumpAndLand => stage_metadata(
                    Some("Jump onto the ledge by pressing the Space Bar."),
                    Some("Computress_Tut12"),
                    TutorialScene::None,
                    MOVE_INPUT,
                ),
            },
            Self::Combat(stage) => match stage {
                CombatStage::AwaitIntro => stage_metadata(
                    None,
                    None,
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                CombatStage::CombatIntro => stage_metadata(
                    None,
                    None,
                    TutorialScene::BasicCombatA,
                    TutorialInputPermissions::NONE,
                ),
                CombatStage::TargetHostile => stage_metadata(
                    Some("Use your mouse to target a monster."),
                    Some("Computress_Tut13"),
                    TutorialScene::None,
                    COMBAT_INPUT,
                ),
                CombatStage::FirstAttack => stage_metadata(
                    Some(
                        "Click the Left Mouse button to attack. Hold the button down for continuous fire.",
                    ),
                    Some("Computress_Tut14"),
                    TutorialScene::None,
                    COMBAT_INPUT,
                ),
                CombatStage::KillThree => stage_metadata(
                    Some(
                        "Click the Left Mouse button to attack. Hold the button down for continuous fire.",
                    ),
                    Some("Computress_Tut15"),
                    TutorialScene::None,
                    COMBAT_INPUT,
                ),
                CombatStage::CyberusIntro => stage_metadata(
                    None,
                    None,
                    TutorialScene::BasicCombatB,
                    TutorialInputPermissions::NONE,
                ),
                CombatStage::FightCyberus => stage_metadata(
                    Some("Defeat the monster."),
                    None,
                    TutorialScene::None,
                    COMBAT_INPUT,
                ),
                CombatStage::PlayerWasHit => stage_metadata(
                    Some("Defeat the monster."),
                    Some("Computress_Tut18"),
                    TutorialScene::None,
                    COMBAT_INPUT,
                ),
                CombatStage::PlanetFusionOutro => stage_metadata(
                    None,
                    None,
                    TutorialScene::BasicCombatC,
                    TutorialInputPermissions::NONE,
                ),
            },
            Self::Minimap(stage) => match stage {
                MinimapStage::AwaitSequence => stage_metadata(
                    None,
                    None,
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                MinimapStage::MinimapSequence => stage_metadata(
                    Some("Go to Numbuh Two."),
                    Some("NumTwo_Tut01"),
                    TutorialScene::None,
                    COMBAT_INPUT,
                ),
                MinimapStage::ApproachNumbuhTwo => stage_metadata(
                    Some("Go to Numbuh Two."),
                    Some("Computress_Tut23"),
                    TutorialScene::None,
                    MOVE_INPUT,
                ),
            },
            Self::Mission(stage) => match stage {
                MissionStage::Approach => stage_metadata(
                    Some("Go to Numbuh Two."),
                    None,
                    TutorialScene::None,
                    MOVE_INPUT,
                ),
                MissionStage::TargetAndTalk => stage_metadata(
                    Some("Target Numbuh Two and click the Left Mouse button to talk."),
                    Some("Computress_Tut24"),
                    TutorialScene::None,
                    TALK_INPUT,
                ),
                MissionStage::MissionMenu => stage_metadata(
                    Some("Select \"Transmitter Critters\" from mission menu."),
                    Some("Computress_Tut25"),
                    TutorialScene::None,
                    TALK_INPUT,
                ),
                MissionStage::ObjectiveCombat => stage_metadata(
                    None,
                    Some("Computress_Tut29"),
                    TutorialScene::None,
                    COMBAT_MENU_INPUT,
                ),
                MissionStage::NumbuhTwoResponse => stage_metadata(
                    None,
                    Some("NumTwo_Tut03"),
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                MissionStage::OpenMenu => stage_metadata(
                    Some("Press \"Enter\" key to access the main menu. Then select \"Journal\"."),
                    Some("Computress_Tut30"),
                    TutorialScene::None,
                    MENU_JOURNAL_INPUT,
                ),
                MissionStage::SelectJournal => stage_metadata(
                    Some("Press \"Enter\" key to access the main menu. Then select \"Journal\"."),
                    None,
                    TutorialScene::None,
                    MENU_JOURNAL_INPUT,
                ),
                MissionStage::JournalIntro => stage_metadata(
                    None,
                    Some("Computress_Tut31"),
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                MissionStage::JournalMissionTab => stage_metadata(
                    None,
                    Some("Computress_Tut32"),
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                MissionStage::JournalObjective => stage_metadata(
                    None,
                    Some("Computress_Tut33"),
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                MissionStage::JournalDetail => stage_metadata(
                    None,
                    Some("Computress_Tut34"),
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                MissionStage::CloseJournal => stage_metadata(
                    None,
                    Some("Computress_Tut35"),
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                MissionStage::ReturnToNumbuhTwo => stage_metadata(
                    None,
                    Some("Computress_Tut36"),
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                MissionStage::NearNumbuhTwo => stage_metadata(
                    None,
                    Some("Computress_Tut37"),
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                MissionStage::RewardMenu => stage_metadata(
                    Some("Select \"Transmitter Critters\" from mission menu."),
                    Some("Computress_Tut38"),
                    TutorialScene::None,
                    NPC_MENU_INPUT,
                ),
                MissionStage::ClaimReward => stage_metadata(
                    None,
                    Some("Computress_Tut39"),
                    TutorialScene::None,
                    NPC_MENU_INPUT,
                ),
                MissionStage::CloseReward => stage_metadata(
                    Some("Click the \"Close\" button."),
                    Some("Computress_Tut40"),
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                MissionStage::Handoff => stage_metadata(
                    None,
                    None,
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                MissionStage::NumbuhTwoDelay => stage_metadata(
                    None,
                    Some("NumTwo_Tut02"),
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                MissionStage::AcceptMission => stage_metadata(
                    None,
                    Some("Computress_Tut27"),
                    TutorialScene::None,
                    TALK_INPUT,
                ),
                MissionStage::CloseAccept => stage_metadata(
                    Some("Click the \"Close\" button."),
                    Some("Computress_Tut28"),
                    TutorialScene::None,
                    TALK_CLOSE_INPUT,
                ),
            },
            Self::Infection(stage) => match stage {
                InfectionStage::ApproachButtercup => stage_metadata(
                    Some("Talk to Buttercup."),
                    None,
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                InfectionStage::TalkButtercup => stage_metadata(
                    Some("Talk to Buttercup."),
                    None,
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                InfectionStage::SelectMission => stage_metadata(
                    Some("Select \"A Fusion Matter\" from mission menu."),
                    Some("Computress_Tut44"),
                    TutorialScene::None,
                    NPC_MENU_INPUT,
                ),
                InfectionStage::InfectedFlythrough => stage_metadata(
                    None,
                    None,
                    TutorialScene::InfectionA,
                    TutorialInputPermissions::NONE,
                ),
                InfectionStage::TravelToGate => stage_metadata(
                    None,
                    Some("Computress_Tut46"),
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                InfectionStage::SelectAttendant => stage_metadata(
                    Some("Select the Dexbot attendant to use the warp gate."),
                    Some("Computress_Tut47"),
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                InfectionStage::WarpFromTechSquare => stage_metadata(
                    Some("Click the \"Warp\" button to enter Fusion Buttercup's lair."),
                    None,
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                InfectionStage::OutsidePortal => stage_metadata(
                    Some("Go to the Fusion Portal."),
                    Some("Dexter_Tut01"),
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                InfectionStage::TalkDexter => stage_metadata(
                    Some("Talk to Dexter."),
                    Some("Dexter_Tut02"),
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                InfectionStage::ApproachFusion => {
                    stage_metadata(None, None, TutorialScene::None, GENERAL_INPUT)
                }
                InfectionStage::FightFusion => {
                    stage_metadata(None, None, TutorialScene::None, GENERAL_INPUT)
                }
                InfectionStage::NanoCreation => stage_metadata(
                    None,
                    None,
                    TutorialScene::InfectionC,
                    TutorialInputPermissions::NONE,
                ),
                InfectionStage::ExitPrompt => stage_metadata(
                    Some("Use the warp gate to exit."),
                    Some("Computress_Tut56"),
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                InfectionStage::WarpOut => stage_metadata(
                    Some("Click the \"Warp\" button to exit the lair."),
                    None,
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                InfectionStage::ButtercupDelay => stage_metadata(
                    None,
                    Some("Btrcup_Tut07"),
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                InfectionStage::CloseMission => stage_metadata(
                    Some("Click the \"Close\" button."),
                    Some("Computress_Tut45"),
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                InfectionStage::SelectDexterMission => stage_metadata(
                    Some("Select \"A Fusion Matter\" from mission menu."),
                    Some("Computress_Tut53"),
                    TutorialScene::None,
                    NPC_MENU_INPUT,
                ),
                InfectionStage::SelectTentacles => stage_metadata(
                    Some("Select the tentacles to use the Fusion Portal."),
                    Some("Computress_Tut50"),
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                InfectionStage::WarpIntoLair => stage_metadata(
                    Some("Click the \"Warp\" button to enter Fusion Buttercup's lair."),
                    Some("Computress_Tut51"),
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                InfectionStage::DexterCutscene => stage_metadata(
                    None,
                    None,
                    TutorialScene::InfectionB,
                    TutorialInputPermissions::NONE,
                ),
            },
            Self::NanoPower(stage) => match stage {
                NanoPowerStage::AwaitCollapse => stage_metadata(
                    None,
                    None,
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
                NanoPowerStage::CollapseCutscene => stage_metadata(
                    None,
                    None,
                    TutorialScene::NanoPowerA,
                    TutorialInputPermissions::NONE,
                ),
                NanoPowerStage::CrossBridge => stage_metadata(
                    Some("Cross the bridge to escape."),
                    None,
                    TutorialScene::None,
                    GENERAL_INPUT,
                ),
                NanoPowerStage::DemoMonsterCutscene => stage_metadata(
                    None,
                    None,
                    TutorialScene::NanoPowerA2,
                    TutorialInputPermissions::NONE,
                ),
                NanoPowerStage::SummonNano => stage_metadata(
                    Some("Press \"1\" to summon your Buttercup Nano."),
                    Some("Computress_Tut59"),
                    TutorialScene::None,
                    NANO_SUMMON_INPUT,
                ),
                NanoPowerStage::UseNanoPower => stage_metadata(
                    Some("Stun the monster."),
                    Some("Computress_Tut60"),
                    TutorialScene::None,
                    NANO_COMBAT_INPUT,
                ),
                NanoPowerStage::ReturnToNumbuhTwo => stage_metadata(
                    Some("Talk to Numbuh Two."),
                    Some("Computress_Tut62"),
                    TutorialScene::None,
                    NANO_COMBAT_INPUT,
                ),
                NanoPowerStage::FinaleCutscene => stage_metadata(
                    None,
                    None,
                    TutorialScene::NanoPowerB,
                    TutorialInputPermissions::NONE,
                ),
                NanoPowerStage::Exit => stage_metadata(
                    None,
                    None,
                    TutorialScene::None,
                    TutorialInputPermissions::NONE,
                ),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TutorialProgress {
    chapter: u8,
    step: i16,
    event_flags: [i32; TUTORIAL_EVENT_COUNT],
    timer_seconds: u32,
}

impl Default for TutorialProgress {
    fn default() -> Self {
        Self {
            chapter: 0,
            step: 0,
            event_flags: [0; TUTORIAL_EVENT_COUNT],
            timer_seconds: 0,
        }
    }
}

impl TutorialProgress {
    pub const fn chapter(&self) -> u8 {
        self.chapter
    }

    pub const fn step(&self) -> i16 {
        self.step
    }

    pub fn stage(&self) -> Option<TutorialStage> {
        TutorialStage::from_legacy(self.chapter, self.step)
    }

    /// Exact `InitStep`: write `pFlag[1]`, clear all event flags and restart
    /// presentation timing. Position/rotation snapshots are owned by the Bevy
    /// runtime because this engine-neutral type has no transform dependency.
    pub fn init_step(&mut self, step: i16) {
        self.step = step;
        self.event_flags.fill(0);
        self.timer_seconds = 0;
    }

    /// Direct legacy `pFlag[1] = step` write.
    ///
    /// `Chapter_03` uses this for the `FightCyberus -> PlayerWasHit`
    /// transition, intentionally preserving the current event flags and
    /// reminder timer instead of applying `InitStep`.
    pub fn write_step_preserving_state(&mut self, step: i16) {
        self.step = step;
    }

    /// Clear only the transient `RecieveEvent` slots.
    ///
    /// This represents the event portion of the legacy `ClearVar` call when
    /// the native runtime has already applied the accompanying chapter reset.
    pub fn clear_event_flags(&mut self) {
        self.event_flags.fill(0);
    }

    /// Invalidate a stale observation without discarding other events that
    /// arrived in the same frame or restarting the current stage's timer.
    pub fn clear_event(&mut self, event: TutorialEvent) {
        self.event_flags[event.index()] = 0;
    }

    /// Exact `InitChapter`: select `arrayFunc[chapter]`, reset step/event state
    /// and stop the old presentation timer.
    pub fn init_chapter(&mut self, chapter: u8) -> Result<(), u8> {
        if chapter >= TUTORIAL_CHAPTER_COUNT {
            return Err(chapter);
        }
        self.chapter = chapter;
        self.init_step(0);
        Ok(())
    }

    pub fn set_timer_seconds(&mut self, seconds: u32) {
        self.timer_seconds = seconds;
    }

    pub const fn timer_seconds(&self) -> u32 {
        self.timer_seconds
    }

    /// Exact `RecieveEvent` storage rules. Movement keeps the source direction,
    /// NPC death increments, and every other event is a one-bit observation.
    pub fn receive_event(&mut self, event: TutorialEvent, value: i32) {
        let slot = &mut self.event_flags[event.index()];
        match event {
            TutorialEvent::Move => *slot = value,
            TutorialEvent::DeadNpc => *slot = slot.saturating_add(1),
            _ => *slot = 1,
        }
    }

    pub const fn event_value(&self, event: TutorialEvent) -> i32 {
        self.event_flags[event.index()]
    }
}

#[cfg(test)]
mod tests;
