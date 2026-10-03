//! Exact local-transform policies for player equipment and scripted NPC
//! cosmetics.
//!
//! Attachments already inherit the character visual container and skeleton.
//! They must never receive the character `Ry(180 degrees)` a second time.

use bevy::prelude::{Quat, Transform, Vec3};
use ffone_runtime_contracts::PlayerRigGender;

use crate::coordinates::{unity_to_native_rotation, unity_to_native_vector};

/// The six attachment slots constructed by legacy `ActorSkinCombiner`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyPlayerAttachmentSlot {
    Hat,
    Glasses,
    LeftPistol,
    RightPistol,
    Back,
    Zipline,
    Vehicle,
}

impl LegacyPlayerAttachmentSlot {
    /// Exact skeleton path passed to `Transform.Find` by
    /// `ActorSkinCombiner.Generate`.
    #[must_use]
    pub const fn socket_path(self) -> &'static str {
        match self {
            Self::Vehicle => "Bip01/Bip01 NonAccum/Bip01 Pelvis/Bip01 R Thigh/Bip01 R Calf/Bip01 R Foot/vehicle",
            Self::Hat => {
                "Bip01/Bip01 NonAccum/Bip01 Pelvis/Bip01 Spine/Bip01 Spine1/Bip01 Neck/Bip01 Head/Bip01 helmet01"
            }
            Self::Glasses => {
                "Bip01/Bip01 NonAccum/Bip01 Pelvis/Bip01 Spine/Bip01 Spine1/Bip01 Neck/Bip01 Head/Bip01 glass01"
            }
            Self::LeftPistol => {
                "Bip01/Bip01 NonAccum/Bip01 Pelvis/Bip01 Spine/Bip01 Spine1/Bip01 Neck/Bip01 L Clavicle/Bip01 L UpperArm/Bip01 L Forearm/Bip01 L Hand/Bip01 Lweapon01"
            }
            Self::RightPistol | Self::Zipline => {
                "Bip01/Bip01 NonAccum/Bip01 Pelvis/Bip01 Spine/Bip01 Spine1/Bip01 Neck/Bip01 R Clavicle/Bip01 R UpperArm/Bip01 R Forearm/Bip01 R Hand/Bip01 Rweapon01"
            }
            Self::Back => "Bip01/Bip01 NonAccum/Bip01 Pelvis/Bip01 Spine/Bip01 Spine1/Bip01 back01",
        }
    }
}

/// Exact instance-local shared-rig path for one legacy attachment socket.
///
/// Modular player-part scenes retain duplicate joint names after their meshes
/// are rebound to the shared skeleton. Attachment consumers must resolve this
/// full path through [`crate::player_shared_rig::NativePlayerRigBones`] rather
/// than selecting the first descendant with a matching [`bevy::prelude::Name`].
#[must_use]
pub fn player_attachment_socket_full_path(
    gender: PlayerRigGender,
    slot: LegacyPlayerAttachmentSlot,
) -> String {
    let root = match gender {
        PlayerRigGender::Male => "m",
        PlayerRigGender::Female => "w",
    };
    format!("{root}/{}", slot.socket_path())
}

/// Local item transform plus the socket-scale overwrite performed by the old
/// combiner. `None` means the owning script did not overwrite socket scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeAttachmentPlacement {
    pub item_local: Transform,
    pub socket_local_scale_override: Option<Vec3>,
}

/// Why a scripted local attachment transform cannot be represented safely.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyAttachmentTransformError {
    NonFinite,
}

/// `ActorSkinCombiner.AttachGO`: zero position, `Quaternion.Euler(90,0,0)`,
/// item scale one, and socket scale one.
#[must_use]
pub fn standard_player_attachment_placement() -> NativeAttachmentPlacement {
    NativeAttachmentPlacement {
        item_local: native_attachment_transform(
            Vec3::ZERO,
            unity_quaternion_euler(Vec3::new(90.0, 0.0, 0.0)),
            Vec3::ONE,
        )
        .expect("the fixed legacy attachment transform is finite"),
        socket_local_scale_override: Some(Vec3::ONE),
    }
}

/// `NpcCustomization.CustomizeNpcModel`: preserve its explicit per-option
/// position, Euler rotation, and scale after the global basis conversion.
///
/// Unlike character roots, negative/non-uniform cosmetic scale is legal here:
/// the shipped scripts deliberately use it. A zero bone scale is likewise a
/// separate legacy tail-hiding operation, not a malformed model root.
pub fn scripted_npc_cosmetic_placement(
    unity_position: Vec3,
    unity_euler_degrees: Vec3,
    unity_scale: Vec3,
) -> Result<NativeAttachmentPlacement, LegacyAttachmentTransformError> {
    Ok(NativeAttachmentPlacement {
        item_local: native_attachment_transform(
            unity_position,
            unity_quaternion_euler(unity_euler_degrees),
            unity_scale,
        )?,
        socket_local_scale_override: None,
    })
}

/// Exact zero-scale operation used by `NpcCustomization` when `hideTail` is
/// true. It is intentionally not accepted by the character-root policy.
#[must_use]
pub const fn scripted_hidden_tail_scale() -> Vec3 {
    Vec3::ZERO
}

fn native_attachment_transform(
    unity_position: Vec3,
    unity_rotation: Quat,
    unity_scale: Vec3,
) -> Result<Transform, LegacyAttachmentTransformError> {
    if !vec3_is_finite(unity_position)
        || !quat_is_finite_and_nonzero(unity_rotation)
        || !vec3_is_finite(unity_scale)
    {
        return Err(LegacyAttachmentTransformError::NonFinite);
    }
    Ok(Transform {
        translation: unity_to_native_vector(unity_position),
        rotation: unity_to_native_rotation(unity_rotation),
        // H is a basis conversion, not an extra negative object scale.
        scale: unity_scale,
    })
}

/// Unity documents `Quaternion.Euler(x,y,z)` as applying Z, then X, then Y.
/// With column vectors the composed quaternion is therefore `Qy * Qx * Qz`.
fn unity_quaternion_euler(degrees: Vec3) -> Quat {
    let radians = Vec3::new(
        degrees.x.to_radians(),
        degrees.y.to_radians(),
        degrees.z.to_radians(),
    );
    Quat::from_rotation_y(radians.y)
        * Quat::from_rotation_x(radians.x)
        * Quat::from_rotation_z(radians.z)
}

fn vec3_is_finite(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

fn quat_is_finite_and_nonzero(value: Quat) -> bool {
    value.x.is_finite()
        && value.y.is_finite()
        && value.z.is_finite()
        && value.w.is_finite()
        && value.length_squared() > f32::EPSILON
}

#[cfg(test)]
mod tests;
