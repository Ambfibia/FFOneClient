//! Scene setup, native world slice, avatar environment and world-ready recovery.

use super::loading_screen::{GameplayLoadingState, ResourceLoadingScope};
use super::local_inventory::LocalInventoryRuntime;
use super::login::{ClientConfig, Credentials};
use super::runtime_status::RuntimeStatus;
use super::sky::LEGACY_REFERENCE_GLOW_ENABLED;
use super::state::ClientState;
use super::tutorial_session::TutorialSession;
use super::{
    LOCAL_INFECTION_AURA_EFFECT_ID, LOCAL_INFECTION_EFFECT_NAME, LocalCharacterScene,
    LocalInfectionPresentationState, LocalNetworkIdentity, LocalPlayer, NetworkLifecycleSession,
    WorldSliceEntity,
};
use bevy::{
    audio::SpatialListener, camera::visibility::RenderLayers, ecs::system::SystemParam, prelude::*,
};
use ffone_client::{
    avatar_action::{
        LegacyAvatarActionContext, LegacyAvatarActionState, LegacyAvatarClipBindings,
        LegacyAvatarPresentationContext, LegacyAvatarTargetFeed, LegacyVehiclePresentationFamily,
    },
    character_creation_data::CharacterCreationData,
    coordinates::{ProtocolPosition, ProtocolYawDegrees},
    entity_lifecycle::NetworkEntityLifecycleIngress0104,
    gameplay_audio::GameplayAudioRuntime,
    legacy_environment::{
        LEGACY_ENVIRONMENT_TRANSITION_SECONDS, LEGACY_LOCAL_INFECTION_TICK_SECONDS,
        LEGACY_LOCAL_REGEN_TICK_SECONDS, LEGACY_TERRAIN_CONTACT_HEIGHT,
        LegacyAvatarEnvironmentState, LegacyWaterSurface, legacy_environment_flags,
        legacy_water_surface_contact, legacy_water_surface_height,
    },
    legacy_glow::LegacyGlowSettings,
    movement::{LegacyOrbitCamera, LegacyPlayerController, LegacyWorldColliderPending},
    nano_free_tuning_ui::NanoFreeTuningProtocolFault,
    native_terrain::{
        NativeHeightmapCollider, NativeTerrainGameplayAttributeSample,
        NativeTerrainLegacyInterpolatedHeightSample,
    },
    network::{
        CharacterSummary, NanoTuneCorrelationError0104, NanoTuneCorrelationFault0104,
        NetworkBridge, NetworkCommand,
    },
    player_shared_rig::{NativePlayerRigAssetCache, NativePlayerRigCatalog, NativePlayerRigStatus},
    skill_buff_ui::{SKILL_BUFF_INFECTION_FLAG, SkillBuffUiModel},
    tutorial_effects_runtime::{
        TutorialEffectPlacement, TutorialEffectRuntime, TutorialEffectRuntimeCommand,
    },
    tutorial_player_presentation::{
        PlayerWeaponAnimationCatalog, TutorialPlayerAnimationRequest,
        TutorialPlayerEquipmentRequest, TutorialPlayerPresentationCommandQueue,
        tutorial_player_gender_from_protocol,
    },
    tutorial_player_rig_runtime::{
        TutorialPlayerAnimationApplied, TutorialSelectedPlayerRigActive,
        TutorialSelectedPlayerRigStatus, spawn_tutorial_selected_player_rig,
    },
    world::{
        EXTENDED_WORLD_CAMERA_FAR_NATIVE, NativeWorldBehaviourStatus, NativeWorldCatalog,
        NativeWorldColliderStatus, NativeWorldPresentationStatus, NativeWorldSceneEntity,
        NativeWorldSceneRoot, NativeWorldScope, NativeWorldStreamingStatus,
        begin_native_world_scene_admission, native_dong_key,
    },
};
use ffone_protocol::{NanoTuneSuccess0104, WirePayload, packet};

#[derive(SystemParam)]
pub(super) struct NativeWorldStartupQueries<'w, 's> {
    pub(super) catalog: Res<'w, NativeWorldCatalog>,
    pub(super) streaming: Res<'w, NativeWorldStreamingStatus>,
    pub(super) location: Res<'w, ffone_client::world::NativeWorldLocationPresentation>,
    pub(super) roots: Query<
        'w,
        's,
        (
            Entity,
            &'static NativeWorldSceneRoot,
            &'static NativeWorldPresentationStatus,
            &'static NativeWorldBehaviourStatus,
        ),
    >,
    pub(super) scenes: Query<'w, 's, (Entity, &'static WorldAssetRoot), With<NativeWorldSceneEntity>>,
    pub(super) colliders: Query<'w, 's, (Entity, &'static NativeWorldColliderStatus)>,
    pub(super) parents: Query<'w, 's, &'static ChildOf>,
    pub(super) player_rigs: Query<'w, 's, &'static NativePlayerRigStatus, With<LocalCharacterScene>>,
    pub(super) tutorial_player_rigs: Query<
        'w,
        's,
        (
            &'static TutorialSelectedPlayerRigStatus,
            Option<&'static TutorialPlayerAnimationApplied>,
        ),
        With<TutorialSelectedPlayerRigActive>,
    >,
}

pub(super) fn setup_scene(mut commands: Commands) {
    commands.spawn((
        DirectionalLight {
            illuminance: 8_000.0,
            // Native FusionFall materials deliberately opt out of Bevy's
            // StandardMaterial shadow/prepass pipelines. Enabling this light's
            // shadow map lets a just-converted glTF primitive retain a stale
            // StandardMaterial prepass item for one render frame, then binds
            // LegacyModelMaterial against that incompatible pipeline.
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, 20.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

pub(super) fn project_nano_tune_correlation_fault(
    error: NanoTuneCorrelationError0104,
) -> NanoFreeTuningProtocolFault {
    match error {
        NanoTuneCorrelationError0104::Malformed { frame, .. } => {
            let expected = match frame.packet_type {
                packet::P_FE2CL_REP_NANO_TUNE_SUCC => NanoTuneSuccess0104::SIZE,
                packet::P_FE2CL_REP_NANO_TUNE_FAIL => ffone_protocol::NanoTuneFailure0104::SIZE,
                packet_id => return NanoFreeTuningProtocolFault::UnexpectedPacketId(packet_id),
            };
            NanoFreeTuningProtocolFault::PayloadSize {
                packet_id: frame.packet_type,
                expected,
                actual: frame.payload.len(),
            }
        }
        NanoTuneCorrelationError0104::Mismatch { fault, .. } => match fault {
            NanoTuneCorrelationFault0104::NanoId { expected, actual } => {
                NanoFreeTuningProtocolFault::NanoMismatch { expected, actual }
            }
            NanoTuneCorrelationFault0104::SkillId { expected, actual } => {
                NanoFreeTuningProtocolFault::SkillMismatch { expected, actual }
            }
            NanoTuneCorrelationFault0104::PlayerId { expected, actual } => {
                NanoFreeTuningProtocolFault::PlayerMismatch { expected, actual }
            }
        },
    }
}

/// Rolls back both already-applied world ownership and entities still queued
/// by a failed native scene/rig assembly.
///
/// `spawn_native_world_slice` uses deferred commands, so an ordinary query in
/// this system cannot observe every partial entity yet. Queue the sweep after
/// those commands; the closure then sees the complete failed transaction and
/// removes both tagged player/world roots and any native scene child created
/// before an error was returned.
#[allow(clippy::too_many_arguments)]
pub(super) fn rollback_failed_world_ready_spawn(
    commands: &mut Commands,
    lifecycle_ingress: &mut NetworkEntityLifecycleIngress0104,
    lifecycle_session: &mut NetworkLifecycleSession,
    runtime: &mut RuntimeStatus,
    inventory: &mut LocalInventoryRuntime,
    effect_runtime: &mut TutorialEffectRuntime,
    player_commands: &mut TutorialPlayerPresentationCommandQueue,
    rig_assets: &mut NativePlayerRigAssetCache,
    loading: &mut GameplayLoadingState,
    loading_scope: ResourceLoadingScope,
) {
    commands.queue(|world: &mut World| {
        let failed_entities = {
            let mut query = world.query_filtered::<
                Entity,
                Or<(With<WorldSliceEntity>, With<NativeWorldSceneEntity>)>,
            >();
            query.iter(world).collect::<Vec<_>>()
        };
        for entity in failed_entities {
            if let Ok(entity) = world.get_entity_mut(entity) {
                entity.despawn();
            }
        }
    });
    // The failed spawn queued a fresh streaming resource before scene
    // assembly. Reset it after the sweep so no old blocker/readiness state is
    // mistaken for the next character-entry attempt.
    commands.insert_resource(NativeWorldStreamingStatus::default());

    lifecycle_ingress.clear();
    if let Some(epoch) = lifecycle_session.take() {
        lifecycle_ingress.disconnect(epoch);
    }
    runtime.clear_world();
    runtime.diagnostics.bootstrap_packets = 0;
    runtime.diagnostics.bootstrap_decode_errors = 0;
    inventory.reset();
    effect_runtime.clear_scene_instances();
    player_commands.clear();
    rig_assets.release_cached_handles();
    loading.begin(loading_scope);
}

pub(super) fn recover_failed_world_ready_connection(
    bridge: &NetworkBridge,
    config: &ClientConfig,
    credentials: Option<&Credentials>,
    next_state: &mut NextState<ClientState>,
    loading: &mut GameplayLoadingState,
    failure: String,
) -> String {
    match bridge.send(NetworkCommand::Disconnect) {
        Ok(()) => {
            next_state.set(ClientState::Login);
            loading.finish();
            if let Some(credentials) = credentials {
                if let Err(relogin_error) = bridge.send(NetworkCommand::Login {
                    login_address: config.login_address.clone(),
                    username: credentials.username.clone(),
                    password: credentials.password.clone(),
                }) {
                    format!("{failure}; shard disconnected safely, sign in again: {relogin_error}")
                } else {
                    format!("{failure}; reconnecting to character selection")
                }
            } else {
                format!("{failure}; shard disconnected safely, sign in again")
            }
        }
        Err(disconnect_error) => {
            let blocked =
                format!("{failure}; automatic shard disconnect failed: {disconnect_error}");
            loading.block(blocked.clone());
            format!("Native world recovery blocked: {blocked}")
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn update_legacy_avatar_environment(
    time: Res<Time>,
    bridge: Res<NetworkBridge>,
    meshes: Res<Assets<Mesh>>,
    tutorial: Res<TutorialSession>,
    terrains: Query<(&GlobalTransform, &NativeHeightmapCollider)>,
    waters: Query<(&GlobalTransform, &Mesh3d, &LegacyWaterSurface)>,
    mut water_occlusion: ffone_client::world::NativeWaterOcclusion,
    mut runtime: ResMut<RuntimeStatus>,
    mut audio: ResMut<GameplayAudioRuntime>,
    mut players: Query<
        (
            Entity,
            &Transform,
            &LegacyPlayerController,
            &mut LegacyAvatarEnvironmentState,
            Option<&LocalNetworkIdentity>,
            Option<&LegacyAvatarPresentationContext>,
        ),
        With<LocalPlayer>,
    >,
) {
    let delta_seconds = time.delta_secs().max(0.0);
    let local_tutorial_hp_authority = tutorial.owns_local_hp();
    for (player, transform, _controller, mut environment, network_identity, presentation) in
        &mut players
    {
        let position = transform.translation;
        let mut terrain_height = None;
        let mut terrain_attribute = None;
        // MapAttributeTable selects one dong BEFORE calling SampleHeight.
        // SampleHeight clamps outside queries to the tile edge; taking the
        // highest result across resident neighbors can erase the current
        // tile's poison bit with an unrelated out-of-bounds sample.
        let dong_key = native_dong_key(position.x, position.z);
        let mut owners = terrains.iter().filter(|(global, _)| {
            let origin = global.translation();
            dong_key.is_some() && native_dong_key(origin.x, origin.z) == dong_key
        });
        if let Some((global, terrain)) = owners.next()
            && owners.next().is_none()
        {
            terrain_attribute = match terrain.gameplay_attribute(global, position.x, position.z) {
                NativeTerrainGameplayAttributeSample::Value(value) => Some(value),
                NativeTerrainGameplayAttributeSample::OutsideTerrain
                | NativeTerrainGameplayAttributeSample::SidecarMissing => None,
            };
            if let NativeTerrainLegacyInterpolatedHeightSample::Height(height) =
                terrain.legacy_interpolated_height(global, position.x, position.z)
            {
                terrain_height = Some(height);
            }
        }

        let mut water_contact = None;
        for (global, mesh_handle, water) in &waters {
            let Some(mesh) = meshes.get(&mesh_handle.0) else {
                continue;
            };
            let Some(height) = legacy_water_surface_height(mesh, global, position.x, position.z)
            else {
                continue;
            };
            let touching = legacy_water_surface_contact(height, position.y)
                && !water_occlusion.blocks(height, position);
            if touching && water_contact.is_none_or(|(current, _): (f32, bool)| height > current) {
                water_contact = Some((height, water.infected));
            }
        }
        let in_water = water_contact.is_some();
        let terrain_contact = terrain_height
            .is_some_and(|height: f32| (position.y - height).abs() < LEGACY_TERRAIN_CONTACT_HEIGHT);
        let infected_water = water_contact.is_some_and(|(_, infected)| infected);
        let flags = legacy_environment_flags(
            terrain_attribute,
            in_water || terrain_contact,
            presentation.is_some_and(|presentation| {
                presentation.mounted_vehicle != LegacyVehiclePresentationFamily::None
            }),
        );

        environment.in_water = in_water;
        environment.infected_water = infected_water;
        environment.terrain_attribute = terrain_attribute;
        environment.poison_transition_elapsed_seconds += delta_seconds;
        environment.heal_transition_elapsed_seconds += delta_seconds;

        // Clean `cnOwnAvatarStatus.Update` calls EnterHealArea before
        // EnterPoisonArea. Preserve the packet order when both bits change on
        // the same update.
        if flags.healing != environment.healing
            && environment.heal_transition_elapsed_seconds >= LEGACY_ENVIRONMENT_TRANSITION_SECONDS
        {
            environment.healing = flags.healing;
            environment.heal_transition_elapsed_seconds = 0.0;
            if network_identity.is_some()
                && let Err(error) =
                    bridge.send(NetworkCommand::EnvironmentHeal(environment.healing))
            {
                runtime.message = error;
            }
        }
        if flags.poisoned != environment.poisoned
            && environment.poison_transition_elapsed_seconds
                >= LEGACY_ENVIRONMENT_TRANSITION_SECONDS
        {
            environment.poisoned = flags.poisoned;
            environment.poison_transition_elapsed_seconds = 0.0;
            if network_identity.is_some()
                && let Err(error) =
                    bridge.send(NetworkCommand::EnvironmentDamage(environment.poisoned))
            {
                runtime.message = error;
            }
        }

        if network_identity.is_some() && !local_tutorial_hp_authority {
            environment.last_observed_hp = runtime.hp;
            continue;
        }
        let Some(mut hp) = runtime.hp else {
            continue;
        };
        let local_combat_active = environment.local_combat_timeout_remaining_seconds > 0.0;
        environment.local_combat_timeout_remaining_seconds =
            (environment.local_combat_timeout_remaining_seconds - delta_seconds).max(0.0);
        if environment
            .last_observed_hp
            .is_some_and(|previous| hp < previous)
        {
            environment.local_regen_elapsed_seconds = 0.0;
        }
        if environment.poisoned {
            environment.local_regen_elapsed_seconds = 0.0;
            environment.local_infection_elapsed_seconds += delta_seconds;
            while environment.local_infection_elapsed_seconds >= LEGACY_LOCAL_INFECTION_TICK_SECONDS
            {
                environment.local_infection_elapsed_seconds -= LEGACY_LOCAL_INFECTION_TICK_SECONDS;
                let previous_hp = hp;
                hp = (hp - runtime.max_hp * 3 / 20).max(0);
                // The tutorial owns HP locally and receives no shard DOT packet.
                // Reuse the packet path's infection audio only on actual damage,
                // including the lethal tick but never subsequent ticks at zero HP.
                if hp < previous_hp {
                    let gender = tutorial
                        .character()
                        .map(|character| character.style.gender)
                        .or_else(|| {
                            runtime
                                .player_gender
                                .and_then(|gender| i8::try_from(gender).ok())
                        })
                        .and_then(|gender| tutorial_player_gender_from_protocol(gender).ok());
                    if let Some(gender) = gender {
                        audio.queue_player_infection_damage(player, gender);
                    }
                }
            }
        } else if local_combat_active {
            // Retrobution refreshes GameFrame's five-second combat lease on
            // every incoming/outgoing result and cnVirtualServer heals only
            // while !bCombatMode. Retain at most one due tick so leaving a
            // long fight cannot produce a burst of deferred regeneration.
            environment.local_regen_elapsed_seconds = (environment.local_regen_elapsed_seconds
                + delta_seconds)
                .min(LEGACY_LOCAL_REGEN_TICK_SECONDS);
        } else {
            environment.local_infection_elapsed_seconds = 0.0;
            environment.local_regen_elapsed_seconds += delta_seconds;
            while environment.local_regen_elapsed_seconds >= LEGACY_LOCAL_REGEN_TICK_SECONDS {
                environment.local_regen_elapsed_seconds -= LEGACY_LOCAL_REGEN_TICK_SECONDS;
                hp = (hp + runtime.max_hp / 5).min(runtime.max_hp);
            }
        }
        runtime.hp = Some(hp);
        environment.last_observed_hp = Some(hp);
    }
}

/// Mirrors clean `Status.UpdateSkillBuff`: time-buff 17 owns one persistent
/// ES376 instance parented to the local avatar for exactly as long as its
/// authoritative condition bit is present.
pub(super) fn sync_local_infection_status_effect(
    skill_buffs: Res<SkillBuffUiModel>,
    mut state: ResMut<LocalInfectionPresentationState>,
    players: Query<(Entity, &GlobalTransform), With<LocalPlayer>>,
    mut effects: ResMut<TutorialEffectRuntime>,
) {
    let desired = (skill_buffs.local_condition_bit_flag & SKILL_BUFF_INFECTION_FLAG != 0)
        .then(|| players.single().ok())
        .flatten();
    let desired_player = desired.map(|(player, _)| player);
    if desired_player == state.active_player
        && (desired_player.is_none()
            || effects.has_named_native_instance(LOCAL_INFECTION_EFFECT_NAME))
    {
        return;
    }
    if state.active_player.take().is_some() {
        effects.enqueue(TutorialEffectRuntimeCommand::DestroyNamed {
            name: LOCAL_INFECTION_EFFECT_NAME.to_owned(),
            source_line: 0,
        });
    }
    let Some((player, transform)) = desired else {
        return;
    };
    effects.enqueue(TutorialEffectRuntimeCommand::Add {
        effect_id: LOCAL_INFECTION_AURA_EFFECT_ID,
        placement: TutorialEffectPlacement::ExactEntityWorld {
            root_entity: player,
            position: transform.translation(),
            rotation: Quat::IDENTITY,
        },
        scale: 1.0,
        tracked: false,
        name: Some(LOCAL_INFECTION_EFFECT_NAME.to_owned()),
        destroy_after_seconds: None,
        source_line: 0,
    });
    state.active_player = Some(player);
}

pub(super) fn spawn_native_world_slice(
    commands: &mut Commands,
    asset_server: &AssetServer,
    world_catalog: &NativeWorldCatalog,
    rig_assets: &mut NativePlayerRigAssetCache,
    rig_catalog: &NativePlayerRigCatalog,
    weapon_animation_catalog: &PlayerWeaponAnimationCatalog,
    character_data: &CharacterCreationData,
    character: &CharacterSummary,
    pc_uid: i64,
    player_id: i32,
    world_scope: NativeWorldScope,
    server_position: [i32; 3],
    server_angle: i32,
    networked: bool,
    gameplay_loading: &mut GameplayLoadingState,
    initial_equipment_commands: Option<&mut TutorialPlayerPresentationCommandQueue>,
) -> Result<(), String> {
    let position = ProtocolPosition::new(server_position).to_native();
    let world_scene = world_catalog
        .select_in_scope(world_scope, position)
        .ok_or_else(|| {
            let available = world_catalog
                .scenes()
                .iter()
                .filter(|scene| scene.scope == Some(world_scope))
                .map(|scene| format!("{:?}", scene.tile))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "server position {server_position:?} converts to {position:?}, outside authored {world_scope:?} tiles [{available}]"
            )
        })?;
    gameplay_loading.begin(match world_scope {
        NativeWorldScope::Tutorial => ResourceLoadingScope::Tutorial,
        NativeWorldScope::WorldMap => ResourceLoadingScope::World,
    });
    // A blocker belongs to one concrete streamed slice. Retrobution rebuilt
    // DongLoader state on LoadPosition, so an old map failure must not poison
    // a later Tutorial or world entry.
    commands.insert_resource(NativeWorldStreamingStatus::default());
    let world_root =
        begin_native_world_scene_admission(commands, world_catalog, world_scene, world_scope)
            .map_err(|error| error.to_string())?;
    commands.entity(world_root).insert(WorldSliceEntity);

    let heading = ProtocolYawDegrees::new(server_angle).legacy_heading();
    let mut controller = LegacyPlayerController::from_baseline_table().with_yaw(heading.degrees());
    controller.movement_enabled = false;
    controller.set_grounded(false);
    let player = commands
        .spawn((
            LocalPlayer,
            WorldSliceEntity,
            LegacyWorldColliderPending,
            controller,
            LegacyAvatarEnvironmentState::default(),
            LegacyAvatarActionContext::default(),
            LegacyAvatarPresentationContext::default(),
            LegacyAvatarTargetFeed::default(),
            LegacyAvatarClipBindings::default(),
            LegacyAvatarActionState::default(),
            Transform::from_translation(position).with_rotation(heading.native_root_rotation()),
            // The assembled rig starts on stand1. Keep its controller root
            // hidden until the gameplay gate confirms the first authored pose
            // (staying in BasicMove) has actually reached AnimationPlayer.
            Visibility::Hidden,
        ))
        .id();
    if networked {
        commands
            .entity(player)
            .insert(LocalNetworkIdentity { pc_uid, player_id });
    }
    let selected_rig = match spawn_tutorial_selected_player_rig(
        commands,
        asset_server,
        rig_assets,
        rig_catalog,
        weapon_animation_catalog,
        character_data,
        player,
        character,
        pc_uid.unsigned_abs(),
        RenderLayers::layer(0),
        None,
        world_scope == NativeWorldScope::Tutorial,
    ) {
        Ok(spawned) => spawned,
        Err(error) => {
            commands.entity(player).despawn();
            commands.entity(world_root).despawn();
            return Err(error);
        }
    };
    // The controller root owns the assembled rig through Bevy's child
    // relationship and is already a WorldSliceEntity. Marking the child too
    // would schedule both parent and descendant for world cleanup.
    commands
        .entity(selected_rig.rig_root)
        .insert(LocalCharacterScene);

    if world_scope == NativeWorldScope::WorldMap
        && let Some(equipment_commands) = initial_equipment_commands
    {
        equipment_commands.clear();
        let item = character.equipment[ffone_protocol::CharacterEquipSlot0104::Hand as usize];
        if item.item_id != 0 {
            equipment_commands.push_equipment(TutorialPlayerEquipmentRequest {
                slot: ffone_protocol::CharacterEquipSlot0104::Hand,
                item,
            });
            let gender = tutorial_player_gender_from_protocol(character.style.gender)
                .map_err(|error| format!("ordinary-world armed pose: {error}"))?;
            if let Some(profile) = weapon_animation_catalog.profile_for_item(item.item_id) {
                let armed_stand =
                    TutorialPlayerAnimationRequest::locomotion(gender, profile.stand())
                        .expect("weapon profile stand is a validated locomotion clip");
                equipment_commands.push_animation(armed_stand);
            }
        }
    }

    let camera = commands
        .spawn((
            WorldSliceEntity,
            Camera3d::default(),
            // Primary `Good` quality has antiAliasing=0. Camera3d otherwise
            // requires Bevy's default four-sample MSAA, multiplying the cost
            // of terrain and alpha-tested foliage without source evidence.
            Msaa::Off,
            Camera {
                // The camera-owned Unity skybox is reproduced by a background
                // pass. Preserve that color target here while still letting this
                // camera own the exact world depth/far-clip contract.
                clear_color: ClearColorConfig::None,
                ..default()
            },
            Projection::Perspective(PerspectiveProjection {
                // FOV and near clip are exact MainCamera object 40 values from
                // Retrobution mainData. The far clip is a deliberate modest
                // extension; the exact Dong fog still covers its outer band.
                fov: 45.0_f32.to_radians(),
                near: 0.15,
                far: EXTENDED_WORLD_CAMERA_FAR_NATIVE,
                ..default()
            }),
            // Retrobution's SFX AudioSources are positional. Keep the one
            // gameplay listener on MainCamera so actor/player cues pan and
            // attenuate while cutscene camera choreography moves it.
            SpatialListener::new(0.2),
            LegacyOrbitCamera::new(player).with_yaw(heading.degrees()),
            Transform::from_xyz(position.x, position.y + 4.0, position.z - 6.0),
        ))
        .id();
    if LEGACY_REFERENCE_GLOW_ENABLED {
        commands
            .entity(camera)
            .insert(LegacyGlowSettings::default());
    }
    Ok(())
}
