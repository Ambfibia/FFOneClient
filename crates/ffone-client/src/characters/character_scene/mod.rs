//! Fail-closed instantiation of native character GLB scenes.
//!
//! Published GLBs keep every authored node transform.  The old Unity
//! character spawners replaced only the instantiated character root, while
//! world/static scenes retained their authored transform.  This module keeps
//! those two responsibilities explicit and prevents an origin/scale fix from
//! being applied twice.

use bevy::{prelude::*, world_serialization::WorldInstanceReady};

use crate::coordinates::{
    LegacyCharacterRootError, LegacyCharacterRootPolicy, native_model_forward_child_rotation,
};

/// What kind of native scene is attached below a Bevy runtime entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSceneRole {
    /// Terrain, buildings, props, sockets, and other authored world content.
    WorldAuthored,
    /// A character model whose authored +Z must face Bevy gameplay-root -Z.
    CharacterGameplay,
    /// A server transportation prefab. Clean `cnBusMoveController` does not
    /// rotate toward packet destinations, so source world facing is retained.
    TransportationGameplay,
}

/// Stable container transform for a native scene.
///
/// Neither role introduces translation or scale.  The character half-turn is
/// intentionally outside the imported named root, so root replacement cannot
/// destroy it and it can never leak into world or attachment transforms.
#[must_use]
pub fn native_scene_container_transform(role: NativeSceneRole) -> Transform {
    match role {
        NativeSceneRole::WorldAuthored | NativeSceneRole::TransportationGameplay => {
            Transform::IDENTITY
        }
        NativeSceneRole::CharacterGameplay => {
            Transform::from_rotation(native_model_forward_child_rotation())
        }
    }
}

/// Contract attached to the Bevy entity that owns a spawned GLB scene.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct LegacyCharacterSceneSpec {
    /// Exact original Unity `m_Name` of the one root that may be replaced.
    pub expected_root_name: String,
    pub root_policy: LegacyCharacterRootPolicy,
}

/// A hard reason why a character scene remained hidden.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LegacyCharacterSceneBlock {
    MissingNamedRoot {
        expected: String,
    },
    MultipleNamedRoots {
        expected: String,
        count: usize,
    },
    InvalidSceneWrapper {
        expected: String,
        wrapper: Entity,
    },
    InvalidNamedRoot {
        expected: String,
        error: LegacyCharacterRootError,
    },
}

/// Observable state of the scene-root normalization gate.
#[derive(Component, Debug, Clone, PartialEq)]
pub enum LegacyCharacterSceneStatus {
    Pending,
    Ready {
        root: Entity,
        authored_root: Transform,
        applied_root: Transform,
    },
    Blocked(LegacyCharacterSceneBlock),
}

/// Keeps a normalized character scene hidden until its owning runtime has
/// finished post-load setup such as legacy material conversion and XDT texture
/// selection. Root normalization still advances [`LegacyCharacterSceneStatus`]
/// to `Ready`; only the final visibility transition is deferred.
#[derive(Component, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LegacyCharacterSceneDeferredReveal;

/// Entities created for one character visual.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpawnedLegacyCharacterScene {
    /// Child of the gameplay transform; carries the selected stable scene role transform.
    pub visual_container: Entity,
    /// Hidden GLB scene parent until exact root normalization succeeds.
    pub scene: Entity,
}

/// Spawns a lossless native GLB below an existing gameplay root.
///
/// The scene stays hidden until [`WorldInstanceReady`] proves that exactly one
/// topmost descendant has `expected_root_name`, every intermediary Bevy scene
/// wrapper has identity TRS, and the authored root TRS is valid. This permits
/// the loader's structural wrappers without letting a repacked GLB smuggle in
/// an additional origin, scale, or facing rotation. Some source prefabs
/// (notably `nano_johnnybravo`) reuse the root name on a nested mesh transform;
/// a nested node is not a second logical root.
pub fn spawn_legacy_character_scene(
    commands: &mut Commands,
    gameplay_root: Entity,
    scene: Handle<WorldAsset>,
    expected_root_name: impl Into<String>,
    root_policy: LegacyCharacterRootPolicy,
) -> SpawnedLegacyCharacterScene {
    spawn_legacy_scene(
        commands,
        gameplay_root,
        scene,
        expected_root_name,
        root_policy,
        NativeSceneRole::CharacterGameplay,
    )
}

/// Spawns a clean `cnBusMoveController` model without the character-only
/// +Z-to--Z half turn. The transport root keeps the source prefab facing.
pub fn spawn_legacy_transportation_scene(
    commands: &mut Commands,
    gameplay_root: Entity,
    scene: Handle<WorldAsset>,
    expected_root_name: impl Into<String>,
) -> SpawnedLegacyCharacterScene {
    spawn_legacy_scene(
        commands,
        gameplay_root,
        scene,
        expected_root_name,
        LegacyCharacterRootPolicy::Transportation,
        NativeSceneRole::TransportationGameplay,
    )
}

pub(crate) fn spawn_legacy_scene(
    commands: &mut Commands,
    gameplay_root: Entity,
    scene: Handle<WorldAsset>,
    expected_root_name: impl Into<String>,
    root_policy: LegacyCharacterRootPolicy,
    role: NativeSceneRole,
) -> SpawnedLegacyCharacterScene {
    let visual_container = commands
        .spawn((
            Name::new("native character visual container"),
            ChildOf(gameplay_root),
            native_scene_container_transform(role),
            Visibility::Inherited,
        ))
        .id();

    let scene = commands
        .spawn((
            Name::new("lossless native character GLB"),
            ChildOf(visual_container),
            WorldAssetRoot(scene),
            Transform::IDENTITY,
            Visibility::Hidden,
            LegacyCharacterSceneSpec {
                expected_root_name: expected_root_name.into(),
                root_policy,
            },
            LegacyCharacterSceneStatus::Pending,
        ))
        .observe(normalize_legacy_character_root)
        .id();

    SpawnedLegacyCharacterScene {
        visual_container,
        scene,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExactNamedRootCandidate {
    root: Entity,
    wrappers: Vec<Entity>,
}

fn collect_topmost_exact_named_descendants(
    root: Entity,
    mut children_of: impl FnMut(Entity) -> Vec<Entity>,
    mut is_exact_name: impl FnMut(Entity) -> bool,
) -> Vec<ExactNamedRootCandidate> {
    let mut stack: Vec<_> = children_of(root)
        .into_iter()
        .map(|entity| (entity, Vec::new()))
        .collect();
    let mut matches = Vec::new();
    while let Some((entity, wrappers)) = stack.pop() {
        if is_exact_name(entity) {
            matches.push(ExactNamedRootCandidate {
                root: entity,
                wrappers,
            });
            // A nested same-name node belongs to this logical root and cannot
            // compete with it (the shipped Johnny Bravo prefab does this).
            continue;
        }
        let mut descendant_wrappers = wrappers;
        descendant_wrappers.push(entity);
        stack.extend(
            children_of(entity)
                .into_iter()
                .map(|child| (child, descendant_wrappers.clone())),
        );
    }
    matches
}

fn is_identity_scene_wrapper(transform: Transform) -> bool {
    transform.translation.abs_diff_eq(Vec3::ZERO, 0.000_01)
        && transform.scale.abs_diff_eq(Vec3::ONE, 0.000_01)
        && (transform.rotation.abs_diff_eq(Quat::IDENTITY, 0.000_01)
            || transform.rotation.abs_diff_eq(-Quat::IDENTITY, 0.000_01))
}

fn normalize_legacy_character_root(
    event: On<WorldInstanceReady>,
    mut scenes: Query<(
        &LegacyCharacterSceneSpec,
        &mut LegacyCharacterSceneStatus,
        &mut Visibility,
        Option<&LegacyCharacterSceneDeferredReveal>,
    )>,
    children: Query<&Children>,
    names: Query<&Name>,
    mut transforms: Query<&mut Transform>,
) {
    let scene_entity = event.event().entity;
    let Ok((spec, mut status, mut visibility, deferred_reveal)) = scenes.get_mut(scene_entity)
    else {
        return;
    };
    if !matches!(*status, LegacyCharacterSceneStatus::Pending) {
        return;
    }

    let matches = collect_topmost_exact_named_descendants(
        scene_entity,
        |entity| {
            children
                .get(entity)
                .map(|children| children.iter().collect())
                .unwrap_or_default()
        },
        |entity| {
            names
                .get(entity)
                .is_ok_and(|name| name.as_str() == spec.expected_root_name)
        },
    );

    let candidate = match matches.as_slice() {
        [] => {
            *status =
                LegacyCharacterSceneStatus::Blocked(LegacyCharacterSceneBlock::MissingNamedRoot {
                    expected: spec.expected_root_name.clone(),
                });
            return;
        }
        [candidate] => candidate.clone(),
        roots => {
            *status = LegacyCharacterSceneStatus::Blocked(
                LegacyCharacterSceneBlock::MultipleNamedRoots {
                    expected: spec.expected_root_name.clone(),
                    count: roots.len(),
                },
            );
            return;
        }
    };

    for wrapper in candidate.wrappers {
        let Ok(wrapper_transform) = transforms.get(wrapper) else {
            *status = LegacyCharacterSceneStatus::Blocked(
                LegacyCharacterSceneBlock::InvalidSceneWrapper {
                    expected: spec.expected_root_name.clone(),
                    wrapper,
                },
            );
            return;
        };
        if !is_identity_scene_wrapper(*wrapper_transform) {
            *status = LegacyCharacterSceneStatus::Blocked(
                LegacyCharacterSceneBlock::InvalidSceneWrapper {
                    expected: spec.expected_root_name.clone(),
                    wrapper,
                },
            );
            return;
        }
    }
    let root = candidate.root;

    let Ok(mut root_transform) = transforms.get_mut(root) else {
        *status =
            LegacyCharacterSceneStatus::Blocked(LegacyCharacterSceneBlock::InvalidNamedRoot {
                expected: spec.expected_root_name.clone(),
                error: LegacyCharacterRootError::InvalidAuthoredTransform,
            });
        return;
    };
    let authored_root = *root_transform;
    let applied_root = match spec.root_policy.try_resolve_root(authored_root) {
        Ok(applied_root) => applied_root,
        Err(error) => {
            *status =
                LegacyCharacterSceneStatus::Blocked(LegacyCharacterSceneBlock::InvalidNamedRoot {
                    expected: spec.expected_root_name.clone(),
                    error,
                });
            return;
        }
    };

    *root_transform = applied_root;
    *status = LegacyCharacterSceneStatus::Ready {
        root,
        authored_root,
        applied_root,
    };
    *visibility = normalized_character_scene_visibility(deferred_reveal.is_some());
}

fn normalized_character_scene_visibility(deferred_reveal: bool) -> Visibility {
    if deferred_reveal {
        Visibility::Hidden
    } else {
        Visibility::Inherited
    }
}

#[cfg(test)]
mod tests;
