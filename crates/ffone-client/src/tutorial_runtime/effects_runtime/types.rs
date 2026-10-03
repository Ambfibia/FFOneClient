use super::*;

/// Validated, immutable exact source contract.  Presence of this resource does
/// not claim that the native particle renderer exists.
#[derive(Clone, Debug, Resource)]
pub struct TutorialEffectLibrary {
    pub(super) root: PathBuf,
    pub(super) effect_catalog: TutorialEffectCatalog,
    pub(super) projectile_catalog: TutorialProjectileCatalog,
    pub(super) effects: BTreeMap<i32, ValidatedEffect>,
    pub(super) projectile_effects: BTreeMap<i32, ValidatedEffect>,
    pub(super) bullets: BTreeMap<i32, TutorialBulletRowFile>,
}

#[derive(Clone, Debug)]
pub(super) struct ValidatedEffect {
    pub(super) entry: TutorialEffectCatalogEntry,
    pub(super) closure: TutorialEffectClosureFile,
}

impl TutorialEffectLibrary {
    pub fn load(asset_root: impl AsRef<Path>) -> Result<Self, TutorialEffectLibraryError> {
        let root = fs::canonicalize(asset_root.as_ref()).map_err(|error| {
            TutorialEffectLibraryError(format!(
                "could not canonicalize tutorial asset root {}: {error}",
                asset_root.as_ref().display()
            ))
        })?;
        reject_symlink(&root, "asset root")?;
        let effect_catalog: TutorialEffectCatalog = read_json(&root, TUTORIAL_EFFECT_CATALOG_PATH)?;
        let projectile_catalog: TutorialProjectileCatalog =
            read_json(&root, TUTORIAL_PROJECTILE_CATALOG_PATH)?;
        validate_catalog_headers(&effect_catalog, &projectile_catalog)?;

        let actual_effect_ids = effect_catalog
            .effects
            .iter()
            .map(|entry| entry.effect_id)
            .collect::<BTreeSet<_>>();
        let expected_effect_ids = RETROBUTION_TUTORIAL_EFFECT_IDS
            .into_iter()
            .chain(RETROBUTION_FUSION_ACTOR_EFFECT_IDS)
            .chain(RETROBUTION_CHARACTER_ACTOR_EFFECT_IDS)
            .chain(RETROBUTION_WORLD_EP_EFFECT_IDS)
            .chain(RETROBUTION_PLAYER_STATUS_EFFECT_IDS)
            .chain(RETROBUTION_NPC_WARP_EFFECT_IDS)
            .chain(RETROBUTION_WEAPON_EFFECT_IDS)
            .chain(RETROBUTION_NPC_GAME_ICON_EFFECT_IDS)
            .collect::<BTreeSet<_>>();
        if actual_effect_ids != expected_effect_ids
            || effect_catalog.effects.len() != expected_effect_ids.len()
        {
            return fail(format!(
                "tutorial effect catalog IDs differ: actual={actual_effect_ids:?}, expected={expected_effect_ids:?}"
            ));
        }

        let mut effects = BTreeMap::new();
        for entry in &effect_catalog.effects {
            let expected_route = format!(
                "prefabs/particle/effectscripts/es[{}].prefab",
                entry.effect_id
            );
            let expected_path = format!("map/shared/effects/es{}.closure.json", entry.effect_id);
            if entry.container_route != expected_route || entry.closure_path != expected_path {
                return fail(format!(
                    "effect {} uses non-canonical route/path: {:?}, {:?}",
                    entry.effect_id, entry.container_route, entry.closure_path
                ));
            }
            let bytes = read_verified(
                &root,
                &entry.closure_path,
                entry.closure_bytes,
                &entry.closure_blake3,
            )?;
            let closure: TutorialEffectClosureFile =
                serde_json::from_slice(&bytes).map_err(|error| {
                    TutorialEffectLibraryError(format!(
                        "invalid effect closure {:?}: {error}",
                        entry.closure_path
                    ))
                })?;
            validate_effect_closure(
                entry,
                &closure,
                &effect_catalog.source_bundle,
                &effect_catalog.source_dump,
                &effect_catalog.source_assets,
            )?;
            if effects
                .insert(
                    entry.effect_id,
                    ValidatedEffect {
                        entry: entry.clone(),
                        closure,
                    },
                )
                .is_some()
            {
                return fail(format!("duplicate effect {}", entry.effect_id));
            }
        }

        let actual_projectile_effect_ids = projectile_catalog
            .particle_effects
            .iter()
            .map(|entry| entry.effect_id)
            .collect::<BTreeSet<_>>();
        let expected_projectile_effect_ids = RETROBUTION_TUTORIAL_PROJECTILE_EFFECT_IDS
            .into_iter()
            .collect::<BTreeSet<_>>();
        if actual_projectile_effect_ids != expected_projectile_effect_ids
            || projectile_catalog.particle_effects.len() != expected_projectile_effect_ids.len()
        {
            return fail(format!(
                "tutorial projectile effect IDs differ: actual={actual_projectile_effect_ids:?}, expected={expected_projectile_effect_ids:?}"
            ));
        }
        let mut projectile_effects = BTreeMap::new();
        for entry in &projectile_catalog.particle_effects {
            let expected_route = format!(
                "prefabs/particle/effectscripts/es[{}].prefab",
                entry.effect_id
            );
            let expected_path = format!(
                "map/shared/projectiles/effects/es{}.closure.json",
                entry.effect_id
            );
            if entry.container_route != expected_route || entry.closure_path != expected_path {
                return fail(format!(
                    "projectile effect {} uses non-canonical route/path: {:?}, {:?}",
                    entry.effect_id, entry.container_route, entry.closure_path
                ));
            }
            let bytes = read_verified(
                &root,
                &entry.closure_path,
                entry.closure_bytes,
                &entry.closure_blake3,
            )?;
            let closure: TutorialEffectClosureFile =
                serde_json::from_slice(&bytes).map_err(|error| {
                    TutorialEffectLibraryError(format!(
                        "invalid projectile effect closure {:?}: {error}",
                        entry.closure_path
                    ))
                })?;
            validate_effect_closure(
                entry,
                &closure,
                &projectile_catalog.source_bundle,
                &projectile_catalog.source_dump,
                &projectile_catalog.source_assets,
            )?;
            if projectile_effects
                .insert(
                    entry.effect_id,
                    ValidatedEffect {
                        entry: entry.clone(),
                        closure,
                    },
                )
                .is_some()
            {
                return fail(format!("duplicate projectile effect {}", entry.effect_id));
            }
        }

        let bullet_closure_bytes = read_verified(
            &root,
            &projectile_catalog.bullet_table_closure_path,
            projectile_catalog.bullet_table_closure_bytes,
            &projectile_catalog.bullet_table_closure_blake3,
        )?;
        let bullet_closure: TutorialEffectClosureFile =
            serde_json::from_slice(&bullet_closure_bytes).map_err(|error| {
                TutorialEffectLibraryError(format!("invalid BulletTable closure: {error}"))
            })?;
        if bullet_closure.schema != TUTORIAL_EFFECT_CLOSURE_SCHEMA
            || bullet_closure.effect_id.is_some()
            || bullet_closure.container_route != "bullettable.asset"
            || bullet_closure.root_asset != PRIMARY_EFFECTS_ASSET
            || bullet_closure.root_path_id != projectile_catalog.bullet_table_root_path_id
            || bullet_closure.source_bundle_blake3 != effect_catalog.source_bundle.blake3
            || bullet_closure.source_dump_blake3 != effect_catalog.source_dump.blake3
            || bullet_closure.source_assets != effect_catalog.source_assets
        {
            return fail("BulletTable closure disagrees with its catalog".to_owned());
        }
        validate_object_proofs(&bullet_closure)?;

        let actual_bullet_types = projectile_catalog
            .rows
            .iter()
            .map(|row| row.bullet_type)
            .collect::<BTreeSet<_>>();
        let expected_bullet_types = RETROBUTION_TUTORIAL_BULLET_TYPES
            .into_iter()
            .collect::<BTreeSet<_>>();
        if actual_bullet_types != expected_bullet_types
            || projectile_catalog.rows.len() != expected_bullet_types.len()
        {
            return fail(format!(
                "tutorial BulletTable types differ: actual={actual_bullet_types:?}, expected={expected_bullet_types:?}"
            ));
        }

        let mut bullets = BTreeMap::new();
        for entry in &projectile_catalog.rows {
            let expected_path = format!("map/shared/projectiles/bullet-{}.json", entry.bullet_type);
            if entry.row_path != expected_path {
                return fail(format!(
                    "bullet {} uses non-canonical row path {:?}",
                    entry.bullet_type, entry.row_path
                ));
            }
            let bytes = read_verified(&root, &entry.row_path, entry.row_bytes, &entry.row_blake3)?;
            let row: TutorialBulletRowFile = serde_json::from_slice(&bytes).map_err(|error| {
                TutorialEffectLibraryError(format!(
                    "invalid bullet row {:?}: {error}",
                    entry.row_path
                ))
            })?;
            let serialized = canonical_json(&row.serialized_row);
            let serialized_bytes = serde_json::to_vec(&serialized).map_err(|error| {
                TutorialEffectLibraryError(format!(
                    "could not canonicalize bullet {}: {error}",
                    entry.bullet_type
                ))
            })?;
            if row.schema != TUTORIAL_BULLET_ROW_SCHEMA
                || row.bullet_type != entry.bullet_type
                || row.source_route != "bullettable.asset"
                || row.source_root_path_id != projectile_catalog.bullet_table_root_path_id
                || row.serialized_row_blake3 != blake3_hash(&serialized_bytes)
                || row.serialized_row_blake3 != entry.serialized_row_blake3
                || row.parameters != entry.parameters
            {
                return fail(format!(
                    "bullet {} row disagrees with exact catalog evidence",
                    entry.bullet_type
                ));
            }
            let expected_row_blake3 =
                ffone_runtime_contracts::retrobution_bullet_row_proof(entry.bullet_type)
                    .ok_or_else(|| {
                        TutorialEffectLibraryError(format!(
                            "bullet {} has no clean-primary proof",
                            entry.bullet_type
                        ))
                    })?;
            if row.serialized_row_blake3 != expected_row_blake3 {
                return fail(format!(
                    "bullet {} differs from clean primary: actual={}, expected={expected_row_blake3}",
                    entry.bullet_type, row.serialized_row_blake3
                ));
            }
            validate_exact_bullet_parameters(&row.parameters, entry.bullet_type)?;
            bullets.insert(entry.bullet_type, row);
        }

        Ok(Self {
            root,
            effect_catalog,
            projectile_catalog,
            effects,
            projectile_effects,
            bullets,
        })
    }

    pub fn asset_root(&self) -> &Path {
        &self.root
    }

    pub fn effect_catalog(&self) -> &TutorialEffectCatalog {
        &self.effect_catalog
    }

    pub fn projectile_catalog(&self) -> &TutorialProjectileCatalog {
        &self.projectile_catalog
    }

    pub fn contains_effect(&self, effect_id: i32) -> bool {
        self.effects.contains_key(&effect_id)
    }

    pub fn contains_bullet(&self, bullet_type: i32) -> bool {
        self.bullets.contains_key(&bullet_type)
    }

    pub fn effect_material_animation(
        &self,
        effect_id: i32,
    ) -> Result<TutorialEffectMaterialAnimation, TutorialEffectLibraryError> {
        let effect = self.effects.get(&effect_id).ok_or_else(|| {
            TutorialEffectLibraryError(format!(
                "effect {effect_id} is outside the exact tutorial catalog"
            ))
        })?;
        tutorial_native_effects::compile_material_animation_component(effect_id, effect).map_err(
            |reason| {
                TutorialEffectLibraryError(format!(
                    "effect {effect_id} material animation is unavailable: {reason:?}"
                ))
            },
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TutorialEffectPlacement {
    World {
        position: Vec3,
        rotation: Quat,
    },
    /// A world-space placement owned by a streamed entity. The native effect
    /// renderer converts it to a local transform at spawn time so recursive
    /// despawn of the owner also removes the persistent effect root.
    ExactEntityWorld {
        root_entity: Entity,
        position: Vec3,
        rotation: Quat,
    },
    ExactBone {
        actor_id: i32,
        node_name: String,
        spawn_world_rotation: Quat,
        local_translation_after_parenting: Vec3,
        local_rotation_after_parenting: Quat,
    },
    /// Exact `AnimationEventHandler.tag_p` attachment for a non-NPC scene
    /// root, such as the active gameplay Nano. Unity resolves the named bone
    /// below the event owner's model and parents the effect to it.
    ExactEntityBone {
        root_entity: Entity,
        node_name: String,
        spawn_world_rotation: Quat,
        local_translation_after_parenting: Vec3,
        local_rotation_after_parenting: Quat,
    },
}

impl TutorialEffectPlacement {
    pub(super) fn is_valid(&self) -> bool {
        match self {
            Self::World { position, rotation }
            | Self::ExactEntityWorld {
                position, rotation, ..
            } => position.is_finite() && rotation.is_finite(),
            Self::ExactBone {
                node_name,
                spawn_world_rotation,
                local_translation_after_parenting,
                local_rotation_after_parenting,
                ..
            }
            | Self::ExactEntityBone {
                node_name,
                spawn_world_rotation,
                local_translation_after_parenting,
                local_rotation_after_parenting,
                ..
            } => {
                !node_name.is_empty()
                    && spawn_world_rotation.is_finite()
                    && local_translation_after_parenting.is_finite()
                    && local_rotation_after_parenting.is_finite()
            }
        }
    }

    pub(super) fn attachment_proof(&self) -> Option<TutorialEffectAttachmentProof> {
        match self {
            Self::World { .. } | Self::ExactEntityWorld { .. } => None,
            Self::ExactBone {
                actor_id,
                node_name,
                ..
            } => Some(TutorialEffectAttachmentProof::Actor {
                actor_id: *actor_id,
                node_name: node_name.clone(),
            }),
            Self::ExactEntityBone {
                root_entity,
                node_name,
                ..
            } => Some(TutorialEffectAttachmentProof::EntityRoot {
                root_entity: *root_entity,
                node_name: node_name.clone(),
            }),
        }
    }

    pub(super) fn stream_owner(&self) -> Option<Entity> {
        match self {
            Self::ExactEntityWorld { root_entity, .. } => Some(*root_entity),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) enum TutorialEffectAttachmentProof {
    Actor {
        actor_id: i32,
        node_name: String,
    },
    EntityRoot {
        root_entity: Entity,
        node_name: String,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TutorialProjectileMotion {
    /// Clean `BulletMoveScript`: wait for the BulletTable hide timer and then
    /// converge on the live target over the BulletTable maximum timer.
    BulletMove,
    /// Clean `cnWarHead`: use the weapon row for constant rocket velocity or
    /// a ballistic grenade launch. Warheads deliberately ignore the
    /// BulletTable curve, hide and maximum-time fields.
    Warhead {
        speed: f32,
        initial_vertical_speed: Option<f32>,
        duration_seconds: f32,
        authority: Option<WarheadAuthority>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WarheadAuthority {
    pub bullet_id: i8,
    pub blast_radius: f32,
    pub target_capacity: usize,
}

impl TutorialProjectileMotion {
    pub(super) fn is_valid(self) -> bool {
        match self {
            Self::BulletMove => true,
            Self::Warhead {
                speed,
                initial_vertical_speed,
                duration_seconds,
                ..
            } => {
                speed.is_finite()
                    && speed >= 0.0
                    && initial_vertical_speed.is_none_or(|speed| speed.is_finite())
                    && duration_seconds.is_finite()
                    && duration_seconds >= 0.0
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TutorialNativeClosureBlockerReason {
    LegacyMeshRenderer,
    LegacyAnimationRuntimeUnavailable,
    UnityRandomDependentEmitter,
    UnsupportedParticleShader { shader_name: String },
    UnsupportedUvAnimation { x_tiles: i64, y_tiles: i64 },
    UnsupportedParticleAnimator { field: String },
    InvalidSerializedNode { detail: String },
    MissingProjectileTrail,
}

/// True after the renderer has extracted the exact projectile-trail textures
/// and specialized their legacy blend pipelines for an active camera.
#[derive(Clone, Copy, Debug, Default, Resource)]
pub struct TutorialProjectileVisualReadiness {
    pub ready: bool,
}

#[derive(Clone, Debug)]
pub(super) struct ActiveNativeInstance {
    pub(super) name: Option<String>,
    pub(super) tracked: bool,
    pub(super) stream_owner: Option<Entity>,
    /// Actual Bevy root materialized by the native renderer. Named effects
    /// can outlive their visual subtree in bookkeeping when that subtree is
    /// rebuilt, so liveness must be reconciled against this entity rather
    /// than inferred from the numeric instance ID alone.
    pub(super) root_entity: Option<Entity>,
    pub(super) presentation_ready: bool,
}

/// Exact constants and update order from `OniMoveScript`.  Initial X/Z/Y
/// velocity is sampled by Unity's RNG in the original; callers must provide
/// that captured sample rather than substituting a different RNG algorithm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TutorialOniMotionContract {
    pub initial_horizontal_min: f32,
    pub initial_horizontal_max: f32,
    pub initial_vertical_min: f32,
    pub initial_vertical_max: f32,
    pub initial_move_speed: f32,
    pub acceleration_per_second: f32,
    pub damping_per_second: f32,
    pub gravity_per_second: f32,
    pub arrival_distance: f32,
    pub maximum_distance: f32,
    pub maximum_flight_seconds: f32,
    pub destroy_delay_seconds: f32,
    pub bounce_multiplier: f32,
    pub normal_force: f32,
    pub reverse_force: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TutorialOniMotionPhase {
    Flying,
    EmitterStopped,
    Destroyed,
}
