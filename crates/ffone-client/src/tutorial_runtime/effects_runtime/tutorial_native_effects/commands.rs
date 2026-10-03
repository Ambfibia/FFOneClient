use super::*;

#[derive(Clone, Debug)]
pub(in super::super) struct NativePreloadRequest {
    pub(in super::super) effect_id: i32,
    pub(in super::super) plan: NativeEffectPlan,
}

#[derive(Clone, Debug)]
pub(in super::super) enum NativeSpawnRequest {
    Effect {
        instance_id: u64,
        effect_id: i32,
        streamed_world: bool,
        placement: TutorialEffectPlacement,
        scale: f32,
        name: Option<String>,
        destroy_after_seconds: Option<f32>,
        source_line: u32,
        plan: NativeEffectPlan,
    },
    Projectile {
        instance_id: u64,
        effect_id: i32,
        source: Vec3,
        target: Vec3,
        scale: f32,
        reverse: bool,
        sampled_initial_velocity: Vec3,
        plan: NativeProjectilePlan,
    },
    LinearProjectile {
        instance_id: u64,
        bullet_type: i32,
        effect_id: i32,
        source: Vec3,
        target: Vec3,
        scale: f32,
        motion: NativeLinearProjectileMotion,
        impact: Option<NativeLinearImpactPlan>,
        plan: Option<NativeProjectilePlan>,
        carried_effect: Option<NativeEffectPlan>,
    },
}

impl NativeSpawnRequest {
    pub(super) fn instance_id(&self) -> u64 {
        match self {
            Self::Effect { instance_id, .. }
            | Self::Projectile { instance_id, .. }
            | Self::LinearProjectile { instance_id, .. } => *instance_id,
        }
    }

    pub(super) fn stream_owner(&self) -> Option<Entity> {
        match self {
            // A pending animation event must be cancelled when its owner is
            // picked up/despawned before asynchronous visual admission.
            Self::Effect {
                placement: TutorialEffectPlacement::ExactEntityBone { root_entity, .. },
                ..
            } => Some(*root_entity),
            Self::Effect { placement, .. } => placement.stream_owner(),
            Self::Projectile { .. } | Self::LinearProjectile { .. } => None,
        }
    }

    pub(super) fn is_streamed_world(&self) -> bool {
        matches!(
            self,
            Self::Effect {
                streamed_world: true,
                ..
            }
        )
    }

    pub(super) fn spawn_work(&self) -> usize {
        match self {
            Self::Effect { plan, .. } => 1_usize.saturating_add(plan.rendered_nodes),
            Self::Projectile { plan, .. } => 1_usize.saturating_add(plan.rendered_nodes),
            Self::LinearProjectile {
                plan,
                carried_effect,
                ..
            } => 1_usize
                .saturating_add(plan.as_ref().map_or(0, |plan| plan.rendered_nodes))
                .saturating_add(
                    carried_effect
                        .as_ref()
                        .map_or(0, |plan| plan.rendered_nodes),
                ),
        }
        .max(1)
    }
}
