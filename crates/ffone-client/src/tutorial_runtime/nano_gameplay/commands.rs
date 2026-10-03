use super::*;

pub const TUTORIAL_BUTTERCUP_SUMMON_EVENT_SECONDS: f32 = 0.25;

pub const TUTORIAL_BUTTERCUP_SKILL_EVENT_SECONDS: f32 = 0.25;

#[derive(Debug, Clone, PartialEq)]
pub enum TutorialNanoGameplayCommand {
    Equip {
        nano_id: i16,
        skill_id: i16,
        stamina: i32,
    },
    EquipWorld {
        nano_id: i16,
        skill_id: i16,
        stamina: i32,
        presentation: WorldNanoGameplayPresentation,
    },
    Summon {
        owner: Entity,
    },
    Dismiss,
    Withdraw,
    Hide,
    UseSkill {
        owner: Entity,
        targets: Vec<LegacyAttackTarget>,
    },
    PlayWorldSkill {
        owner: Entity,
    },
}

#[derive(Debug, Default, Resource)]
pub struct TutorialNanoGameplayCommandQueue {
    pub(super) pending: VecDeque<TutorialNanoGameplayCommand>,
}

impl TutorialNanoGameplayCommandQueue {
    pub fn equip(&mut self, nano_id: i16, skill_id: i16, stamina: i32) {
        self.pending.push_back(TutorialNanoGameplayCommand::Equip {
            nano_id,
            skill_id,
            stamina,
        });
    }

    pub fn equip_world(
        &mut self,
        nano_id: i16,
        skill_id: i16,
        stamina: i32,
        presentation: WorldNanoGameplayPresentation,
    ) {
        self.pending
            .push_back(TutorialNanoGameplayCommand::EquipWorld {
                nano_id,
                skill_id,
                stamina,
                presentation,
            });
    }

    pub fn summon(&mut self, owner: Entity) {
        self.pending
            .push_back(TutorialNanoGameplayCommand::Summon { owner });
    }

    pub fn dismiss(&mut self) {
        self.pending.push_back(TutorialNanoGameplayCommand::Dismiss);
    }

    pub fn withdraw(&mut self) {
        self.pending
            .push_back(TutorialNanoGameplayCommand::Withdraw);
    }

    pub fn hide(&mut self) {
        self.pending.push_back(TutorialNanoGameplayCommand::Hide);
    }

    pub fn use_skill(&mut self, owner: Entity, targets: Vec<LegacyAttackTarget>) {
        self.pending
            .push_back(TutorialNanoGameplayCommand::UseSkill { owner, targets });
    }

    pub fn play_world_skill(&mut self, owner: Entity) {
        self.pending
            .push_back(TutorialNanoGameplayCommand::PlayWorldSkill { owner });
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn clear(&mut self) {
        self.pending.clear();
    }

    pub(super) fn take_all(&mut self) -> VecDeque<TutorialNanoGameplayCommand> {
        std::mem::take(&mut self.pending)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TutorialNanoGameplayEvent {
    EffectRequested {
        effect_id: i32,
        position: Vec3,
        rotation: Quat,
        source_line: u32,
    },
    TaggedEffectRequested {
        effect_id: i32,
        root: Entity,
        node_name: &'static str,
        source_clip_path_id: i64,
        source_event_seconds: f32,
    },
    Activated {
        owner: Entity,
        entity: Entity,
        nano_id: i16,
        skill_id: i16,
        stamina: i32,
    },
    Dismissed {
        owner: Option<Entity>,
        entity: Entity,
    },
    DamageNpc {
        target: Entity,
        actor_id: i32,
        condition: i32,
    },
    UseSkill {
        owner: Entity,
        skill_id: i16,
        stamina: i32,
    },
    AudioRequested {
        true_name: &'static str,
        category: TutorialNanoGameplayAudioCategory,
        position: Vec3,
        source_clip_path_id: i64,
        source_event_seconds: f32,
    },
}

#[derive(Debug, Default, Resource)]
pub struct TutorialNanoGameplayEventQueue {
    pub(super) pending: VecDeque<TutorialNanoGameplayEvent>,
}

impl TutorialNanoGameplayEventQueue {
    pub(super) fn push(&mut self, event: TutorialNanoGameplayEvent) {
        self.pending.push_back(event);
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn take_all(&mut self) -> VecDeque<TutorialNanoGameplayEvent> {
        std::mem::take(&mut self.pending)
    }
}
