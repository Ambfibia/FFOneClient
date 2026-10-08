use super::*;

#[derive(Clone, Debug)]
pub(in super::super) struct NativeClosureBlocker {
    pub asset: String,
    pub path_id: i64,
    pub object_type: String,
    pub reason: TutorialNativeClosureBlockerReason,
}

#[derive(Clone, Debug)]
pub(in super::super) struct NativeCompileResult<T> {
    pub plan: Option<T>,
    pub blockers: Vec<NativeClosureBlocker>,
}

#[derive(Clone, Debug)]
pub(in super::super) struct NativeEffectPlan {
    pub rendered_nodes: usize,
    pub(super) emitters: Vec<EmitterPlan>,
    pub(super) mesh_scene: Option<&'static str>,
    pub(super) material_animation: Option<MaterialAnimationPlan>,
    pub(super) maximum_timer: f32,
    pub(super) longest_lifetime: f32,
    pub(super) disable_update: bool,
}

#[derive(Clone, Debug)]
pub(in super::super) struct NativeProjectilePlan {
    pub rendered_nodes: usize,
    pub(super) trail: Option<TrailPlan>,
    pub(super) mesh_scene: Option<&'static str>,
    pub(super) material_animation: Option<MaterialAnimationPlan>,
}

#[derive(Clone, Debug)]
pub(in super::super) struct NativeLinearImpactPlan {
    pub instance_id: u64,
    pub effect_id: i32,
    pub scale: f32,
    pub sound_path: Option<String>,
    pub plan: NativeEffectPlan,
}

#[derive(Clone, Debug)]
pub(super) struct EmitterPlan {
    pub(super) source_asset: String,
    pub(super) source_path_id: i64,
    pub(super) generation: EmitterGeneration,
    pub(super) random_position: f32,
    pub(super) random_angle: f32,
    pub(super) random_velocity: f32,
    // Kept in Unity local space until `ConfigureParticleSettings` has applied
    // its Unity-space random rotation. The completed position and velocity are
    // reflected into the native coordinate contract together.
    pub(super) initial_velocity: Vec3,
    pub(super) plane: Vec3,
    pub(super) initial_translation: Vec3,
    pub(super) script_keys: Vec<ScriptKey>,
    pub(super) initial_emit: bool,
    pub(super) generations_per_second: f32,
    pub(super) number_per_generation: f32,
    pub(super) lifetime: f32,
    pub(super) initial_size: f32,
    // ParticleAnimator.force is a Unity world-space vector and is therefore
    // stored in native coordinates for the detached Bevy particle update.
    pub(super) force: Vec3,
    pub(super) colors: [Vec4; 5],
    pub(super) animate_color: bool,
    pub(super) uv_tiles: UVec2,
    pub(super) uv_cycles: f32,
    pub(super) width_curve: AnimationCurve,
    pub(super) height_curve: AnimationCurve,
    pub(super) rotation_curve: AnimationCurve,
    pub(super) render_mode: LegacyParticleRenderMode,
    pub(super) blend_mode: ParticleBlendMode,
    pub(super) material_tint: Vec4,
    pub(super) legacy_gamma_accumulation_gain: f32,
    pub(super) texture: ExactTexture,
}

/// `ParticleEmitterController.EmitterType` from the Retrobution first-pass
/// assembly. `GEN_TRAIL` is compiled by the dedicated projectile path.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum EmitterGeneration {
    Point,
    InversePoint,
    Plane,
    XPlaneDonut,
    ZPlaneDonut,
    YPlaneDonut,
    InverseXPlaneDonut,
    InverseZPlaneDonut,
    InverseYPlaneDonut,
    OutwardXPlaneDonut,
    OutwardZPlaneDonut,
    OutwardYPlaneDonut,
}

impl TryFrom<i64> for EmitterGeneration {
    type Error = String;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Point),
            1 => Ok(Self::InversePoint),
            2 => Ok(Self::Plane),
            3 => Ok(Self::XPlaneDonut),
            4 => Ok(Self::ZPlaneDonut),
            5 => Ok(Self::YPlaneDonut),
            6 => Ok(Self::InverseXPlaneDonut),
            7 => Ok(Self::InverseZPlaneDonut),
            8 => Ok(Self::InverseYPlaneDonut),
            9 => Ok(Self::OutwardXPlaneDonut),
            10 => Ok(Self::OutwardZPlaneDonut),
            11 => Ok(Self::OutwardYPlaneDonut),
            12 => Err("GEN_TRAIL requires the exact projectile-trail compiler".to_owned()),
            other => Err(format!(
                "unknown ParticleEmitterController m_iGenType {other}"
            )),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ScriptKey {
    pub(super) time: f32,
    pub(super) translate: Vec3,
    pub(super) emit: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CurveKey {
    pub(super) time: f32,
    pub(super) value: f32,
    pub(super) in_slope: f32,
    pub(super) out_slope: f32,
}

#[derive(Clone, Debug)]
pub(super) struct TrailPlan {
    pub(super) source_asset: String,
    pub(super) source_path_id: i64,
    pub(super) length: usize,
    pub(super) height: f32,
    pub(super) interpolation_step: f32,
    pub(super) blend_mode: ParticleBlendMode,
    pub(super) material_tint: Vec4,
    pub(super) texture: ExactTexture,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in super::super) enum NativeLinearProjectileMotion {
    BulletMove {
        hide_seconds: f32,
        duration_seconds: f32,
    },
    Warhead {
        speed: f32,
        initial_vertical_speed: Option<f32>,
        duration_seconds: f32,
        authority: Option<super::super::WarheadAuthority>,
    },
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct TutorialParticlePipelineKey {
    pub(super) blend_mode: ParticleBlendMode,
}

impl From<&TutorialParticleMaterial> for TutorialParticlePipelineKey {
    fn from(material: &TutorialParticleMaterial) -> Self {
        Self {
            blend_mode: material.blend_mode,
        }
    }
}

#[derive(Default, Resource)]
pub(super) struct NativeVisualAssets {
    pub(super) quad: Option<Handle<Mesh>>,
    pub(super) textures: BTreeMap<(String, i64, String), Handle<Image>>,
    pub(super) particle_materials: BTreeMap<ParticleMaterialCacheKey, Handle<TutorialParticleMaterial>>,
    pub(super) sword_trail_texture: Option<Handle<Image>>,
    pub(super) sword_trail_material: Option<Handle<TutorialParticleMaterial>>,
    pub(super) sword_trail_sampler_configured: bool,
}

#[derive(Default, Resource)]
pub(super) struct NativeProjectileVisualPrewarm {
    pub(super) initialized: bool,
    pub(super) camera_frames: u8,
}

#[derive(Component)]
pub(super) struct NativeProjectileVisualPrewarmSurface;

/// Strong handles created by Retrobution's `PreloadEffect` equivalent.
///
/// `AssetServer::load` alone is not a preload unless the returned handle is
/// retained. ES668 is requested one second before it is instantiated; keeping
/// both the GLTF and its scene alive here lets the distant rigid waypoint
/// appear immediately instead of only after the player has approached it.
#[derive(Default, Resource)]
pub(super) struct NativeEffectPreloadCache {
    pub(super) gltfs: BTreeMap<i32, Handle<Gltf>>,
    pub(super) scenes: BTreeMap<i32, Handle<WorldAsset>>,
    pub(super) paths: BTreeMap<i32, &'static str>,
}

/// App-lifetime compatibility stream for the legacy Unity particle emitters.
///
/// Retrobution consumes Unity's app-global `Random` stream, but neither its
/// seed nor its internal state is serialized. We preserve the proven call
/// order, inclusive unit interval, distributions, and emitter formulas while
/// keeping the stream alive across tutorial transitions.
#[derive(Debug, Resource)]
pub(super) struct NativeParticleRandomStream {
    pub(super) state: u32,
}

impl Default for NativeParticleRandomStream {
    fn default() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let folded = nanos ^ (nanos >> 32) ^ (nanos >> 64) ^ (nanos >> 96);
        Self::with_seed(folded as u32)
    }
}

impl NativeParticleRandomStream {
    pub(super) const FALLBACK_SEED: u32 = 0x7e57_372f;

    pub(super) fn with_seed(seed: u32) -> Self {
        Self {
            state: if seed == 0 { Self::FALLBACK_SEED } else { seed },
        }
    }

    pub(super) fn unit(&mut self) -> f32 {
        let value = advance_native_xorshift32(&mut self.state);
        (f64::from(value) / f64::from(u32::MAX)) as f32
    }

    pub(super) fn on_unit_sphere(&mut self) -> Vec3 {
        let z = self.unit() * 2.0 - 1.0;
        let angle = self.unit() * std::f32::consts::TAU;
        let radius = (1.0 - z * z).max(0.0).sqrt();
        Vec3::new(radius * angle.cos(), radius * angle.sin(), z)
    }

    pub(super) fn inside_unit_circle_direction(&mut self) -> Vec2 {
        loop {
            let candidate = Vec2::new(self.unit() * 2.0 - 1.0, self.unit() * 2.0 - 1.0);
            let length_squared = candidate.length_squared();
            if length_squared <= 1.0 {
                return if length_squared > 0.0 {
                    candidate / length_squared.sqrt()
                } else {
                    Vec2::ZERO
                };
            }
        }
    }
}

#[derive(Component)]
pub(super) struct NativeRoot {
    pub(super) instance_id: u64,
}

#[derive(Component)]
pub(super) struct NativeDetachedOwner(pub(super) u64);

#[derive(Component)]
pub(super) struct NativeEffectRoot {
    pub(super) stream_owner: Option<Entity>,
    pub(super) age: f32,
    pub(super) destroy_after: Option<f32>,
    pub(super) natural_destroy_after: Option<f32>,
    pub(super) material_animation: Option<MaterialAnimationPlan>,
    pub(super) waiting_for_mesh_surface: bool,
}

impl NativeEffectRoot {
    pub(super) fn advance_age(&mut self, delta: f32) {
        if !self.waiting_for_mesh_surface {
            self.age += delta;
        }
    }

    pub(super) fn requires_mesh_surface_preparation(&self) -> bool {
        self.waiting_for_mesh_surface
    }
}

#[derive(Component, Default)]
pub(super) struct NativeEffectBillboard {
    pub(super) upright: bool,
}

#[derive(Component)]
pub(super) struct NativeEmitter {
    pub(super) owner: u64,
    pub(super) stream_owner: Option<Entity>,
    pub(super) plan: Arc<EmitterPlan>,
    pub(super) static_material: Option<Handle<TutorialParticleMaterial>>,
    pub(super) stream_owned: bool,
    pub(super) preserve_when_streamed_budget_is_full: bool,
    pub(super) scale: f32,
    pub(super) age: f32,
    pub(super) timer: f32,
    pub(super) maximum_timer: f32,
    pub(super) disable_update: bool,
}

#[derive(Component)]
pub(super) struct NativeParticle {
    pub(super) stream_owner: Option<Entity>,
    pub(super) plan: Arc<EmitterPlan>,
    pub(super) scale: f32,
    pub(super) age: f32,
    pub(super) velocity: Vec3,
    pub(super) stream_owned: bool,
    pub(super) alive: bool,
}

#[derive(Component)]
pub(super) struct NativeProjectile {
    pub(super) target: Vec3,
    pub(super) motion: NativeProjectileMotion,
    pub(super) impact: Option<NativeLinearImpactPlan>,
    /// The clean client instantiates an already-loaded NIF before starting
    /// BulletMoveScript/cnWarHead. Hold the native clock until Bevy has a real
    /// converted surface, otherwise a 0.3 s projectile can despawn while its
    /// asynchronous GLB is still invisible.
    pub(super) waiting_for_mesh_surface: bool,
    pub(super) waiting_for_trail_prewarm: bool,
}

pub(super) enum NativeProjectileMotion {
    Oni(TutorialOniProjectileState),
    Linear {
        position: Vec3,
        hide_remaining: f32,
        elapsed: f32,
        duration: f32,
    },
    Warhead {
        position: Vec3,
        velocity: Vec3,
        gravity: bool,
        authority: Option<super::super::WarheadAuthority>,
        elapsed: f32,
        duration: f32,
    },
}

impl NativeProjectile {
    pub(super) fn position(&self) -> Vec3 {
        match &self.motion {
            NativeProjectileMotion::Oni(state) => state.position,
            NativeProjectileMotion::Linear { position, .. } => *position,
            NativeProjectileMotion::Warhead { position, .. } => *position,
        }
    }
}

#[derive(Component)]
pub(super) struct NativeTrail {
    pub(super) projectile: Entity,
    pub(super) plan: TrailPlan,
    pub(super) scale: f32,
    pub(super) points: Vec<Vec3>,
    pub(super) elapsed: f32,
}

#[derive(Component)]
pub(super) struct NativeSwordTrailBound;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct NativeSwordTrailEdge {
    pub(super) top: Vec3,
    pub(super) bottom: Vec3,
}

#[derive(Component)]
pub(super) struct NativeSwordTrail {
    pub(super) attachment: Entity,
    pub(super) controller_root: Entity,
    pub(super) rig_root: Entity,
    pub(super) top_point: Entity,
    pub(super) bottom_point: Entity,
    pub(super) edges: Vec<NativeSwordTrailEdge>,
    pub(super) emitting: bool,
    pub(super) attack_generation: Option<u64>,
    pub(super) elapsed_seconds: f32,
}
