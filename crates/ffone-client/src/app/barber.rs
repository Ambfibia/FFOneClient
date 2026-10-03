use super::*;
use ffone_client::barber::{BarberInbox, BarberModel, BarberPhase};
use ffone_protocol::wire_0104::{
    PcBarberConfirmReply0104, PcBarberConfirmRequest0104, PcBarberOpenRequest0104,
    PcBarberOpenSuccess0104,
};
use ffone_runtime_contracts::CharacterAppearanceCategory;

pub(super) struct BarberRuntimePlugin;
impl Plugin for BarberRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            drive
                .after(NetworkSessionLifecycleSet::Apply)
                .before(sync_tutorial_input_gate)
                .before(sync_world_player_equipment),
        );
    }
}

fn send(bridge: &NetworkBridge, packet_type: u32, payload: Vec<u8>) -> Result<(), String> {
    let request = ffone_protocol::RegisteredGameplayRequest0104::new(packet_type, payload)
        .map_err(|e| e.to_string())?;
    bridge
        .send(NetworkCommand::SendRegisteredGameplay0104(request))
        .map_err(|e| e.to_string())
}

/// Validate all equipment writes before replacing either authority snapshot.
fn committed_inventory(
    current: &InventoryRuntime0104,
    reply: &PcBarberConfirmReply0104,
) -> Result<InventoryRuntime0104, String> {
    if reply.error_code != 0 || reply.taros < 0 {
        return Err("invalid barber success".into());
    }
    let mut next = current.clone();
    let mut slots = BTreeSet::new();
    for i in 0..3 {
        let wire_item = &reply.unequip_items[i];
        let item = ItemBase0104 {
            item_type: wire_item.type_,
            item_id: wire_item.id,
            option: wire_item.opt,
            time_limit: wire_item.time_limit,
        };
        let slot = reply.unequip_slots[i];
        if slot < 0 {
            continue;
        }
        let slot = usize::try_from(slot).map_err(|e| e.to_string())?;
        if !slots.insert(slot)
            || !current
                .inventory()
                .get(slot)
                .is_some_and(|item| InventoryRuntime0104::item_is_empty(*item))
            || current.equipment()[i + 1] != item
            || InventoryRuntime0104::item_is_empty(item)
        {
            return Err("barber equipment reply does not match the owned inventory".into());
        }
        next.apply_item_move_success(ItemMoveSuccessPacket0104 {
            from_location: 0,
            from_slot_num: (i + 1) as i32,
            from_slot_item: ItemBase0104 {
                item_type: 0,
                item_id: 0,
                option: 0,
                time_limit: 0,
            },
            to_location: 1,
            to_slot_num: slot as i32,
            to_slot_item: item,
        })
        .map_err(|e| e.to_string())?;
    }
    Ok(next)
}

#[allow(clippy::too_many_arguments)]
fn drive(
    state: Res<State<ClientState>>,
    time: Res<Time>,
    bridge: Res<NetworkBridge>,
    data: Res<LoadedCharacterCreationData>,
    mut model: ResMut<BarberModel>,
    mut inbox: ResMut<BarberInbox>,
    mut resets: MessageReader<NetworkSessionReset>,
    mut runtime: ResMut<RuntimeStatus>,
    mut inventory: ResMut<LocalInventoryRuntime>,
    mut projection: ResMut<WorldPlayerEquipmentProjection>,
    mut audio: ResMut<GameplayAudioRuntime>,
    npcs: Query<(Entity, &NetworkNpc0104)>,
    content: Res<TutorialMissionContent>,
) {
    if resets.read().next().is_some() || *state.get() != ClientState::World {
        if model.active() {
            *model = default();
        }
        if !inbox.0.is_empty() {
            inbox.0.clear();
        }
        return;
    }
    if model.close_requested && model.phase != BarberPhase::Confirming {
        *model = default();
        inbox.0.clear();
        audio.queue_gameplay_ui_sound("Close_Screen");
        return;
    }
    if !model.active() {
        inbox.0.clear();
        return;
    }
    if model.taros != runtime.candy {
        model.taros = runtime.candy;
    }
    if model.phase == BarberPhase::Opening && model.original.is_none() {
        let result = (|| {
            let snapshot = inventory.snapshot().ok_or("missing owned inventory")?;
            let character =
                character_flow::authoritative_user_equip_preview_character(&runtime, snapshot)?;
            let style = ffone_protocol::wire_0104::PcStyle0104 {
                pc_uid: character.pc_uid,
                name_check: character.style.name_check,
                first_name: ffone_protocol::FixedUtf16::from_str(&character.first_name)
                    .map_err(|e| e.to_string())?,
                last_name: ffone_protocol::FixedUtf16::from_str(&character.last_name)
                    .map_err(|e| e.to_string())?,
                gender: character.style.gender,
                face_style: character.style.face_style,
                hair_style: character.style.hair_style,
                hair_color: character.style.hair_color,
                skin_color: character.style.skin_color,
                eye_color: character.style.eye_color,
                height: character.style.height,
                body: character.style.body,
                class: character.style.class,
            };
            model.original = Some(style.clone());
            model.draft = Some(style);
            for choice in &data.0.appearance_document().choices {
                let gender = usize::from(choice.gender.protocol_code() == 2);
                let Ok(value) = i8::try_from(choice.value) else {
                    continue;
                };
                let choices = match choice.category {
                    CharacterAppearanceCategory::Hair => &mut model.hair[gender],
                    CharacterAppearanceCategory::Face => &mut model.face[gender],
                    _ => continue,
                };
                if !choices.iter().any(|row| row.0 == value) {
                    choices.push((value, choice.label.clone()));
                }
                let (field, category) = if choice.category == CharacterAppearanceCategory::Hair {
                    (ffone_client::barber::BarberField::Hair, "hair")
                } else {
                    (ffone_client::barber::BarberField::Face, "face")
                };
                model.appearance_keys.insert(
                    (choice.gender.protocol_code() as i8, field as u8, value),
                    format!(
                        "content.appearance.{}.{category}.{}.name",
                        if gender == 0 { "male" } else { "female" },
                        choice.creation_index
                    ),
                );
            }
            model.palettes = data.0.ui_palettes();
            send(
                &bridge,
                0x130000a5,
                PcBarberOpenRequest0104 {
                    npc_id: model.npc_id.ok_or("missing barber owner")?,
                }
                .encode(),
            )
        })();
        if let Err(error) = result {
            runtime.message = format!("Barber open failed: {error}");
            model.error = Some(LocalizedText::new(
                "ui.barber.unavailable",
                "The barber is unavailable. Please try again.",
            ));
        } else {
            audio.queue_gameplay_ui_sound("Open_Screen");
        }
    }
    while let Some(frame) = inbox.0.pop_front() {
        if frame.packet_type == 0x31000138 && model.phase == BarberPhase::Opening {
            match PcBarberOpenSuccess0104::decode(&frame.payload) {
                Ok(prices) => {
                    model.prices = Some(prices);
                    model.phase = BarberPhase::Editing;
                    model.error = None;
                    model.elapsed = 0.;
                }
                Err(error) => runtime.message = format!("Invalid barber prices: {error}"),
            }
        } else if frame.packet_type == 0x31000139 && model.phase == BarberPhase::Confirming {
            let result = (|| {
                let reply =
                    PcBarberConfirmReply0104::decode(&frame.payload).map_err(|e| e.to_string())?;
                if reply.error_code != 0 {
                    model.phase = BarberPhase::Editing;
                    model.error = Some(if reply.error_code == 1 {
                        LocalizedText::new(
                            "ui.barber.inventory_full",
                            "Make room in your inventory for your clothes.",
                        )
                    } else {
                        LocalizedText::new(
                            "ui.barber.failed",
                            "The appearance change was not accepted.",
                        )
                    });
                    return Ok(());
                }
                let next =
                    committed_inventory(inventory.snapshot().ok_or("missing inventory")?, &reply)?;
                let draft = model.draft.clone().ok_or("missing submitted appearance")?;
                let uid = runtime.roster.selected_uid.ok_or("missing character")?;
                let character = runtime
                    .roster
                    .characters
                    .iter_mut()
                    .find(|c| c.pc_uid == uid)
                    .ok_or("missing character summary")?;
                ffone_client::barber::apply_appearance(&mut character.style, &draft);
                inventory.snapshot = Some(next);
                runtime.candy = reply.taros;
                runtime.player_gender = Some(i32::from(draft.gender));
                projection.force_refresh = true;
                *model = default();
                audio.queue_gameplay_ui_sound("Close_Screen");
                Ok::<(), String>(())
            })();
            if let Err(error) = result {
                runtime.message = format!("Barber reply rejected: {error}");
            }
        }
    }
    if model.confirm_requested {
        model.confirm_requested = false;
        if model.can_confirm()
            && let Some(draft) = model.draft.clone()
        {
            match send(
                &bridge,
                0x130000a6,
                PcBarberConfirmRequest0104 { s_pc_style: draft }.encode(),
            ) {
                Ok(()) => {
                    if let Some((entity, npc)) = npcs
                        .iter()
                        .find(|(_, npc)| Some(npc.npc_id) == model.npc_id)
                        && let Some(definition) = content.gameplay_npc(npc.npc_type)
                    {
                        audio.queue_legacy_npc_voice(
                            entity,
                            &definition.move_voice_owner,
                            ffone_client::gameplay_audio::LegacyNpcVoiceCue::BarberOk,
                        );
                    }
                    model.phase = BarberPhase::Confirming;
                    model.elapsed = 0.;
                    model.error = None;
                }
                Err(error) => {
                    runtime.message = format!("Barber confirm failed: {error}");
                    model.error = Some(LocalizedText::new(
                        "ui.barber.unavailable",
                        "The barber is unavailable. Please try again.",
                    ));
                }
            }
        }
    }
    if matches!(model.phase, BarberPhase::Opening | BarberPhase::Confirming) {
        model.elapsed += time.delta_secs();
        if model.elapsed > 10. {
            model.error = Some(if model.phase == BarberPhase::Confirming {
                LocalizedText::new(
                    "ui.barber.pending",
                    "Waiting for confirmation. Reconnect if the server does not respond.",
                )
            } else {
                LocalizedText::new(
                    "ui.barber.unavailable",
                    "The barber is unavailable. Please try again.",
                )
            });
        }
    }
}

#[cfg(test)]
mod tests;
