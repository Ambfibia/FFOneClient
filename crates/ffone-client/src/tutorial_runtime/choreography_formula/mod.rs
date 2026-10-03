//! Typed evaluation of the live transform expressions recovered from
//! `cntutorialscript.cs`.
//!
//! Choreography data stays in legacy Unity coordinates. This adapter performs
//! the reflection exactly once and resolves live/captured transforms supplied
//! by the ECS caller. There is deliberately no string parser or best-effort
//! fallback: every supported expression is represented by a closed enum.

use bevy::prelude::{Quat, Resource, Transform, Vec3};

use crate::{
    coordinates::{unity_to_native_rotation, unity_to_native_vector},
    tutorial_choreography::{
        CameraCaptureSlot, ClientVec3, EntityRef, OrientationBasis, PositionAnchor, PositionExpr,
        RotationExpr,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoreographyFormulaError {
    MissingEntity(EntityRef),
    MissingCameraCapture(CameraCaptureSlot),
    DegenerateLookDirection,
}

#[derive(Debug, Clone, Default, Resource)]
pub struct TutorialCameraCaptureStore {
    infection_return_start: Option<Transform>,
    infection_entry: Option<Transform>,
}

impl TutorialCameraCaptureStore {
    pub fn capture(&mut self, slot: CameraCaptureSlot, transform: Transform) {
        match slot {
            CameraCaptureSlot::InfectionReturnStart => {
                self.infection_return_start = Some(transform);
            }
            CameraCaptureSlot::InfectionEntry => {
                self.infection_entry = Some(transform);
            }
        }
    }

    #[must_use]
    pub fn get(&self, slot: CameraCaptureSlot) -> Option<Transform> {
        match slot {
            CameraCaptureSlot::InfectionReturnStart => self.infection_return_start,
            CameraCaptureSlot::InfectionEntry => self.infection_entry,
        }
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// Resolve a typed choreography position into native world coordinates.
///
/// `entity_transform` returns the live native gameplay transform for player,
/// camera, NPC, Nano, or named references. Captures are native camera
/// transforms recorded when `CameraAction::CaptureTransform` is dispatched.
pub fn resolve_position(
    expression: PositionExpr,
    entity_transform: &mut impl FnMut(EntityRef) -> Option<Transform>,
    captures: &TutorialCameraCaptureStore,
) -> Result<Vec3, ChoreographyFormulaError> {
    match expression {
        PositionExpr::Client(value) => Ok(native_vector(value)),
        PositionExpr::Entity(entity) => Ok(required_entity(entity, entity_transform)?.translation),
        PositionExpr::EntityOffset { entity, offset } => {
            Ok(required_entity(entity, entity_transform)?.translation + native_vector(offset))
        }
        PositionExpr::Midpoint {
            left,
            right,
            offset,
        } => {
            let left = required_entity(left, entity_transform)?.translation;
            let right = required_entity(right, entity_transform)?.translation;
            Ok((left + right) * 0.5 + native_vector(offset))
        }
        PositionExpr::OrientedOffset {
            origin,
            world_offset,
            orientation,
            local_offset,
        } => {
            let origin = resolve_anchor(origin, entity_transform, captures)?;
            let local =
                resolve_oriented_offset(orientation, local_offset, entity_transform, captures)?;
            Ok(origin + native_vector(world_offset) + local)
        }
    }
}

/// Resolve a typed choreography rotation into the native coordinate contract.
pub fn resolve_rotation(
    expression: RotationExpr,
    origin: Vec3,
    entity_transform: &mut impl FnMut(EntityRef) -> Option<Transform>,
    captures: &TutorialCameraCaptureStore,
) -> Result<Quat, ChoreographyFormulaError> {
    match expression {
        RotationExpr::Euler(value) => Ok(native_euler(value)),
        RotationExpr::FaceEntity(entity) => {
            let target = required_entity(entity, entity_transform)?.translation;
            native_look_at(origin, target)
        }
        RotationExpr::FacePosition(position) => {
            let target = resolve_position(position, entity_transform, captures)?;
            native_look_at(origin, target)
        }
        RotationExpr::EntityYaw {
            entity,
            pitch,
            yaw_offset,
            roll,
        } => {
            let transform = required_entity(entity, entity_transform)?;
            let native_forward = transform.rotation * Vec3::NEG_Z;
            let yaw = legacy_unity_yaw(native_forward)?;
            Ok(native_euler(ClientVec3::new(pitch, yaw + yaw_offset, roll)))
        }
        RotationExpr::LookAtYaw {
            target,
            pitch,
            yaw_offset,
            roll,
        } => {
            let native_target = native_vector(target);
            let yaw = legacy_unity_yaw(native_target - origin)?;
            Ok(native_euler(ClientVec3::new(pitch, yaw + yaw_offset, roll)))
        }
    }
}

fn resolve_anchor(
    anchor: PositionAnchor,
    entity_transform: &mut impl FnMut(EntityRef) -> Option<Transform>,
    captures: &TutorialCameraCaptureStore,
) -> Result<Vec3, ChoreographyFormulaError> {
    match anchor {
        PositionAnchor::Client(value) => Ok(native_vector(value)),
        PositionAnchor::Entity(entity) => {
            Ok(required_entity(entity, entity_transform)?.translation)
        }
        PositionAnchor::Midpoint { left, right } => {
            let left = required_entity(left, entity_transform)?.translation;
            let right = required_entity(right, entity_transform)?.translation;
            Ok((left + right) * 0.5)
        }
        PositionAnchor::CapturedCamera(slot) => captures
            .get(slot)
            .map(|transform| transform.translation)
            .ok_or(ChoreographyFormulaError::MissingCameraCapture(slot)),
    }
}

fn resolve_oriented_offset(
    orientation: OrientationBasis,
    local: ClientVec3,
    entity_transform: &mut impl FnMut(EntityRef) -> Option<Transform>,
    captures: &TutorialCameraCaptureStore,
) -> Result<Vec3, ChoreographyFormulaError> {
    let local = Vec3::new(local.x, local.y, local.z);
    match orientation {
        OrientationBasis::Euler(euler) => Ok(native_euler(euler) * unity_to_native_vector(local)),
        OrientationBasis::Entity(entity) => {
            let transform = required_entity(entity, entity_transform)?;
            let right = transform.rotation * Vec3::X;
            let up = transform.rotation * Vec3::Y;
            let forward = transform.rotation * Vec3::NEG_Z;
            Ok(right * local.x + up * local.y + forward * local.z)
        }
        OrientationBasis::CapturedCameraYaw(slot) => {
            let transform = captures
                .get(slot)
                .ok_or(ChoreographyFormulaError::MissingCameraCapture(slot))?;
            let mut forward = transform.rotation * Vec3::NEG_Z;
            forward.y = 0.0;
            forward = forward
                .try_normalize()
                .ok_or(ChoreographyFormulaError::DegenerateLookDirection)?;
            let right = forward.cross(Vec3::Y);
            Ok(right * local.x + Vec3::Y * local.y + forward * local.z)
        }
    }
}

fn required_entity(
    entity: EntityRef,
    entity_transform: &mut impl FnMut(EntityRef) -> Option<Transform>,
) -> Result<Transform, ChoreographyFormulaError> {
    entity_transform(entity).ok_or(ChoreographyFormulaError::MissingEntity(entity))
}

fn native_vector(value: ClientVec3) -> Vec3 {
    unity_to_native_vector(Vec3::new(value.x, value.y, value.z))
}

fn native_euler(value: ClientVec3) -> Quat {
    // Unity's Quaternion.Euler(x, y, z) applies Z, then X, then Y. Glam's
    // `from_euler` parameters are always named x/y/z even when a different
    // order is selected, which made the former ZXY call silently feed each
    // angle to the wrong axis. In particular Euler(10, 270, 0) lost its pitch
    // and drove the BasicMove return camera through the ground.
    let unity = Quat::from_rotation_y(value.y.to_radians())
        * Quat::from_rotation_x(value.x.to_radians())
        * Quat::from_rotation_z(value.z.to_radians());
    unity_to_native_rotation(unity)
}

fn native_look_at(origin: Vec3, target: Vec3) -> Result<Quat, ChoreographyFormulaError> {
    if (target - origin).length_squared() <= f32::EPSILON {
        return Err(ChoreographyFormulaError::DegenerateLookDirection);
    }
    Ok(Transform::from_translation(origin)
        .looking_at(target, Vec3::Y)
        .rotation)
}

fn legacy_unity_yaw(native_direction: Vec3) -> Result<f32, ChoreographyFormulaError> {
    let mut unity = unity_to_native_vector(native_direction);
    unity.y = 0.0;
    unity = unity
        .try_normalize()
        .ok_or(ChoreographyFormulaError::DegenerateLookDirection)?;
    Ok(unity.x.atan2(unity.z).to_degrees())
}

#[cfg(test)]
mod tests;
