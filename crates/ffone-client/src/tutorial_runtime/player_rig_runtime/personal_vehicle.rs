//! Server-confirmed personal vehicle attached to the exact animated foot socket.
use super::*;
use crate::avatar_action::LegacyVehiclePresentationFamily;
use crate::legacy_model_material::LegacyStaticMaterialCache;

#[derive(Resource, Default, Debug)]
pub struct PersonalVehiclePresentation {
    pub item_id: Option<i16>,
    pub engine_sound: Option<String>,
    pub speed_server_units: i32,
    pub family: LegacyVehiclePresentationFamily,
}

#[derive(Component)]
pub(crate) struct VehicleAttachment {
    rig_root: Entity,
    item_id: i16,
    local: bool,
    gltf: Handle<Gltf>,
    look: NativePlayerLook,
}

impl VehicleAttachment {
    pub(crate) fn model_path(&self) -> &str {
        &self.look.parts[0].glb
    }
}

#[derive(Component)]
pub(super) struct VehicleMaterialBound(#[allow(dead_code)] NativePlayerMaterialBinding);

pub(super) fn sync_vehicle_attachment(
    mut commands: Commands,
    presentation: Res<PersonalVehiclePresentation>,
    data: Res<CharacterCreationDataResource>,
    assets: Res<AssetServer>,
    rigs: Query<
        (
            Entity,
            &TutorialSelectedPlayerRig,
            &NativePlayerRigBones,
            &TutorialSelectedPlayerRigStatus,
        ),
        With<TutorialSelectedPlayerRigActive>,
    >,
    attachments: Query<(Entity, &VehicleAttachment)>,
    transforms: Query<&GlobalTransform>,
    mut rejected: Local<Option<(Entity, i16)>>,
) {
    for (entity, attachment) in &attachments {
        if !attachment.local {
            continue;
        }
        if presentation.item_id != Some(attachment.item_id)
            || rigs.get(attachment.rig_root).is_err()
        {
            commands.entity(entity).despawn();
        }
    }
    let Some(item_id) = presentation.item_id else {
        *rejected = None;
        return;
    };
    let Ok((root, rig, bones, status)) = rigs.single() else {
        return;
    };
    if !matches!(status, TutorialSelectedPlayerRigStatus::Ready) {
        return;
    }
    if *rejected == Some((root, item_id))
        || attachments
            .iter()
            .any(|(_, a)| a.rig_root == root && a.item_id == item_id)
    {
        return;
    }
    let path = player_attachment_socket_full_path(rig.gender, LegacyPlayerAttachmentSlot::Vehicle);
    let Some(socket) = bones.by_full_path(&path) else {
        return;
    };
    let Ok(world) = transforms.get(socket) else {
        return;
    };
    // LoadVehicle instantiates at unit world scale, then reparents retaining it.
    let Some(placement) = skyway_attachment_transform(world) else {
        return;
    };
    let part = match data.0.resolve_vehicle_attachment(item_id as u32) {
        Ok(part) => part,
        Err(error) => {
            warn!("Vehicle {item_id}: {error}");
            *rejected = Some((root, item_id));
            return;
        }
    };
    let scene = assets.load(GltfAssetLabel::Scene(0).from_asset(part.glb.clone()));
    let gltf = assets.load(part.glb.clone());
    let mut look = rig.look.clone();
    look.parts = vec![part];
    commands.spawn((
        Name::new(format!("Personal vehicle {item_id}")),
        VehicleAttachment {
            rig_root: root,
            item_id,
            local: true,
            gltf,
            look,
        },
        ChildOf(socket),
        WorldAssetRoot(scene),
        placement,
        Visibility::Inherited,
        rig.render_layers.clone(),
    ));
}

pub(super) fn bind_vehicle_materials(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
    mut cache: ResMut<LegacyStaticMaterialCache>,
    children: Query<&Children>,
    vehicles: Query<(Entity, &VehicleAttachment)>,
    metadata: Query<&PendingLegacyModelMaterial>,
    surfaces: Query<
        (
            Entity,
            &MeshMaterial3d<LegacyModelMaterial>,
            Option<&PendingLegacyModelMaterial>,
            Option<&LegacyMaterialPassCompanion>,
        ),
        Without<VehicleMaterialBound>,
    >,
) {
    for (root, vehicle) in &vehicles {
        for entity in children.iter_descendants(root) {
            let Ok((entity, handle, own, companion)) = surfaces.get(entity) else {
                continue;
            };
            let Some(pending) =
                own.or_else(|| companion.and_then(|p| metadata.get(p.source_mesh_entity).ok()))
            else {
                continue;
            };
            let Some(source) = materials.get(&handle.0) else {
                continue;
            };
            let mut material = source.clone();
            match bind_native_player_look_material(
                &assets,
                pending,
                &mut material,
                &vehicle.look,
                &vehicle.look.parts[0],
                companion.is_some(),
            ) {
                Ok(binding) => {
                    let handle = cache.admit_immutable(material, &mut materials);
                    commands
                        .entity(entity)
                        .insert((MeshMaterial3d(handle), VehicleMaterialBound(binding)));
                }
                Err(error) => {
                    warn!("Vehicle material: {error}");
                }
            }
        }
    }
}

pub(super) fn vehicle_clip(
    family: LegacyVehiclePresentationFamily,
    clip: LegacyVisualClip,
) -> Option<TutorialPlayerClip> {
    match (family, clip) {
        (
            LegacyVehiclePresentationFamily::Board,
            LegacyVisualClip::Stand1 | LegacyVisualClip::Ready,
        ) => Some(TutorialPlayerClip::BoardStand1),
        (LegacyVehiclePresentationFamily::Board, LegacyVisualClip::Run) => {
            Some(TutorialPlayerClip::BoardRun)
        }
        (LegacyVehiclePresentationFamily::Board, LegacyVisualClip::RunBack) => {
            Some(TutorialPlayerClip::BoardRunBack)
        }
        (LegacyVehiclePresentationFamily::Board, LegacyVisualClip::JumpStart) => {
            Some(TutorialPlayerClip::BoardJumpStart)
        }
        (LegacyVehiclePresentationFamily::Board, LegacyVisualClip::Jump) => {
            Some(TutorialPlayerClip::BoardJump)
        }
        (LegacyVehiclePresentationFamily::Board, LegacyVisualClip::JumpEnd) => {
            Some(TutorialPlayerClip::BoardJumpEnd)
        }
        (LegacyVehiclePresentationFamily::Board, LegacyVisualClip::JumpLandRun) => {
            Some(TutorialPlayerClip::BoardJumpLandRun)
        }
        (
            LegacyVehiclePresentationFamily::Scooter,
            LegacyVisualClip::Stand1 | LegacyVisualClip::Ready,
        ) => Some(TutorialPlayerClip::ScooterStand1),
        (LegacyVehiclePresentationFamily::Scooter, LegacyVisualClip::Run) => {
            Some(TutorialPlayerClip::ScooterRun)
        }
        (LegacyVehiclePresentationFamily::Scooter, LegacyVisualClip::RunBack) => {
            Some(TutorialPlayerClip::ScooterRunBack)
        }
        (LegacyVehiclePresentationFamily::Scooter, LegacyVisualClip::JumpStart) => {
            Some(TutorialPlayerClip::ScooterJumpStart)
        }
        (LegacyVehiclePresentationFamily::Scooter, LegacyVisualClip::Jump) => {
            Some(TutorialPlayerClip::ScooterJump)
        }
        (LegacyVehiclePresentationFamily::Scooter, LegacyVisualClip::JumpEnd) => {
            Some(TutorialPlayerClip::ScooterJumpEnd)
        }
        (LegacyVehiclePresentationFamily::Scooter, LegacyVisualClip::JumpLandRun) => {
            Some(TutorialPlayerClip::ScooterJumpLandRun)
        }
        _ => None,
    }
}

pub(super) const fn end_event(gender: PlayerRigGender, clip: TutorialPlayerClip) -> Option<f32> {
    match (gender, clip) {
        (PlayerRigGender::Male, TutorialPlayerClip::BoardJump) => Some(1.350000024),
        (PlayerRigGender::Male, TutorialPlayerClip::BoardJumpEnd) => Some(0.350000083),
        (PlayerRigGender::Male, TutorialPlayerClip::BoardJumpLandRun) => Some(0.350000083),
        (PlayerRigGender::Male, TutorialPlayerClip::BoardJumpStart) => Some(0.683333337),
        (PlayerRigGender::Male, TutorialPlayerClip::BoardRun) => Some(0.816666722),
        (PlayerRigGender::Male, TutorialPlayerClip::BoardRunBack) => Some(0.816666722),
        (PlayerRigGender::Male, TutorialPlayerClip::BoardStand1) => Some(1.350000024),
        (PlayerRigGender::Male, TutorialPlayerClip::ScooterJump) => Some(1.350000024),
        (PlayerRigGender::Male, TutorialPlayerClip::ScooterJumpEnd) => Some(0.350000083),
        (PlayerRigGender::Male, TutorialPlayerClip::ScooterJumpLandRun) => Some(0.350000083),
        (PlayerRigGender::Male, TutorialPlayerClip::ScooterJumpStart) => Some(0.683333337),
        (PlayerRigGender::Male, TutorialPlayerClip::ScooterRun) => Some(0.816666722),
        (PlayerRigGender::Male, TutorialPlayerClip::ScooterRunBack) => Some(0.816666722),
        (PlayerRigGender::Male, TutorialPlayerClip::ScooterStand1) => Some(1.350000024),
        (PlayerRigGender::Female, TutorialPlayerClip::BoardJump) => Some(1.350000024),
        (PlayerRigGender::Female, TutorialPlayerClip::BoardJumpEnd) => Some(0.350000083),
        (PlayerRigGender::Female, TutorialPlayerClip::BoardJumpLandRun) => Some(0.350000083),
        (PlayerRigGender::Female, TutorialPlayerClip::BoardJumpStart) => Some(0.683333337),
        (PlayerRigGender::Female, TutorialPlayerClip::BoardRun) => Some(0.816666722),
        (PlayerRigGender::Female, TutorialPlayerClip::BoardRunBack) => Some(0.816666722),
        (PlayerRigGender::Female, TutorialPlayerClip::BoardStand1) => Some(1.350000024),
        (PlayerRigGender::Female, TutorialPlayerClip::ScooterJump) => Some(1.350000024),
        (PlayerRigGender::Female, TutorialPlayerClip::ScooterJumpEnd) => Some(0.350000083),
        (PlayerRigGender::Female, TutorialPlayerClip::ScooterJumpLandRun) => Some(0.350000083),
        (PlayerRigGender::Female, TutorialPlayerClip::ScooterJumpStart) => Some(0.683333337),
        (PlayerRigGender::Female, TutorialPlayerClip::ScooterRun) => Some(0.816666722),
        (PlayerRigGender::Female, TutorialPlayerClip::ScooterRunBack) => Some(0.816666722),
        (PlayerRigGender::Female, TutorialPlayerClip::ScooterStand1) => Some(1.350000024),
        _ => None,
    }
}

pub(super) fn vehicle_animation_audio(
    rigs: Query<
        (&TutorialSelectedPlayerRig, &TutorialPlayerAnimationAdapter),
        With<TutorialSelectedPlayerRigActive>,
    >,
    players: Query<&AnimationPlayer>,
    mut audio: Option<ResMut<crate::gameplay_audio::GameplayAudioRuntime>>,
    mut cursor: Local<Option<(Entity, TutorialPlayerClip, f32)>>,
) {
    let Ok((rig, adapter)) = rigs.single() else {
        *cursor = None;
        return;
    };
    let clip = adapter.active_clip;
    let cue = match clip {
        TutorialPlayerClip::BoardJumpStart | TutorialPlayerClip::ScooterJumpStart => {
            "Vehicle_JumpStart.wav"
        }
        TutorialPlayerClip::BoardJumpEnd | TutorialPlayerClip::ScooterJumpEnd => {
            "Vehicle_JumpLnd.wav"
        }
        _ => {
            *cursor = None;
            return;
        }
    };
    let Some(active) = players
        .get(adapter.animation_player)
        .ok()
        .and_then(|p| p.animation(adapter.active_node))
    else {
        return;
    };
    let current = active.seek_time();
    let previous = cursor
        .filter(|(owner, prior, time)| {
            *owner == rig.controller_root && *prior == clip && *time <= current
        })
        .map_or(-1.0, |(_, _, t)| t);
    if previous < 0.25 && current >= 0.25 {
        if let Some(audio) = audio.as_deref_mut() {
            audio.queue_legacy_animation_sound(rig.controller_root, cue);
        }
    }
    *cursor = Some((rig.controller_root, clip, current));
}

#[derive(Component)]
pub(super) struct VehicleEngine {
    item_id: i16,
    owner: Entity,
}

pub(super) fn vehicle_engine_audio(
    mut commands: Commands,
    presentation: Res<PersonalVehiclePresentation>,
    catalog: Option<Res<crate::semantic_audio::NativeAudioCatalog>>,
    assets: Res<AssetServer>,
    rigs: Query<&TutorialSelectedPlayerRig, With<TutorialSelectedPlayerRigActive>>,
    controllers: Query<&LegacyPlayerController>,
    mut engines: Query<(
        Entity,
        &VehicleEngine,
        &mut PlaybackSettings,
        Option<&AudioSink>,
    )>,
    mut rejected: Local<Option<i16>>,
) {
    use bevy::audio::AudioSinkPlayback;
    let owner = rigs.single().ok().map(|r| r.controller_root);
    let pitch = owner
        .and_then(|o| controllers.get(o).ok())
        .map_or(1.0, |c| {
            if presentation.speed_server_units <= 0
                || presentation
                    .engine_sound
                    .as_deref()
                    .is_some_and(|s| s.eq_ignore_ascii_case("vehicle_timesquad"))
            {
                1.0
            } else {
                (Vec2::new(c.velocity.x, c.velocity.z).length()
                    / (presentation.speed_server_units as f32 * 0.015)
                    * 2.0)
                    .max(1.0)
            }
        });
    let mut active = false;
    for (entity, engine, mut settings, sink) in &mut engines {
        if presentation.item_id != Some(engine.item_id) || owner != Some(engine.owner) {
            commands.entity(entity).despawn();
        } else {
            active = true;
            if settings.speed != pitch {
                settings.speed = pitch;
            }
            if let Some(sink) = sink {
                if sink.speed() != pitch {
                    sink.set_speed(pitch);
                }
            }
        }
    }
    let Some(item_id) = presentation.item_id else {
        *rejected = None;
        return;
    };
    if active || *rejected == Some(item_id) {
        return;
    }
    let (Some(owner), Some(name), Some(catalog)) =
        (owner, presentation.engine_sound.as_deref(), catalog)
    else {
        return;
    };
    let matches: Vec<_> = catalog
        .by_true_name(name)
        .into_iter()
        .filter(|a| a.category == crate::semantic_audio::NativeAudioCategory::Sfx)
        .collect();
    let [sound] = matches.as_slice() else {
        warn!("Vehicle engine {name:?} has no unique native SFX route");
        *rejected = Some(item_id);
        return;
    };
    commands.spawn((
        Name::new("Personal vehicle engine"),
        VehicleEngine { item_id, owner },
        ChildOf(owner),
        AudioPlayer::new(assets.load(sound.path.clone())),
        PlaybackSettings {
            speed: pitch,
            ..PlaybackSettings::LOOP
        },
        crate::audio_channel::GameplayAudioChannel::new(
            crate::semantic_audio::NativeAudioCategory::Sfx,
            if presentation.family == LegacyVehiclePresentationFamily::Board { 0.5 } else { 1.0 },
        ),
    ));
}

pub(super) fn sync_remote_vehicle_attachments(
    mut commands: Commands,
    data: Res<CharacterCreationDataResource>,
    assets: Res<AssetServer>,
    players: Query<(
        &crate::network_world_runtime::NetworkPcVisual0104,
        &crate::entity_lifecycle::NetworkPcAppearance0104,
    )>,
    bones: Query<(
        &NativePlayerRigBones,
        &crate::network_world_runtime::NetworkPcRigAppearanceStatus0104,
    )>,
    attachments: Query<(Entity, &VehicleAttachment)>,
    transforms: Query<&GlobalTransform>,
    mut rejected: Local<BTreeSet<(Entity, i16)>>,
) {
    rejected.retain(|(root, _)| {
        players
            .iter()
            .any(|(v, a)| v.rig_root == *root && a.0.pc_state == 8)
    });
    for (entity, attachment) in &attachments {
        if attachment.local {
            continue;
        }
        if !players.iter().any(|(v, a)| {
            v.rig_root == attachment.rig_root
                && a.0.pc_state == 8
                && a.0.equipment[8].item_id == attachment.item_id
        }) {
            commands.entity(entity).despawn();
        }
    }
    for (visual, appearance) in &players {
        if appearance.0.pc_state != 8 {
            continue;
        }
        let item = appearance.0.equipment[8];
        let root = visual.rig_root;
        if item.item_id <= 0
            || item.item_type != 10
            || rejected.contains(&(root, item.item_id))
            || attachments
                .iter()
                .any(|(_, a)| a.rig_root == root && a.item_id == item.item_id)
        {
            continue;
        }
        let Ok(gender) = crate::tutorial_player_presentation::tutorial_player_gender_from_protocol(
            appearance.0.style.gender,
        ) else {
            continue;
        };
        let Ok((bones, crate::network_world_runtime::NetworkPcRigAppearanceStatus0104::Ready)) =
            bones.get(root)
        else {
            continue;
        };
        let Some(socket) = bones.by_full_path(&player_attachment_socket_full_path(
            gender,
            LegacyPlayerAttachmentSlot::Vehicle,
        )) else {
            continue;
        };
        let Some(placement) = transforms
            .get(socket)
            .ok()
            .and_then(skyway_attachment_transform)
        else {
            continue;
        };
        let part = match data.0.resolve_vehicle_attachment(item.item_id as u32) {
            Ok(part) => part,
            Err(error) => {
                warn!("Remote vehicle {}: {error}", item.item_id);
                rejected.insert((root, item.item_id));
                continue;
            }
        };
        let scene = assets.load(GltfAssetLabel::Scene(0).from_asset(part.glb.clone()));
        let gltf = assets.load(part.glb.clone());
        let look = NativePlayerLook {
            identity: format!("vehicle:{}", item.item_id),
            gender,
            parts: vec![part],
            skin_texture: None,
            skin_color: LinearRgba::WHITE,
            hair_color: LinearRgba::WHITE,
            weapon_animation_profile: None,
            height_selector: 0,
            body_selector: 0,
        };
        commands.spawn((
            Name::new(format!("Remote personal vehicle {}", item.item_id)),
            VehicleAttachment {
                rig_root: root,
                item_id: item.item_id,
                local: false,
                gltf,
                look,
            },
            ChildOf(socket),
            WorldAssetRoot(scene),
            placement,
            Visibility::Inherited,
        ));
    }
}

#[derive(Component)]
pub(super) struct PendingVehicleAnimation;

pub(super) fn play_vehicle_model_animation(
    mut commands: Commands,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    parents: Query<&ChildOf>,
    attachments: Query<&VehicleAttachment>,
    mut players: Query<
        (Entity, &mut AnimationPlayer),
        Or<(Added<AnimationPlayer>, With<PendingVehicleAnimation>)>,
    >,
) {
    for (entity, mut player) in &mut players {
        let mut owner = entity;
        let attachment = loop {
            if let Ok(attachment) = attachments.get(owner) {
                break Some(attachment);
            }
            let Ok(parent) = parents.get(owner) else {
                break None;
            };
            owner = parent.parent();
        };
        let Some(attachment) = attachment else {
            continue;
        };
        let Some(gltf) = gltfs.get(&attachment.gltf) else {
            commands.entity(entity).insert(PendingVehicleAnimation);
            continue;
        };
        // Published model defaults preserve the source's automatic idle loop.
        if let Some(clip) = gltf
            .named_animations
            .get("nif-default")
            .or_else(|| gltf.named_animations.get("idle"))
        {
            let (graph, node) = AnimationGraph::from_clip(clip.clone());
            player.start(node).repeat();
            commands
                .entity(entity)
                .insert(AnimationGraphHandle(graphs.add(graph)));
        }
        commands.entity(entity).remove::<PendingVehicleAnimation>();
    }
}
