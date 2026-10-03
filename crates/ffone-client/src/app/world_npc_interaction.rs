//! World NPC service entries and interaction collection.

use super::guide::GuideProductionRuntime;
use super::local_inventory::LocalInventoryRuntime;
use super::npc_warp::normal_npc_warp_ui_entry;
use super::race::{
    CleanRaceNpcRoute, RaceProductionRuntime, clean_race_npc_route, open_race_mode_from_npc,
};
use super::runtime_status::RuntimeStatus;
use super::transportation::TransportationProductionRuntime;
use super::tutorial_choreography::*;
use super::tutorial_combat::{tutorial_active_nano_style, tutorial_named_descendant_position};
use super::user_equip::LocalVehiclePresentationRuntime;
use super::world_combat::{
    WorldNpcInteractionCombatVisuals, authoritative_world_hand_weapon_item_id,
    validated_world_weapon_bullet_links, world_weapon_projectile_endpoints,
    world_weapon_swap_request,
};
use super::world_nano::{active_world_nano_slot, ordinary_world_nano_skill_supported};
use super::{LocalPlayer, movement_buffs, nano_recall};
use bevy::{
    ecs::system::SystemParam,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};
use ffone_client::{
    avatar_action::{
        LegacyAvatarActionContext, LegacyAvatarActionIntent, LegacyAvatarActionQueue,
        LegacyAvatarTargetFeed, LegacyTargetKind, LegacyVehiclePresentationFamily,
        LegacyVisualClip,
    },
    entity_lifecycle::NetworkNpcAppearance0104,
    gameplay_audio::{LegacyNpcVoiceCue, legacy_npc_open_voice_cue},
    gameplay_ui::{NpcBarkerBubbleRuntime, NpcServiceKind},
    guide_runtime::{GuideNpcServiceRoute, GuideRuntime, guide_npc_service_route},
    localization::LocalizedText,
    mission_ui::{MissionUiModel, NpcInteractionUi, NpcServiceUiEntry},
    movement::SERVER_TO_CLIENT_SCALE,
    nano_free_tuning_runtime::NanoFreeTuningBank0104,
    network::{NetworkBridge, NetworkCommand},
    race_ui::{
        mode::{RaceEcomType, RaceModeModel},
        rank::RaceRankCatalog,
    },
    rule_runtime::rule_npc_service_route,
    transportation_ui::TransportationCatalog,
    tutorial_effects_runtime::{TutorialEffectRuntimeCommand, TutorialProjectileMotion},
    tutorial_mission_content::TutorialMissionContent,
    tutorial_nano_gameplay::TutorialNanoGameplayCommandQueue,
    user_equip_runtime::{UserEquipPendingRequest0104, UserEquipProductionRuntime0104},
    world::{AUTHORED_CHARACTER_CONTROLLER_HEIGHT, AUTHORED_CHARACTER_CONTROLLER_RADIUS},
    world_behaviour::WorldTriggerUseQueue,
    world_combat::{WorldPrimaryAttack0104, build_world_primary_attack_0104},
    world_mission_runtime::WorldMissionRuntime,
    world_nano_cooldown::WorldNanoCooldownRuntime,
    world_nano_runtime::{
        WorldNanoPlanInput, WorldNanoTarget, WorldNanoTargetKind, plan_world_nano_skill_0104,
    },
};
use ffone_protocol::{NpcInteractionRequest0104, PcVehicleOffRequest0104};
use std::collections::BTreeSet;

pub(super) fn guide_service_entry(
    service_category: i32,
    payment_flag: Option<i8>,
) -> Option<NpcServiceUiEntry> {
    let route = if (18..=22).contains(&service_category) {
        guide_npc_service_route(service_category, 0)
    } else {
        payment_flag.and_then(|flag| guide_npc_service_route(service_category, flag))
    }?;
    let service = match route {
        GuideNpcServiceRoute::GuideChanger { .. } => NpcServiceKind::GuideChanger,
        GuideNpcServiceRoute::PastWarp | GuideNpcServiceRoute::UpsellRequired => {
            NpcServiceKind::PastWarp
        }
    };
    Some(NpcServiceUiEntry::original(service))
}

pub(super) const fn bank_service_kind(service_category: i32) -> Option<NpcServiceKind> {
    match service_category {
        12 => Some(NpcServiceKind::Bank),
        50 => Some(NpcServiceKind::LocalBank),
        _ => None,
    }
}

#[must_use]
pub(super) const fn combi_service_allowed_0104(service_category: i32, service: NpcServiceKind) -> bool {
    matches!((service_category, service), (26, NpcServiceKind::Combine))
}

#[must_use]
pub(super) const fn enchant_service_allowed_0104(service_category: i32, service: NpcServiceKind) -> bool {
    matches!((service_category, service), (27, NpcServiceKind::Enchant))
}

pub(super) fn clean_category_service_entries(
    service_category: i32,
    ring_race_active: bool,
) -> Vec<NpcServiceUiEntry> {
    let services: &[NpcServiceKind] = match service_category {
        7..=9 => &[NpcServiceKind::NanoStation],
        13 => &[NpcServiceKind::Race, NpcServiceKind::RaceRank],
        14 => &[NpcServiceKind::RaceRank],
        15 => &[NpcServiceKind::TransportationWarp],
        16 => &[NpcServiceKind::TransportationWyvern],
        // Category 17 calls StartRXCom and returns before the menu is built.
        26 => &[NpcServiceKind::Combine],
        27 => &[NpcServiceKind::Enchant],
        28 => &[NpcServiceKind::Barber],
        _ => &[],
    };
    let mut entries = services
        .iter()
        .copied()
        .map(NpcServiceUiEntry::original)
        .collect::<Vec<_>>();
    if service_category == 13 && ring_race_active {
        if let Some(entry) = entries
            .iter_mut()
            .find(|entry| entry.service == NpcServiceKind::Race)
        {
            // Exact clean NpcIconMode cancellation row. The leading space is
            // retained by the native NpcServiceUiEntry layout contract.
            entry.label = " CANCEL RACE".to_owned();
        }
    }
    entries
}

pub(super) fn world_npc_service_entries(
    content: &TutorialMissionContent,
    npc_type: i32,
    service_category: i32,
    service_number: Option<i32>,
    payment_flag: Option<i8>,
    ring_race_active: bool,
) -> Vec<NpcServiceUiEntry> {
    let mut services = Vec::new();
    // Clean `NpcIconMode.InitMode` calls `CheckVendor` before every other
    // service check, so ENTER STORE owns the first utility row.
    if content.gameplay_npc_is_vendor(npc_type) {
        services.push(NpcServiceUiEntry::original(NpcServiceKind::Vendor));
    }
    if matches!(service_category, 7..=9) {
        services.extend(clean_category_service_entries(
            service_category,
            ring_race_active,
        ));
    }
    services.extend(bank_service_kind(service_category).map(NpcServiceUiEntry::original));
    services.extend(guide_service_entry(service_category, payment_flag));
    if !matches!(service_category, 7..=9) {
        services.extend(clean_category_service_entries(
            service_category,
            ring_race_active,
        ));
    }
    if let Some(route) = service_number.and_then(rule_npc_service_route) {
        services.push(NpcServiceUiEntry {
            service: NpcServiceKind::Rule,
            label: format!("{}{}", route.label_prefix, route.label_key),
        });
    }
    services
}

#[derive(SystemParam)]
pub(super) struct WorldNpcMissionInputs<'w, 's> {
    pub(super) inventory: Res<'w, LocalInventoryRuntime>,
    pub(super) player_prompt: ResMut<'w, super::group_pc2pc::WorldPc2pcOfferPrompt>,
    pub(super) system_messages: ResMut<'w, ffone_client::system_message_ui::SystemMessageUiModel>,
    pub(super) runtime: Res<'w, WorldMissionRuntime>,
    pub(super) guide: Res<'w, GuideRuntime>,
    pub(super) guide_production: Res<'w, GuideProductionRuntime>,
    pub(super) nano_bank: Res<'w, NanoFreeTuningBank0104>,
    pub(super) race_mode: ResMut<'w, RaceModeModel>,
    pub(super) race_catalog: Res<'w, RaceRankCatalog>,
    pub(super) race_production: ResMut<'w, RaceProductionRuntime>,
    pub(super) transportation_production: ResMut<'w, TransportationProductionRuntime>,
    pub(super) npc_bubbles: ResMut<'w, NpcBarkerBubbleRuntime>,
    pub(super) cursors: Query<'w, 's, &'static CursorOptions, With<PrimaryWindow>>,
}

#[derive(SystemParam)]
pub(super) struct WorldNpcEquipmentProduction<'w> {
    pub(super) vehicle: Res<'w, LocalVehiclePresentationRuntime>,
    pub(super) item_move_owner: ResMut<'w, UserEquipProductionRuntime0104>,
}

/// Applies the client-only facing edge from clean
/// `NpcIconMode.InitMode -> NpcMoveController.SetForceAngle(player)`.
///
/// Primary evidence is `retrobution-20260613/main.unity3d` (7,000,415 bytes,
/// SHA-256 `59788201962B6A1737B114486C361FE74EEF69F507D1D125CA3171377EEC602F`)
/// and its bundled `Assembly - CSharp.dll` (1,517,568 bytes, SHA-256
/// `33D6F70216B1C7BA05BCC0F270FBA97E767B129159755AF4C8835922E60ACADB`).
/// The exact managed owners were inspected offline with repository
/// `ilspycmd 11.0.0.9375 -t NpcIconMode` and `-t NpcMoveController`.
///
/// Retrobution writes only the NPC gameplay-root transform here. It neither
/// edits the cached server appearance angle nor serializes a facing packet,
/// and `NpcIconMode.EndMode` does not restore the pre-dialogue rotation.
pub(super) fn face_world_npc_toward_player_locally(
    npc_transform: &mut Transform,
    player_position: Vec3,
) -> bool {
    let Some(heading) = tutorial_player_heading_toward(npc_transform, player_position) else {
        return false;
    };
    npc_transform.rotation = heading.native_root_rotation();
    true
}

pub(super) fn collect_world_npc_interactions(
    mut actions: ResMut<LegacyAvatarActionQueue>,
    mut trigger_uses: ResMut<WorldTriggerUseQueue>,
    bridge: Res<NetworkBridge>,
    content: Res<TutorialMissionContent>,
    transportation_catalog: Res<TransportationCatalog>,
    mut mission: WorldNpcMissionInputs,
    npc_appearances: Query<&NetworkNpcAppearance0104>,
    attack_actors: Query<
        (&Transform, &LegacyAvatarActionContext),
        (With<LocalPlayer>, Without<NetworkNpcAppearance0104>),
    >,
    target_feeds: Query<&LegacyAvatarTargetFeed, With<LocalPlayer>>,
    mut combat_visuals: WorldNpcInteractionCombatVisuals,
    mut mission_ui: ResMut<MissionUiModel>,
    mut gameplay_nano: ResMut<TutorialNanoGameplayCommandQueue>,
    mut nano_cooldowns: ResMut<WorldNanoCooldownRuntime>,
    mut equipment: WorldNpcEquipmentProduction,
    mut runtime: ResMut<RuntimeStatus>,
) {
    let authoritative_weapon_item_id = authoritative_world_hand_weapon_item_id(&mission.inventory);
    for request in actions.take_all() {
        if attack_actors
            .get(request.actor)
            .is_ok_and(|(_, context)| matches!(context.time_buff_condition, 5 | 6))
        {
            continue;
        }
        match request.intent {
            LegacyAvatarActionIntent::UseTrigger(trigger) => {
                trigger_uses.push(request.actor, trigger);
            }
            LegacyAvatarActionIntent::PrimaryAttack { mode, targets } => {
                let Ok((transform, context)) = attack_actors.get(request.actor) else {
                    runtime.message =
                        "World attack ignored an action without a live local avatar".to_owned();
                    continue;
                };
                let Some(attack) = build_world_primary_attack_0104(
                    mode,
                    transform,
                    context.attack_range,
                    &targets,
                    |entity| {
                        npc_appearances
                            .get(entity)
                            .ok()
                            .map(|appearance| appearance.0.npc_id)
                    },
                ) else {
                    runtime.message = "World attack rejected non-finite avatar geometry".to_owned();
                    continue;
                };
                if let Some(rig) = combat_visuals
                    .rigs
                    .iter()
                    .find(|rig| rig.controller_root == request.actor)
                {
                    if let Some(item_id) = authoritative_weapon_item_id {
                        combat_visuals.gameplay_audio.queue_player_weapon_attack(
                            request.actor,
                            rig.gender,
                            item_id,
                            runtime.weapon_battery,
                            &combat_visuals.weapon_catalog,
                        );

                        if let Some(profile) = combat_visuals
                            .weapon_catalog
                            .combat_profile_for_item(item_id)
                        {
                            let bullet_type =
                                profile.native_presentation_bullet_type(runtime.weapon_battery);
                            if let Some((fire_link, success_link)) =
                                validated_world_weapon_bullet_links(
                                    combat_visuals.effect_library.0.as_ref(),
                                    bullet_type,
                                )
                            {
                                // BulletContainer recursively resolves the exact
                                // row-owned FireLink from the whole avatar root.
                                // Its CharacterController.center fallback is
                                // (0, 0.8, 0) in the authored player controller.
                                let source = tutorial_named_descendant_position(
                                    request.actor,
                                    fire_link,
                                    &combat_visuals.parents,
                                    &combat_visuals.named_transforms,
                                )
                                .unwrap_or(transform.translation + Vec3::Y * 0.8);
                                let attacker_style = tutorial_active_nano_style(&runtime);
                                for endpoint in world_weapon_projectile_endpoints(
                                    &attack,
                                    transform,
                                    context.attack_range,
                                    &targets,
                                    |entity| {
                                        let appearance = npc_appearances.get(entity).ok()?;
                                        let target_transform =
                                            combat_visuals.npc_transforms.get(entity).ok()?;
                                        let definition =
                                            content.gameplay_npc(appearance.0.npc_type)?;
                                        let target = tutorial_named_descendant_position(
                                            entity,
                                            success_link,
                                            &combat_visuals.parents,
                                            &combat_visuals.named_transforms,
                                        )
                                        .unwrap_or_else(|| {
                                            target_transform.translation
                                                + Vec3::Y
                                                    * (definition.height_server_units as f32
                                                        * 0.005)
                                        });
                                        Some((appearance.0.npc_id, target, definition.npc_style))
                                    },
                                ) {
                                    combat_visuals.effect_runtime.enqueue(
                                        TutorialEffectRuntimeCommand::Projectile {
                                            bullet_type,
                                            source,
                                            target: endpoint.target,
                                            target_exists: endpoint.target_exists,
                                            // AvatarUtil's Transform/Vector3
                                            // overload leaves both styles at
                                            // BulletGenData's -1 defaults when
                                            // the attack has no target.
                                            source_style: if endpoint.target_exists {
                                                attacker_style
                                            } else {
                                                -1
                                            },
                                            target_style: endpoint.target_style,
                                            motion: TutorialProjectileMotion::BulletMove,
                                            source_line: line!(),
                                        },
                                    );
                                }
                            }
                        }
                    } else {
                        let full_body =
                            combat_visuals
                                .action_states
                                .get(request.actor)
                                .is_ok_and(|state| {
                                    matches!(
                                        state.base_action(),
                                        Some(LegacyVisualClip::AttackFull(_))
                                    )
                                });
                        combat_visuals.gameplay_audio.queue_player_unarmed_attack(
                            request.actor,
                            rig.gender,
                            full_body,
                        );
                    }
                }
                let command = match attack {
                    // Clean `cnAvatarAttack` still plays its miss animation and
                    // untargeted bullet locally, but it never serializes a
                    // zero-target PC_ATTACK_NPCS request.
                    WorldPrimaryAttack0104::Hitscan(request) if request.npc_ids.is_empty() => None,
                    WorldPrimaryAttack0104::Hitscan(request) => {
                        Some(NetworkCommand::AttackNpcs(request))
                    }
                    WorldPrimaryAttack0104::Rocket(request) => {
                        Some(NetworkCommand::RocketStyleFire(request))
                    }
                    WorldPrimaryAttack0104::Grenade(request) => {
                        Some(NetworkCommand::GrenadeStyleFire(request))
                    }
                };
                if let Some(command) = command
                    && let Err(error) = bridge.send(command)
                {
                    runtime.message = format!("World attack send failed: {error}");
                }
            }
            LegacyAvatarActionIntent::TalkNpc(entity) => {
                // A queued world click may outlive the input snapshot which
                // produced it. Never reopen a conversation through its own UI.
                if mission_ui.gameplay_input_blocked() {
                    continue;
                }
                let target_still_talkable = target_feeds.get(request.actor).is_ok_and(|feed| {
                    feed.source_connected
                        && feed.samples.iter().any(|sample| {
                            sample.entity == entity
                                && sample.in_view
                                && sample.talk_enabled
                                && sample.kind == LegacyTargetKind::Npc { team: 1 }
                        })
                });
                if !target_still_talkable {
                    runtime.message =
                        "NpcIconMode ignored a stale or out-of-range world talk target".to_owned();
                    continue;
                }
                let Ok(appearance) = npc_appearances.get(entity) else {
                    runtime.message =
                        "NpcIconMode ignored a talk target without live NPC appearance".to_owned();
                    continue;
                };
                let Some(definition) = content.gameplay_npc(appearance.0.npc_type) else {
                    runtime.message = format!(
                        "NpcIconMode ignored NPC table {} because TableData is unavailable",
                        appearance.0.npc_type
                    );
                    continue;
                };
                if definition.team != 1 || appearance.0.hp <= 0 {
                    runtime.message = format!(
                        "NpcIconMode ignored non-talkable NPC table {} (team {}, hp {})",
                        appearance.0.npc_type, definition.team, appearance.0.hp
                    );
                    continue;
                }
                let Ok((player_transform, _)) = attack_actors.get(request.actor) else {
                    runtime.message =
                        "NpcIconMode ignored a talk action without its live local player transform"
                            .to_owned();
                    continue;
                };
                let Ok(mut npc_transform) = combat_visuals.npc_transforms.get_mut(entity) else {
                    runtime.message =
                        "NpcIconMode ignored a talk target without its live NPC transform"
                            .to_owned();
                    continue;
                };
                // Recheck live roots: the queued feed may precede movement.
                if !ffone_client::world_targeting::world_npc_in_interaction_range(
                    player_transform.translation,
                    npc_transform.translation,
                ) {
                    runtime.message = "NpcIconMode ignored an out-of-range NPC root".to_owned();
                    continue;
                }
                // Clean InitMode turns every friendly NPC immediately, before
                // quest/service discovery and before the interaction-open
                // event. This is deliberately a local Transform write only.
                face_world_npc_toward_player_locally(
                    &mut npc_transform,
                    player_transform.translation,
                );
                // Clean `NpcIconMode.InitMode` calls `NpcGreetingBubble`
                // before opening any service/race/transport branch.
                mission.npc_bubbles.request_greeting(entity, definition);
                let services = world_npc_service_entries(
                    &content,
                    definition.npc_type,
                    definition.service_category,
                    definition.service_number,
                    mission.guide_production.payment_flag,
                    mission.race_production.player.ring_race_active,
                );
                if let Some(registration) = transportation_catalog
                    .registration_intent(appearance.0.npc_id, definition.npc_type)
                    .filter(|intent| !intent.is_registered(runtime.transportation_unlocks))
                {
                    mission
                        .transportation_production
                        .reserve_registration(registration);
                    match registration.encode_registered() {
                        Ok(request) => {
                            if let Err(error) =
                                bridge.send(NetworkCommand::SendRegisteredGameplay0104(request))
                            {
                                mission
                                    .transportation_production
                                    .release_registration(registration);
                                runtime.message = format!(
                                    "NpcIconMode transportation registration send failed: {error}"
                                );
                            }
                        }
                        Err(error) => {
                            mission
                                .transportation_production
                                .release_registration(registration);
                            runtime.message = format!(
                                "NpcIconMode transportation registration rejected: {error:?}"
                            );
                        }
                    }
                }
                let owned_nanos = mission
                    .nano_bank
                    .entries()
                    .iter()
                    .filter_map(|nano| (nano.id > 0).then_some(i32::from(nano.id)))
                    .collect::<BTreeSet<_>>();
                let guide = mission
                    .guide
                    .authoritative()
                    .map_or(0, |state| i32::from(state.raw_mentor()));
                let (available_missions, completed_missions) = mission
                    .inventory
                    .quest_inventory
                    .as_ref()
                    .map_or_else(
                        || Ok((Vec::new(), Vec::new())),
                        |quest_inventory| {
                            mission.runtime.npc_entries(
                                definition.npc_type,
                                appearance.0.npc_id,
                                runtime.map_name.clone(),
                                i32::from(runtime.player_level),
                                guide,
                                &owned_nanos,
                                quest_inventory,
                                &content,
                            )
                        },
                    )
                    .unwrap_or_else(|error| {
                        runtime.message = format!("NpcIconMode mission list ignored {error}");
                        (Vec::new(), Vec::new())
                    });
                let has_quest = !available_missions.is_empty() || !completed_missions.is_empty();
                let greeting = legacy_npc_open_voice_cue(
                    definition.service_category,
                    mission.race_production.player.ring_race_active,
                    has_quest,
                );
                // EndEcom plays this line after the correlated EndSuccess reply.
                // Queuing it here also plays the same racefinished take on NPC open.
                if greeting != LegacyNpcVoiceCue::RaceEnd {
                    combat_visuals.gameplay_audio.queue_legacy_npc_voice(
                        entity,
                        &definition.move_voice_owner,
                        greeting,
                    );
                }
                let race_route = clean_race_npc_route(
                    definition.service_category,
                    mission.race_production.player.ring_race_active,
                    has_quest,
                );
                match race_route {
                    CleanRaceNpcRoute::AutoEnd => {
                        // Clean NpcIconMode has already called NpcInteraction by
                        // this point. Preserve that authoritative open edge before
                        // it replaces itself with RaceMode EndEcom.
                        if definition.ai_type > 0
                            && let Err(error) = bridge.send(NetworkCommand::InteractWithNpc(
                                NpcInteractionRequest0104 {
                                    npc_id: appearance.0.npc_id,
                                    flag: 1,
                                },
                            ))
                        {
                            runtime.message =
                                format!("NpcIconMode race interaction-open send failed: {error}");
                            continue;
                        }
                        let Ok(cursor) = mission.cursors.single() else {
                            runtime.message =
                                "RaceMode EndEcom rejected without one production cursor owner"
                                    .to_owned();
                            continue;
                        };
                        let cursor_was_locked = cursor.grab_mode != CursorGrabMode::None;
                        match open_race_mode_from_npc(
                            &mut mission.race_mode,
                            &mission.race_catalog,
                            &mut mission.race_production,
                            RaceEcomType::End,
                            appearance.0.npc_id,
                            definition.npc_type,
                            definition.move_voice_owner.clone(),
                            runtime.fusion_matter,
                            cursor_was_locked,
                        ) {
                            Ok(()) => {
                                mission_ui.clear_npc_interaction_locally();
                                runtime.message = format!(
                                    "RaceMode EndEcom auto-opened from category-14 NPC {} with no CheckQuest rows",
                                    appearance.0.npc_id
                                );
                            }
                            Err(error) => runtime.message = error,
                        }
                        continue;
                    }
                    CleanRaceNpcRoute::RecallNano => {
                        mission_ui.clear_npc_interaction_locally();
                        let has_recall = runtime.nano_slots.iter().any(|slot| {
                            slot.nano_id.is_some()
                                && content
                                    .gameplay_skill(slot.skill_id)
                                    .is_some_and(|skill| matches!(skill.skill_type, 27 | 28))
                        });
                        if !has_recall {
                            nano_recall::notice(
                                &mut combat_visuals.nano_notices,
                                LocalizedText::new(
                                    "ui.nano.recall.no_nano",
                                    "Unable to register. No Recall Nano Equipped.",
                                ),
                            );
                        } else if let Err(error) =
                            runtime.nano_recall.register(appearance.0.npc_id, &bridge)
                        {
                            runtime.message = format!("Recall registration failed: {error}");
                        }
                        continue;
                    }
                    CleanRaceNpcRoute::Menu | CleanRaceNpcRoute::None => {}
                }
                mission_ui.show_npc_interaction(NpcInteractionUi {
                    npc_id: appearance.0.npc_id,
                    npc_type: definition.npc_type,
                    name: definition.name.clone(),
                    available_missions,
                    completed_missions,
                    services,
                    warp: normal_npc_warp_ui_entry(
                        &content,
                        appearance.0.npc_id,
                        definition.npc_type,
                    ),
                });
                if definition.ai_type > 0
                    && let Err(error) =
                        bridge.send(NetworkCommand::InteractWithNpc(NpcInteractionRequest0104 {
                            npc_id: appearance.0.npc_id,
                            flag: 1,
                        }))
                {
                    runtime.message = format!("NpcIconMode interaction-open send failed: {error}");
                }
            }
            LegacyAvatarActionIntent::WeaponCycle => {
                let Some(inventory) = mission.inventory.snapshot() else {
                    runtime.message =
                        "Weapon change ignored before authoritative inventory load".to_owned();
                    continue;
                };
                let Some(request) = world_weapon_swap_request(
                    inventory.equipment(),
                    |item_id| combat_visuals.weapon_catalog.profile_for_item(item_id).is_some(),
                ) else {
                    runtime.message = "Weapon change found no eligible equipped weapon".to_owned();
                    continue;
                };
                if let Err(error) = equipment
                    .item_move_owner
                    .begin(UserEquipPendingRequest0104::Move(request))
                {
                    runtime.message = format!("Weapon change blocked by item-move owner: {error}");
                    continue;
                }
                if let Err(error) = bridge.send(NetworkCommand::MoveItem(request)) {
                    equipment.item_move_owner.cancel();
                    runtime.message = format!("Weapon change send failed: {error}");
                }
            }
            LegacyAvatarActionIntent::NanoSkill { .. } => {
                // Recheck at dispatch: a queued skill may precede the mount reply.
                if equipment.vehicle.family != LegacyVehiclePresentationFamily::None
                    || !attack_actors
                        .get(request.actor)
                        .is_ok_and(|(_, context)| !context.vehicle_mounted)
                {
                    continue;
                }
                let Some((active_index, active_slot)) = active_world_nano_slot(&runtime) else {
                    runtime.message = "Nano skill ignored without an active tuned Nano".to_owned();
                    continue;
                };
                let Some(skill) = content.gameplay_skill(active_slot.skill_id).copied() else {
                    runtime.message = format!(
                        "Nano skill {} is absent from native TableData",
                        active_slot.skill_id
                    );
                    continue;
                };
                if active_slot.stamina <= 0
                    || !ordinary_world_nano_skill_supported(&skill)
                    || nano_cooldowns.is_active(active_index, active_slot.skill_id)
                {
                    runtime.message = format!(
                        "Nano skill {} is inactive, unsupported, depleted, or cooling down",
                        active_slot.skill_id
                    );
                    continue;
                }
                if matches!(skill.skill_type, 27 | 28) {
                    if let Some(reason) = runtime.nano_recall.restriction() {
                        nano_recall::notice(&mut combat_visuals.nano_notices, reason);
                        continue;
                    }
                }
                let Some(self_id) = runtime.player_id else {
                    runtime.message =
                        "Nano skill ignored before authoritative player identity load".to_owned();
                    continue;
                };
                let Ok((caster_transform, _)) = attack_actors.get(request.actor) else {
                    runtime.message =
                        "Nano skill ignored without a live local avatar transform".to_owned();
                    continue;
                };

                let mut planner_targets = Vec::new();
                if skill.target_type == 1 {
                    planner_targets.extend(combat_visuals.nano_npcs.iter().filter_map(
                        |(_, appearance, transform)| {
                            if appearance.0.hp <= 0 {
                                return None;
                            }
                            let definition = content.gameplay_npc(appearance.0.npc_type)?;
                            Some(WorldNanoTarget {
                                actor_id: appearance.0.npc_id,
                                kind: WorldNanoTargetKind::Npc {
                                    team: definition.team,
                                },
                                position: transform.translation().to_array(),
                                radius: definition.radius_server_units as f32
                                    * SERVER_TO_CLIENT_SCALE,
                                height: definition.height_server_units as f32
                                    * SERVER_TO_CLIENT_SCALE,
                            })
                        },
                    ));
                } else if skill.target_type == 2 {
                    planner_targets.extend(combat_visuals.nano_players.iter().filter_map(
                        |(_, player, appearance, transform)| {
                            (appearance.0.hp > 0 && appearance.0.special_state as u8 & 0x06 == 0)
                                .then_some(WorldNanoTarget {
                                    actor_id: player.pc_id,
                                    kind: WorldNanoTargetKind::Player,
                                    position: transform.translation().to_array(),
                                    radius: AUTHORED_CHARACTER_CONTROLLER_RADIUS,
                                    height: AUTHORED_CHARACTER_CONTROLLER_HEIGHT,
                                })
                        },
                    ));
                }

                let focused_target_id = combat_visuals
                    .action_states
                    .get(request.actor)
                    .ok()
                    .and_then(|state| match skill.target_type {
                        1 => state.target_selection.focused_npc.and_then(|focused| {
                            combat_visuals
                                .nano_npcs
                                .get(focused.entity)
                                .ok()
                                .map(|(_, appearance, _)| appearance.0.npc_id)
                        }),
                        2 => state.target_selection.focused_player.and_then(|focused| {
                            combat_visuals
                                .nano_players
                                .get(focused.entity)
                                .ok()
                                .map(|(_, player, _, _)| player.pc_id)
                        }),
                        _ => None,
                    });
                let caster_forward = caster_transform.rotation * Vec3::NEG_Z;
                let skill_request = match plan_world_nano_skill_0104(
                    &skill,
                    WorldNanoPlanInput {
                        self_id,
                        caster_position: caster_transform.translation.to_array(),
                        caster_forward: caster_forward.to_array(),
                        focused_target_id,
                        targets: &planner_targets,
                    },
                ) {
                    Ok(request) => request,
                    Err(error) => {
                        runtime.message = format!(
                            "Nano skill {} target plan rejected: {error:?}",
                            active_slot.skill_id
                        );
                        continue;
                    }
                };
                if let Err(error) = bridge.send(NetworkCommand::UseNanoSkill(skill_request)) {
                    runtime.message = format!("Nano skill send failed: {error}");
                } else {
                    if let Ok((mut controller, base)) =
                        combat_visuals.nano_controllers.get_mut(request.actor)
                    {
                        movement_buffs::launch_nano_rocket(
                            &mut controller,
                            base,
                            &combat_visuals.movement_buffs,
                            active_index,
                            &skill,
                        );
                    }
                    nano_cooldowns
                        .start(
                            active_index,
                            active_slot.skill_id,
                            skill.cool_type,
                            skill.cooldown,
                        )
                        .expect("validated Nano cooldown must fit its active equipped slot");
                    gameplay_nano.play_world_skill(request.actor);
                }
            }
            LegacyAvatarActionIntent::DismountVehicle => {
                if let Err(error) =
                    bridge.send(NetworkCommand::VehicleOff(PcVehicleOffRequest0104 {
                        unused: 0,
                    }))
                {
                    runtime.message = format!("Vehicle dismount send failed: {error}");
                }
            }
            LegacyAvatarActionIntent::TalkPlayer(entity) => {
                let target_still_talkable = runtime.allow_player_interaction
                    && target_feeds.get(request.actor).is_ok_and(|feed| {
                        feed.source_connected
                            && feed.samples.iter().any(|sample| {
                                sample.entity == entity
                                    && ffone_client::world_targeting::world_pc_in_interaction_range(sample.distance)
                                    && sample.talk_enabled
                                    && sample.kind == LegacyTargetKind::Player
                            })
                    });
                if !target_still_talkable {
                    runtime.message =
                        "Pc2pcMode ignored a stale or out-of-range player target".to_owned();
                    continue;
                }
                let Ok((_, remote, appearance, _)) = combat_visuals.nano_players.get(entity) else {
                    runtime.message =
                        "Pc2pcMode ignored a target without a live player appearance".to_owned();
                    continue;
                };
                let Some(local_pc_id) = runtime.player_id else {
                    runtime.message =
                        "Pc2pcMode ignored an offer before local identity loaded".to_owned();
                    continue;
                };
                mission.player_prompt.select(local_pc_id, remote.pc_id, request.actor, entity,
                    super::group_pc2pc::world_remote_pc_display_name(appearance), &mut mission.system_messages);
            }
        }
    }
}
