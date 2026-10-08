use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LegacyVisualClip {
    Stun,
    Dash,
    DashAir,
    DashUpper,
    DashWaterUpper,
    Stand1,
    Ready,
    Run,
    RunBack,
    JumpStart,
    Jump,
    JumpEnd,
    JumpLandRun,
    Die,
    Death,
    Swim,
    SwimBack,
    SwimIdle,
    SwimLeft,
    SwimRight,
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
    Inventory,
    BoardInventory,
    ScooterInventory,
    AttackFull(u32),
    AttackUpper(u32),
}

impl LegacyVisualClip {
    #[must_use]
    pub fn legacy_name(self) -> String {
        match self {
            Self::Stun => "stun".to_owned(),
            Self::Dash => "dash".to_owned(),
            Self::DashAir => "rocketsomersault".to_owned(),
            Self::DashUpper => "dodgeupper".to_owned(),
            Self::DashWaterUpper => "stickdodgeupper".to_owned(),
            Self::Stand1 => "stand1".to_owned(),
            Self::Ready => "ready".to_owned(),
            Self::Run => "run".to_owned(),
            Self::RunBack => "runback".to_owned(),
            Self::JumpStart => "jumpstart".to_owned(),
            Self::Jump => "jump".to_owned(),
            Self::JumpEnd => "jumpend".to_owned(),
            Self::JumpLandRun => "jumplandrun".to_owned(),
            Self::Die => "die".to_owned(),
            Self::Death => "death".to_owned(),
            Self::Swim => "swim".to_owned(),
            Self::SwimBack => "swimback".to_owned(),
            Self::SwimIdle => "swimidle".to_owned(),
            Self::SwimLeft => "swimleft".to_owned(),
            Self::SwimRight => "swimright".to_owned(),
            Self::Slide => "slide".to_owned(),
            Self::RopeDown => "ropedown".to_owned(),
            Self::Launcher => "luncher".to_owned(),
            Self::RopeDrop => "ropedrop".to_owned(),
            Self::RopeLeft => "ropeleft".to_owned(),
            Self::RopeRight => "roperight".to_owned(),
            Self::RopeStand1 => "ropestand1".to_owned(),
            Self::RopeStand2 => "ropestand2".to_owned(),
            Self::RopeTurn => "ropeturn".to_owned(),
            Self::RopeUp => "ropeup".to_owned(),
            Self::Mount1 => "mount1".to_owned(),
            Self::Inventory => "inven".to_owned(),
            Self::BoardInventory => "board_inven".to_owned(),
            Self::ScooterInventory => "scooter_inven".to_owned(),
            Self::AttackFull(sequence) => format!("attack{sequence}"),
            Self::AttackUpper(sequence) => format!("attack{sequence}upper"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyClipResolution {
    Connected {
        clip: LegacyVisualClip,
        asset_path: String,
    },
    Unconnected,
}

#[derive(Debug, Clone, Default, Component)]
pub struct LegacyAvatarClipBindings {
    pub(super) paths: HashMap<LegacyVisualClip, String>,
}

impl LegacyAvatarClipBindings {
    pub fn connect(&mut self, clip: LegacyVisualClip, asset_path: impl Into<String>) {
        self.paths.insert(clip, asset_path.into());
    }

    pub fn disconnect(&mut self, clip: LegacyVisualClip) {
        self.paths.remove(&clip);
    }

    #[must_use]
    pub fn is_connected(&self, clip: LegacyVisualClip) -> bool {
        self.paths.contains_key(&clip)
    }

    #[must_use]
    pub fn resolve(&self, requested: LegacyVisualClip) -> LegacyClipResolution {
        let direct = self.paths.get(&requested).map(|path| (requested, path));
        let fallback = match requested {
            LegacyVisualClip::AttackFull(sequence) if sequence != 1 => self
                .paths
                .get(&LegacyVisualClip::AttackFull(1))
                .map(|path| (LegacyVisualClip::AttackFull(1), path)),
            LegacyVisualClip::AttackUpper(sequence) if sequence != 1 => self
                .paths
                .get(&LegacyVisualClip::AttackUpper(1))
                .map(|path| (LegacyVisualClip::AttackUpper(1), path)),
            _ => None,
        };
        direct
            .or(fallback)
            .map_or(LegacyClipResolution::Unconnected, |(clip, asset_path)| {
                LegacyClipResolution::Connected {
                    clip,
                    asset_path: asset_path.clone(),
                }
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyAnimationLayer {
    FullBody,
    UpperBody,
}
