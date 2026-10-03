use super::*;

pub(in super::super) fn resolve_choreography_position(
    expression: PositionExpr,
    source_line: u32,
    frame: &TutorialChoreographyFrame,
    captures: &TutorialCameraCaptureStore,
    actor_registry: &TutorialActorRegistry,
    actor_transforms: &Query<&Transform, With<TutorialActor>>,
    issues: &mut TutorialChoreographyIssueQueue,
) -> Option<Vec3> {
    let mut entity_transform = |entity| {
        resolve_choreography_entity_transform(entity, frame, actor_registry, actor_transforms)
    };
    match resolve_typed_choreography_position(expression, &mut entity_transform, captures) {
        Ok(position) => Some(position),
        Err(error) => {
            issues.push(TutorialChoreographyIssue::ExpressionResolutionUnavailable {
                source_line,
                error,
            });
            None
        }
    }
}

pub(in super::super) fn resolve_choreography_entity_position(
    entity: EntityRef,
    source_line: u32,
    frame: &TutorialChoreographyFrame,
    actor_registry: &TutorialActorRegistry,
    actor_transforms: &Query<&Transform, With<TutorialActor>>,
    issues: &mut TutorialChoreographyIssueQueue,
) -> Option<Vec3> {
    let position =
        resolve_choreography_entity_transform(entity, frame, actor_registry, actor_transforms)
            .map(|transform| transform.translation);
    if position.is_none() {
        issues.push(TutorialChoreographyIssue::MissingEntityReference {
            source_line,
            detail: match entity {
                EntityRef::Npc(_) => "tutorial NPC entity is unavailable",
                EntityRef::NewNano => "tutorial Nano presentation entity is unavailable",
                EntityRef::Player | EntityRef::Camera | EntityRef::StartPosition => {
                    "tutorial entity is unavailable"
                }
            },
        });
    }
    position
}

pub(in super::super) fn resolve_choreography_entity_transform(
    entity: EntityRef,
    frame: &TutorialChoreographyFrame,
    actor_registry: &TutorialActorRegistry,
    actor_transforms: &Query<&Transform, With<TutorialActor>>,
) -> Option<Transform> {
    match entity {
        EntityRef::Player => Some(frame.player.clone()),
        EntityRef::Camera => Some(frame.camera.clone()),
        EntityRef::StartPosition => Some(frame.start.clone()),
        EntityRef::Npc(id) => actor_registry
            .entity(id)
            .and_then(|entity| actor_transforms.get(entity).ok())
            .cloned(),
        // The legacy coroutine addresses the separately-created `newNano`
        // presentation through id -1 helpers, but it never belongs to the NPC
        // container. `frame.nano` is the adapted Bevy gameplay root; formulas
        // such as `newNano.rotation * Vector3.forward` require the rendered
        // Unity-model orientation after the character-basis child is composed.
        EntityRef::NewNano => frame.nano.map(tutorial_nano_choreography_transform),
    }
}

pub(in super::super) fn resolve_choreography_rotation(
    expression: RotationExpr,
    source_line: u32,
    origin: Vec3,
    frame: &TutorialChoreographyFrame,
    captures: &TutorialCameraCaptureStore,
    actor_registry: &TutorialActorRegistry,
    actor_transforms: &Query<&Transform, With<TutorialActor>>,
    issues: &mut TutorialChoreographyIssueQueue,
) -> Option<Quat> {
    let mut entity_transform = |entity| {
        resolve_choreography_entity_transform(entity, frame, actor_registry, actor_transforms)
    };
    match resolve_typed_choreography_rotation(expression, origin, &mut entity_transform, captures) {
        Ok(rotation) => Some(rotation),
        Err(error) => {
            issues.push(TutorialChoreographyIssue::ExpressionResolutionUnavailable {
                source_line,
                error,
            });
            None
        }
    }
}
