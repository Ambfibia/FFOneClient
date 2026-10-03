use crate::app::gameplay_ui_actions::*;

pub(super) fn handle_npc(
    action: NpcAction,
    gameplay_modal: bool,
    context: &mut WorldGameplayActionContext<'_, '_>,
) {
    let WorldGameplayActionContext {
        gameplay_ui,
        normal_warp,
        mission_ui,
        nanocom_messages,
        content,
        npc_appearances,
        guide_ui,
        modal_owners,
        guide_runtime,
        guide_production,
        upsell_ui,
        bridge,
        runtime,
        user_equip_ui,
        nano_viewer,
        ..
    } = context;
    for action in std::iter::once(action) {
        match action {
            NpcAction::NpcService { npc_id, service } => {
                if gameplay_modal {
                    continue;
                }
                let mut matches = npc_appearances
                    .iter()
                    .filter(|(_, appearance)| appearance.0.npc_id == npc_id);
                let Some((_, appearance)) = matches.next() else {
                    runtime.message =
                        format!("NpcIconMode service ignored missing runtime NPC {npc_id}");
                    continue;
                };
                if matches.next().is_some() {
                    runtime.message =
                        format!("NpcIconMode service ignored duplicate runtime NPC {npc_id}");
                    continue;
                }
                let Some(definition) = content.gameplay_npc(appearance.0.npc_type) else {
                    runtime.message = format!(
                        "NpcIconMode service ignored missing TableData NPC {}",
                        appearance.0.npc_type
                    );
                    continue;
                };
                if service == NpcServiceKind::NanoStation {
                    if !(7..=9).contains(&definition.service_category) || appearance.0.hp <= 0 {
                        runtime.message = "Nano Station rejected an invalid NPC owner".to_owned();
                        continue;
                    }
                    if definition.ai_type > 0
                        && let Err(error) = bridge.send(NetworkCommand::InteractWithNpc(
                            NpcInteractionRequest0104 { npc_id, flag: 0 },
                        ))
                    {
                        runtime.message = format!("Nano Station interaction-close failed: {error}");
                        continue;
                    }
                    user_equip_ui.open_nano_station(npc_id);
                    nano_viewer.close();
                    gameplay_ui.chat.close_nanocom_menu();
                    mission_ui.npc_icon_mode_visible = false;
                    continue;
                }
                if service == NpcServiceKind::Barber {
                    if definition.service_category == 28 && appearance.0.hp > 0 {
                        modal_owners.crafting.barber.request_open(npc_id);
                        mission_ui.clear_npc_interaction_locally();
                        nanocom_messages.set_expanded(false);
                        gameplay_ui.chat.active=false;
                        gameplay_ui.chat.input.clear();
                        gameplay_ui.chat.close_nanocom_menu();
                    }
                    continue;
                }
                if service == NpcServiceKind::Combine {
                    if !combi_service_allowed_0104(definition.service_category, service)
                        || appearance.0.hp <= 0
                    {
                        mission_ui.npc_icon_mode_visible = true;
                        runtime.message = format!(
                            "NpcIconMode COMBINE rejected NPC table {} category {} or a non-live runtime owner",
                            definition.npc_type, definition.service_category
                        );
                        continue;
                    }
                    let player = match combi_player_authority_0104(&runtime, &guide_runtime) {
                        Ok(player) => player,
                        Err(error) => {
                            mission_ui.npc_icon_mode_visible = true;
                            runtime.message = format!("CombiMode authority rejected: {error}");
                            continue;
                        }
                    };
                    let Some(inventory) = normal_warp.inventory.snapshot() else {
                        mission_ui.npc_icon_mode_visible = true;
                        runtime.message =
                            "CombiMode rejected without an authoritative inventory snapshot"
                                .to_owned();
                        continue;
                    };
                    let cursor_was_locked = match modal_owners.shell.cursors.single_mut() {
                        Ok(cursor) => cursor.grab_mode != CursorGrabMode::None,
                        Err(_) => {
                            mission_ui.npc_icon_mode_visible = true;
                            runtime.message =
                                "CombiMode rejected without one production cursor owner".to_owned();
                            continue;
                        }
                    };
                    let context = CombiOpenContext0104::clean(npc_id, definition.npc_type, player);
                    let runtime_catalog = RuntimeCombiItemCatalog0104 {
                        source: &modal_owners.crafting.combi_catalog,
                        player,
                    };
                    match modal_owners.crafting.combi_runtime.open(
                        context,
                        inventory,
                        &runtime_catalog,
                        &modal_owners.crafting.combi_catalog.recipes,
                    ) {
                        Ok(output) => {
                            modal_owners.crafting.combi_shell.cursor_was_locked =
                                Some(cursor_was_locked);
                            modal_owners.crafting.combi_shell.push_output(output);
                            // CreateGameMode(25) hides NpcIconMode locally and
                            // does not synthesize the ordinary interaction-out packet.
                            mission_ui.clear_npc_interaction_locally();
                            nanocom_messages.set_expanded(false);
                            gameplay_ui.chat.active = false;
                            gameplay_ui.chat.input.clear();
                            runtime.message = format!(
                                "CombiMode 25 routed from category-26 runtime NPC {npc_id} (table {})",
                                definition.npc_type
                            );
                        }
                        Err(error) => {
                            mission_ui.npc_icon_mode_visible = true;
                            runtime.message = format!("CombiMode initialization rejected: {error}");
                        }
                    }
                    continue;
                }
                if service == NpcServiceKind::Enchant {
                    if !enchant_service_allowed_0104(definition.service_category, service)
                        || appearance.0.hp <= 0
                    {
                        mission_ui.npc_icon_mode_visible = true;
                        runtime.message = format!(
                            "NpcIconMode ENCHANT rejected NPC table {} category {} or a non-live runtime owner",
                            definition.npc_type, definition.service_category
                        );
                        continue;
                    }
                    let Some(player) = enchant_player_authority_0104(&runtime) else {
                        mission_ui.npc_icon_mode_visible = true;
                        runtime.message =
                            "EnchantMode rejected before authoritative local PC load".to_owned();
                        continue;
                    };
                    let Some(inventory) = normal_warp.inventory.snapshot() else {
                        mission_ui.npc_icon_mode_visible = true;
                        runtime.message =
                            "EnchantMode rejected without an authoritative inventory snapshot"
                                .to_owned();
                        continue;
                    };
                    let cursor_was_locked = match modal_owners.shell.cursors.single_mut() {
                        Ok(cursor) => cursor.grab_mode != CursorGrabMode::None,
                        Err(_) => {
                            mission_ui.npc_icon_mode_visible = true;
                            runtime.message =
                                "EnchantMode rejected without one production cursor owner"
                                    .to_owned();
                            continue;
                        }
                    };
                    let context = EnchantOpenContext0104::clean(
                        npc_id,
                        definition.npc_type,
                        player,
                        cursor_was_locked,
                    );
                    match modal_owners
                        .crafting
                        .enchant_runtime
                        .open(context, inventory)
                    {
                        Ok(output) => {
                            modal_owners.crafting.enchant_shell.push_output(output);
                            // CreateGameMode(29) hides NpcIconMode locally and
                            // does not synthesize the ordinary interaction-out packet.
                            mission_ui.clear_npc_interaction_locally();
                            nanocom_messages.set_expanded(false);
                            gameplay_ui.chat.active = false;
                            gameplay_ui.chat.input.clear();
                            runtime.message = format!(
                                "EnchantMode 29 routed from category-27 runtime NPC {npc_id} (table {})",
                                definition.npc_type
                            );
                        }
                        Err(error) => {
                            mission_ui.npc_icon_mode_visible = true;
                            runtime.message =
                                format!("EnchantMode initialization rejected: {error}");
                        }
                    }
                    continue;
                }
                if matches!(service, NpcServiceKind::Race | NpcServiceKind::RaceRank) {
                    if !race_service_allowed(definition.service_category, service) {
                        mission_ui.npc_icon_mode_visible = true;
                        runtime.message = format!(
                            "NpcIconMode {service:?} rejected NPC table {} category {}; category 17 remains Recall Nano and never enters mode 17",
                            definition.npc_type, definition.service_category
                        );
                        continue;
                    }
                    if service == NpcServiceKind::RaceRank {
                        debug_assert!(!RACE_RANK_HTTP_TRANSPORT_AVAILABLE);
                        // The button already hid NpcIconMode. Refuse the mode
                        // transition and restore that exact source menu rather
                        // than opening a loading shell that can never receive
                        // its clean HTTP completion.
                        mission_ui.npc_icon_mode_visible = true;
                        runtime.message = RACE_RANK_HTTP_GAP.to_owned();
                        continue;
                    }
                    let Ok(cursor) = modal_owners.shell.cursors.single_mut() else {
                        mission_ui.npc_icon_mode_visible = true;
                        runtime.message =
                            "RaceMode StartEcom rejected without one production cursor owner"
                                .to_owned();
                        continue;
                    };
                    let cursor_was_locked = cursor.grab_mode != CursorGrabMode::None;
                    let cancelling = modal_owners.race.production.player.ring_race_active;
                    match open_race_mode_from_npc(
                        &mut modal_owners.race.mode,
                        &modal_owners.race.catalog,
                        &mut modal_owners.race.production,
                        RaceEcomType::Start,
                        npc_id,
                        definition.npc_type,
                        definition.move_voice_owner.clone(),
                        runtime.fusion_matter,
                        cursor_was_locked,
                    ) {
                        Ok(()) => {
                            // CreateGameMode(16) hides NpcIconMode locally; it
                            // does not synthesize the normal interaction-close
                            // request.
                            mission_ui.clear_npc_interaction_locally();
                            runtime.message = format!(
                                "RaceMode {} routed from category-13 runtime NPC {npc_id} (table {})",
                                if cancelling { "Cancel" } else { "Start" },
                                definition.npc_type
                            );
                        }
                        Err(error) => {
                            mission_ui.npc_icon_mode_visible = true;
                            runtime.message = error;
                        }
                    }
                    continue;
                }
                if matches!(service, NpcServiceKind::Bank | NpcServiceKind::LocalBank) {
                    let expected = bank_service_kind(definition.service_category);
                    if expected != Some(service) {
                        runtime.message = format!(
                            "NpcIconMode {service:?} rejected NPC table {} with service type {}",
                            definition.npc_type, definition.service_category
                        );
                        continue;
                    }
                    let Some(player_id) = runtime.player_id else {
                        runtime.message =
                            "BankMode rejected before authoritative local PC load".to_owned();
                        continue;
                    };
                    let CommerceOwners {
                        bank_state,
                        bank_outbox,
                        ..
                    } = &mut modal_owners.commerce;
                    bank_state.begin_open(player_id, npc_id, bank_outbox);
                    // `NpcIconMode.StartBank` changes GameMode without its
                    // normal interaction-close packet. Bank owns the camera
                    // target until its local BankOut edge.
                    mission_ui.clear_npc_interaction_locally();
                    runtime.message = format!(
                        "BankMode {:?} routed locally from runtime NPC {npc_id} (table {})",
                        service, definition.npc_type
                    );
                    continue;
                }
                if service == NpcServiceKind::Vendor {
                    if !content.gameplay_npc_is_vendor(definition.npc_type) {
                        runtime.message = format!(
                            "NpcIconMode ENTER STORE rejected NPC table {} without a VendorTable row",
                            definition.npc_type
                        );
                        continue;
                    }
                    if definition.ai_type > 0
                        && let Err(error) = bridge.send(NetworkCommand::InteractWithNpc(
                            NpcInteractionRequest0104 { npc_id, flag: 0 },
                        ))
                    {
                        runtime.message =
                            format!("NpcIconMode interaction-close send failed: {error}");
                        continue;
                    }
                    let CommerceOwners {
                        vendor_state,
                        vendor_outbox,
                        ..
                    } = &mut modal_owners.commerce;
                    vendor_state.begin_open(npc_id, definition.npc_type, vendor_outbox);
                    runtime.message = format!(
                        "VendorMode ENTER STORE routed from runtime NPC {npc_id} (table {})",
                        definition.npc_type
                    );
                    continue;
                }
                if matches!(
                    service,
                    NpcServiceKind::TransportationWarp | NpcServiceKind::TransportationWyvern
                ) {
                    let expected_transportation_service = transportation_npc_service(
                        definition.service_category,
                        modal_owners
                            .modes
                            .transportation_catalog
                            .npc_class(definition.npc_type),
                    );
                    let expected_ui_service = match expected_transportation_service {
                        Some(TransportationService::Warp) => {
                            Some(NpcServiceKind::TransportationWarp)
                        }
                        Some(TransportationService::Wyvern) => {
                            Some(NpcServiceKind::TransportationWyvern)
                        }
                        _ => None,
                    };
                    if expected_ui_service != Some(service) {
                        runtime.message = format!(
                            "NpcIconMode transportation rejected NPC table {} with service type {} and catalog class {:?}",
                            definition.npc_type,
                            definition.service_category,
                            modal_owners
                                .modes
                                .transportation_catalog
                                .npc_class(definition.npc_type),
                        );
                        continue;
                    }
                    let expected_transportation_service = expected_transportation_service
                        .expect("the matching cat-15/16 route has an exact transportation service");
                    let mut npc_matches = normal_warp
                        .npc_sources
                        .iter()
                        .filter(|(_, source, _)| source.0.npc_id == npc_id);
                    let Some((_, _, npc_transform)) = npc_matches.next() else {
                        runtime.message = format!(
                            "TransportMode rejected missing runtime NPC transform {npc_id}"
                        );
                        continue;
                    };
                    if npc_matches.next().is_some() {
                        runtime.message = format!(
                            "TransportMode rejected duplicate runtime NPC transform {npc_id}"
                        );
                        continue;
                    }
                    let npc_unity = native_to_unity_vector(npc_transform.translation());
                    let player_unity = {
                        let mut players = modal_owners.shell.players.iter_mut();
                        let Some((transform, _, pending)) = players.next() else {
                            runtime.message =
                                "TransportMode rejected before local player spawn".to_owned();
                            continue;
                        };
                        if pending.is_some() || players.next().is_some() {
                            runtime.message =
                                "TransportMode rejected ambiguous or pending local player"
                                    .to_owned();
                            continue;
                        }
                        native_to_unity_vector(transform.translation)
                    };
                    let cursor_was_locked = modal_owners
                        .shell
                        .cursors
                        .single_mut()
                        .is_ok_and(|cursor| cursor.grab_mode != CursorGrabMode::None);
                    let move_ok_voice_true_names = transportation_move_ok_voice_set(
                        &modal_owners.audio.catalog,
                        &definition.move_voice_owner,
                    );
                    let context = TransportationOpenContext {
                        player: TransportationPlayerSnapshot {
                            position: TransportationWorldPoint::new(
                                player_unity.x,
                                player_unity.y,
                                player_unity.z,
                            ),
                            taros: runtime.candy,
                            unlocks: runtime.transportation_unlocks,
                            cursor_was_locked,
                        },
                        target: TransportationTarget::Npc {
                            npc_instance_id: npc_id,
                            npc_table_id: definition.npc_type,
                            npc_position: TransportationWorldPoint::new(
                                npc_unity.x,
                                npc_unity.y,
                                npc_unity.z,
                            ),
                            has_move_ok_voice: move_ok_voice_true_names.is_some(),
                        },
                    };
                    match modal_owners
                        .modes
                        .transportation_model
                        .open(&modal_owners.modes.transportation_catalog, context)
                    {
                        Ok(()) => {
                            modal_owners.modes.transportation_production.begin_npc(
                                npc_id,
                                definition.ai_type > 0,
                                expected_transportation_service,
                                move_ok_voice_true_names,
                            );
                            // CreateGameMode hides NpcIconMode; it does not
                            // synthesize the normal interaction-close packet.
                            mission_ui.clear_npc_interaction_locally();
                            runtime.message = format!(
                                "TransportMode opened from runtime NPC {npc_id} (table {}, type {})",
                                definition.npc_type, expected_transportation_service as i32
                            );
                        }
                        Err(error) => {
                            runtime.message =
                                format!("TransportMode initialization rejected: {error:?}");
                        }
                    }
                    continue;
                }
                if service == NpcServiceKind::Rule {
                    let Some(service_number) = definition.service_number else {
                        runtime.message = format!(
                            "NpcIconMode RULE rejected NPC table {} without m_iServiceNumber",
                            definition.npc_type
                        );
                        continue;
                    };
                    if rule_npc_service_route(service_number).is_none() {
                        runtime.message = format!(
                            "NpcIconMode RULE rejected NPC table {} service number {}",
                            definition.npc_type, service_number
                        );
                        continue;
                    }
                    let request = RuleOpenRequest::NpcService {
                        runtime_npc_id: npc_id,
                        table_npc_id: definition.npc_type,
                        service_number,
                    };
                    match modal_owners
                        .modes
                        .rule_runtime
                        .open(&mut modal_owners.modes.rule_model, request)
                    {
                        Ok(session) => {
                            runtime.message = format!(
                                "RuleMode page {} opened locally from runtime NPC {npc_id} (table {}, service {service_number}); no interaction-close packet sent",
                                session.page.table_index(),
                                definition.npc_type
                            );
                        }
                        Err(error) => {
                            runtime.message =
                                format!("NpcIconMode RULE transition rejected: {error:?}");
                        }
                    }
                    continue;
                }
                if definition.ai_type > 0
                    && let Err(error) =
                        bridge.send(NetworkCommand::InteractWithNpc(NpcInteractionRequest0104 {
                            npc_id,
                            flag: 0,
                        }))
                {
                    runtime.message = format!("NpcIconMode interaction-close send failed: {error}");
                    continue;
                }
                let expected =
                    guide_service_entry(definition.service_category, guide_production.payment_flag);
                if expected.as_ref().map(|entry| entry.service) != Some(service) {
                    runtime.message = format!(
                        "NpcIconMode service {:?} rejected for NPC table {} category {}",
                        service, definition.npc_type, definition.service_category
                    );
                    continue;
                }
                let payment_flag = guide_production.payment_flag.unwrap_or(-1);
                let Some(route) =
                    guide_npc_service_route(definition.service_category, payment_flag)
                else {
                    runtime.message = format!(
                        "Guide service category {} rejected unknown payment flag {:?}",
                        definition.service_category, guide_production.payment_flag
                    );
                    continue;
                };
                let passed_raw_mentor = match route {
                    GuideNpcServiceRoute::GuideChanger { passed_mentor } => passed_mentor.raw(),
                    GuideNpcServiceRoute::PastWarp => 0,
                    GuideNpcServiceRoute::UpsellRequired => {
                        match open_unpaid_past_warp_upsell(upsell_ui) {
                            Ok(()) => {
                                runtime.message = format!(
                                    "Upsell News mode opened from unpaid Past Warp NPC {npc_id} (table {}); awaiting the source-equivalent news image provider",
                                    definition.npc_type
                                );
                            }
                            Err(error) => {
                                runtime.message =
                                    format!("Upsell initialization rejected: {error:?}");
                            }
                        }
                        continue;
                    }
                };
                let intent = match guide_runtime.init_intent(passed_raw_mentor) {
                    Ok(intent) => intent,
                    Err(error) => {
                        runtime.message = format!("Guide mode initialization rejected: {error:?}");
                        continue;
                    }
                };
                match intent {
                    GuideInitIntent::ChangeExisting { current } => guide_ui.open_change(current),
                    GuideInitIntent::InitialWarpWarning => guide_ui.open_initial_selection(),
                    GuideInitIntent::InitialSelection { .. } => {
                        guide_ui.open_initial_mentor_selection();
                    }
                }
                guide_production.active_source_npc = Some(ActiveGuideSourceNpc {
                    runtime_npc_id: npc_id,
                    table_npc_id: definition.npc_type,
                    ai_type: definition.ai_type,
                });
                if let Err(error) =
                    switch_guide_special_state(&bridge, runtime.player_id, guide_production, true)
                {
                    runtime.message = format!("Guide mode entry state failed: {error}");
                } else {
                    runtime.message = format!(
                        "Guide mode opened from runtime NPC {npc_id} (table {})",
                        definition.npc_type
                    );
                }
            }
            NpcAction::NpcIconClose { npc_id } => {
                let appearance = npc_appearances
                    .iter()
                    .find(|(_, appearance)| appearance.0.npc_id == npc_id);
                let Some((entity, appearance, definition)) =
                    appearance.and_then(|(entity, appearance)| {
                        content
                            .gameplay_npc(appearance.0.npc_type)
                            .map(|definition| (entity, appearance, definition))
                    })
                else {
                    continue;
                };
                // Normal EndMode requests farewell on the NPC's replaceable
                // source. Confirmed warp closes use their separate WarpOk path.
                // The confirmed race result owns this NPC's final voice.
                if !modal_owners.race.production.owns_final_voice_for(npc_id) {
                    modal_owners.audio.runtime.queue_legacy_npc_voice(
                        entity,
                        &definition.move_voice_owner,
                        LegacyNpcVoiceCue::Farewell,
                    );
                }
                if definition.ai_type > 0
                    && let Err(error) =
                        bridge.send(NetworkCommand::InteractWithNpc(NpcInteractionRequest0104 {
                            npc_id: appearance.0.npc_id,
                            flag: 0,
                        }))
                {
                    runtime.message = format!("NpcIconMode interaction-close send failed: {error}");
                }
            }
        }
    }
}
