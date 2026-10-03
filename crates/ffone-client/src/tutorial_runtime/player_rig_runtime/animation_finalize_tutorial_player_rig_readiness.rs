use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn finalize_tutorial_player_rig_readiness(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut issues: ResMut<TutorialPlayerRigIssueQueue>,
    parents: Query<&ChildOf>,
    appearance_parts: Query<&TutorialPlayerAppearancePart>,
    appearance_scenes: Query<(Entity, &TutorialPlayerAppearancePart, &WorldAssetRoot)>,
    pending_attachments: Query<&PendingTutorialPlayerAppearanceAttachment>,
    metadata_errors: Query<(Entity, &LegacyMaterialMetadataError)>,
    surfaces: Query<
        (Entity, Option<&TutorialPlayerAppearanceMaterialBound>),
        With<MeshMaterial3d<LegacyModelMaterial>>,
    >,
    mut rigs: Query<(
        Entity,
        &TutorialSelectedPlayerRig,
        &NativePlayerRigStatus,
        Option<&TutorialPlayerAnimationAdapter>,
        &mut TutorialSelectedPlayerRigStatus,
    )>,
) {
    'rig: for (rig_root, selected, native_status, adapter, mut status) in &mut rigs {
        if !matches!(*status, TutorialSelectedPlayerRigStatus::Loading) {
            continue;
        }
        let NativePlayerRigStatus::ReadyAnimated {
            parts: native_part_count,
            skinned_surfaces,
            ..
        } = native_status
        else {
            if let NativePlayerRigStatus::Blocked(reason) = native_status {
                block_tutorial_player_appearance(
                    rig_root,
                    &mut status,
                    &mut issues,
                    format!("native selected-player rig blocked: {reason}"),
                );
            }
            continue;
        };
        if adapter.is_none() {
            continue;
        }
        for (entity, error) in &metadata_errors {
            if is_descendant_of(entity, rig_root, &parents) {
                block_tutorial_player_appearance(
                    rig_root,
                    &mut status,
                    &mut issues,
                    format!("selected-player material metadata failed: {}", error.0),
                );
                continue 'rig;
            }
        }

        if pending_attachments
            .iter()
            .any(|pending| pending.rig_root == rig_root)
        {
            continue;
        }
        let expected_parts = selected
            .look
            .parts
            .iter()
            .filter(|part| part.kind != NativePlayerPartKind::Weapon)
            .map(|part| (part.kind, part.exact_route.clone()))
            .collect::<BTreeSet<_>>();
        let expected_skinned_parts = selected
            .look
            .parts
            .iter()
            .filter(|part| part.uses_shared_skin())
            .count();
        let actual_parts = appearance_parts
            .iter()
            .filter(|part| part.rig_root == rig_root)
            .collect::<Vec<_>>();
        let actual_part_set = actual_parts
            .iter()
            .map(|part| (part.kind, part.exact_route.clone()))
            .collect::<BTreeSet<_>>();
        if actual_parts.len() != expected_parts.len()
            || actual_part_set != expected_parts
            || *native_part_count != expected_skinned_parts
        {
            // Rigid WorldAssetRoot descendants arrive asynchronously. Retain the
            // old visible rig until all exact routes materialize instead of
            // rejecting a valid candidate during its load frame.
            continue;
        }

        // Rigid Hat/Glasses/Back SceneRoots are not owned by the shared-rig
        // loader. Track their exact handles here so a failed cosmetic blocks
        // the hidden candidate instead of leaving apparel refresh in Loading
        // forever, while incomplete dependencies continue to retain the old
        // visible rig.
        let mut loading_scene = false;
        for (kind, exact_route) in &expected_parts {
            let mut matches = appearance_scenes.iter().filter(|(_, part, _)| {
                part.rig_root == rig_root && part.kind == *kind && part.exact_route == *exact_route
            });
            let Some((_, part, scene)) = matches.next() else {
                loading_scene = true;
                continue;
            };
            if matches.next().is_some() {
                block_tutorial_player_appearance(
                    rig_root,
                    &mut status,
                    &mut issues,
                    format!("selected-player appearance repeats exact route {exact_route:?}"),
                );
                continue 'rig;
            }
            match asset_server.load_state(scene.0.id()) {
                LoadState::Failed(error) => {
                    block_tutorial_player_appearance(
                        rig_root,
                        &mut status,
                        &mut issues,
                        format!(
                            "selected-player part {:?} ({:?}) failed to load: {error}",
                            part.glb, part.exact_route
                        ),
                    );
                    continue 'rig;
                }
                _ if matches!(
                    asset_server.get_recursive_dependency_load_state(scene.0.id()),
                    Some(RecursiveDependencyLoadState::Failed(_))
                ) =>
                {
                    let error = asset_server
                        .get_recursive_dependency_load_state(scene.0.id())
                        .and_then(|state| match state {
                            RecursiveDependencyLoadState::Failed(error) => Some(error),
                            _ => None,
                        })
                        .expect("recursive failure was matched above");
                    block_tutorial_player_appearance(
                        rig_root,
                        &mut status,
                        &mut issues,
                        format!(
                            "selected-player part {:?} ({:?}) dependency failed to load: {error}",
                            part.glb, part.exact_route
                        ),
                    );
                    continue 'rig;
                }
                _ if !asset_server.is_loaded_with_dependencies(scene.0.id()) => {
                    loading_scene = true;
                }
                _ => {}
            }
        }
        if loading_scene {
            continue;
        }

        let mut surface_count = 0_usize;
        let mut bound_parts = BTreeSet::new();
        let mut loaded_texture_paths = BTreeSet::new();
        let mut skin_tinted_surfaces = 0_usize;
        let mut hair_tinted_surfaces = 0_usize;
        let mut loading_material = false;
        let mut loading_texture = false;
        for (entity, bound) in &surfaces {
            let Some(part_scene) = ancestor_tutorial_player_appearance_part(
                entity,
                rig_root,
                &parents,
                &appearance_parts,
            ) else {
                continue;
            };
            surface_count += 1;
            let Some(bound) = bound else {
                // Scene descendants and their exact material metadata arrive
                // asynchronously. A route-level set is insufficient here:
                // one already-bound surface must not make a multi-surface
                // cosmetic Ready while its sibling still has authored/raw
                // material state.
                loading_material = true;
                continue;
            };
            if bound.rig_root != rig_root || bound.exact_route != part_scene.exact_route {
                block_tutorial_player_appearance(
                    rig_root,
                    &mut status,
                    &mut issues,
                    format!(
                        "selected-player material surface {entity:?} has stale binder ownership"
                    ),
                );
                continue 'rig;
            }
            let Some(part_look) =
                selected.look.parts.iter().find(|part| {
                    part.kind == part_scene.kind && part.exact_route == bound.exact_route
                })
            else {
                block_tutorial_player_appearance(
                    rig_root,
                    &mut status,
                    &mut issues,
                    format!(
                        "bound route {:?} is absent from selected look",
                        bound.exact_route
                    ),
                );
                continue 'rig;
            };
            let expected_texture = if bound.binding.hide_surface {
                None
            } else {
                match bound.binding.role {
                    ActorSkinTextureRole::SecondarySkin => {
                        if uses_global_skin_secondary(part_look.kind) {
                            selected.look.skin_texture.as_ref()
                        } else {
                            part_look.secondary_texture.as_ref()
                        }
                    }
                    ActorSkinTextureRole::PreserveOverride => None,
                    ActorSkinTextureRole::Primary => part_look.primary_texture.as_ref(),
                }
            };
            match (expected_texture, bound.binding.texture.as_ref()) {
                (Some(expected), Some(actual)) if expected.path == actual.path => {
                    match asset_server.load_state(actual.handle.id()) {
                        LoadState::Failed(error) => {
                            block_tutorial_player_appearance(
                                rig_root,
                                &mut status,
                                &mut issues,
                                format!(
                                    "selected-player texture {:?} failed to load: {error}",
                                    actual.path
                                ),
                            );
                            continue 'rig;
                        }
                        LoadState::Loaded => {
                            loaded_texture_paths.insert(actual.path.clone());
                        }
                        _ => loading_texture = true,
                    }
                }
                (None, None) => {}
                (expected, actual) => {
                    block_tutorial_player_appearance(
                        rig_root,
                        &mut status,
                        &mut issues,
                        format!(
                            "selected-player exact texture mismatch for {:?}: expected {:?}, bound {:?}",
                            bound.exact_route,
                            expected.map(|texture| texture.path.as_str()),
                            actual.map(|texture| texture.path.as_str())
                        ),
                    );
                    continue 'rig;
                }
            }
            bound_parts.insert((part_scene.kind, bound.exact_route.clone()));
            skin_tinted_surfaces += usize::from(bound.binding.skin_tint_applied);
            hair_tinted_surfaces += usize::from(bound.binding.hair_tint_applied);
        }
        if surface_count < *skinned_surfaces || bound_parts != expected_parts {
            continue;
        }
        if selected.look.skin_texture.is_some() && skin_tinted_surfaces == 0 {
            block_tutorial_player_appearance(
                rig_root,
                &mut status,
                &mut issues,
                "selected-player global skin tint reached no exact material surface".to_owned(),
            );
            continue;
        }
        if selected
            .look
            .parts
            .iter()
            .any(|part| part.kind == NativePlayerPartKind::Hair)
            && hair_tinted_surfaces == 0
        {
            block_tutorial_player_appearance(
                rig_root,
                &mut status,
                &mut issues,
                "selected-player hair tint reached no exact material surface".to_owned(),
            );
            continue;
        }
        if loading_material || loading_texture {
            continue;
        }

        *status = TutorialSelectedPlayerRigStatus::Ready;
        commands
            .entity(rig_root)
            .insert(TutorialPlayerAppearanceReady {
                parts: expected_parts.len(),
                surfaces: surface_count,
                textures: loaded_texture_paths.len(),
                skin_tinted_surfaces,
                hair_tinted_surfaces,
            });
    }
}

pub(super) fn legacy_visual_tutorial_clip(
    clip: LegacyVisualClip,
    weapon_profile: Option<PlayerWeaponAnimationProfile>,
    tutorial_stand_semantics: bool,
) -> Option<TutorialPlayerClip> {
    let base = match clip {
        // `StandName` deliberately suppresses the weapon prefix while the
        // tutorial event is active. Ready/run/jump/attack still use their
        // rifle variants through `ReadyName`/`RunName`.
        LegacyVisualClip::Stand1 => {
            return Some(if tutorial_stand_semantics {
                TutorialPlayerClip::Stand1
            } else {
                weapon_profile.map_or(TutorialPlayerClip::Stand1, |profile| profile.stand())
            });
        }
        LegacyVisualClip::Ready => {
            return Some(
                weapon_profile.map_or(TutorialPlayerClip::Stand1, |profile| profile.ready()),
            );
        }
        LegacyVisualClip::Stun => TutorialPlayerClip::Stun,
        LegacyVisualClip::Dash => match weapon_profile {
            Some(PlayerWeaponAnimationProfile::Rifle) => TutorialPlayerClip::RifleDash,
            Some(PlayerWeaponAnimationProfile::Pistol) => TutorialPlayerClip::RifleTumbling,
            _ => TutorialPlayerClip::StickDash,
        },
        LegacyVisualClip::DashAir => TutorialPlayerClip::RocketSomersault,
        LegacyVisualClip::DashWaterUpper => TutorialPlayerClip::StickDodgeUpper,
        LegacyVisualClip::DashUpper => match weapon_profile {
            Some(PlayerWeaponAnimationProfile::Bomb) => return None,
            Some(PlayerWeaponAnimationProfile::Rifle) => TutorialPlayerClip::RifleDodgeUpper,
            _ => TutorialPlayerClip::StickDodgeUpper,
        },
        LegacyVisualClip::Run => TutorialPlayerClip::Run,
        LegacyVisualClip::RunBack => TutorialPlayerClip::RunBack,
        LegacyVisualClip::JumpStart => TutorialPlayerClip::JumpStart,
        LegacyVisualClip::Jump => TutorialPlayerClip::Jump,
        LegacyVisualClip::JumpEnd => TutorialPlayerClip::JumpEnd,
        LegacyVisualClip::JumpLandRun => TutorialPlayerClip::JumpLandRun,
        LegacyVisualClip::Die => TutorialPlayerClip::Die,
        LegacyVisualClip::Death => TutorialPlayerClip::Death,
        LegacyVisualClip::Swim => TutorialPlayerClip::Swim,
        LegacyVisualClip::SwimBack => TutorialPlayerClip::SwimBack,
        LegacyVisualClip::SwimIdle => TutorialPlayerClip::SwimIdle,
        LegacyVisualClip::SwimLeft => TutorialPlayerClip::SwimLeft,
        LegacyVisualClip::SwimRight => TutorialPlayerClip::SwimRight,
        LegacyVisualClip::Slide => TutorialPlayerClip::Slide,
        LegacyVisualClip::RopeDown => TutorialPlayerClip::RopeDown,
        LegacyVisualClip::RopeDrop => TutorialPlayerClip::RopeDrop,
        LegacyVisualClip::RopeLeft => TutorialPlayerClip::RopeLeft,
        LegacyVisualClip::RopeRight => TutorialPlayerClip::RopeRight,
        LegacyVisualClip::RopeStand1 => TutorialPlayerClip::RopeStand1,
        LegacyVisualClip::RopeStand2 => TutorialPlayerClip::RopeStand2,
        LegacyVisualClip::RopeTurn => TutorialPlayerClip::RopeTurn,
        LegacyVisualClip::RopeUp => TutorialPlayerClip::RopeUp,
        LegacyVisualClip::Mount1 => TutorialPlayerClip::Mount1,
        LegacyVisualClip::Inventory => TutorialPlayerClip::Inventory,
        LegacyVisualClip::BoardInventory => TutorialPlayerClip::BoardInventory,
        LegacyVisualClip::ScooterInventory => TutorialPlayerClip::ScooterInventory,
        // With an empty Hand slot, clean cnAvatarAnimation leaves the prefix
        // empty and therefore resolves the actor-owned attack1/attack1upper.
        LegacyVisualClip::AttackFull(_) => {
            weapon_profile.map_or(TutorialPlayerClip::Attack1, |profile| profile.attack(false))
        }
        LegacyVisualClip::AttackUpper(_) => weapon_profile
            .map_or(TutorialPlayerClip::Attack1Upper, |profile| {
                profile.attack(true)
            }),
    };
    Some(weapon_profile.map_or(base, |profile| profile.locomotion_variant(base)))
}

#[derive(Clone, Copy)]
pub(super) struct CoalescedLegacyLayerAnimation {
    pub(super) semantic_clip: LegacyVisualClip,
    pub(super) animation_clip: TutorialPlayerClip,
    pub(super) blend_seconds: f32,
}

pub(super) fn coalesce_legacy_layer_animation(
    base: &mut Option<CoalescedLegacyLayerAnimation>,
    upper: &mut Option<CoalescedLegacyLayerAnimation>,
    layer: LegacyAnimationLayer,
    request: CoalescedLegacyLayerAnimation,
) {
    match layer {
        LegacyAnimationLayer::FullBody => *base = Some(request),
        LegacyAnimationLayer::UpperBody => *upper = Some(request),
    }
}

pub(super) fn queue_coalesced_legacy_layer_animation(
    queue: &mut TutorialPlayerPresentationCommandQueue,
    pending: &mut Option<PendingLegacyVisualCompletion>,
    gender: PlayerRigGender,
    request: CoalescedLegacyLayerAnimation,
) {
    queue.push_animation(
        TutorialPlayerAnimationRequest::runtime_cross_fade_with_duration(
            gender,
            request.animation_clip,
            Duration::from_secs_f32(request.blend_seconds.max(0.0)),
        ),
    );
    *pending = pending_legacy_visual_completion(request.semantic_clip, request.animation_clip);
}
