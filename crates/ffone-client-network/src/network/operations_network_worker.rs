use super::*;

pub(super) fn network_worker(commands: Receiver<NetworkCommand>, events: SyncSender<NetworkEvent>) {
    let mut login: Option<LoginSession> = None;
    let mut gameplay: Option<GameplayHandle> = None;

    loop {
        // Menus and gameplay can remain idle for minutes. Pump the login socket
        // on this same owner so heartbeat replies share command packet ordering.
        if let Some(session) = login.as_mut() {
            match session.poll_idle() {
                Ok(Some(frame)) => {
                    let _ = events.send(NetworkEvent::incoming_frame_0104(
                        NetworkStream0104::Login,
                        frame,
                    ));
                }
                Ok(None) => {}
                Err(error) => {
                    login = None;
                    let _ = events.send(NetworkEvent::Disconnected {
                        reason: DisconnectReason0104::LoginTransportFailed(error.to_string()),
                    });
                }
            }
        }
        let command = match commands.recv_timeout(Duration::from_millis(100)) {
            Ok(command) => command,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        let cookie_login = matches!(&command, NetworkCommand::LoginCookie { .. });
        match command {
            NetworkCommand::Login {
                login_address,
                username,
                password,
            } | NetworkCommand::LoginCookie { login_address, username, cookie: password } => {
                disconnect_gameplay(&mut gameplay);
                login = None;
                let _ = events.send(NetworkEvent::Connecting);
                let result = if cookie_login { LoginSession::connect_cookie(login_address, &username, &password) }
                    else { LoginSession::connect_password(login_address, &username, &password) };
                match result {
                    Ok(session) => {
                        let payment_flag = session.login_success().payment_flag;
                        let characters = character_summaries(&session);
                        login = Some(session);
                        let _ = events.send(NetworkEvent::LoginMetadata { payment_flag });
                        let _ = events.send(NetworkEvent::Characters(characters));
                    }
                    Err(error) => {
                        let _ = events.send(NetworkEvent::Error(error.to_string()));
                    }
                }
            }
            NetworkCommand::CheckCharacterName(request) => {
                let Some(login_session) = login.as_mut() else {
                    send_login_required(&events, "character name check");
                    continue;
                };
                match login_session.check_character_name(&request) {
                    Ok(checked) => {
                        let _ = events.send(NetworkEvent::CharacterNameChecked(checked));
                    }
                    Err(NetError::CharacterNameCheckRejected { error_code }) => {
                        let _ = events.send(NetworkEvent::CharacterOperationRejected {
                            stage: CharacterOperationStage::NameCheck,
                            error_code,
                        });
                    }
                    Err(error) => {
                        let _ = events.send(NetworkEvent::Error(error.to_string()));
                    }
                }
            }
            NetworkCommand::ExitDuplicateSession {
                login_address,
                username,
                password,
            } => match LoginSession::request_duplicate_exit(login_address, &username, &password) {
                Ok(()) => {
                    let _ = events.send(NetworkEvent::DuplicateSessionExitRequested);
                }
                Err(error) => {
                    let _ = events.send(NetworkEvent::Error(error.to_string()));
                }
            },
            NetworkCommand::ReserveCharacterName {
                request,
                slot,
                gender,
            } => {
                let Some(login_session) = login.as_mut() else {
                    send_login_required(&events, "character name reservation");
                    continue;
                };
                let checked = match login_session.check_character_name(&request) {
                    Ok(checked) => checked,
                    Err(NetError::CharacterNameCheckRejected { error_code }) => {
                        let _ = events.send(NetworkEvent::CharacterOperationRejected {
                            stage: CharacterOperationStage::NameCheck,
                            error_code,
                        });
                        continue;
                    }
                    Err(error) => {
                        let _ = events.send(NetworkEvent::Error(error.to_string()));
                        continue;
                    }
                };
                let save =
                    CharacterNameSaveRequest0104::from_check(slot, gender, &request, &checked);
                match login_session.save_character_name(&save) {
                    Ok(saved) => {
                        let _ = events.send(NetworkEvent::CharacterNameSaved(saved));
                    }
                    Err(NetError::CharacterNameSaveRejected { error_code }) => {
                        let _ = events.send(NetworkEvent::CharacterOperationRejected {
                            stage: CharacterOperationStage::NameSave,
                            error_code,
                        });
                    }
                    Err(error) => {
                        let _ = events.send(NetworkEvent::Error(error.to_string()));
                    }
                }
            }
            NetworkCommand::SaveCharacterName(request) => {
                let Some(login_session) = login.as_mut() else {
                    send_login_required(&events, "character name save");
                    continue;
                };
                match login_session.save_character_name(&request) {
                    Ok(saved) => {
                        let _ = events.send(NetworkEvent::CharacterNameSaved(saved));
                    }
                    Err(NetError::CharacterNameSaveRejected { error_code }) => {
                        let _ = events.send(NetworkEvent::CharacterOperationRejected {
                            stage: CharacterOperationStage::NameSave,
                            error_code,
                        });
                    }
                    Err(error) => {
                        let _ = events.send(NetworkEvent::Error(error.to_string()));
                    }
                }
            }
            NetworkCommand::CreateCharacter(request) => {
                let Some(login_session) = login.as_mut() else {
                    send_login_required(&events, "character creation");
                    continue;
                };
                let slot = login_session
                    .characters()
                    .iter()
                    .find(|character| character.pc_uid() == request.style.pc_uid)
                    .map(|character| character.slot());
                match login_session.create_character(&request) {
                    Ok(created) => {
                        let slot = slot.or_else(|| {
                            login_session
                                .characters()
                                .iter()
                                .find(|character| character.pc_uid() == created.style.pc_uid)
                                .map(|character| character.slot())
                        });
                        let Some(slot) = slot else {
                            let _ = events.send(NetworkEvent::Error(
                                "created character is absent from the refreshed login roster"
                                    .to_owned(),
                            ));
                            continue;
                        };
                        let _ = events.send(NetworkEvent::CharacterCreated {
                            slot,
                            response: created,
                        });
                        let _ = events
                            .send(NetworkEvent::Characters(character_summaries(login_session)));
                    }
                    Err(NetError::CharacterCreateRejected { error_code }) => {
                        let _ = events.send(NetworkEvent::CharacterOperationRejected {
                            stage: CharacterOperationStage::Appearance,
                            error_code,
                        });
                    }
                    Err(error) => {
                        let _ = events.send(NetworkEvent::Error(error.to_string()));
                    }
                }
            }
            NetworkCommand::DeleteCharacter { pc_uid } => {
                let Some(login_session) = login.as_mut() else {
                    send_login_required(&events, "character deletion");
                    continue;
                };
                match login_session.delete_character(pc_uid) {
                    Ok(deleted) => {
                        let _ = events.send(NetworkEvent::CharacterDeleted {
                            pc_uid,
                            response: deleted,
                        });
                        let _ = events
                            .send(NetworkEvent::Characters(character_summaries(login_session)));
                    }
                    Err(NetError::CharacterDeleteRejected { error_code }) => {
                        let _ = events.send(NetworkEvent::CharacterOperationRejected {
                            stage: CharacterOperationStage::Delete,
                            error_code,
                        });
                    }
                    Err(error) => {
                        let _ = events.send(NetworkEvent::Error(error.to_string()));
                    }
                }
            }
            NetworkCommand::ChangeCharacterName(request) => {
                let Some(login_session) = login.as_mut() else {
                    send_login_required(&events, "character rename");
                    continue;
                };
                match login_session.change_character_name(&request) {
                    Ok(changed) => {
                        let _ = events.send(NetworkEvent::CharacterNameChanged(changed));
                        let _ = events
                            .send(NetworkEvent::Characters(character_summaries(login_session)));
                    }
                    Err(NetError::CharacterNameChangeRejected { error_code }) => {
                        let _ = events.send(NetworkEvent::CharacterOperationRejected {
                            stage: CharacterOperationStage::Rename,
                            error_code,
                        });
                    }
                    Err(error) => {
                        let _ = events.send(NetworkEvent::Error(error.to_string()));
                    }
                }
            }
            NetworkCommand::RefreshCharacters => {
                let characters = login.as_ref().map(character_summaries);
                let Some(characters) = characters else {
                    send_login_required(&events, "character-list refresh");
                    continue;
                };
                let _ = events.send(NetworkEvent::Characters(characters));
            }
            NetworkCommand::SelectCharacter { pc_uid, location } => {
                enter_character_world(
                    pc_uid,
                    CharacterEntryRoute::Selection,
                    location,
                    &mut login,
                    &mut gameplay,
                    &events,
                );
            }
            NetworkCommand::CompleteTutorial { pc_uid } => {
                enter_character_world(
                    pc_uid,
                    CharacterEntryRoute::TutorialCompletion,
                    CharacterEntryLocation0104::Saved,
                    &mut login,
                    &mut gameplay,
                    &events,
                );
            }
            NetworkCommand::Move(request) => send_gameplay(gameplay.as_ref(), &events, |sender| {
                sender.send_move(&request)
            }),
            NetworkCommand::Stop(request) => send_gameplay(gameplay.as_ref(), &events, |sender| {
                sender.send_stop(&request)
            }),
            NetworkCommand::Jump(request) => send_gameplay(gameplay.as_ref(), &events, |sender| {
                sender.send_jump(&request)
            }),
            NetworkCommand::CombatBegin(pc_id) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_combat_begin(pc_id)
                })
            }
            NetworkCommand::CombatEnd(pc_id) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_combat_end(pc_id)
                })
            }
            NetworkCommand::AttackNpcs(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_attack_npcs(&request)
                })
            }
            NetworkCommand::AttackChars(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_attack_chars(&request)
                })
            }
            NetworkCommand::RocketStyleFire(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_rocket_style_fire(&request)
                })
            }
            NetworkCommand::GrenadeStyleFire(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_grenade_style_fire(&request)
                })
            }
            NetworkCommand::Regen(request) => send_gameplay(gameplay.as_ref(), &events, |sender| {
                sender.send_pc_regen(&request)
            }),
            NetworkCommand::ChangeMentor(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_change_mentor(&request)
                })
            }
            NetworkCommand::UseNpcWarp(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_npc_warp(&request)
                })
            }
            NetworkCommand::StopTask(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_task_stop(&request)
                })
            }
            NetworkCommand::SwitchSpecialState(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_special_state_switch(&request)
                })
            }
            NetworkCommand::InteractWithNpc(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_npc_interaction(&request)
                })
            }
            NetworkCommand::OpenBank(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_bank_open(&request)
                })
            }
            NetworkCommand::CloseBank(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_bank_close(&request)
                })
            }
            NetworkCommand::MoveItem(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_item_move(&request)
                })
            }
            NetworkCommand::EquipNano(request) => {
                send_registered_world_action_0104(gameplay.as_ref(), &events, || {
                    registered_fixed_world_action_0104(packet::P_CL2FE_REQ_NANO_EQUIP, &request)
                })
            }
            NetworkCommand::UnequipNano(request) => {
                send_registered_world_action_0104(gameplay.as_ref(), &events, || {
                    registered_fixed_world_action_0104(packet::P_CL2FE_REQ_NANO_UNEQUIP, &request)
                })
            }
            NetworkCommand::ActivateNano(request) => {
                send_registered_world_action_0104(gameplay.as_ref(), &events, || {
                    registered_fixed_world_action_0104(packet::P_CL2FE_REQ_NANO_ACTIVE, &request)
                })
            }
            NetworkCommand::UseNanoSkill(request) => {
                send_registered_world_action_0104(gameplay.as_ref(), &events, || {
                    registered_nano_skill_use_request_0104(&request)
                })
            }
            NetworkCommand::TuneNano(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_nano_tune(&request)
                })
            }
            NetworkCommand::VehicleOff(request) => {
                send_registered_world_action_0104(gameplay.as_ref(), &events, || {
                    registered_fixed_world_action_0104(packet::P_CL2FE_REQ_PC_VEHICLE_OFF, &request)
                })
            }
            NetworkCommand::VendorStart(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_vendor_start(&request)
                })
            }
            NetworkCommand::VendorTableUpdate(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_vendor_table_update(&request)
                })
            }
            NetworkCommand::VendorBuy(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_vendor_item_buy(&request)
                })
            }
            NetworkCommand::VendorSell(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_vendor_item_sell(&request)
                })
            }
            NetworkCommand::VendorRestore(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_vendor_item_restore(&request)
                })
            }
            NetworkCommand::VendorBatteryBuy(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_vendor_battery_buy(&request)
                })
            }
            NetworkCommand::DeleteInventoryItem(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_item_delete(&request)
                })
            }
            NetworkCommand::DisassembleInventoryItem(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_disassemble_item(&request)
                })
            }
            NetworkCommand::RequestPresentNpcTypes(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_present_npc_types(&request)
                })
            }
            NetworkCommand::RegisterQuickSlot(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_register_quick_slot(&request)
                })
            }
            NetworkCommand::UseItem(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_item_use(&request)
                })
            }
            NetworkCommand::OpenChest(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_item_chest_open(&request)
                })
            }
            NetworkCommand::SendRegisteredGameplay0104(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_registered_request(&request)
                })
            }
            NetworkCommand::InviteEscortNpc(npc_id) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_escort_invite(npc_id)
                })
            }
            NetworkCommand::KickEscortNpc(npc_id) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_escort_kick(npc_id)
                })
            }
            NetworkCommand::RequestBuddy(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_buddy_request(&request)
                })
            }
            NetworkCommand::RequestBuddyByName(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_buddy_find_name(&request)
                })
            }
            NetworkCommand::AcceptBuddyByName(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_buddy_find_name_accept(&request)
                })
            }
            NetworkCommand::AcceptBuddy(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_buddy_accept(&request)
                })
            }
            NetworkCommand::RefreshBuddyState(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_buddy_state_refresh(&request)
                })
            }
            NetworkCommand::BlockBuddy(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_buddy_block(&request)
                })
            }
            NetworkCommand::RemoveBuddy(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_buddy_remove(&request)
                })
            }
            NetworkCommand::WarpToBuddy(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_buddy_warp(&request)
                })
            }
            NetworkCommand::LeaveGroup(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_group_leave(&request)
                })
            }
            NetworkCommand::SendFreeChat(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_freechat(&request)
                })
            }
            NetworkCommand::SetGmValue(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_gm_set_value(&request)
                })
            }
            NetworkCommand::SendBuddyFreeChat(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_buddy_freechat(&request)
                })
            }
            NetworkCommand::SendAllGroupFreeChat(request) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_all_group_freechat(&request)
                })
            }
            NetworkCommand::EnvironmentDamage(enabled) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_dot_damage(enabled)
                })
            }
            NetworkCommand::EnvironmentHeal(enabled) => {
                send_gameplay(gameplay.as_ref(), &events, |sender| {
                    sender.send_dot_heal(enabled)
                })
            }
            NetworkCommand::ExitWorld => {
                let Some(gameplay) = gameplay.as_ref() else {
                    let _ = events.send(NetworkEvent::Error(
                        "gameplay packet requested before entering the world".to_owned(),
                    ));
                    continue;
                };
                if let Err(error) = gameplay.begin_graceful_exit() {
                    let _ = events.send(NetworkEvent::Error(error.to_string()));
                }
            }
            NetworkCommand::ReturnToCharacterSelection => {
                disconnect_gameplay(&mut gameplay);
                if let Some(session) = login.as_ref() {
                    let _ = events.send(NetworkEvent::ReturnedToCharacterSelection(
                        character_summaries(session),
                    ));
                } else {
                    send_login_required(&events, "return to character selection");
                }
            }
            NetworkCommand::Disconnect => {
                disconnect_gameplay(&mut gameplay);
                login = None;
                let _ = events.send(NetworkEvent::Disconnected {
                    reason: DisconnectReason0104::Requested,
                });
            }
            NetworkCommand::Shutdown => {
                disconnect_gameplay(&mut gameplay);
                break;
            }
        }
    }
}

pub(super) fn enter_character_world(
    pc_uid: i64,
    route: CharacterEntryRoute,
    location: CharacterEntryLocation0104,
    login: &mut Option<LoginSession>,
    gameplay: &mut Option<GameplayHandle>,
    events: &SyncSender<NetworkEvent>,
) {
    let Some(mut login_session) = login.take() else {
        let error = match route {
            CharacterEntryRoute::Selection => "character selection requires a successful login",
            CharacterEntryRoute::TutorialCompletion => "tutorial completion requires a successful login",
        };
        publish_character_entry_failure(events, route, pc_uid, error.to_owned());
        return;
    };

    if route == CharacterEntryRoute::TutorialCompletion {
        if let Err(error) = retire_tutorial_gameplay(gameplay) {
            *login = Some(login_session);
            publish_character_entry_failure(events, route, pc_uid, error);
            return;
        }
        let request = CharacterTutorialSaveRequest0104 {
            pc_uid,
            tutorial_flag: 1,
        };
        if let Err(error) = login_session.send_tutorial_completion(&request) {
            *login = Some(login_session);
            publish_character_entry_failure(events, route, pc_uid, error.to_string());
            return;
        }
    }

    // Retrobution sends CHAR_SELECT immediately after the fire-and-forget
    // SAVE_CHAR_TUTOR. Keeping both writes inside this one worker command
    // prevents any other producer from interleaving a packet between them.
    let _ = events.send(NetworkEvent::EnteringWorld { pc_uid });
    let result = (|| {
        let character = login_session
            .characters()
            .iter()
            .find(|character| character.pc_uid() == pc_uid)
            .ok_or(NetError::UnknownCharacter(pc_uid))?;
        let mut login_style = character.style();
        let location = character_entry_location(route, location);
        if route == CharacterEntryRoute::TutorialCompletion {
            // SAVE_CHAR_TUTOR has no success reply and LoginSession retains
            // the original CHAR_INFO bytes. Publish the completed flag to the
            // new WorldReady snapshot just as the retained-login route does.
            login_style.tutorial_flag = 1;
        }
        let ticket = login_session.select_character(pc_uid)?;
        connect_character_shard(pc_uid, login_style, location, ticket)
    })();
    match result {
        Ok((handle, mut receiver, world)) => {
            // One owner keeps the login decoder, pending creation and outbound
            // sequence alive across world entry, menu return and another creation.
            login_session.observe_character_position(pc_uid, world.position);
            *login = Some(login_session);
            disconnect_gameplay(gameplay);
            let reader_shutdown = Arc::clone(&handle.shutting_down);
            let reader_finished = Arc::clone(&handle.reader_finished);
            *gameplay = Some(handle);
            if !publish_world_then_spawn_gameplay_reader(
                events,
                &reader_shutdown,
                &reader_finished,
                world,
                move || receiver.read_next(),
            ) {
                disconnect_gameplay(gameplay);
                return;
            }
        }
        Err(error) => {
            *login = Some(login_session);
            publish_character_entry_failure(events, route, pc_uid, error.to_string());
        }
    }
}

pub(super) fn retire_tutorial_gameplay(gameplay: &mut Option<GameplayHandle>) -> Result<(), String> {
    let Some(active) = gameplay.take() else {
        return Ok(());
    };

    if let Err(error) = active.begin_graceful_exit() {
        active.stop();
        return Err(format!(
            "tutorial shard could not begin its final save before completion: {error}"
        ));
    }
    if !active.wait_for_reader(TUTORIAL_GAMEPLAY_EXIT_TIMEOUT) {
        active.stop();
        return Err(format!(
            "tutorial shard did not finish saving within {} seconds",
            TUTORIAL_GAMEPLAY_EXIT_TIMEOUT.as_secs()
        ));
    }

    // OpenFusion flushes the old in-memory Player while closing this socket.
    // Only after that flush may SAVE_CHAR_TUTOR grant the permanent weapon,
    // Buttercup Nano, and completed tutorial mission flags. Otherwise the
    // duplicate-player cleanup writes the stale tutorial snapshot over them.
    Ok(())
}

pub(super) fn character_entry_location(
    route: CharacterEntryRoute,
    requested: CharacterEntryLocation0104,
) -> CharacterEntryLocation0104 {
    match route {
        CharacterEntryRoute::Selection => requested,
        CharacterEntryRoute::TutorialCompletion => {
            CharacterEntryLocation0104::TutorialCompletionSectorV
        }
    }
}
