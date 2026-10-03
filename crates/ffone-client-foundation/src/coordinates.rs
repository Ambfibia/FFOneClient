//! The stable runtime coordinate contract between OpenFusion, legacy Unity,
//! exported native assets, and Bevy.
//!
//! The asset/world exporters publish the legacy Unity space through
//! `H = diag(-1, 1, 1)`. Runtime positions, velocities, rotations, camera
//! headings, and packet builders must use the same reflection. This module
//! deliberately contains no origin shift, automatic centering, or model scale.

use std::f32::consts::PI;

use bevy::prelude::{Quat, Transform, Vec3};

/// Why a published character root cannot safely be instantiated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegacyCharacterRootError {
    /// Translation, rotation, or scale contains NaN/infinity, or the authored
    /// quaternion has no usable magnitude.
    InvalidAuthoredTransform,
    /// At least one authored scale component is zero or negative.
    NonPositiveAuthoredScale,
    /// An NPC table row supplied a zero, negative, NaN, or infinite scale.
    InvalidNpcTableScale,
}

/// Matches `nativeCoordinateContract.schema` written by FusionForge.
pub const NATIVE_COORDINATE_CONTRACT_SCHEMA: &str = "ffone.native-coordinate-contract.v1";

/// One integer OpenFusion distance unit is one centimetre-like legacy client
/// unit (`CoordUtil.Distance_SToC` / `ServerToClient`).
pub const PROTOCOL_TO_NATIVE_SCALE: f32 = 0.01;

/// Native model files retain the legacy authored +Z forward axis, while a
/// Bevy transform's conventional forward axis is -Z. A model scene attached
/// below a gameplay root therefore needs this local half-turn exactly once.
pub const NATIVE_MODEL_FORWARD_CHILD_YAW_DEGREES: f32 = 180.0;

/// Applies the exporter reflection `H = diag(-1, 1, 1)`.
///
/// `H` is its own inverse, so this is also the exact native-to-Unity vector
/// conversion. It changes handedness but neither recenters nor rescales.
#[must_use]
pub fn unity_to_native_vector(unity: Vec3) -> Vec3 {
    Vec3::new(-unity.x, unity.y, unity.z)
}

/// Applies `H^-1`; for this reflection `H^-1 = H`.
#[must_use]
pub fn native_to_unity_vector(native: Vec3) -> Vec3 {
    unity_to_native_vector(native)
}

/// Unity local scale is preserved component-for-component by the exporter.
/// Reflection belongs to positions/rotations and must not be reintroduced as
/// a negative runtime scale (which would corrupt winding and skinned poses).
#[must_use]
pub fn unity_to_native_scale(unity_scale: Vec3) -> Vec3 {
    unity_scale
}

/// Applies `R_native = H * R_unity * H` to a Unity quaternion.
///
/// This is the same quaternion rule emitted in `nativeCoordinateContract`:
/// `[unity.x, -unity.y, -unity.z, unity.w]`.
#[must_use]
pub fn unity_to_native_rotation(unity: Quat) -> Quat {
    Quat::from_xyzw(unity.x, -unity.y, -unity.z, unity.w)
}

/// Converts an OpenFusion scalar distance without changing its sign or origin.
#[must_use]
pub fn protocol_distance_to_native(distance: i32) -> f32 {
    distance as f32 * PROTOCOL_TO_NATIVE_SCALE
}

/// OpenFusion position fields in wire order `[X, Y, Z]`.
///
/// Legacy `CoordUtil.ServerToClient` first maps them to Unity
/// `[X, Z, Y] * .01`; the native exporter contract then applies `H`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolPosition([i32; 3]);

impl ProtocolPosition {
    #[must_use]
    pub const fn new(raw: [i32; 3]) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> [i32; 3] {
        self.0
    }

    /// `native = [-X, Z, Y] * .01`.
    #[must_use]
    pub fn to_native(self) -> Vec3 {
        unity_to_native_vector(Vec3::new(
            protocol_distance_to_native(self.0[0]),
            protocol_distance_to_native(self.0[2]),
            protocol_distance_to_native(self.0[1]),
        ))
    }

    /// `protocol = trunc([-native.x, native.z, native.y] * 100)`.
    ///
    /// Rust's finite float-to-int cast truncates toward zero, matching the old
    /// C# `(int)(value * 100f)` packet path.
    #[must_use]
    pub fn from_native(native: Vec3) -> Self {
        let unity = native_to_unity_vector(native);
        Self([
            (unity.x * 100.0) as i32,
            (unity.z * 100.0) as i32,
            (unity.y * 100.0) as i32,
        ])
    }
}

impl From<[i32; 3]> for ProtocolPosition {
    fn from(raw: [i32; 3]) -> Self {
        Self::new(raw)
    }
}

impl From<ProtocolPosition> for [i32; 3] {
    fn from(position: ProtocolPosition) -> Self {
        position.raw()
    }
}

/// The unscaled float velocity fields used by normal MOVE packets.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProtocolMoveVelocity([f32; 3]);

impl ProtocolMoveVelocity {
    #[must_use]
    pub const fn new(raw: [f32; 3]) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> [f32; 3] {
        self.0
    }

    /// `native = [-VX, VZ, VY]`.
    #[must_use]
    pub fn to_native(self) -> Vec3 {
        unity_to_native_vector(Vec3::new(self.0[0], self.0[2], self.0[1]))
    }

    /// `protocol = [-native.x, native.z, native.y]`.
    #[must_use]
    pub fn from_native(native: Vec3) -> Self {
        let unity = native_to_unity_vector(native);
        Self([unity.x, unity.z, unity.y])
    }
}

impl From<[f32; 3]> for ProtocolMoveVelocity {
    fn from(raw: [f32; 3]) -> Self {
        Self::new(raw)
    }
}

impl From<ProtocolMoveVelocity> for [f32; 3] {
    fn from(velocity: ProtocolMoveVelocity) -> Self {
        velocity.raw()
    }
}

/// Integer, scaled vector fields used by JUMP packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolScaledVelocity([i32; 3]);

impl ProtocolScaledVelocity {
    #[must_use]
    pub const fn new(raw: [i32; 3]) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> [i32; 3] {
        self.0
    }

    /// `native = [-VX, VZ, VY] * .01`.
    #[must_use]
    pub fn to_native(self) -> Vec3 {
        ProtocolPosition::new(self.0).to_native()
    }

    /// `protocol = trunc([-native.x, native.z, native.y] * 100)`.
    #[must_use]
    pub fn from_native(native: Vec3) -> Self {
        Self(ProtocolPosition::from_native(native).raw())
    }
}

impl From<[i32; 3]> for ProtocolScaledVelocity {
    fn from(raw: [i32; 3]) -> Self {
        Self::new(raw)
    }
}

impl From<ProtocolScaledVelocity> for [i32; 3] {
    fn from(velocity: ProtocolScaledVelocity) -> Self {
        velocity.raw()
    }
}

/// A legacy Unity heading in degrees: yaw of the visual/model +Z forward.
///
/// Keeping this type explicit lets the input/camera port retain the exact
/// legacy signs while every Bevy transform is produced through `H`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LegacyUnityHeadingDegrees(f32);

impl LegacyUnityHeadingDegrees {
    #[must_use]
    pub const fn new(degrees: f32) -> Self {
        Self(degrees)
    }

    #[must_use]
    pub const fn degrees(self) -> f32 {
        self.0
    }

    /// Exact 6877 request rule: `iAngle = (int)fCurAngle - 180`.
    #[must_use]
    pub fn to_protocol(self) -> ProtocolYawDegrees {
        ProtocolYawDegrees::new(self.0 as i32 - 180)
    }

    /// Bevy gameplay-root yaw is `psi = 180 degrees - theta_unity`.
    ///
    /// The half-turn accounts for native model +Z versus Bevy forward -Z; the
    /// minus sign is the global X reflection. The implementation below is the
    /// direct quaternion proof: reflect the old remote root yaw
    /// `(theta_unity - 180 degrees)` with `H * R * H`.
    #[must_use]
    pub fn native_root_rotation(self) -> Quat {
        let legacy_remote_root = Quat::from_rotation_y((self.0 - 180.0).to_radians());
        unity_to_native_rotation(legacy_remote_root)
    }

    /// Physical forward in native world space, obtained from Bevy's -Z axis.
    #[must_use]
    pub fn native_forward(self) -> Vec3 {
        self.native_root_rotation() * Vec3::NEG_Z
    }
}

/// Raw OpenFusion `iAngle`, stored and relayed unchanged by OpenFusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProtocolYawDegrees(i32);

impl ProtocolYawDegrees {
    #[must_use]
    pub const fn new(degrees: i32) -> Self {
        Self(degrees)
    }

    #[must_use]
    pub const fn degrees(self) -> i32 {
        self.0
    }

    /// Inverse of the old packet rule (apart from its intentional sub-degree
    /// truncation): `theta_unity = iAngle + 180 degrees`.
    #[must_use]
    pub fn legacy_heading(self) -> LegacyUnityHeadingDegrees {
        LegacyUnityHeadingDegrees::new(self.0 as f32 + 180.0)
    }

    /// Native root yaw is exactly `psi = -iAngle` for an integer packet yaw.
    #[must_use]
    pub fn native_root_rotation(self) -> Quat {
        self.legacy_heading().native_root_rotation()
    }
}

impl From<i32> for ProtocolYawDegrees {
    fn from(degrees: i32) -> Self {
        Self::new(degrees)
    }
}

impl From<ProtocolYawDegrees> for i32 {
    fn from(yaw: ProtocolYawDegrees) -> Self {
        yaw.degrees()
    }
}

/// Local rotation for a published native model scene attached to a gameplay
/// root. It maps authored +Z onto Bevy root forward -Z exactly once.
#[must_use]
pub fn native_model_forward_child_rotation() -> Quat {
    Quat::from_rotation_y(PI)
}

/// Exact local-root replacement performed by the legacy character spawners.
///
/// A published GLB intentionally retains the authored Unity root TRS so the
/// asset remains lossless and inspectable.  That is separate from the runtime
/// behavior of `NpcMoveController`, `NanoMoveController`, `cnAvatarStatus`,
/// and `cnBusMoveController`: all four instantiate the model at the owning gameplay transform, which
/// makes the model root's local translation zero and its local rotation the
/// identity. They differ only in the final local scale.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LegacyCharacterRootPolicy {
    /// `NpcMoveController.SetupNPC`: replace authored scale with `m_fScale`.
    Npc { table_scale: f32 },
    /// `NanoMoveController.SetupModel`: retain the prefab's authored scale.
    Nano,
    /// `cnAvatarStatus`: force the assembled player root to `Vector3.one`.
    Player,
    /// `cnBusMoveController.SetupBus`: parent the loaded transport model to
    /// the bus prefab while retaining the model prefab's authored scale.
    Transportation,
    /// `cnShinyController.SetupShiny`: preserve prefab scale at server position.
    Shiny,
}

impl LegacyCharacterRootPolicy {
    /// Validates the lossless authored root and then resolves the legacy
    /// runtime replacement. Publication normally catches these conditions;
    /// keeping the same gate in Bevy prevents a malformed or hand-edited GLB
    /// from silently producing a mirrored, collapsed, or exploding rig.
    pub fn try_resolve_root(
        self,
        authored_root: Transform,
    ) -> Result<Transform, LegacyCharacterRootError> {
        if !vec3_is_finite(authored_root.translation)
            || !quat_is_finite_and_normalized(authored_root.rotation)
            || !vec3_is_finite(authored_root.scale)
        {
            return Err(LegacyCharacterRootError::InvalidAuthoredTransform);
        }
        if authored_root.scale.min_element() <= 0.0 {
            return Err(LegacyCharacterRootError::NonPositiveAuthoredScale);
        }
        if let Self::Npc { table_scale } = self
            && (!table_scale.is_finite() || table_scale <= 0.0)
        {
            return Err(LegacyCharacterRootError::InvalidNpcTableScale);
        }
        Ok(self.resolve_root(authored_root))
    }

    /// Resolves the model root TRS that must be applied after the Bevy glTF
    /// scene has instantiated its authored root node.
    ///
    /// This deliberately does not apply the model-forward half-turn.  That
    /// rotation belongs to the stable model-scene container, while this value
    /// replaces the imported root node exactly like the old Unity spawners.
    #[must_use]
    pub fn resolve_root(self, authored_root: Transform) -> Transform {
        let scale = match self {
            Self::Npc { table_scale } => Vec3::splat(table_scale),
            Self::Nano | Self::Transportation | Self::Shiny => authored_root.scale,
            Self::Player => Vec3::ONE,
        };
        Transform {
            translation: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale,
        }
    }
}

fn vec3_is_finite(value: Vec3) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.z.is_finite()
}

fn quat_is_finite_and_normalized(value: Quat) -> bool {
    value.x.is_finite()
        && value.y.is_finite()
        && value.z.is_finite()
        && value.w.is_finite()
        // `Transform` rotations must be unit quaternions. Merely rejecting a
        // zero quaternion would allow a hand-edited GLB to cross the scene
        // gate with an invalid authored TRS before the typed spawn policy
        // replaces it. Keep the tolerance large enough for f32 serialization
        // noise, but fail closed on materially non-unit input.
        && (value.length_squared() - 1.0).abs() <= 0.000_1
}

#[cfg(test)]
mod tests;
