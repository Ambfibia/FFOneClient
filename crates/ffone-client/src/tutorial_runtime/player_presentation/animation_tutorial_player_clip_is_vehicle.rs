use super::*;

impl TutorialPlayerClip {
    pub const fn is_vehicle(self) -> bool { matches!(self, Self::BoardStand1 | Self::BoardRun | Self::BoardRunBack | Self::BoardJumpStart | Self::BoardJump | Self::BoardJumpEnd | Self::BoardJumpLandRun | Self::ScooterStand1 | Self::ScooterRun | Self::ScooterRunBack | Self::ScooterJumpStart | Self::ScooterJump | Self::ScooterJumpEnd | Self::ScooterJumpLandRun) }

    pub const ALL: [Self; 141] = [
        Self::WoundUpper,
        Self::Stun,
        Self::StickDash,
        Self::RifleDash,
        Self::RifleTumbling,
        Self::RocketSomersault,
        Self::StickDodgeUpper,
        Self::RifleDodgeUpper,

        Self::Stand1,
        Self::Staying,
        Self::Standup,
        Self::Run,
        Self::RunBack,
        Self::JumpStart,
        Self::Jump,
        Self::JumpEnd,
        Self::JumpLandRun,
        Self::Die,
        Self::Death,
        Self::Slide,
        Self::RopeDown,
        Self::RopeDrop,
        Self::RopeLeft,
        Self::RopeRight,
        Self::RopeStand1,
        Self::RopeStand2,
        Self::RopeTurn,
        Self::RopeUp,
        Self::Mount1,
        Self::Mount2,
        Self::Inventory,
        Self::BoardStand1,
        Self::BoardRun,
        Self::BoardRunBack,
        Self::BoardJumpStart,
        Self::BoardJump,
        Self::BoardJumpEnd,
        Self::BoardJumpLandRun,
        Self::ScooterStand1,
        Self::ScooterRun,
        Self::ScooterRunBack,
        Self::ScooterJumpStart,
        Self::ScooterJump,
        Self::ScooterJumpEnd,
        Self::ScooterJumpLandRun,
        Self::BoardInventory,
        Self::ScooterInventory,
        Self::StickStand1,
        Self::StickReady,
        Self::StickRun,
        Self::StickRunBack,
        Self::StickJumpStart,
        Self::StickJump,
        Self::StickJumpEnd,
        Self::StickJumpLandRun,
        Self::PistolStand1,
        Self::PistolReady,
        Self::PistolRun,
        Self::PistolRunBack,
        Self::PistolJumpStart,
        Self::PistolJump,
        Self::PistolJumpEnd,
        Self::PistolJumpLandRun,
        Self::RifleStand1,
        Self::RifleReady,
        Self::Attack1,
        Self::Attack1Upper,
        Self::StickAttack1,
        Self::StickAttack1Upper,
        Self::PistolAttack1,
        Self::PistolAttack1Upper,
        Self::RifleAttack1,
        Self::RifleAttack1Upper,
        Self::RifleRun,
        Self::RifleRunBack,
        Self::RifleJumpStart,
        Self::RifleJump,
        Self::RifleJumpEnd,
        Self::RifleJumpLandRun,
        Self::BombStand1,
        Self::BombReady,
        Self::BombAttack1,
        Self::BombAttack1Upper,
        Self::BombRun,
        Self::BombRunBack,
        Self::BombJumpStart,
        Self::BombJump,
        Self::BombJumpEnd,
        Self::BombJumpLandRun,
        Self::RocketStand1,
        Self::RocketReady,
        Self::RocketAttack1,
        Self::RocketAttack1Upper,
        Self::RocketRun,
        Self::RocketRunBack,
        Self::RocketJumpStart,
        Self::RocketJump,
        Self::RocketJumpEnd,
        Self::RocketJumpLandRun,
        Self::Swim,
        Self::SwimBack,
        Self::SwimIdle,
        Self::SwimLeft,
        Self::SwimRight,
        Self::Cry,
        Self::Angry,
        Self::Shocked,
        Self::Hello,
        Self::Thank,
        Self::Dance1,
        Self::Kiss,
        Self::Agree,
        Self::Laugh,
        Self::No,
        Self::Flex,
        Self::Tease,
        Self::Ok,
        Self::Applaud,
        Self::Cheer,
        Self::Dance2,
        Self::Dance3,
        Self::Dance4,
        Self::Dance5,
        Self::Goodbye,
        Self::Beach1,
        Self::Beach2,
        Self::Beach3,
        Self::FfrDance02,
        Self::FfrDance04,
        Self::FfrDance07,
        Self::FfrDance08,
        Self::FfrDance10,
        Self::FfrDance13,
        Self::FfrDance15,
        Self::FfrDance18,
        Self::FfrDance19,
        Self::FfrDance20,
        Self::FfrDanceBully,
        Self::FfrDanceTellMe,
        Self::FfrEmoteCatPose,
        Self::FfrEmoteIdolPose,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::WoundUpper => "woundupper",
            Self::Stun => "stun",
            Self::StickDash => "stickdash",
            Self::RifleDash => "rifledash",
            Self::RifleTumbling => "rifletumbling",
            Self::RocketSomersault => "rocketsomersault",
            Self::StickDodgeUpper => "stickdodgeupper",
            Self::RifleDodgeUpper => "rifledodgeupper",
            Self::Stand1 => "stand1",
            Self::Staying => "staying",
            Self::Standup => "standup",
            Self::Run => "run",
            Self::RunBack => "runback",
            Self::JumpStart => "jumpstart",
            Self::Jump => "jump",
            Self::JumpEnd => "jumpend",
            Self::JumpLandRun => "jumplandrun",
            Self::Die => "die",
            Self::Death => "death",
            Self::Slide => "slide",
            Self::RopeDown => "ropedown",
            Self::RopeDrop => "ropedrop",
            Self::RopeLeft => "ropeleft",
            Self::RopeRight => "roperight",
            Self::RopeStand1 => "ropestand1",
            Self::RopeStand2 => "ropestand2",
            Self::RopeTurn => "ropeturn",
            Self::RopeUp => "ropeup",
            Self::Mount1 => "mount1",
            Self::Mount2 => "mount2",
            Self::Inventory => "inven",
            Self::BoardStand1 => "board_stand1",
            Self::BoardRun => "board_run",
            Self::BoardRunBack => "board_runback",
            Self::BoardJumpStart => "board_jumpstart",
            Self::BoardJump => "board_jump",
            Self::BoardJumpEnd => "board_jumpend",
            Self::BoardJumpLandRun => "board_jumplandrun",
            Self::ScooterStand1 => "scooter_stand1",
            Self::ScooterRun => "scooter_run",
            Self::ScooterRunBack => "scooter_runback",
            Self::ScooterJumpStart => "scooter_jumpstart",
            Self::ScooterJump => "scooter_jump",
            Self::ScooterJumpEnd => "scooter_jumpend",
            Self::ScooterJumpLandRun => "scooter_jumplandrun",
            Self::BoardInventory => "board_inven",
            Self::ScooterInventory => "scooter_inven",
            Self::StickStand1 => "stickstand1",
            Self::StickReady => "stickready",
            Self::StickRun => "stickrun",
            Self::StickRunBack => "stickrunback",
            Self::StickJumpStart => "stickjumpstart",
            Self::StickJump => "stickjump",
            Self::StickJumpEnd => "stickjumpend",
            Self::StickJumpLandRun => "stickjumplandrun",
            Self::PistolStand1 => "pistolstand1",
            Self::PistolReady => "pistolready",
            Self::PistolRun => "pistolrun",
            Self::PistolRunBack => "pistolrunback",
            Self::PistolJumpStart => "pistoljumpstart",
            Self::PistolJump => "pistoljump",
            Self::PistolJumpEnd => "pistoljumpend",
            Self::PistolJumpLandRun => "pistoljumplandrun",
            Self::RifleStand1 => "riflestand1",
            Self::RifleReady => "rifleready",
            Self::Attack1 => "attack1",
            Self::Attack1Upper => "attack1upper",
            Self::StickAttack1 => "stickattack1",
            Self::StickAttack1Upper => "stickattack1upper",
            Self::PistolAttack1 => "pistolattack1",
            Self::PistolAttack1Upper => "pistolattack1upper",
            Self::RifleAttack1 => "rifleattack1",
            Self::RifleAttack1Upper => "rifleattack1upper",
            Self::RifleRun => "riflerun",
            Self::RifleRunBack => "riflerunback",
            Self::RifleJumpStart => "riflejumpstart",
            Self::RifleJump => "riflejump",
            Self::RifleJumpEnd => "riflejumpend",
            Self::RifleJumpLandRun => "riflejumplandrun",
            Self::BombStand1 => "bombstand1",
            Self::BombReady => "bombready",
            Self::BombAttack1 => "bombattack1",
            Self::BombAttack1Upper => "bombattack1upper",
            Self::BombRun => "bombrun",
            Self::BombRunBack => "bombrunback",
            Self::BombJumpStart => "bombjumpstart",
            Self::BombJump => "bombjump",
            Self::BombJumpEnd => "bombjumpend",
            Self::BombJumpLandRun => "bombjumplandrun",
            Self::RocketStand1 => "rocketstand1",
            Self::RocketReady => "rocketready",
            Self::RocketAttack1 => "rocketattack1",
            Self::RocketAttack1Upper => "rocketattack1upper",
            Self::RocketRun => "rocketrun",
            Self::RocketRunBack => "rocketrunback",
            Self::RocketJumpStart => "rocketjumpstart",
            Self::RocketJump => "rocketjump",
            Self::RocketJumpEnd => "rocketjumpend",
            Self::RocketJumpLandRun => "rocketjumplandrun",
            Self::Swim => "swim",
            Self::SwimBack => "swimback",
            Self::SwimIdle => "swimidle",
            Self::SwimLeft => "swimleft",
            Self::SwimRight => "swimright",
            Self::Cry => "cry",
            Self::Angry => "angry",
            Self::Shocked => "shocked",
            Self::Hello => "hello",
            Self::Thank => "thank",
            Self::Dance1 => "dance1",
            Self::Kiss => "kiss",
            Self::Agree => "agree",
            Self::Laugh => "laugh",
            Self::No => "no",
            Self::Flex => "flex",
            Self::Tease => "tease",
            Self::Ok => "ok",
            Self::Applaud => "applaud",
            Self::Cheer => "cheer",
            Self::Dance2 => "dance2",
            Self::Dance3 => "dance3",
            Self::Dance4 => "dance4",
            Self::Dance5 => "dance5",
            Self::Goodbye => "goodbye",
            Self::Beach1 => "beach1",
            Self::Beach2 => "beach2",
            Self::Beach3 => "beach3",
            Self::FfrDance02 => "ffr_dance_02",
            Self::FfrDance04 => "ffr_dance_04",
            Self::FfrDance07 => "ffr_dance_07",
            Self::FfrDance08 => "ffr_dance_08",
            Self::FfrDance10 => "ffr_dance_10",
            Self::FfrDance13 => "ffr_dance_13",
            Self::FfrDance15 => "ffr_dance_15",
            Self::FfrDance18 => "ffr_dance_18",
            Self::FfrDance19 => "ffr_dance_19",
            Self::FfrDance20 => "ffr_dance_20",
            Self::FfrDanceBully => "ffr_dance_bully",
            Self::FfrDanceTellMe => "ffr_dance_tellme",
            Self::FfrEmoteCatPose => "ffr_emote_catpose",
            Self::FfrEmoteIdolPose => "ffr_emote_idolpose",
        }
    }

    /// Exact AnimationClip PathID in `actor/m.kfm` or `actor/w.kfm`.
    #[must_use]
    pub const fn source_path_id(self, gender: PlayerRigGender) -> i64 {
        match (gender, self) {
            // Native clip names own these additions; provenance remains in FusionForge.
            (_, Self::WoundUpper | Self::Stun | Self::StickDash | Self::RifleDash | Self::RifleTumbling | Self::RocketSomersault | Self::StickDodgeUpper | Self::RifleDodgeUpper) => 0,
            (_, Self::BoardStand1 | Self::BoardRun | Self::BoardRunBack | Self::BoardJumpStart | Self::BoardJump | Self::BoardJumpEnd | Self::BoardJumpLandRun | Self::ScooterStand1 | Self::ScooterRun | Self::ScooterRunBack | Self::ScooterJumpStart | Self::ScooterJump | Self::ScooterJumpEnd | Self::ScooterJumpLandRun) => 0,
            (PlayerRigGender::Male, Self::Stand1) => 237_023,
            (PlayerRigGender::Male, Self::Run) => 34_268,
            (PlayerRigGender::Male, Self::Staying) => 34_269,
            (PlayerRigGender::Male, Self::Standup) => 34_404,
            (PlayerRigGender::Male, Self::RunBack) => 34_249,
            (PlayerRigGender::Male, Self::JumpStart) => 34_390,
            (PlayerRigGender::Male, Self::Jump) => 34_314,
            (PlayerRigGender::Male, Self::JumpEnd) => 34_224,
            (PlayerRigGender::Male, Self::JumpLandRun) => 34_256,
            // Primary CharacterSelection.resourceFile, actor/m.kfm preload
            // ownership and AnimationClip object identities.
            (PlayerRigGender::Male, Self::Die) => 34_531,
            (PlayerRigGender::Male, Self::Death) => 34_462,
            (PlayerRigGender::Male, Self::Slide) => 34_408,
            (PlayerRigGender::Male, Self::RopeDown) => 34_214,
            (PlayerRigGender::Male, Self::RopeDrop) => 34_348,
            (PlayerRigGender::Male, Self::RopeLeft) => 34_334,
            (PlayerRigGender::Male, Self::RopeRight) => 34_350,
            (PlayerRigGender::Male, Self::RopeStand1) => 34_361,
            (PlayerRigGender::Male, Self::RopeStand2) => 34_327,
            (PlayerRigGender::Male, Self::RopeTurn) => 34_606,
            (PlayerRigGender::Male, Self::RopeUp) => 34_335,
            (PlayerRigGender::Male, Self::Mount1) => 34_330,
            (PlayerRigGender::Male, Self::Mount2) => 34_338,
            (PlayerRigGender::Male, Self::Inventory) => 34_276,
            (PlayerRigGender::Male, Self::BoardInventory) => 34_399,
            (PlayerRigGender::Male, Self::ScooterInventory) => 34_583,
            (PlayerRigGender::Male, Self::StickStand1) => 34_393,
            (PlayerRigGender::Male, Self::StickReady) => 34_220,
            (PlayerRigGender::Male, Self::StickRun) => 34_208,
            (PlayerRigGender::Male, Self::StickRunBack) => 34_252,
            (PlayerRigGender::Male, Self::StickJumpStart) => 34_233,
            (PlayerRigGender::Male, Self::StickJump) => 34_320,
            (PlayerRigGender::Male, Self::StickJumpEnd) => 34_312,
            (PlayerRigGender::Male, Self::StickJumpLandRun) => 34_300,
            (PlayerRigGender::Male, Self::PistolStand1) => 34_299,
            (PlayerRigGender::Male, Self::PistolReady) => 34_255,
            (PlayerRigGender::Male, Self::PistolRun) => 34_374,
            (PlayerRigGender::Male, Self::PistolRunBack) => 34_349,
            (PlayerRigGender::Male, Self::PistolJumpStart) => 34_212,
            (PlayerRigGender::Male, Self::PistolJump) => 34_302,
            (PlayerRigGender::Male, Self::PistolJumpEnd) => 34_234,
            (PlayerRigGender::Male, Self::PistolJumpLandRun) => 34_221,
            (PlayerRigGender::Male, Self::RifleStand1) => 34_381,
            (PlayerRigGender::Male, Self::RifleReady) => 34_367,
            (PlayerRigGender::Male, Self::Attack1) => 34_235,
            (PlayerRigGender::Male, Self::Attack1Upper) => 34_213,
            (PlayerRigGender::Male, Self::StickAttack1) => 34_332,
            (PlayerRigGender::Male, Self::StickAttack1Upper) => 34_412,
            (PlayerRigGender::Male, Self::PistolAttack1) => 34_380,
            (PlayerRigGender::Male, Self::PistolAttack1Upper) => 34_240,
            (PlayerRigGender::Male, Self::RifleAttack1) => 34_555,
            (PlayerRigGender::Male, Self::RifleAttack1Upper) => 34_305,
            (PlayerRigGender::Male, Self::RifleRun) => 34_321,
            (PlayerRigGender::Male, Self::RifleRunBack) => 34_331,
            (PlayerRigGender::Male, Self::RifleJumpStart) => 34_354,
            (PlayerRigGender::Male, Self::RifleJump) => 34_387,
            (PlayerRigGender::Male, Self::RifleJumpEnd) => 34_415,
            (PlayerRigGender::Male, Self::RifleJumpLandRun) => 34_365,
            (PlayerRigGender::Male, Self::BombStand1) => 34_401,
            (PlayerRigGender::Male, Self::BombReady) => 34_226,
            (PlayerRigGender::Male, Self::BombAttack1) => 34_297,
            (PlayerRigGender::Male, Self::BombAttack1Upper) => 34_436,
            (PlayerRigGender::Male, Self::BombRun) => 34_262,
            (PlayerRigGender::Male, Self::BombRunBack) => 237_032,
            (PlayerRigGender::Male, Self::BombJumpStart) => 34_267,
            (PlayerRigGender::Male, Self::BombJump) => 34_560,
            (PlayerRigGender::Male, Self::BombJumpEnd) => 34_599,
            (PlayerRigGender::Male, Self::BombJumpLandRun) => 34_288,
            (PlayerRigGender::Male, Self::RocketStand1) => 34_624,
            (PlayerRigGender::Male, Self::RocketReady) => 34_521,
            (PlayerRigGender::Male, Self::RocketAttack1) => 34_301,
            (PlayerRigGender::Male, Self::RocketAttack1Upper) => 34_210,
            (PlayerRigGender::Male, Self::RocketRun) => 34_534,
            (PlayerRigGender::Male, Self::RocketRunBack) => 34_518,
            (PlayerRigGender::Male, Self::RocketJumpStart) => 34_441,
            (PlayerRigGender::Male, Self::RocketJump) => 34_422,
            (PlayerRigGender::Male, Self::RocketJumpEnd) => 34_303,
            (PlayerRigGender::Male, Self::RocketJumpLandRun) => 34_484,
            (PlayerRigGender::Male, Self::Swim) => 34_486,
            (PlayerRigGender::Male, Self::SwimBack) => 34_453,
            (PlayerRigGender::Male, Self::SwimIdle) => 34_524,
            (PlayerRigGender::Male, Self::SwimLeft) => 34_535,
            (PlayerRigGender::Male, Self::SwimRight) => 34_514,
            (PlayerRigGender::Female, Self::Stand1) => 237_027,
            (PlayerRigGender::Female, Self::Run) => 34_191,
            (PlayerRigGender::Female, Self::Staying) => 34_194,
            (PlayerRigGender::Female, Self::Standup) => 34_533,
            (PlayerRigGender::Female, Self::RunBack) => 34_611,
            (PlayerRigGender::Female, Self::JumpStart) => 34_557,
            (PlayerRigGender::Female, Self::Jump) => 34_605,
            (PlayerRigGender::Female, Self::JumpEnd) => 34_626,
            (PlayerRigGender::Female, Self::JumpLandRun) => 34_567,
            // Primary CharacterSelection.resourceFile, actor/w.kfm preload
            // ownership and AnimationClip object identities.
            (PlayerRigGender::Female, Self::Die) => 34_444,
            (PlayerRigGender::Female, Self::Death) => 34_537,
            (PlayerRigGender::Female, Self::Slide) => 34_572,
            (PlayerRigGender::Female, Self::RopeDown) => 34_618,
            (PlayerRigGender::Female, Self::RopeDrop) => 34_190,
            (PlayerRigGender::Female, Self::RopeLeft) => 34_169,
            (PlayerRigGender::Female, Self::RopeRight) => 34_162,
            (PlayerRigGender::Female, Self::RopeStand1) => 34_152,
            (PlayerRigGender::Female, Self::RopeStand2) => 34_204,
            (PlayerRigGender::Female, Self::RopeTurn) => 34_507,
            (PlayerRigGender::Female, Self::RopeUp) => 34_193,
            (PlayerRigGender::Female, Self::Mount1) => 34_280,
            (PlayerRigGender::Female, Self::Mount2) => 34_294,
            (PlayerRigGender::Female, Self::Inventory) => 34_614,
            (PlayerRigGender::Female, Self::BoardInventory) => 34_413,
            (PlayerRigGender::Female, Self::ScooterInventory) => 34_435,
            (PlayerRigGender::Female, Self::StickStand1) => 34_457,
            (PlayerRigGender::Female, Self::StickReady) => 34_155,
            (PlayerRigGender::Female, Self::StickRun) => 34_198,
            (PlayerRigGender::Female, Self::StickRunBack) => 34_188,
            (PlayerRigGender::Female, Self::StickJumpStart) => 34_523,
            (PlayerRigGender::Female, Self::StickJump) => 34_278,
            (PlayerRigGender::Female, Self::StickJumpEnd) => 34_474,
            (PlayerRigGender::Female, Self::StickJumpLandRun) => 34_451,
            (PlayerRigGender::Female, Self::PistolStand1) => 34_175,
            (PlayerRigGender::Female, Self::PistolReady) => 34_170,
            (PlayerRigGender::Female, Self::PistolRun) => 34_151,
            (PlayerRigGender::Female, Self::PistolRunBack) => 34_147,
            (PlayerRigGender::Female, Self::PistolJumpStart) => 34_143,
            (PlayerRigGender::Female, Self::PistolJump) => 34_527,
            (PlayerRigGender::Female, Self::PistolJumpEnd) => 34_166,
            (PlayerRigGender::Female, Self::PistolJumpLandRun) => 34_200,
            (PlayerRigGender::Female, Self::RifleStand1) => 34_382,
            (PlayerRigGender::Female, Self::RifleReady) => 34_595,
            (PlayerRigGender::Female, Self::Attack1) => 34_171,
            (PlayerRigGender::Female, Self::Attack1Upper) => 34_174,
            (PlayerRigGender::Female, Self::StickAttack1) => 34_579,
            (PlayerRigGender::Female, Self::StickAttack1Upper) => 34_315,
            (PlayerRigGender::Female, Self::PistolAttack1) => 34_414,
            (PlayerRigGender::Female, Self::PistolAttack1Upper) => 34_286,
            (PlayerRigGender::Female, Self::RifleAttack1) => 34_445,
            (PlayerRigGender::Female, Self::RifleAttack1Upper) => 34_466,
            (PlayerRigGender::Female, Self::RifleRun) => 34_585,
            (PlayerRigGender::Female, Self::RifleRunBack) => 34_578,
            (PlayerRigGender::Female, Self::RifleJumpStart) => 34_603,
            (PlayerRigGender::Female, Self::RifleJump) => 34_627,
            (PlayerRigGender::Female, Self::RifleJumpEnd) => 34_571,
            (PlayerRigGender::Female, Self::RifleJumpLandRun) => 34_559,
            (PlayerRigGender::Female, Self::BombStand1) => 34_593,
            (PlayerRigGender::Female, Self::BombReady) => 34_565,
            (PlayerRigGender::Female, Self::BombAttack1) => 34_500,
            (PlayerRigGender::Female, Self::BombAttack1Upper) => 34_542,
            (PlayerRigGender::Female, Self::BombRun) => 34_556,
            (PlayerRigGender::Female, Self::BombRunBack) => 237_031,
            (PlayerRigGender::Female, Self::BombJumpStart) => 34_630,
            (PlayerRigGender::Female, Self::BombJump) => 34_530,
            (PlayerRigGender::Female, Self::BombJumpEnd) => 34_613,
            (PlayerRigGender::Female, Self::BombJumpLandRun) => 34_633,
            (PlayerRigGender::Female, Self::RocketStand1) => 34_546,
            (PlayerRigGender::Female, Self::RocketReady) => 34_511,
            (PlayerRigGender::Female, Self::RocketAttack1) => 34_452,
            (PlayerRigGender::Female, Self::RocketAttack1Upper) => 34_199,
            (PlayerRigGender::Female, Self::RocketRun) => 34_502,
            (PlayerRigGender::Female, Self::RocketRunBack) => 34_497,
            (PlayerRigGender::Female, Self::RocketJumpStart) => 34_163,
            (PlayerRigGender::Female, Self::RocketJump) => 34_154,
            (PlayerRigGender::Female, Self::RocketJumpEnd) => 34_164,
            (PlayerRigGender::Female, Self::RocketJumpLandRun) => 34_187,
            (PlayerRigGender::Female, Self::Swim) => 34_185,
            (PlayerRigGender::Female, Self::SwimBack) => 34_165,
            (PlayerRigGender::Female, Self::SwimIdle) => 34_512,
            (PlayerRigGender::Female, Self::SwimLeft) => 34_503,
            (PlayerRigGender::Female, Self::SwimRight) => 34_498,
            (PlayerRigGender::Male, Self::Cry) => 34_471,
            (PlayerRigGender::Female, Self::Cry) => 34_494,
            (PlayerRigGender::Male, Self::Angry) => 34_304,
            (PlayerRigGender::Female, Self::Angry) => 34_161,
            (PlayerRigGender::Male, Self::Shocked) => 34_383,
            (PlayerRigGender::Female, Self::Shocked) => 34_629,
            (PlayerRigGender::Male, Self::Hello) => 34_598,
            (PlayerRigGender::Female, Self::Hello) => 34_192,
            (PlayerRigGender::Male, Self::Thank) => 34_362,
            (PlayerRigGender::Female, Self::Thank) => 34_476,
            (PlayerRigGender::Male, Self::Dance1) => 34_566,
            (PlayerRigGender::Female, Self::Dance1) => 34_283,
            (PlayerRigGender::Male, Self::Kiss) => 34_427,
            (PlayerRigGender::Female, Self::Kiss) => 34_602,
            (PlayerRigGender::Male, Self::Agree) => 34_319,
            (PlayerRigGender::Female, Self::Agree) => 34_201,
            (PlayerRigGender::Male, Self::Laugh) => 34_353,
            (PlayerRigGender::Female, Self::Laugh) => 34_377,
            (PlayerRigGender::Male, Self::No) => 34_410,
            (PlayerRigGender::Female, Self::No) => 34_250,
            (PlayerRigGender::Male, Self::Flex) => 34_558,
            (PlayerRigGender::Female, Self::Flex) => 34_218,
            (PlayerRigGender::Male, Self::Tease) => 34_623,
            (PlayerRigGender::Female, Self::Tease) => 34_547,
            (PlayerRigGender::Male, Self::Ok) => 34_423,
            (PlayerRigGender::Female, Self::Ok) => 34_431,
            (PlayerRigGender::Male, Self::Applaud) => 34_217,
            (PlayerRigGender::Female, Self::Applaud) => 34_149,
            (PlayerRigGender::Male, Self::Cheer) => 34_405,
            (PlayerRigGender::Female, Self::Cheer) => 34_515,
            (PlayerRigGender::Male, Self::Dance2) => 34_285,
            (PlayerRigGender::Female, Self::Dance2) => 34_430,
            (PlayerRigGender::Male, Self::Dance3) => 34_225,
            (PlayerRigGender::Female, Self::Dance3) => 34_536,
            (PlayerRigGender::Male, Self::Dance4) => 34_635,
            (PlayerRigGender::Female, Self::Dance4) => 34_634,
            (PlayerRigGender::Male, Self::Dance5) => 34_316,
            (PlayerRigGender::Female, Self::Dance5) => 34_628,
            (PlayerRigGender::Male, Self::Goodbye) => 34_310,
            (PlayerRigGender::Female, Self::Goodbye) => 34_575,
            (PlayerRigGender::Male, Self::Beach1) => 34_211,
            (PlayerRigGender::Female, Self::Beach1) => 34_573,
            (PlayerRigGender::Male, Self::Beach2) => 34_402,
            (PlayerRigGender::Female, Self::Beach2) => 34_563,
            (PlayerRigGender::Male, Self::Beach3) => 34_433,
            (PlayerRigGender::Female, Self::Beach3) => 34_552,
            (_, Self::FfrDance02) => 644,
            (_, Self::FfrDance04) => 624,
            (_, Self::FfrDance07) => 641,
            (_, Self::FfrDance08) => 629,
            (_, Self::FfrDance10) => 607,
            (_, Self::FfrDance13) => 604,
            (_, Self::FfrDance15) => 610,
            (_, Self::FfrDance18) => 646,
            (_, Self::FfrDance19) => 637,
            (_, Self::FfrDance20) => 585,
            (PlayerRigGender::Male, Self::FfrDanceBully) => 642,
            (PlayerRigGender::Female, Self::FfrDanceBully) => 645,
            (_, Self::FfrDanceTellMe) => 648,
            (_, Self::FfrEmoteCatPose) => 639,
            (_, Self::FfrEmoteIdolPose) => 623,
        }
    }

    #[must_use]
    pub const fn playback(self) -> TutorialPlayerClipPlayback {
        match self {
            Self::Stand1
            | Self::Death
            | Self::Run
            | Self::RunBack
            | Self::Jump
            | Self::Slide
            | Self::RopeDown
            | Self::RopeDrop
            | Self::RopeLeft
            | Self::RopeRight
            | Self::RopeStand1
            | Self::RopeStand2
            | Self::RopeTurn
            | Self::RopeUp
            | Self::Mount1
            | Self::Mount2
            | Self::Inventory
            | Self::BoardStand1
            | Self::BoardRun
            | Self::BoardRunBack
            | Self::BoardJump
            | Self::ScooterStand1
            | Self::ScooterRun
            | Self::ScooterRunBack
            | Self::ScooterJump
            | Self::BoardInventory
            | Self::ScooterInventory
            | Self::StickStand1
            | Self::StickReady
            | Self::StickRun
            | Self::StickRunBack
            | Self::StickJump
            | Self::PistolStand1
            | Self::PistolReady
            | Self::PistolRun
            | Self::PistolRunBack
            | Self::PistolJump
            | Self::RifleStand1
            | Self::RifleReady
            | Self::RifleRun
            | Self::RifleRunBack
            | Self::RifleJump
            | Self::BombStand1
            | Self::BombReady
            | Self::BombRun
            | Self::BombRunBack
            | Self::BombJump
            | Self::RocketStand1
            | Self::RocketReady
            | Self::RocketRun
            | Self::RocketRunBack
            | Self::RocketJump
            | Self::Swim
            | Self::SwimBack
            | Self::SwimIdle
            | Self::SwimLeft
            | Self::SwimRight => TutorialPlayerClipPlayback::Loop,
            Self::WoundUpper | Self::Stun | Self::StickDash | Self::RifleDash | Self::RifleTumbling | Self::RocketSomersault | Self::StickDodgeUpper | Self::RifleDodgeUpper
            | Self::BoardJumpStart | Self::BoardJumpEnd | Self::BoardJumpLandRun | Self::ScooterJumpStart | Self::ScooterJumpEnd | Self::ScooterJumpLandRun
            | Self::Staying
            | Self::Standup
            | Self::Die
            | Self::JumpStart
            | Self::JumpEnd
            | Self::JumpLandRun
            | Self::StickJumpStart
            | Self::StickJumpEnd
            | Self::StickJumpLandRun
            | Self::PistolJumpStart
            | Self::PistolJumpEnd
            | Self::PistolJumpLandRun
            | Self::Attack1
            | Self::Attack1Upper
            | Self::StickAttack1
            | Self::StickAttack1Upper
            | Self::PistolAttack1
            | Self::PistolAttack1Upper
            | Self::RifleAttack1
            | Self::RifleAttack1Upper
            | Self::RifleJumpStart
            | Self::RifleJumpEnd
            | Self::RifleJumpLandRun
            | Self::BombAttack1
            | Self::BombAttack1Upper
            | Self::BombJumpStart
            | Self::BombJumpEnd
            | Self::BombJumpLandRun
            | Self::RocketAttack1
            | Self::RocketAttack1Upper
            | Self::RocketJumpStart
            | Self::RocketJumpEnd
            | Self::RocketJumpLandRun
            | Self::Cry
            | Self::Angry
            | Self::Shocked
            | Self::Hello
            | Self::Thank
            | Self::Dance1
            | Self::Kiss
            | Self::Agree
            | Self::Laugh
            | Self::No
            | Self::Flex
            | Self::Tease
            | Self::Ok
            | Self::Applaud
            | Self::Cheer
            | Self::Dance2
            | Self::Dance3
            | Self::Dance4
            | Self::Dance5
            | Self::Goodbye
            | Self::Beach1
            | Self::Beach2
            | Self::Beach3
            | Self::FfrDance02
            | Self::FfrDance04
            | Self::FfrDance07
            | Self::FfrDance08
            | Self::FfrDance10
            | Self::FfrDance13
            | Self::FfrDance15
            | Self::FfrDance18
            | Self::FfrDance19
            | Self::FfrDance20
            | Self::FfrDanceBully
            | Self::FfrDanceTellMe
            | Self::FfrEmoteCatPose
            | Self::FfrEmoteIdolPose => TutorialPlayerClipPlayback::Clamp,
        }
    }
}
