use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TutorialActorCommand {
    Spawn(TutorialNpcSpawn),
    Delete {
        id: i32,
    },
    WarpNative {
        id: i32,
        target: Vec3,
    },
    TranslateNative {
        id: i32,
        delta: Vec3,
    },
    MoveNative {
        id: i32,
        target: Vec3,
        speed_units_per_second: f32,
    },
    /// Exact `NpcMoveController.ForceStop`: cancel translation without
    /// stopping or pausing `NpcAnimation`.
    StopMotion {
        id: i32,
    },
    SetNativeRotation {
        id: i32,
        rotation: Quat,
    },
    FaceNativePosition {
        id: i32,
        target: Vec3,
    },
    FaceActor {
        id: i32,
        target: i32,
    },
    PlayPose {
        id: i32,
        clip: &'static str,
        once: bool,
    },
    SetPoseState {
        id: i32,
        state: TutorialActorPoseState,
    },
    ForceUpdate {
        id: i32,
    },
    Damage {
        id: i32,
        amount: i32,
    },
    SetInteracting {
        id: i32,
        interacting: bool,
    },
    ClearInteractions,
    AttemptPlayerAttack {
        id: i32,
        player_position: Vec3,
    },
    ConfigureDemoMonster {
        dont_kill: bool,
    },
}

#[derive(Debug, Default, Resource)]
pub struct TutorialActorCommandQueue {
    pub(super) pending: VecDeque<TutorialActorCommand>,
}

impl TutorialActorCommandQueue {
    pub fn push(&mut self, command: TutorialActorCommand) {
        self.pending.push_back(command);
    }

    pub fn spawn(&mut self, spawn: TutorialNpcSpawn) {
        // Suppress only a duplicate in the actor's current queued lifetime.
        // A later Delete is a lifecycle boundary: choreography completion and
        // tutorial-stage fallthrough may legitimately enqueue the same Spawn
        // again after that Delete in one frame.
        let duplicate_in_current_lifetime =
            self.pending.iter().rev().find_map(|command| match command {
                TutorialActorCommand::Spawn(queued) if queued.id == spawn.id => {
                    Some(*queued == spawn)
                }
                TutorialActorCommand::Delete { id } if *id == spawn.id => Some(false),
                _ => None,
            });
        if duplicate_in_current_lifetime == Some(true) {
            return;
        }
        self.push(TutorialActorCommand::Spawn(spawn));
    }

    pub fn delete(&mut self, id: i32) {
        self.push(TutorialActorCommand::Delete { id });
    }

    pub fn warp_native(&mut self, id: i32, target: Vec3) {
        self.push(TutorialActorCommand::WarpNative { id, target });
    }

    pub fn translate_native(&mut self, id: i32, delta: Vec3) {
        self.push(TutorialActorCommand::TranslateNative { id, delta });
    }

    pub fn move_native(&mut self, id: i32, target: Vec3, speed_units_per_second: f32) {
        self.push(TutorialActorCommand::MoveNative {
            id,
            target,
            speed_units_per_second,
        });
    }

    pub fn stop_motion(&mut self, id: i32) {
        self.push(TutorialActorCommand::StopMotion { id });
    }

    pub fn set_native_rotation(&mut self, id: i32, rotation: Quat) {
        self.push(TutorialActorCommand::SetNativeRotation { id, rotation });
    }

    pub fn face_native_position(&mut self, id: i32, target: Vec3) {
        self.push(TutorialActorCommand::FaceNativePosition { id, target });
    }

    pub fn face_actor(&mut self, id: i32, target: i32) {
        self.push(TutorialActorCommand::FaceActor { id, target });
    }

    pub fn play_pose(&mut self, id: i32, clip: &'static str, once: bool) {
        self.push(TutorialActorCommand::PlayPose { id, clip, once });
    }

    pub fn set_pose_state(&mut self, id: i32, state: TutorialActorPoseState) {
        self.push(TutorialActorCommand::SetPoseState { id, state });
    }

    pub fn force_update(&mut self, id: i32) {
        self.push(TutorialActorCommand::ForceUpdate { id });
    }

    pub fn damage(&mut self, id: i32, amount: i32) {
        self.push(TutorialActorCommand::Damage { id, amount });
    }

    pub fn set_interacting(&mut self, id: i32, interacting: bool) {
        self.push(TutorialActorCommand::SetInteracting { id, interacting });
    }

    pub fn clear_interactions(&mut self) {
        self.push(TutorialActorCommand::ClearInteractions);
    }

    pub fn attempt_player_attack(&mut self, id: i32, player_position: Vec3) {
        self.push(TutorialActorCommand::AttemptPlayerAttack {
            id,
            player_position,
        });
    }

    pub fn configure_demo_monster(&mut self, dont_kill: bool) {
        self.push(TutorialActorCommand::ConfigureDemoMonster { dont_kill });
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn take_all(&mut self) -> VecDeque<TutorialActorCommand> {
        std::mem::take(&mut self.pending)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TutorialActorEvent {
    Damaged {
        id: i32,
        entity: Entity,
        amount: i32,
        remaining_hp: i32,
        first_hit: bool,
    },
    Dead {
        id: i32,
        entity: Entity,
    },
    /// `DeadMotion.Update` reached `death.length - 1` and the player-owned
    /// ES372/Oni 76+77 payload is now due.
    PlayerKillDeathPresentation {
        id: i32,
        entity: Entity,
    },
    Interaction {
        id: i32,
        entity: Entity,
        interacting: bool,
    },
    AttackedPlayer {
        id: i32,
        entity: Entity,
        damage: i32,
    },
}

#[derive(Debug, Default, Resource)]
pub struct TutorialActorEventQueue {
    pub(super) pending: VecDeque<TutorialActorEvent>,
}

impl TutorialActorEventQueue {
    pub(crate) fn push(&mut self, event: TutorialActorEvent) {
        self.pending.push_back(event);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<TutorialActorEvent> {
        self.pending.pop_front()
    }

    pub fn take_all(&mut self) -> VecDeque<TutorialActorEvent> {
        std::mem::take(&mut self.pending)
    }
}
