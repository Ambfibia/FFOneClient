//! Authoritative world equipment projection and inventory reconciliation.

use crate::app::*;
use ffone_client::avatar_action::LegacyVisualRequestQueue;

pub(super) fn reset_world_player_equipment_projection(
    mut projection: ResMut<WorldPlayerEquipmentProjection>,
) {
    *projection = WorldPlayerEquipmentProjection::default();
}

pub(super) fn equipped_item_from_inventory_item(
    item: ItemBase0104,
) -> ffone_protocol::EquippedItem0104 {
    ffone_protocol::EquippedItem0104 {
        item_type: item.item_type,
        item_id: item.item_id,
        option: item.option,
        time_limit: item.time_limit,
    }
}

const WORLD_PLAYER_APPAREL_SLOTS_0104: [ffone_protocol::CharacterEquipSlot0104; 6] = [
    ffone_protocol::CharacterEquipSlot0104::UpperBody,
    ffone_protocol::CharacterEquipSlot0104::LowerBody,
    ffone_protocol::CharacterEquipSlot0104::Foot,
    ffone_protocol::CharacterEquipSlot0104::Head,
    ffone_protocol::CharacterEquipSlot0104::Face,
    ffone_protocol::CharacterEquipSlot0104::Back,
];

fn authoritative_world_player_apparel_0104(
    inventory: &InventoryRuntime0104,
) -> WorldPlayerApparel0104 {
    WORLD_PLAYER_APPAREL_SLOTS_0104.map(|slot| inventory.equipment()[slot as usize])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorldPlayerApparelCandidateReadiness {
    Missing,
    Loading,
    Ready,
    Blocked,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WorldPlayerApparelRefreshPlan {
    Seed,
    Stable,
    Spawn,
    Await,
    Cancel,
    Replace,
    Promote,
    Reject,
}

pub(super) fn world_player_apparel_refresh_plan_0104(
    active: Option<WorldPlayerApparel0104>,
    candidate: Option<(WorldPlayerApparel0104, WorldPlayerApparelCandidateReadiness)>,
    rejected: Option<WorldPlayerApparel0104>,
    authoritative: WorldPlayerApparel0104,
) -> WorldPlayerApparelRefreshPlan {
    let Some(active) = active else {
        return WorldPlayerApparelRefreshPlan::Seed;
    };
    if let Some((candidate, readiness)) = candidate {
        if candidate != authoritative {
            return if active == authoritative {
                WorldPlayerApparelRefreshPlan::Cancel
            } else {
                WorldPlayerApparelRefreshPlan::Replace
            };
        }
        return match readiness {
            WorldPlayerApparelCandidateReadiness::Missing => WorldPlayerApparelRefreshPlan::Replace,
            WorldPlayerApparelCandidateReadiness::Loading => WorldPlayerApparelRefreshPlan::Await,
            WorldPlayerApparelCandidateReadiness::Ready => WorldPlayerApparelRefreshPlan::Promote,
            WorldPlayerApparelCandidateReadiness::Blocked => WorldPlayerApparelRefreshPlan::Reject,
        };
    }
    if active == authoritative || rejected == Some(authoritative) {
        WorldPlayerApparelRefreshPlan::Stable
    } else {
        WorldPlayerApparelRefreshPlan::Spawn
    }
}

#[derive(SystemParam)]
pub(super) struct WorldPlayerApparelAssets<'w> {
    asset_server: Res<'w, AssetServer>,
    character_data: Res<'w, CharacterCreationDataResource>,
    rig_assets: ResMut<'w, NativePlayerRigAssetCache>,
    rig_catalog: Res<'w, NativePlayerRigCatalog>,
}

#[derive(SystemParam)]
pub(super) struct WorldPlayerApparelRigQueries<'w, 's> {
    active: Query<
        'w,
        's,
        (
            Entity,
            &'static TutorialSelectedPlayerRig,
            &'static Visibility,
        ),
        (
            With<TutorialSelectedPlayerRigActive>,
            Without<TutorialSelectedPlayerRigCandidate>,
        ),
    >,
    candidates: Query<
        'w,
        's,
        (
            Entity,
            &'static TutorialSelectedPlayerRigStatus,
            &'static TutorialSelectedPlayerRigCandidate,
        ),
        Without<TutorialSelectedPlayerRigActive>,
    >,
}

pub(super) fn world_player_hand_projection_required_0104(
    projected: Option<ItemBase0104>,
    projected_rig_root: Option<Entity>,
    active_rig_root: Entity,
    authoritative: ItemBase0104,
) -> bool {
    projected != Some(authoritative) || projected_rig_root != Some(active_rig_root)
}

/// Mirrors authoritative normal-world equipment into the selected player.
/// Hand is an in-place rigid attachment. Apparel slots 1..=6 include both
/// skinned parts and rigid appearance attachments, so they build as one hidden
/// candidate and replace the active rig only after the complete native
/// readiness contract succeeds. No local UI action enters this path until its
/// server reply has already updated `InventoryRuntime0104`.
pub(super) fn sync_world_player_equipment(
    mut commands: Commands,
    inventory: Res<LocalInventoryRuntime>,
    mut runtime: ResMut<RuntimeStatus>,
    mut assets: WorldPlayerApparelAssets,
    weapon_catalog: Res<PlayerWeaponAnimationCatalog>,
    mut projection: ResMut<WorldPlayerEquipmentProjection>,
    mut presentation: ResMut<TutorialPlayerPresentationCommandQueue>,
    mut visuals: ResMut<LegacyVisualRequestQueue>,
    mut action_states: Query<&mut LegacyAvatarActionState, With<LocalPlayer>>,
    rigs: WorldPlayerApparelRigQueries,
) {
    let Some(snapshot) = inventory.snapshot() else {
        return;
    };
    let Ok((active_entity, active_rig, active_visibility)) = rigs.active.single() else {
        return;
    };
    let authoritative_apparel = authoritative_world_player_apparel_0104(snapshot);
    let hand = snapshot.equipment()[ffone_protocol::CharacterEquipSlot0104::Hand as usize];

    let first_seed = projection.rig_root.is_none();
    if projection.rig_root != Some(active_entity) {
        if let Some(candidate) = projection.candidate.take() {
            commands.entity(candidate.entity).despawn();
        }
        projection.controller_root = Some(active_rig.controller_root);
        projection.rig_root = Some(active_entity);
        projection.apparel = Some(authoritative_apparel);
        projection.rejected_apparel = None;
        if first_seed {
            // WorldReady already queued the initial hand. Seeding the cursor
            // must not rebuild identical apparel or replay that command.
            projection.hand = Some(hand);
            projection.hand_rig_root = Some(active_entity);
            return;
        }
        // A replacement/respawn has its own skeleton and sockets even when the
        // authoritative Hand bytes did not change.
        projection.hand_rig_root = None;
    }

    if world_player_hand_projection_required_0104(
        projection.hand,
        projection.hand_rig_root,
        active_entity,
        hand,
    ) {
        let changed_hand = projection.hand.is_some() && projection.hand != Some(hand);
        projection.hand = Some(hand);
        projection.hand_rig_root = Some(active_entity);
        presentation.push_equipment(TutorialPlayerEquipmentRequest {
            slot: ffone_protocol::CharacterEquipSlot0104::Hand,
            item: equipped_item_from_inventory_item(hand),
        });
        if let Ok(mut action_state) = action_states.get_mut(active_rig.controller_root) {
            // Equipment consumption and the locomotion replay share one FIFO
            // frame. Re-resolving the unchanged controller state preserves
            // Run/Jump/Inventory while selecting its new armed variant.
            action_state.invalidate_visual();
            if changed_hand {
                visuals.cancel_attack_for(active_rig.controller_root);
                presentation.cancel_pending_runtime_attacks();
                action_state.begin_weapon_change_visual();
            }
        }
    }

    let candidate_cursor = projection.candidate;
    let candidate_readiness = candidate_cursor.map(|candidate| {
        let Ok((entity, status, marker)) = rigs.candidates.get(candidate.entity) else {
            return WorldPlayerApparelCandidateReadiness::Missing;
        };
        if entity != candidate.entity || marker.fallback != active_entity {
            return WorldPlayerApparelCandidateReadiness::Blocked;
        }
        match status {
            TutorialSelectedPlayerRigStatus::Loading => {
                WorldPlayerApparelCandidateReadiness::Loading
            }
            TutorialSelectedPlayerRigStatus::Ready => WorldPlayerApparelCandidateReadiness::Ready,
            TutorialSelectedPlayerRigStatus::Blocked(_) => {
                WorldPlayerApparelCandidateReadiness::Blocked
            }
        }
    });
    let mut plan = world_player_apparel_refresh_plan_0104(
        projection.apparel,
        candidate_cursor
            .zip(candidate_readiness)
            .map(|(candidate, readiness)| (candidate.apparel, readiness)),
        projection.rejected_apparel,
        authoritative_apparel,
    );

    if projection.force_refresh {
        projection.force_refresh = false;
        plan = WorldPlayerApparelRefreshPlan::Replace;
    }
    match plan {
        WorldPlayerApparelRefreshPlan::Seed => {
            projection.apparel = Some(authoritative_apparel);
            return;
        }
        WorldPlayerApparelRefreshPlan::Stable | WorldPlayerApparelRefreshPlan::Await => return,
        WorldPlayerApparelRefreshPlan::Cancel => {
            if let Some(candidate) = projection.candidate.take() {
                commands.entity(candidate.entity).despawn();
            }
            projection.rejected_apparel = None;
            return;
        }
        WorldPlayerApparelRefreshPlan::Replace => {
            if let Some(candidate) = projection.candidate.take() {
                commands.entity(candidate.entity).despawn();
            }
            projection.rejected_apparel = None;
        }
        WorldPlayerApparelRefreshPlan::Promote => {
            let Some(candidate) = projection.candidate.take() else {
                return;
            };
            let Ok((_, _, marker)) = rigs.candidates.get(candidate.entity) else {
                return;
            };
            if marker.fallback != active_entity {
                commands.entity(candidate.entity).despawn();
                projection.rejected_apparel = Some(candidate.apparel);
                return;
            }
            commands
                .entity(candidate.entity)
                .remove::<TutorialSelectedPlayerRigCandidate>()
                .remove::<TutorialPlayerFallbackVisual>()
                .insert((
                    TutorialSelectedPlayerRigActive,
                    LocalCharacterScene,
                    active_visibility.clone(),
                ));
            commands
                .entity(active_entity)
                .remove::<TutorialSelectedPlayerRigActive>()
                .remove::<LocalCharacterScene>()
                .despawn();
            projection.controller_root = Some(active_rig.controller_root);
            projection.rig_root = Some(candidate.entity);
            projection.apparel = Some(candidate.apparel);
            projection.rejected_apparel = None;
            // Queue the authoritative Hand against the candidate immediately.
            // The animation resolve later in this frame can then map the
            // controller's unchanged locomotion to its armed/unarmed variant
            // instead of briefly installing Stand/RifleStand on the new rig.
            presentation.push_equipment(TutorialPlayerEquipmentRequest {
                slot: ffone_protocol::CharacterEquipSlot0104::Hand,
                item: equipped_item_from_inventory_item(hand),
            });
            projection.hand = Some(hand);
            projection.hand_rig_root = Some(candidate.entity);
            if let Ok(mut action_state) = action_states.get_mut(active_rig.controller_root) {
                action_state.invalidate_visual();
            }
            // Marker changes are deferred. The next frame observes the new
            // active root, reattaches even an unchanged Hand to its socket and
            // replays the controller's current locomotion onto the new rig.
            return;
        }
        WorldPlayerApparelRefreshPlan::Reject => {
            if let Some(candidate) = projection.candidate.take() {
                let reason = rigs
                    .candidates
                    .get(candidate.entity)
                    .ok()
                    .and_then(|(_, status, _)| match status {
                        TutorialSelectedPlayerRigStatus::Blocked(reason) => Some(reason.as_str()),
                        _ => None,
                    })
                    .unwrap_or("candidate ownership/readiness failed");
                runtime.message = format!(
                    "Authoritative apparel {:?} kept the previous rig: {reason}",
                    candidate.apparel.map(|item| item.item_id)
                );
                commands.entity(candidate.entity).despawn();
                projection.rejected_apparel = Some(candidate.apparel);
            }
            return;
        }
        WorldPlayerApparelRefreshPlan::Spawn => {}
    }

    let Some(selected_uid) = runtime.roster.selected_uid else {
        return;
    };
    if runtime.player_id != Some(snapshot.owner_pc_id()) {
        runtime.message = format!(
            "Apparel refresh owner mismatch: runtime {:?}, inventory {}",
            runtime.player_id,
            snapshot.owner_pc_id()
        );
        return;
    }
    let Some(mut character) = runtime
        .roster
        .characters
        .iter()
        .find(|character| character.pc_uid == selected_uid)
        .cloned()
    else {
        runtime.message =
            format!("Apparel refresh has no selected character summary for UID {selected_uid}");
        return;
    };
    character.equipment = (*snapshot.equipment()).map(equipped_item_from_inventory_item);
    projection.next_generation = projection.next_generation.wrapping_add(1).max(1);
    let generation = projection.next_generation;
    match spawn_tutorial_selected_player_rig(
        &mut commands,
        &assets.asset_server,
        &mut assets.rig_assets,
        &assets.rig_catalog,
        &weapon_catalog,
        &assets.character_data.0,
        active_rig.controller_root,
        &character,
        generation,
        active_rig.render_layers(),
        Some(active_entity),
        active_rig.tutorial_stand_semantics(),
    ) {
        Ok(spawned) => {
            commands
                .entity(spawned.rig_root)
                .remove::<TutorialSelectedPlayerRigActive>()
                .remove::<LocalCharacterScene>()
                .insert(TutorialSelectedPlayerRigCandidate {
                    fallback: active_entity,
                });
            projection.candidate = Some(WorldPlayerApparelCandidate0104 {
                entity: spawned.rig_root,
                apparel: authoritative_apparel,
            });
            projection.rejected_apparel = None;
        }
        Err(error) => {
            projection.rejected_apparel = Some(authoritative_apparel);
            runtime.message = format!(
                "Authoritative apparel {:?} kept the previous rig: {error}",
                authoritative_apparel.map(|item| item.item_id)
            );
        }
    }
}

pub(super) fn apply_inventory_frame(
    frame: &DecodedFrame,
    inventory: &mut LocalInventoryRuntime,
    content: &TutorialMissionContent,
    runtime: &mut RuntimeStatus,
) -> Result<bool, String> {
    if frame.packet_type == 0x31000062 {
        gameplay_ui_actions::gm_runtime::item_reply(frame, runtime);
        return Ok(true);
    }
    if let Some(reward) = decode_pc_give_item_success_0104(frame)? {
        if reward.inventory_location == 2 {
            let items = inventory
                .quest_inventory
                .as_mut()
                .ok_or("quest inventory is not loaded")?;
            let slot = usize::try_from(reward.slot)
                .ok()
                .and_then(|index| items.get_mut(index))
                .ok_or("invalid quest inventory slot")?;
            // The shard's quest GM reply echoes the granted count, unlike
            // the normal-inventory reply which contains a replacement item.
            let count = if slot.item_id == reward.item.item_id {
                slot.option
            } else {
                0
            };
            let option = count
                .checked_add(reward.item.option)
                .ok_or("quest item count overflow")?;
            *slot = ItemBase0104 {
                option,
                ..reward.item
            };
            gameplay_ui_actions::gm_runtime::item_reply(frame, runtime);
            return Ok(true);
        }
        let Some(snapshot) = inventory.snapshot_mut() else {
            return Err("give-item success arrived before authoritative PC load".to_owned());
        };
        // The protocol crate does not yet expose this exact registered 0104
        // packet. Apply its complete eIL/slot/sItemBase post-state through the
        // existing checked slot writer; never infer the item from the Email
        // acknowledgement, which intentionally contains no item bytes.
        snapshot
            .apply_reward_item_post_state(reward)
            .map_err(|error| error.to_string())?;
        runtime.resurrection_item_slot = resolve_runtime_resurrection_item_slot(snapshot, content);
        gameplay_ui_actions::gm_runtime::item_reply(frame, runtime);
        return Ok(true);
    }
    if let Some(packet) = decode_inventory_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 inventory packet: {error}"))?
    {
        let Some(snapshot) = inventory.snapshot_mut() else {
            return Err("inventory packet arrived before authoritative PC load".to_owned());
        };
        match packet {
            InventoryPacket0104::ItemMoveSuccess(packet) => {
                snapshot
                    .apply_item_move_success(packet)
                    .map_err(|error| error.to_string())?;
            }
            InventoryPacket0104::EquipChange(packet) => {
                // EquipChange is also broadcast for remote PCs. Their visual
                // mutation remains owned by entity lifecycle; only the matching
                // local snapshot is accepted here.
                if packet.pc_id != snapshot.owner_pc_id() {
                    return Ok(false);
                }
                snapshot
                    .apply_equip_change(packet)
                    .map_err(|error| error.to_string())?;
            }
        }
        runtime.resurrection_item_slot = resolve_runtime_resurrection_item_slot(snapshot, content);
        return Ok(true);
    }

    let Some(packet) = decode_item_use_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 item-use packet: {error}"))?
    else {
        return Ok(false);
    };
    match packet {
        ItemUsePacket0104::Success(packet) => {
            let Some(snapshot) = inventory.snapshot_mut() else {
                return Err("item-use success arrived before authoritative PC load".to_owned());
            };
            snapshot
                .apply_item_use_success(&packet)
                .map_err(|error| error.to_string())?;
            runtime.resurrection_item_slot =
                resolve_runtime_resurrection_item_slot(snapshot, content);
        }
        // Neither a failure nor the around-player broadcast owns local
        // inventory post-state. They remain available to their other UI and
        // entity consumers because every frame router observes the raw frame.
        ItemUsePacket0104::Failure(_) | ItemUsePacket0104::Broadcast(_) => {}
    }
    Ok(true)
}

pub(super) const PC_GIVE_ITEM_SUCCESS_PACKET_ID_0104: u32 = 0x3100_0061;
pub(super) const PC_GIVE_ITEM_SUCCESS_BODY_SIZE_0104: usize = 20;

fn decode_pc_give_item_success_0104(
    frame: &DecodedFrame,
) -> Result<Option<ItemReward0104>, String> {
    if frame.packet_type != PC_GIVE_ITEM_SUCCESS_PACKET_ID_0104 {
        return Ok(None);
    }
    if frame.payload.len() != PC_GIVE_ITEM_SUCCESS_BODY_SIZE_0104 {
        return Err(format!(
            "protocol-0104 PC_GIVE_ITEM_SUCC requires exactly {} bytes, received {}",
            PC_GIVE_ITEM_SUCCESS_BODY_SIZE_0104,
            frame.payload.len()
        ));
    }
    let inventory_location = i32::from_le_bytes(
        frame.payload[0..4]
            .try_into()
            .expect("give-item eIL slice has exact width"),
    );
    let slot = i32::from_le_bytes(
        frame.payload[4..8]
            .try_into()
            .expect("give-item slot slice has exact width"),
    );
    let item = ItemBase0104::decode(&frame.payload[8..20])
        .map_err(|error| format!("malformed protocol-0104 PC_GIVE_ITEM_SUCC item: {error}"))?;
    Ok(Some(ItemReward0104 {
        item,
        inventory_location,
        slot,
    }))
}
