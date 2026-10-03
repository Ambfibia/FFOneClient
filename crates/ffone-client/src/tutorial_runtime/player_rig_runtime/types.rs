use super::*;

#[derive(Resource, Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TutorialSkywayPresentation {
    pub active: bool,
}

#[derive(Component)]
pub(super) struct TutorialSkywayAttachment {
    pub(super) rig_root: Entity,
    pub(super) gltf: Handle<Gltf>,
}

#[derive(Component)]
pub(super) struct TutorialZiplineAttachment {
    pub(super) rig_root: Entity,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PendingLegacyVisualCompletion {
    pub(super) semantic_clip: LegacyVisualClip,
    pub(super) animation_clip: TutorialPlayerClip,
    pub(super) armed: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TutorialUpperLayerPlayback {
    pub(super) node: AnimationNodeIndex,
    pub(super) elapsed_seconds: f32,
    pub(super) blend_seconds: f32,
    pub(super) base_masked: bool,
    pub(super) fading_out: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TutorialBodyShapePlayback {
    pub(super) normal_nodes: [AnimationNodeIndex; 10],
    pub(super) upper_masked_nodes: [AnimationNodeIndex; 10],
    pub(super) upper_masked: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TutorialDirectionalTurnPlayback {
    /// Unarmed, rifle and scooter left/right pairs.
    pub(super) normal_nodes: [AnimationNodeIndex; 6],
    pub(super) upper_masked_nodes: [AnimationNodeIndex; 6],
    pub(super) durations: [f32; 6],
    pub(super) current_left: f32,
    pub(super) current_right: f32,
}

/// One exact appearance part owned by a selected-player rig. Shared-skin
/// scenes and rigid Hat/Glasses/Back attachments use the same route marker so
/// material binding and readiness can validate the complete visible look.
#[derive(Component, Clone, Debug)]
pub(super) struct TutorialPlayerAppearancePart {
    pub(super) rig_root: Entity,
    pub(super) kind: NativePlayerPartKind,
    pub(super) exact_route: String,
    pub(super) glb: String,
}

/// The equipped Hand can change independently of the rig's original look.
#[derive(Component)]
pub(super) struct TutorialPlayerWeaponLook(pub(super) NativePlayerLook);

#[derive(Component, Clone, Debug)]
pub(super) struct PendingTutorialPlayerAppearanceAttachment {
    pub(super) rig_root: Entity,
    pub(super) socket_full_path: String,
    pub(super) socket_local_scale_override: Option<Vec3>,
}

#[derive(Component, Clone, Debug, Eq, PartialEq)]
pub struct TutorialPlayerAppearanceReady {
    pub parts: usize,
    pub surfaces: usize,
    pub textures: usize,
    pub skin_tinted_surfaces: usize,
    pub hair_tinted_surfaces: usize,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct TutorialPlayerFallbackVisual {
    pub entity: Option<Entity>,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct TutorialPlayerFallbackReplaced {
    pub entity: Option<Entity>,
}

#[derive(Component, Clone, Debug)]
pub struct TutorialPlayerWeaponAttachment {
    pub rig_root: Entity,
    pub item_id: i16,
    pub exact_route: String,
    pub socket_full_path: String,
}

#[derive(Component, Clone, Debug)]
pub struct TutorialPlayerEquipmentAttachmentSpawned {
    pub attachment: Entity,
    pub item_id: i16,
    pub exact_route: String,
    pub socket_full_path: String,
}
