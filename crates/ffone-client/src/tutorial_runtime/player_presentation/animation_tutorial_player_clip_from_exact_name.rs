use super::*;

impl TutorialPlayerClip {


    pub fn from_exact_name(name: &str) -> Option<Self> {
        match name {
            "woundupper" => Some(Self::WoundUpper),
            "stun" => Some(Self::Stun),
            "stickdash" => Some(Self::StickDash),
            "rifledash" => Some(Self::RifleDash),
            "rifletumbling" => Some(Self::RifleTumbling),
            "rocketsomersault" => Some(Self::RocketSomersault),
            "stickdodgeupper" => Some(Self::StickDodgeUpper),
            "rifledodgeupper" => Some(Self::RifleDodgeUpper),
            "stand1" => Some(Self::Stand1),
            "staying" => Some(Self::Staying),
            "standup" => Some(Self::Standup),
            "run" => Some(Self::Run),
            "runback" => Some(Self::RunBack),
            "jumpstart" => Some(Self::JumpStart),
            "jump" => Some(Self::Jump),
            "jumpend" => Some(Self::JumpEnd),
            "jumplandrun" => Some(Self::JumpLandRun),
            "die" => Some(Self::Die),
            "death" => Some(Self::Death),
            "slide" => Some(Self::Slide),
            "ropedown" => Some(Self::RopeDown),
            "ropedrop" => Some(Self::RopeDrop),
            "ropeleft" => Some(Self::RopeLeft),
            "roperight" => Some(Self::RopeRight),
            "ropestand1" => Some(Self::RopeStand1),
            "ropestand2" => Some(Self::RopeStand2),
            "ropeturn" => Some(Self::RopeTurn),
            "ropeup" => Some(Self::RopeUp),
            "mount1" => Some(Self::Mount1),
            "mount2" => Some(Self::Mount2),
            "inven" => Some(Self::Inventory),
            "board_stand1" => Some(Self::BoardStand1),
            "board_run" => Some(Self::BoardRun),
            "board_runback" => Some(Self::BoardRunBack),
            "board_jumpstart" => Some(Self::BoardJumpStart),
            "board_jump" => Some(Self::BoardJump),
            "board_jumpend" => Some(Self::BoardJumpEnd),
            "board_jumplandrun" => Some(Self::BoardJumpLandRun),
            "scooter_stand1" => Some(Self::ScooterStand1),
            "scooter_run" => Some(Self::ScooterRun),
            "scooter_runback" => Some(Self::ScooterRunBack),
            "scooter_jumpstart" => Some(Self::ScooterJumpStart),
            "scooter_jump" => Some(Self::ScooterJump),
            "scooter_jumpend" => Some(Self::ScooterJumpEnd),
            "scooter_jumplandrun" => Some(Self::ScooterJumpLandRun),
            "board_inven" => Some(Self::BoardInventory),
            "scooter_inven" => Some(Self::ScooterInventory),
            "stickstand1" => Some(Self::StickStand1),
            "stickready" => Some(Self::StickReady),
            "stickrun" => Some(Self::StickRun),
            "stickrunback" => Some(Self::StickRunBack),
            "stickjumpstart" => Some(Self::StickJumpStart),
            "stickjump" => Some(Self::StickJump),
            "stickjumpend" => Some(Self::StickJumpEnd),
            "stickjumplandrun" => Some(Self::StickJumpLandRun),
            "pistolstand1" => Some(Self::PistolStand1),
            "pistolready" => Some(Self::PistolReady),
            "pistolrun" => Some(Self::PistolRun),
            "pistolrunback" => Some(Self::PistolRunBack),
            "pistoljumpstart" => Some(Self::PistolJumpStart),
            "pistoljump" => Some(Self::PistolJump),
            "pistoljumpend" => Some(Self::PistolJumpEnd),
            "pistoljumplandrun" => Some(Self::PistolJumpLandRun),
            "riflestand1" => Some(Self::RifleStand1),
            "rifleready" => Some(Self::RifleReady),
            "attack1" => Some(Self::Attack1),
            "attack1upper" => Some(Self::Attack1Upper),
            "stickattack1" => Some(Self::StickAttack1),
            "stickattack1upper" => Some(Self::StickAttack1Upper),
            "pistolattack1" => Some(Self::PistolAttack1),
            "pistolattack1upper" => Some(Self::PistolAttack1Upper),
            "rifleattack1" => Some(Self::RifleAttack1),
            "rifleattack1upper" => Some(Self::RifleAttack1Upper),
            "riflerun" => Some(Self::RifleRun),
            "riflerunback" => Some(Self::RifleRunBack),
            "riflejumpstart" => Some(Self::RifleJumpStart),
            "riflejump" => Some(Self::RifleJump),
            "riflejumpend" => Some(Self::RifleJumpEnd),
            "riflejumplandrun" => Some(Self::RifleJumpLandRun),
            "bombstand1" => Some(Self::BombStand1),
            "bombready" => Some(Self::BombReady),
            "bombattack1" => Some(Self::BombAttack1),
            "bombattack1upper" => Some(Self::BombAttack1Upper),
            "bombrun" => Some(Self::BombRun),
            "bombrunback" => Some(Self::BombRunBack),
            "bombjumpstart" => Some(Self::BombJumpStart),
            "bombjump" => Some(Self::BombJump),
            "bombjumpend" => Some(Self::BombJumpEnd),
            "bombjumplandrun" => Some(Self::BombJumpLandRun),
            "rocketstand1" => Some(Self::RocketStand1),
            "rocketready" => Some(Self::RocketReady),
            "rocketattack1" => Some(Self::RocketAttack1),
            "rocketattack1upper" => Some(Self::RocketAttack1Upper),
            "rocketrun" => Some(Self::RocketRun),
            "rocketrunback" => Some(Self::RocketRunBack),
            "rocketjumpstart" => Some(Self::RocketJumpStart),
            "rocketjump" => Some(Self::RocketJump),
            "rocketjumpend" => Some(Self::RocketJumpEnd),
            "rocketjumplandrun" => Some(Self::RocketJumpLandRun),
            "swim" => Some(Self::Swim),
            "swimback" => Some(Self::SwimBack),
            "swimidle" => Some(Self::SwimIdle),
            "swimleft" => Some(Self::SwimLeft),
            "swimright" => Some(Self::SwimRight),
            "cry" => Some(Self::Cry),
            "angry" => Some(Self::Angry),
            "shocked" => Some(Self::Shocked),
            "hello" => Some(Self::Hello),
            "thank" => Some(Self::Thank),
            "dance1" => Some(Self::Dance1),
            "kiss" => Some(Self::Kiss),
            "agree" => Some(Self::Agree),
            "laugh" => Some(Self::Laugh),
            "no" => Some(Self::No),
            "flex" => Some(Self::Flex),
            "tease" => Some(Self::Tease),
            "ok" => Some(Self::Ok),
            "applaud" => Some(Self::Applaud),
            "cheer" => Some(Self::Cheer),
            "dance2" => Some(Self::Dance2),
            "dance3" => Some(Self::Dance3),
            "dance4" => Some(Self::Dance4),
            "dance5" => Some(Self::Dance5),
            "goodbye" => Some(Self::Goodbye),
            "beach1" => Some(Self::Beach1),
            "beach2" => Some(Self::Beach2),
            "beach3" => Some(Self::Beach3),
            "ffr_dance_02" => Some(Self::FfrDance02),
            "ffr_dance_04" => Some(Self::FfrDance04),
            "ffr_dance_07" => Some(Self::FfrDance07),
            "ffr_dance_08" => Some(Self::FfrDance08),
            "ffr_dance_10" => Some(Self::FfrDance10),
            "ffr_dance_13" => Some(Self::FfrDance13),
            "ffr_dance_15" => Some(Self::FfrDance15),
            "ffr_dance_18" => Some(Self::FfrDance18),
            "ffr_dance_19" => Some(Self::FfrDance19),
            "ffr_dance_20" => Some(Self::FfrDance20),
            "ffr_dance_bully" => Some(Self::FfrDanceBully),
            "ffr_dance_tellme" => Some(Self::FfrDanceTellMe),
            "ffr_emote_catpose" => Some(Self::FfrEmoteCatPose),
            "ffr_emote_idolpose" => Some(Self::FfrEmoteIdolPose),
            _ => None,
        }
    }

    /// Exact weapon-prefixed animation selected by `cnAvatarAnimation` while
    /// the tutorial rifle is attached.
    #[must_use]
    pub const fn rifle_variant(self) -> Self {
        match self {
            Self::Stand1 => Self::RifleStand1,
            Self::Run => Self::RifleRun,
            Self::RunBack => Self::RifleRunBack,
            Self::JumpStart => Self::RifleJumpStart,
            Self::Jump => Self::RifleJump,
            Self::JumpEnd => Self::RifleJumpEnd,
            Self::JumpLandRun => Self::RifleJumpLandRun,
            _ => self,
        }
    }

    #[must_use]
    pub const fn is_upper_layer(self) -> bool {
        self.is_upper_attack() || matches!(self, Self::WoundUpper | Self::StickDodgeUpper | Self::RifleDodgeUpper)
    }

    pub const fn is_upper_attack(self) -> bool {
        matches!(
            self,
            Self::Attack1Upper
                | Self::StickAttack1Upper
                | Self::PistolAttack1Upper
                | Self::RifleAttack1Upper
                | Self::BombAttack1Upper
                | Self::RocketAttack1Upper
        )
    }

    /// Exact `EmoteAnimationTable.m_iEmoteCode` to clip mapping used by
    /// `cnAvatarAnimation.AvatarEmote(int)`. Codes 15 and 16 intentionally
    /// share the same `cheer` clip in the clean table. Codes 31-44 are native
    /// Retrobution extensions; the clean table leaves 25-30 empty.
    #[must_use]
    pub const fn from_avatar_emote_code(code: i32) -> Option<Self> {
        match code {
            1 => Some(Self::Cry),
            2 => Some(Self::Angry),
            3 => Some(Self::Shocked),
            4 => Some(Self::Hello),
            5 => Some(Self::Thank),
            6 => Some(Self::Dance1),
            7 => Some(Self::Kiss),
            8 => Some(Self::Agree),
            9 => Some(Self::Laugh),
            10 => Some(Self::No),
            11 => Some(Self::Flex),
            12 => Some(Self::Tease),
            13 => Some(Self::Ok),
            14 => Some(Self::Applaud),
            15 | 16 => Some(Self::Cheer),
            17 => Some(Self::Dance2),
            18 => Some(Self::Dance3),
            19 => Some(Self::Dance4),
            20 => Some(Self::Dance5),
            21 => Some(Self::Goodbye),
            22 => Some(Self::Beach1),
            23 => Some(Self::Beach2),
            24 => Some(Self::Beach3),
            31 => Some(Self::FfrDance02),
            32 => Some(Self::FfrDance04),
            33 => Some(Self::FfrDance07),
            34 => Some(Self::FfrDance08),
            35 => Some(Self::FfrDance10),
            36 => Some(Self::FfrDance13),
            37 => Some(Self::FfrDance15),
            38 => Some(Self::FfrDance18),
            39 => Some(Self::FfrDance19),
            40 => Some(Self::FfrDance20),
            41 => Some(Self::FfrDanceBully),
            42 => Some(Self::FfrDanceTellMe),
            43 => Some(Self::FfrEmoteCatPose),
            44 => Some(Self::FfrEmoteIdolPose),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_avatar_emote_code_clip(self) -> bool {
        matches!(
            self,
            Self::Cry
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
                | Self::FfrEmoteIdolPose
        )
    }

    #[must_use]
    pub const fn is_ffr_custom(self) -> bool {
        matches!(
            self,
            Self::FfrDance02
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
                | Self::FfrEmoteIdolPose
        )
    }

    pub(super) fn direct_tutorial_emote(name: &str) -> Option<Self> {
        let clip = Self::from_exact_name(name)?;
        matches!(clip, Self::Staying | Self::Standup | Self::Run)
            .then_some(clip)
            .or_else(|| clip.is_ffr_custom().then_some(clip))
    }

}

/// Exact animation API used by the original call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialPlayerAnimationDispatch {
    /// `cnAvatarAnimation.Play(name)` / Unity `Animation.Play(name)`.
    PlayImmediate,
    /// `cnAvatarAnimation.CrossFade(name, 0.15f)`.
    CrossFade(Duration),
}

/// A source-proven animation request, still awaiting asset resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TutorialPlayerAnimationRequest {
    pub clip: TutorialPlayerClip,
    pub source_path_id: i64,
    pub dispatch: TutorialPlayerAnimationDispatch,
    /// Direct string `AvatarEmote` sets `bEmote = true`.
    pub sets_emote_state: bool,
    /// Direct string `AvatarEmote` hides `AttachName(3)`.
    pub hide_hand_attachment: bool,
    /// `AvatarStandForce` sets `iCurMoveDir = 0`.
    pub reset_move_direction: bool,
}

impl TutorialPlayerAnimationRequest {
    #[must_use]
    pub fn avatar_emote(
        gender: PlayerRigGender,
        name: &str,
    ) -> Result<Self, TutorialPlayerPresentationError> {
        let clip = TutorialPlayerClip::direct_tutorial_emote(name).ok_or_else(|| {
            TutorialPlayerPresentationError::UnsupportedDirectAvatarEmote(name.to_owned())
        })?;
        Ok(Self {
            clip,
            source_path_id: clip.source_path_id(gender),
            // The string overload in the primary client always calls
            // `Animation.Play(name)`, including for `staying` and `standup`.
            // Native replacement provenance must not change that state-machine
            // dispatch. Only the explicit FFR-only extension clips retain
            // their authored entry fade.
            dispatch: if clip.is_ffr_custom() {
                TutorialPlayerAnimationDispatch::CrossFade(Duration::from_secs_f32(
                    FFR_CUSTOM_EMOTE_CROSS_FADE_SECONDS,
                ))
            } else {
                TutorialPlayerAnimationDispatch::PlayImmediate
            },
            sets_emote_state: true,
            hide_hand_attachment: true,
            reset_move_direction: false,
        })
    }

    /// Exact integer-code overload used by MenuChat and AvatarEmote packets.
    #[must_use]
    pub fn avatar_emote_code(gender: PlayerRigGender, code: i32) -> Option<Self> {
        Self::avatar_emote_clip(gender, TutorialPlayerClip::from_avatar_emote_code(code)?)
    }

    #[must_use]
    pub fn avatar_emote_clip(gender: PlayerRigGender, clip: TutorialPlayerClip) -> Option<Self> {
        if !clip.is_avatar_emote_code_clip() {
            return None;
        }
        Some(Self {
            clip,
            source_path_id: clip.source_path_id(gender),
            dispatch: TutorialPlayerAnimationDispatch::CrossFade(Duration::from_secs_f32(
                AVATAR_EMOTE_CODE_CROSS_FADE_SECONDS,
            )),
            sets_emote_state: true,
            hide_hand_attachment: true,
            reset_move_direction: false,
        })
    }
    #[must_use]
    pub fn stand_force(gender: PlayerRigGender) -> Self {
        let clip = TutorialPlayerClip::Stand1;
        Self {
            clip,
            source_path_id: clip.source_path_id(gender),
            dispatch: TutorialPlayerAnimationDispatch::CrossFade(tutorial_stand_force_cross_fade()),
            sets_emote_state: false,
            hide_hand_attachment: false,
            reset_move_direction: true,
        }
    }

    /// Exact normal-locomotion cross-fade, distinct from direct tutorial
    /// `AvatarEmote` playback. Only clips proven by this bridge are accepted.
    #[must_use]
    pub fn locomotion(gender: PlayerRigGender, clip: TutorialPlayerClip) -> Option<Self> {
        if !matches!(
            clip,
            TutorialPlayerClip::Stand1
                | TutorialPlayerClip::Run
                | TutorialPlayerClip::RunBack
                | TutorialPlayerClip::JumpStart
                | TutorialPlayerClip::Jump
                | TutorialPlayerClip::JumpEnd
                | TutorialPlayerClip::JumpLandRun
                | TutorialPlayerClip::Slide
                | TutorialPlayerClip::RopeDown
                | TutorialPlayerClip::RopeDrop
                | TutorialPlayerClip::RopeLeft
                | TutorialPlayerClip::RopeRight
                | TutorialPlayerClip::RopeStand1
                | TutorialPlayerClip::RopeStand2
                | TutorialPlayerClip::RopeTurn
                | TutorialPlayerClip::RopeUp
                | TutorialPlayerClip::Mount1
                | TutorialPlayerClip::Mount2
                | TutorialPlayerClip::Inventory
                | TutorialPlayerClip::BoardStand1
                | TutorialPlayerClip::BoardRun
                | TutorialPlayerClip::BoardRunBack
                | TutorialPlayerClip::BoardJumpStart
                | TutorialPlayerClip::BoardJump
                | TutorialPlayerClip::BoardJumpEnd
                | TutorialPlayerClip::BoardJumpLandRun
                | TutorialPlayerClip::ScooterStand1
                | TutorialPlayerClip::ScooterRun
                | TutorialPlayerClip::ScooterRunBack
                | TutorialPlayerClip::ScooterJumpStart
                | TutorialPlayerClip::ScooterJump
                | TutorialPlayerClip::ScooterJumpEnd
                | TutorialPlayerClip::ScooterJumpLandRun
                | TutorialPlayerClip::BoardInventory
                | TutorialPlayerClip::ScooterInventory
                | TutorialPlayerClip::StickStand1
                | TutorialPlayerClip::StickReady
                | TutorialPlayerClip::StickRun
                | TutorialPlayerClip::StickRunBack
                | TutorialPlayerClip::StickJumpStart
                | TutorialPlayerClip::StickJump
                | TutorialPlayerClip::StickJumpEnd
                | TutorialPlayerClip::StickJumpLandRun
                | TutorialPlayerClip::PistolStand1
                | TutorialPlayerClip::PistolReady
                | TutorialPlayerClip::PistolRun
                | TutorialPlayerClip::PistolRunBack
                | TutorialPlayerClip::PistolJumpStart
                | TutorialPlayerClip::PistolJump
                | TutorialPlayerClip::PistolJumpEnd
                | TutorialPlayerClip::PistolJumpLandRun
                | TutorialPlayerClip::RifleStand1
                | TutorialPlayerClip::RifleReady
                | TutorialPlayerClip::RifleRun
                | TutorialPlayerClip::RifleRunBack
                | TutorialPlayerClip::RifleJumpStart
                | TutorialPlayerClip::RifleJump
                | TutorialPlayerClip::RifleJumpEnd
                | TutorialPlayerClip::RifleJumpLandRun
                | TutorialPlayerClip::BombStand1
                | TutorialPlayerClip::BombReady
                | TutorialPlayerClip::BombRun
                | TutorialPlayerClip::BombRunBack
                | TutorialPlayerClip::BombJumpStart
                | TutorialPlayerClip::BombJump
                | TutorialPlayerClip::BombJumpEnd
                | TutorialPlayerClip::BombJumpLandRun
                | TutorialPlayerClip::RocketStand1
                | TutorialPlayerClip::RocketReady
                | TutorialPlayerClip::RocketRun
                | TutorialPlayerClip::RocketRunBack
                | TutorialPlayerClip::RocketJumpStart
                | TutorialPlayerClip::RocketJump
                | TutorialPlayerClip::RocketJumpEnd
                | TutorialPlayerClip::RocketJumpLandRun
                | TutorialPlayerClip::Swim
                | TutorialPlayerClip::SwimBack
                | TutorialPlayerClip::SwimIdle
                | TutorialPlayerClip::SwimLeft
                | TutorialPlayerClip::SwimRight
        ) {
            return None;
        }
        Some(Self {
            clip,
            source_path_id: clip.source_path_id(gender),
            dispatch: TutorialPlayerAnimationDispatch::CrossFade(Duration::from_secs_f32(
                LEGACY_ANIMATION_BLEND_SECONDS,
            )),
            // `AvatarSlope`, `AvatarZipline`, `AvatarRope`, and
            // `SetInvenMotion` all call HidePistol before/while taking
            // persistent full-body ownership. Reuse the adapter's exact hand
            // attachment visibility latch for those states.
            sets_emote_state: false,
            hide_hand_attachment: matches!(
                clip,
                TutorialPlayerClip::Slide
                    | TutorialPlayerClip::RopeDown
                    | TutorialPlayerClip::RopeDrop
                    | TutorialPlayerClip::RopeLeft
                    | TutorialPlayerClip::RopeRight
                    | TutorialPlayerClip::RopeStand1
                    | TutorialPlayerClip::RopeStand2
                    | TutorialPlayerClip::RopeTurn
                    | TutorialPlayerClip::RopeUp
                    | TutorialPlayerClip::Mount1
                    | TutorialPlayerClip::Mount2
                    | TutorialPlayerClip::Inventory
                    | TutorialPlayerClip::BoardStand1
                | TutorialPlayerClip::BoardRun
                | TutorialPlayerClip::BoardRunBack
                | TutorialPlayerClip::BoardJumpStart
                | TutorialPlayerClip::BoardJump
                | TutorialPlayerClip::BoardJumpEnd
                | TutorialPlayerClip::BoardJumpLandRun
                | TutorialPlayerClip::ScooterStand1
                | TutorialPlayerClip::ScooterRun
                | TutorialPlayerClip::ScooterRunBack
                | TutorialPlayerClip::ScooterJumpStart
                | TutorialPlayerClip::ScooterJump
                | TutorialPlayerClip::ScooterJumpEnd
                | TutorialPlayerClip::ScooterJumpLandRun
                | TutorialPlayerClip::BoardInventory
                    | TutorialPlayerClip::ScooterInventory
            ),
            reset_move_direction: false,
        })
    }

    /// Source-proven cross-fade used by the normal avatar action bridge.  The
    /// attack clips are clamp animations; their completion is returned to the
    /// legacy action state by the concrete rig adapter.
    #[must_use]
    pub fn runtime_cross_fade(gender: PlayerRigGender, clip: TutorialPlayerClip) -> Self {
        Self::runtime_cross_fade_with_duration(
            gender,
            clip,
            Duration::from_secs_f32(LEGACY_ANIMATION_BLEND_SECONDS),
        )
    }

    #[must_use]
    pub fn runtime_cross_fade_with_duration(
        gender: PlayerRigGender,
        clip: TutorialPlayerClip,
        duration: Duration,
    ) -> Self {
        Self {
            clip,
            source_path_id: clip.source_path_id(gender),
            dispatch: TutorialPlayerAnimationDispatch::CrossFade(duration),
            sets_emote_state: false,
            hide_hand_attachment: matches!(
                clip,
                TutorialPlayerClip::Slide
                    | TutorialPlayerClip::RopeDown
                    | TutorialPlayerClip::RopeDrop
                    | TutorialPlayerClip::RopeLeft
                    | TutorialPlayerClip::RopeRight
                    | TutorialPlayerClip::RopeStand1
                    | TutorialPlayerClip::RopeStand2
                    | TutorialPlayerClip::RopeTurn
                    | TutorialPlayerClip::RopeUp
                    | TutorialPlayerClip::Mount1
                    | TutorialPlayerClip::Mount2
                    | TutorialPlayerClip::Inventory
                    | TutorialPlayerClip::BoardStand1
                | TutorialPlayerClip::BoardRun
                | TutorialPlayerClip::BoardRunBack
                | TutorialPlayerClip::BoardJumpStart
                | TutorialPlayerClip::BoardJump
                | TutorialPlayerClip::BoardJumpEnd
                | TutorialPlayerClip::BoardJumpLandRun
                | TutorialPlayerClip::ScooterStand1
                | TutorialPlayerClip::ScooterRun
                | TutorialPlayerClip::ScooterRunBack
                | TutorialPlayerClip::ScooterJumpStart
                | TutorialPlayerClip::ScooterJump
                | TutorialPlayerClip::ScooterJumpEnd
                | TutorialPlayerClip::ScooterJumpLandRun
                | TutorialPlayerClip::BoardInventory
                    | TutorialPlayerClip::ScooterInventory
            ),
            reset_move_direction: false,
        }
    }

    /// Exact `EndAnimation -> CrossFade(next, 0.3f)` request. This is
    /// intentionally distinct from normal movement's 0.15-second fade.
    #[must_use]
    pub fn end_animation_cross_fade(gender: PlayerRigGender, clip: TutorialPlayerClip) -> Self {
        Self {
            clip,
            source_path_id: clip.source_path_id(gender),
            dispatch: TutorialPlayerAnimationDispatch::CrossFade(Duration::from_secs_f32(
                TUTORIAL_END_ANIMATION_CROSS_FADE_SECONDS,
            )),
            sets_emote_state: false,
            hide_hand_attachment: false,
            reset_move_direction: false,
        }
    }
}

/// Named clips reported by the concrete selected-player GLB.
///
/// Resolution is intentionally name-exact and case-sensitive, just like
/// Unity's legacy `Animation` lookup used by Retrobution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TutorialPlayerClipAvailability {
    pub(super) names: BTreeSet<String>,
}

impl TutorialPlayerClipAvailability {
    #[must_use]
    pub fn from_names(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            names: names.into_iter().map(Into::into).collect(),
        }
    }

    #[must_use]
    pub fn names(&self) -> impl ExactSizeIterator<Item = &str> {
        self.names.iter().map(String::as_str)
    }

    #[must_use]
    pub fn contains(&self, clip: TutorialPlayerClip) -> bool {
        self.names.contains(clip.name())
    }

    pub fn resolve(
        &self,
        request: TutorialPlayerAnimationRequest,
    ) -> Result<ResolvedTutorialPlayerAnimation, MissingTutorialPlayerClip> {
        if self.contains(request.clip) {
            Ok(ResolvedTutorialPlayerAnimation { request })
        } else {
            Err(MissingTutorialPlayerClip {
                requested_clip: request.clip,
                source_path_id: request.source_path_id,
                available_names: self.names().map(str::to_owned).collect(),
            })
        }
    }
}

/// A request may be called "resolved" only after exact named-clip discovery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolvedTutorialPlayerAnimation {
    pub request: TutorialPlayerAnimationRequest,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingTutorialPlayerClip {
    pub requested_clip: TutorialPlayerClip,
    pub source_path_id: i64,
    pub available_names: Vec<String>,
}

impl fmt::Display for MissingTutorialPlayerClip {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "selected-player rig is missing exact clip {:?} (source PathID {}); available={:?}",
            self.requested_clip.name(),
            self.source_path_id,
            self.available_names
        )
    }
}

impl Error for MissingTutorialPlayerClip {}

/// Failure to prove that an exact tutorial clip can be consumed by the
/// selected native player rig. These checks are stricter than name discovery:
/// source identity, GLB playback mode, and publisher runtime status must all
/// agree before a command can reach a renderer adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TutorialPlayerRigCapabilityError {
    DuplicateExactClip {
        clip: TutorialPlayerClip,
    },
    MissingExactClip {
        clip: TutorialPlayerClip,
        available_exact_names: Vec<String>,
    },
    RequestSourcePathIdMismatch {
        clip: TutorialPlayerClip,
        expected: i64,
        actual: i64,
    },
    ContractSourcePathIdMismatch {
        clip: TutorialPlayerClip,
        expected: i64,
        actual: i64,
    },
    PlaybackMismatch {
        clip: TutorialPlayerClip,
        expected: TutorialPlayerClipPlayback,
        actual: String,
    },
    RuntimeStatusNotReady {
        clip: TutorialPlayerClip,
        actual: String,
    },
}

impl fmt::Display for TutorialPlayerRigCapabilityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateExactClip { clip } => {
                write!(formatter, "duplicate exact player clip {:?}", clip.name())
            }
            Self::MissingExactClip {
                clip,
                available_exact_names,
            } => write!(
                formatter,
                "selected-player contract is missing exact clip {:?}; available={available_exact_names:?}",
                clip.name()
            ),
            Self::RequestSourcePathIdMismatch {
                clip,
                expected,
                actual,
            } => write!(
                formatter,
                "request for {:?} has source PathID {actual}, expected {expected}",
                clip.name()
            ),
            Self::ContractSourcePathIdMismatch {
                clip,
                expected,
                actual,
            } => write!(
                formatter,
                "contract for {:?} has source PathID {actual}, expected {expected}",
                clip.name()
            ),
            Self::PlaybackMismatch {
                clip,
                expected,
                actual,
            } => write!(
                formatter,
                "contract for {:?} has playback {actual:?}, expected {:?}",
                clip.name(),
                expected.contract_value()
            ),
            Self::RuntimeStatusNotReady { clip, actual } => write!(
                formatter,
                "contract for {:?} is not renderer-consumable: status={actual:?}, expected {TUTORIAL_PLAYER_RUNTIME_READY_STATUS:?}",
                clip.name()
            ),
        }
    }
}

impl Error for TutorialPlayerRigCapabilityError {}

/// Exact clip capabilities loaded from one gender entry of
/// `player-shared-rig.json`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TutorialPlayerRigCapabilities {
    pub(super) gender: PlayerRigGender,
    pub(super) clips: BTreeMap<TutorialPlayerClip, PlayerRigClipContract>,
}
