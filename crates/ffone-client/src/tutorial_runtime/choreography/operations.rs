use super::*;

pub(super) const fn at(at_seconds: f32, source_line: u32, action: ChoreographyAction) -> TimedAction {
    TimedAction {
        at_seconds,
        source_line,
        action,
    }
}

pub(super) const fn oriented(
    origin: PositionAnchor,
    world_offset: ClientVec3,
    orientation: OrientationBasis,
    local_offset: ClientVec3,
) -> PositionExpr {
    PositionExpr::OrientedOffset {
        origin,
        world_offset,
        orientation,
        local_offset,
    }
}

pub(super) const fn skip(source_line: u32, action: ChoreographyAction) -> SourcedAction {
    SourcedAction {
        source_line,
        action,
    }
}

pub(super) const fn spawn(
    id: i32,
    npc_type: i32,
    x: f32,
    y: f32,
    z: f32,
    helper_angle_degrees: Option<i16>,
    initial_animation: Option<&'static str>,
    force_update: bool,
) -> NpcSpawn {
    NpcSpawn {
        id,
        npc_type,
        position: LegacyServerPosition::new(x, y, z),
        helper_angle_degrees,
        initial_animation,
        force_update,
    }
}

pub(super) const fn effect(effect_id: i32, x: f32, y: f32, z: f32, scale: f32, tracked: bool) -> EffectSpawn {
    EffectSpawn {
        effect_id,
        position: PositionExpr::Client(ClientVec3::new(x, y, z)),
        scale,
        tracked,
    }
}

pub(super) const fn skip_contract(
    scene_specific: &'static [SourcedAction],
    final_fade: SkipFinalFade,
) -> SkipContract {
    SkipContract {
        scene_specific,
        common_cleanup: COMMON_SKIP_CLEANUP,
        final_fade,
    }
}

pub fn tutorial_scene_choreography(
    scene: TutorialScene,
) -> Option<&'static TutorialSceneChoreography> {
    TUTORIAL_SCENE_CHOREOGRAPHIES
        .iter()
        .find(|choreography| choreography.scene == scene)
}
