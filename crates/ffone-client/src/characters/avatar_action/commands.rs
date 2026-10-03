use super::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Resource)]
pub struct LegacyAvatarActionInput {
    pub primary_held: bool,
    pub primary_just_pressed: bool,
    pub nano_just_pressed: bool,
    pub weapon_cycle_just_pressed: bool,
}

/// Conditions owned by status, inventory, UI, and tutorial systems. Keeping
/// them explicit prevents the action scheduler from pretending those ports
/// are already complete.
#[derive(Debug, Clone, Component)]
pub struct LegacyAvatarActionContext {
    pub ready_for_play: bool,
    pub input_enabled: bool,
    /// Authoritative `cnOwnAvatarStatus.HP <= 0` state. Death presentation is
    /// owned independently from the ordinary input gate because the clean
    /// client always runs `AvatarDead`, including server-forced death edges.
    pub dead: bool,
    pub system_popup: bool,
    pub time_buff_condition: i32,
    pub move_mode: LegacyMoveMode,
    pub combat_condition: bool,
    pub tutorial_event: bool,
    pub attack_locked: bool,
    pub nano_locked: bool,
    pub weapon_change_locked: bool,
    pub attack_players_enabled: bool,
    pub player_interaction_allowed: bool,
    pub overheat_allows_attack: bool,
    pub attack_cooldown_blocked: bool,
    pub weapon_change_cooldown_blocked: bool,
    pub weapon_change_in_progress: bool,
    pub special_state_four: bool,
    /// Action gate supplied by the transport owner. Full-body vehicle
    /// presentation is selected independently through
    /// [`LegacyAvatarPresentationContext::mounted_vehicle`]; callers must not
    /// derive that family merely from an equipped vehicle item.
    pub vehicle_mounted: bool,
    pub weapon_swap_available: bool,
    pub nano_skill_usable: bool,
    pub nano_target_range: f32,
    pub nano_target_capacity: usize,
    /// Exact active Nano targeting policy used by clean
    /// `cnAvatarAttack.SearchNanoSkillTarget`. `None` means that the active
    /// skill has no actor-selection overlay (for example a self skill).
    pub nano_target_policy: Option<LegacyNanoTargetPolicy>,
    pub primary_weapon_shootable: bool,
    /// `cnAvatarStatus.CurAttrib.iAttackAngle` after the equipped weapon row.
    pub attack_half_angle_degrees: f32,
    /// `cnAvatarStatus.CurAttrib.iAttackRange` converted from centiunits.
    pub attack_range: f32,
    /// `ItemElement.m_iDelayTime * 0.1`, used by
    /// `GameCondition.IsCallCoolTime(0)`.
    pub attack_cooldown_seconds: f32,
    pub target_capacity: usize,
    pub weapon_target_mode: LegacyWeaponTargetMode,
}

impl Default for LegacyAvatarActionContext {
    fn default() -> Self {
        Self {
            ready_for_play: true,
            input_enabled: true,
            dead: false,
            system_popup: false,
            time_buff_condition: 0,
            move_mode: LegacyMoveMode::None,
            combat_condition: false,
            tutorial_event: false,
            attack_locked: false,
            nano_locked: false,
            weapon_change_locked: false,
            attack_players_enabled: false,
            player_interaction_allowed: true,
            overheat_allows_attack: true,
            attack_cooldown_blocked: false,
            weapon_change_cooldown_blocked: false,
            weapon_change_in_progress: false,
            special_state_four: false,
            vehicle_mounted: false,
            weapon_swap_available: false,
            nano_skill_usable: false,
            nano_target_range: 0.0,
            nano_target_capacity: 0,
            nano_target_policy: None,
            primary_weapon_shootable: true,
            attack_half_angle_degrees: 90.0,
            attack_range: 2.0,
            attack_cooldown_seconds: 0.0,
            target_capacity: 1,
            weapon_target_mode: LegacyWeaponTargetMode::Normal,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LegacyAvatarActionIntent {
    UseTrigger(Entity),
    TalkNpc(Entity),
    TalkPlayer(Entity),
    DismountVehicle,
    PrimaryAttack {
        mode: LegacyWeaponTargetMode,
        targets: Vec<LegacyAttackTarget>,
    },
    WeaponCycle,
    NanoSkill {
        targets: Vec<LegacyAttackTarget>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAvatarActionRequest {
    pub actor: Entity,
    pub intent: LegacyAvatarActionIntent,
}

#[derive(Debug, Default, Resource)]
pub struct LegacyAvatarActionQueue {
    pub(super) pending: VecDeque<LegacyAvatarActionRequest>,
}

impl LegacyAvatarActionQueue {
    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<LegacyAvatarActionRequest> {
        self.pending.pop_front()
    }

    pub fn take_all(&mut self) -> VecDeque<LegacyAvatarActionRequest> {
        std::mem::take(&mut self.pending)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LegacyVisualCommand {
    CrossFade {
        requested_clip: LegacyVisualClip,
        resolution: LegacyClipResolution,
        blend_seconds: f32,
        layer: LegacyAnimationLayer,
        queued_play_now: bool,
    },
    DelayCurrent {
        seconds: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyVisualRequest {
    pub actor: Entity,
    pub command: LegacyVisualCommand,
}

#[derive(Debug, Default, Resource)]
pub struct LegacyVisualRequestQueue {
    pub(super) pending: VecDeque<LegacyVisualRequest>,
}

impl LegacyVisualRequestQueue {
    pub fn cancel_attack_for(&mut self, actor: Entity) {
        self.pending.retain(|request| {
            request.actor != actor || !matches!(&request.command,
                LegacyVisualCommand::CrossFade {
                    requested_clip: LegacyVisualClip::AttackFull(_) | LegacyVisualClip::AttackUpper(_),
                    ..
                } | LegacyVisualCommand::DelayCurrent { .. })
        });
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<LegacyVisualRequest> {
        self.pending.pop_front()
    }

    pub fn take_all(&mut self) -> VecDeque<LegacyVisualRequest> {
        std::mem::take(&mut self.pending)
    }
}

pub(super) fn action_input_from_devices(
    keyboard: Option<&ButtonInput<KeyCode>>,
    mouse: Option<&ButtonInput<MouseButton>>,
) -> LegacyAvatarActionInput {
    LegacyAvatarActionInput {
        primary_held: keyboard.is_some_and(|keys| keys.pressed(KeyCode::KeyZ))
            || mouse.is_some_and(|buttons| buttons.pressed(MouseButton::Left)),
        primary_just_pressed: keyboard.is_some_and(|keys| keys.just_pressed(KeyCode::KeyZ))
            || mouse.is_some_and(|buttons| buttons.just_pressed(MouseButton::Left)),
        nano_just_pressed: keyboard.is_some_and(|keys| keys.just_pressed(KeyCode::KeyX))
            || mouse.is_some_and(|buttons| buttons.just_pressed(MouseButton::Right)),
        weapon_cycle_just_pressed: keyboard.is_some_and(|keys| keys.just_pressed(KeyCode::Tab)),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum LegacyAvatarActionSet {
    ReadInput,
    Resolve,
    VisualFeedback,
    Locomotion,
}

/// Installs the scheduler only. Callers attach the context/feed/state/bindings
/// components to the local avatar and drain both output queues.
pub struct LegacyAvatarActionPlugin;

impl Plugin for LegacyAvatarActionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::input_focus::GameplayPointerCapturePlugin)
            .init_resource::<LegacyAvatarActionInput>()
            .init_resource::<LegacyAvatarActionQueue>()
            .init_resource::<LegacyVisualRequestQueue>()
            .init_resource::<LegacyVisualCompletionQueue>()
            .configure_sets(
                Update,
                LegacyAvatarActionSet::ReadInput.before(LegacyMovementSet::CameraInput),
            )
            .configure_sets(
                Update,
                LegacyAvatarActionSet::Resolve
                    .after(LegacyMovementSet::CameraInput)
                    .before(LegacyMovementSet::Simulate),
            )
            .configure_sets(
                Update,
                LegacyAvatarActionSet::VisualFeedback
                    .after(LegacyMovementSet::Simulate)
                    .after(NativeWorldSet::ResolveCollision),
            )
            .configure_sets(
                Update,
                LegacyAvatarActionSet::Locomotion.after(LegacyAvatarActionSet::VisualFeedback),
            )
            .add_systems(
                Update,
                read_legacy_avatar_action_input.in_set(LegacyAvatarActionSet::ReadInput),
            )
            .add_systems(
                Update,
                resolve_legacy_avatar_actions.in_set(LegacyAvatarActionSet::Resolve),
            )
            .add_systems(
                Update,
                process_legacy_visual_completions.in_set(LegacyAvatarActionSet::VisualFeedback),
            )
            .add_systems(
                Update,
                update_legacy_avatar_locomotion.in_set(LegacyAvatarActionSet::Locomotion),
            );
    }
}
