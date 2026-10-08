use super::*;

/// Exact fade used by `cnAvatarAnimation.EndAnimation` when a non-looping
/// main animation returns to its next locomotion state.
pub const TUTORIAL_END_ANIMATION_CROSS_FADE_SECONDS: f32 = LEGACY_END_ANIMATION_BLEND_SECONDS;

/// Named clips whose source identity is proven for both player actors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TutorialPlayerClip {
    WoundUpper,
    Stun,
    StickDash,
    RifleDash,
    RifleTumbling,
    RocketSomersault,
    StickDodgeUpper,
    RifleDodgeUpper,

    Stand1,
    Staying,
    Standup,
    Run,
    RunBack,
    JumpStart,
    Jump,
    JumpEnd,
    JumpLandRun,
    Die,
    Death,
    Slide,
    RopeDown,
    Launcher,
    RopeDrop,
    RopeLeft,
    RopeRight,
    RopeStand1,
    RopeStand2,
    RopeTurn,
    RopeUp,
    Mount1,
    Mount2,
    Inventory,
    BoardStand1,
    BoardRun,
    BoardRunBack,
    BoardJumpStart,
    BoardJump,
    BoardJumpEnd,
    BoardJumpLandRun,
    ScooterStand1,
    ScooterRun,
    ScooterRunBack,
    ScooterJumpStart,
    ScooterJump,
    ScooterJumpEnd,
    ScooterJumpLandRun,
    BoardInventory,
    ScooterInventory,
    StickStand1,
    StickReady,
    StickRun,
    StickRunBack,
    StickJumpStart,
    StickJump,
    StickJumpEnd,
    StickJumpLandRun,
    PistolStand1,
    PistolReady,
    PistolRun,
    PistolRunBack,
    PistolJumpStart,
    PistolJump,
    PistolJumpEnd,
    PistolJumpLandRun,
    RifleStand1,
    RifleReady,
    Attack1,
    Attack1Upper,
    StickAttack1,
    StickAttack1Upper,
    PistolAttack1,
    PistolAttack1Upper,
    RifleAttack1,
    RifleAttack1Upper,
    RifleRun,
    RifleRunBack,
    RifleJumpStart,
    RifleJump,
    RifleJumpEnd,
    RifleJumpLandRun,
    BombStand1,
    BombReady,
    BombAttack1,
    BombAttack1Upper,
    BombRun,
    BombRunBack,
    BombJumpStart,
    BombJump,
    BombJumpEnd,
    BombJumpLandRun,
    RocketStand1,
    RocketReady,
    RocketAttack1,
    RocketAttack1Upper,
    RocketRun,
    RocketRunBack,
    RocketJumpStart,
    RocketJump,
    RocketJumpEnd,
    RocketJumpLandRun,
    Swim,
    SwimBack,
    SwimIdle,
    SwimLeft,
    SwimRight,

    Cry,
    Angry,
    Shocked,
    Hello,
    Thank,
    Dance1,
    Kiss,
    Agree,
    Laugh,
    No,
    Flex,
    Tease,
    Ok,
    Applaud,
    Cheer,
    Dance2,
    Dance3,
    Dance4,
    Dance5,
    Goodbye,
    Beach1,
    Beach2,
    Beach3,

    FfrDance02,
    FfrDance04,
    FfrDance07,
    FfrDance08,
    FfrDance10,
    FfrDance13,
    FfrDance15,
    FfrDance18,
    FfrDance19,
    FfrDance20,
    FfrDanceBully,
    FfrDanceTellMe,
    FfrEmoteCatPose,
    FfrEmoteIdolPose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialPlayerClipPlayback {
    Loop,
    Clamp,
}

/// Exact animation family selected by the tutorial-exit hand items. Keeping
/// this typed prevents a stick or pistol from silently inheriting rifle
/// locomotion merely because the hand slot is non-empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerWeaponAnimationProfile {
    Stick,
    Pistol,
    Rifle,
    Bomb,
    Rocket,
}

impl PlayerWeaponAnimationProfile {
    /// Exact `cnAvatarStatus.SetFaceHand` mapping from XDT
    /// `m_pWeaponItemTable.m_iEquipType` to `iAniStyle`.
    #[must_use]
    pub const fn from_equip_type(equip_type: i32) -> Option<Self> {
        match equip_type {
            1 | 2 => Some(Self::Stick),
            3 => Some(Self::Pistol),
            4 => Some(Self::Rifle),
            10 => Some(Self::Bomb),
            11 => Some(Self::Rocket),
            _ => None,
        }
    }

    /// Bootstrap mapping for the three class-dependent tutorial rewards.
    /// Ordinary-world items must resolve through
    /// [`PlayerWeaponAnimationCatalog`] so every XDT weapon is covered.
    #[must_use]
    pub const fn from_item_id(item_id: i16) -> Option<Self> {
        match item_id {
            43 => Some(Self::Stick),
            197 => Some(Self::Pistol),
            328 => Some(Self::Rifle),
            _ => None,
        }
    }

    #[must_use]
    pub const fn stand(self) -> TutorialPlayerClip {
        match self {
            Self::Stick => TutorialPlayerClip::StickStand1,
            Self::Pistol => TutorialPlayerClip::PistolStand1,
            Self::Rifle => TutorialPlayerClip::RifleStand1,
            Self::Bomb => TutorialPlayerClip::BombStand1,
            Self::Rocket => TutorialPlayerClip::RocketStand1,
        }
    }

    #[must_use]
    pub const fn ready(self) -> TutorialPlayerClip {
        match self {
            Self::Stick => TutorialPlayerClip::StickReady,
            Self::Pistol => TutorialPlayerClip::PistolReady,
            Self::Rifle => TutorialPlayerClip::RifleReady,
            Self::Bomb => TutorialPlayerClip::BombReady,
            Self::Rocket => TutorialPlayerClip::RocketReady,
        }
    }

    #[must_use]
    pub const fn run(self, backwards: bool) -> TutorialPlayerClip {
        match (self, backwards) {
            (Self::Stick, false) => TutorialPlayerClip::StickRun,
            (Self::Stick, true) => TutorialPlayerClip::StickRunBack,
            (Self::Pistol, false) => TutorialPlayerClip::PistolRun,
            (Self::Pistol, true) => TutorialPlayerClip::PistolRunBack,
            (Self::Rifle, false) => TutorialPlayerClip::RifleRun,
            (Self::Rifle, true) => TutorialPlayerClip::RifleRunBack,
            (Self::Bomb, false) => TutorialPlayerClip::BombRun,
            (Self::Bomb, true) => TutorialPlayerClip::BombRunBack,
            (Self::Rocket, false) => TutorialPlayerClip::RocketRun,
            (Self::Rocket, true) => TutorialPlayerClip::RocketRunBack,
        }
    }

    #[must_use]
    pub const fn jump(self, phase: TutorialPlayerClip) -> TutorialPlayerClip {
        match (self, phase) {
            (Self::Stick, TutorialPlayerClip::JumpStart) => TutorialPlayerClip::StickJumpStart,
            (Self::Stick, TutorialPlayerClip::Jump) => TutorialPlayerClip::StickJump,
            (Self::Stick, TutorialPlayerClip::JumpEnd) => TutorialPlayerClip::StickJumpEnd,
            (Self::Stick, TutorialPlayerClip::JumpLandRun) => TutorialPlayerClip::StickJumpLandRun,
            (Self::Pistol, TutorialPlayerClip::JumpStart) => TutorialPlayerClip::PistolJumpStart,
            (Self::Pistol, TutorialPlayerClip::Jump) => TutorialPlayerClip::PistolJump,
            (Self::Pistol, TutorialPlayerClip::JumpEnd) => TutorialPlayerClip::PistolJumpEnd,
            (Self::Pistol, TutorialPlayerClip::JumpLandRun) => {
                TutorialPlayerClip::PistolJumpLandRun
            }
            (Self::Rifle, TutorialPlayerClip::JumpStart) => TutorialPlayerClip::RifleJumpStart,
            (Self::Rifle, TutorialPlayerClip::Jump) => TutorialPlayerClip::RifleJump,
            (Self::Rifle, TutorialPlayerClip::JumpEnd) => TutorialPlayerClip::RifleJumpEnd,
            (Self::Rifle, TutorialPlayerClip::JumpLandRun) => TutorialPlayerClip::RifleJumpLandRun,
            (Self::Bomb, TutorialPlayerClip::JumpStart) => TutorialPlayerClip::BombJumpStart,
            (Self::Bomb, TutorialPlayerClip::Jump) => TutorialPlayerClip::BombJump,
            (Self::Bomb, TutorialPlayerClip::JumpEnd) => TutorialPlayerClip::BombJumpEnd,
            (Self::Bomb, TutorialPlayerClip::JumpLandRun) => TutorialPlayerClip::BombJumpLandRun,
            (Self::Rocket, TutorialPlayerClip::JumpStart) => TutorialPlayerClip::RocketJumpStart,
            (Self::Rocket, TutorialPlayerClip::Jump) => TutorialPlayerClip::RocketJump,
            (Self::Rocket, TutorialPlayerClip::JumpEnd) => TutorialPlayerClip::RocketJumpEnd,
            (Self::Rocket, TutorialPlayerClip::JumpLandRun) => {
                TutorialPlayerClip::RocketJumpLandRun
            }
            (_, phase) => phase,
        }
    }

    #[must_use]
    pub const fn locomotion_variant(self, base: TutorialPlayerClip) -> TutorialPlayerClip {
        match base {
            TutorialPlayerClip::Stand1 => self.stand(),
            TutorialPlayerClip::Run => self.run(false),
            TutorialPlayerClip::RunBack => self.run(true),
            TutorialPlayerClip::JumpStart
            | TutorialPlayerClip::Jump
            | TutorialPlayerClip::JumpEnd
            | TutorialPlayerClip::JumpLandRun => self.jump(base),
            _ => base,
        }
    }

    #[must_use]
    pub const fn attack(self, upper: bool) -> TutorialPlayerClip {
        match (self, upper) {
            (Self::Stick, false) => TutorialPlayerClip::StickAttack1,
            (Self::Stick, true) => TutorialPlayerClip::StickAttack1Upper,
            (Self::Pistol, false) => TutorialPlayerClip::PistolAttack1,
            (Self::Pistol, true) => TutorialPlayerClip::PistolAttack1Upper,
            (Self::Rifle, false) => TutorialPlayerClip::RifleAttack1,
            (Self::Rifle, true) => TutorialPlayerClip::RifleAttack1Upper,
            (Self::Bomb, false) => TutorialPlayerClip::BombAttack1,
            (Self::Bomb, true) => TutorialPlayerClip::BombAttack1Upper,
            (Self::Rocket, false) => TutorialPlayerClip::RocketAttack1,
            (Self::Rocket, true) => TutorialPlayerClip::RocketAttack1Upper,
        }
    }
}

/// Runtime projection of every XDT weapon row onto the legacy animation
/// family selected by `cnAvatarStatus.SetFaceHand`.
#[derive(Clone, Debug, Default, Resource)]
pub struct PlayerWeaponAnimationCatalog {
    pub(super) profiles: BTreeMap<i16, PlayerWeaponAnimationProfile>,
    pub(super) combat_profiles: BTreeMap<i16, PlayerWeaponCombatProfile>,
    pub(super) attack_sounds: BTreeMap<i16, PlayerWeaponAttackSounds>,
}

impl PlayerWeaponAnimationCatalog {
    pub fn open(locator: &AssetLocator) -> Result<Self, String> {
        let table_set: Value = locator.read_table_set()?;
        Self::from_table_set(&table_set)
    }

    pub(super) fn from_table_set(table_set: &Value) -> Result<Self, String> {
        let tables = table_set
            .get("tables")
            .and_then(Value::as_array)
            .ok_or_else(|| "TableData table-set has no tables array".to_owned())?;
        let matches = tables
            .iter()
            .filter_map(|table| table.pointer("/value/m_pWeaponItemTable"))
            .collect::<Vec<_>>();
        let [weapon_table] = matches.as_slice() else {
            return Err(format!(
                "TableData must contain exactly one m_pWeaponItemTable, found {}",
                matches.len()
            ));
        };
        let rows = weapon_table
            .get("m_pItemData")
            .and_then(Value::as_array)
            .ok_or_else(|| "m_pWeaponItemTable.m_pItemData is not an array".to_owned())?;
        let sound_rows = weapon_table
            .get("m_pItemSoundData")
            .and_then(Value::as_array)
            .ok_or_else(|| "m_pWeaponItemTable.m_pItemSoundData is not an array".to_owned())?;
        let sound_variants = sound_rows
            .iter()
            .enumerate()
            .map(|(index, row)| weapon_sound_variants(row, index))
            .collect::<Result<Vec<_>, _>>()?;
        let mut profiles = BTreeMap::new();
        let mut combat_profiles = BTreeMap::new();
        let mut attack_sounds = BTreeMap::new();
        for (index, row) in rows.iter().enumerate() {
            let item_number = row
                .get("m_iItemNumber")
                .and_then(Value::as_i64)
                .ok_or_else(|| format!("weapon row {index} has no integer m_iItemNumber"))?;
            if item_number <= 0 {
                continue;
            }
            let item_id = i16::try_from(item_number).map_err(|_| {
                format!("weapon row {index} item number {item_number} is outside protocol i16")
            })?;
            let equip_type = row
                .get("m_iEquipType")
                .and_then(Value::as_i64)
                .ok_or_else(|| format!("weapon row {index} has no integer m_iEquipType"))?;
            let equip_type = i32::try_from(equip_type).map_err(|_| {
                format!("weapon row {index} equip type {equip_type} is outside i32")
            })?;

            let row_i32 = |field: &str| -> Result<i32, String> {
                let value = row
                    .get(field)
                    .and_then(Value::as_i64)
                    .ok_or_else(|| format!("weapon row {index} has no integer {field}"))?;
                i32::try_from(value)
                    .map_err(|_| format!("weapon row {index} {field} {value} is outside i32"))
            };
            let attack_angle = row_i32("m_iAtkAngle")?;
            let attack_range = row_i32("m_iAtkRange")?;
            let delay_time = row_i32("m_iDelayTime")?;
            let target_number = row_i32("m_iTargetNumber")?;
            let target_mode = row_i32("m_iTargetMode")?;
            let effect1_bullet_type = row_i32("m_iEffect1")?;
            let effect2_bullet_type = row_i32("m_iEffect2")?;
            let sound1 = row_i32("m_iSound1")?;
            let sound2 = row_i32("m_iSound2")?;
            let deliver_time = row_i32("m_iDeliverTime")?;
            let duration_time = row_i32("m_iDurationTime")?;
            if attack_angle <= 0 || attack_range <= 0 || delay_time <= 0 || target_number <= 0 {
                return Err(format!(
                    "weapon row {index} has non-positive combat attributes: angle={attack_angle}, range={attack_range}, delay={delay_time}, targets={target_number}"
                ));
            }
            if effect1_bullet_type < 0 || effect2_bullet_type < 0 {
                return Err(format!(
                    "weapon row {index} has a negative BulletTable index: effect1={effect1_bullet_type}, effect2={effect2_bullet_type}"
                ));
            }
            let sound_row = |sound_id: i32, field: &str| -> Result<Vec<String>, String> {
                let sound_id = usize::try_from(sound_id).map_err(|_| {
                    format!("weapon row {index} has negative {field} index {sound_id}")
                })?;
                sound_variants.get(sound_id).cloned().ok_or_else(|| {
                    format!(
                        "weapon row {index} {field} index {sound_id} is outside {} sound rows",
                        sound_variants.len()
                    )
                })
            };
            let attack_sound = PlayerWeaponAttackSounds {
                normal: sound_row(sound1, "m_iSound1")?,
                charged: sound_row(sound2, "m_iSound2")?,
            };
            if attack_sound.normal.is_empty() {
                return Err(format!(
                    "weapon row {index} item {item_id} has no usable m_iSound1 variants"
                ));
            }
            if let Some(previous) = attack_sounds.insert(item_id, attack_sound.clone())
                && previous != attack_sound
            {
                return Err(format!(
                    "weapon item {item_id} maps to contradictory attack sounds"
                ));
            }
            if deliver_time < 0 || duration_time < 0 {
                return Err(format!(
                    "weapon row {index} has negative warhead timing: deliver={deliver_time}, duration={duration_time}"
                ));
            }
            let combat_profile = PlayerWeaponCombatProfile {
                attack_half_angle_degrees: attack_angle as f32,
                attack_range: attack_range as f32 * 0.01,
                blast_radius: row_i32("m_iEffectArea")? as f32 * 0.01,
                attack_cooldown_seconds: delay_time as f32 * 0.1,
                target_capacity: usize::try_from(target_number).map_err(|_| {
                    format!("weapon row {index} target count {target_number} is outside usize")
                })?,
                // `cnAvatarAttack.AttackTarget` branches on m_iTargetMode,
                // independently from SetFaceHand's m_iEquipType animation.
                target_mode: match target_mode {
                    5 => LegacyWeaponTargetMode::Rocket,
                    6 => LegacyWeaponTargetMode::Grenade,
                    _ => LegacyWeaponTargetMode::Normal,
                },
                effect1_bullet_type,
                effect2_bullet_type,
                warhead_duration_seconds: duration_time as f32 * 0.1,
                grenade_initial_vertical_speed: deliver_time as f32 * 0.5,
            };
            if let Some(previous) = combat_profiles.insert(item_id, combat_profile)
                && previous != combat_profile
            {
                return Err(format!(
                    "weapon item {item_id} maps to contradictory combat profiles"
                ));
            }
            let Some(profile) = PlayerWeaponAnimationProfile::from_equip_type(equip_type) else {
                continue;
            };
            if let Some(previous) = profiles.insert(item_id, profile)
                && previous != profile
            {
                return Err(format!(
                    "weapon item {item_id} maps to contradictory animation profiles"
                ));
            }
        }
        Ok(Self {
            profiles,
            combat_profiles,
            attack_sounds,
        })
    }

    #[must_use]
    pub fn profile_for_item(&self, item_id: i16) -> Option<PlayerWeaponAnimationProfile> {
        self.profiles.get(&item_id).copied()
    }

    #[must_use]
    pub fn combat_profile_for_item(&self, item_id: i16) -> Option<PlayerWeaponCombatProfile> {
        self.combat_profiles.get(&item_id).copied()
    }

    /// Resolves the exact `WeaponItemTable.m_iSound1/m_iSound2` row used by
    /// clean `cnAvatarAttack.attacksound`. A missing charged source asset
    /// falls back to the row's normal attack sound, never to an unrelated
    /// weapon family.
    #[must_use]
    pub fn attack_sound_variants_for_item(
        &self,
        item_id: i16,
        weapon_battery: i32,
    ) -> Option<&[String]> {
        let sounds = self.attack_sounds.get(&item_id)?;
        let selected = if weapon_battery > 0 && !sounds.charged.is_empty() {
            &sounds.charged
        } else {
            &sounds.normal
        };
        (!selected.is_empty()).then_some(selected.as_slice())
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.profiles.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }
}

impl TutorialPlayerClipPlayback {
    #[must_use]
    pub const fn contract_value(self) -> &'static str {
        match self {
            Self::Loop => "loop",
            Self::Clamp => "clamp",
        }
    }
}
