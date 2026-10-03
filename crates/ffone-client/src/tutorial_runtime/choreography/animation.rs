use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EffectBoneAttachment {
    pub effect_id: i32,
    pub actor_id: i32,
    pub exact_node_name: &'static str,
    pub spawn_world_rotation: RotationExpr,
    pub local_rotation_after_parenting: RotationExpr,
    pub scale: f32,
    pub tracked: bool,
    pub name: &'static str,
    pub locale_gate: EffectLocaleGate,
    pub destroy_after_seconds: f32,
}
