use super::*;

#[test]
fn nano_tune_command_is_typed_and_uses_the_gameplay_worker_route() {
    let request = NanoTuneRequest0104 {
        nano_id: 36,
        tune_id: 144,
        needed_item_slots: [0; NANO_TUNE_ITEM_SLOT_COUNT_0104],
    };
    let command = NetworkCommand::TuneNano(request);
    let debug = format!("{command:?}");
    assert!(debug.contains("TuneNano"));
    assert!(debug.contains("nano_id: 36"));
    assert!(debug.contains("tune_id: 144"));

    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::sync_channel(32);
    let worker = thread::spawn(move || network_worker(command_rx, event_tx));

    command_tx.send(command).unwrap();
    assert!(matches!(
        event_rx.recv_timeout(Duration::from_secs(1)),
        Ok(NetworkEvent::Error(message))
            if message == "gameplay packet requested before entering the world"
    ));

    command_tx.send(NetworkCommand::Shutdown).unwrap();
    worker.join().unwrap();
}

#[test]
fn normal_world_nano_and_vehicle_commands_use_the_gameplay_worker_route() {
    let commands = [
        NetworkCommand::EquipNano(NanoEquipRequest0104 {
            nano_id: 36,
            nano_slot: 0,
        }),
        NetworkCommand::UnequipNano(NanoUnequipRequest0104 { nano_slot: 0 }),
        NetworkCommand::ActivateNano(NanoActiveRequest0104 { nano_slot: 0 }),
        NetworkCommand::UseNanoSkill(NanoSkillUseRequest0104 {
            bullet_id: 0,
            arg1: 0,
            arg2: 0,
            arg3: 0,
            target_ids: vec![],
        }),
        NetworkCommand::VehicleOff(PcVehicleOffRequest0104 { unused: 0 }),
    ];
    let debug = commands
        .iter()
        .map(|command| format!("{command:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    for name in [
        "EquipNano",
        "UnequipNano",
        "ActivateNano",
        "UseNanoSkill",
        "VehicleOff",
    ] {
        assert!(debug.contains(name));
    }

    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::sync_channel(32);
    let worker = thread::spawn(move || network_worker(command_rx, event_tx));
    for command in commands {
        command_tx.send(command).unwrap();
        assert!(matches!(
            event_rx.recv_timeout(Duration::from_secs(1)),
            Ok(NetworkEvent::Error(message))
                if message == "gameplay packet requested before entering the world"
        ));
    }
    command_tx.send(NetworkCommand::Shutdown).unwrap();
    worker.join().unwrap();
}

#[test]
fn guide_npc_commands_expose_exact_intent_and_share_gameplay_worker_route() {
    let commands = vec![
        NetworkCommand::ChangeMentor(PcChangeMentorRequest0104 { mentor: 4 }),
        NetworkCommand::UseNpcWarp(PcWarpUseNpcRequest0104 {
            npc_id: 2_661,
            warp_id: 76,
            e_il_1: 4,
            item_slot_1: 0,
            e_il_2: 4,
            item_slot_2: 0,
        }),
        NetworkCommand::StopTask(PcTaskStopRequest0104 {
            task_id: 0x1234_5678,
        }),
        NetworkCommand::SwitchSpecialState(PcSpecialStateSwitchRequest0104 {
            pc_id: 81,
            special_state_flag: 16,
        }),
        NetworkCommand::InteractWithNpc(NpcInteractionRequest0104 {
            npc_id: 2_661,
            flag: 1,
        }),
    ];
    let debug = commands
        .iter()
        .map(|command| format!("{command:?}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(debug.contains("ChangeMentor"));
    assert!(debug.contains("mentor: 4"));
    assert!(debug.contains("UseNpcWarp"));
    assert!(debug.contains("warp_id: 76"));
    assert!(debug.contains("StopTask"));
    assert!(debug.contains("task_id: 305419896"));
    assert!(debug.contains("SwitchSpecialState"));
    assert!(debug.contains("special_state_flag: 16"));
    assert!(debug.contains("InteractWithNpc"));
    assert!(debug.contains("flag: 1"));

    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::sync_channel(32);
    let worker = thread::spawn(move || network_worker(command_rx, event_tx));

    for command in commands {
        command_tx.send(command).unwrap();
        assert!(matches!(
            event_rx.recv_timeout(Duration::from_secs(1)),
            Ok(NetworkEvent::Error(message))
                if message == "gameplay packet requested before entering the world"
        ));
    }

    command_tx.send(NetworkCommand::Shutdown).unwrap();
    worker.join().unwrap();
}

#[test]
fn present_npc_types_command_exposes_last_sync_and_uses_gameplay_route() {
    let request = PresentNpcTypesRequest0104 {
        last_sync_time: 0x0102_0304_0506_0708,
    };
    let command = NetworkCommand::RequestPresentNpcTypes(request);
    let debug = format!("{command:?}");
    assert!(debug.contains("RequestPresentNpcTypes"));
    assert!(debug.contains("last_sync_time: 72623859790382856"));

    let (command_tx, command_rx) = mpsc::channel();
    let (event_tx, event_rx) = mpsc::sync_channel(32);
    let worker = thread::spawn(move || network_worker(command_rx, event_tx));

    command_tx
        .send(NetworkCommand::RequestPresentNpcTypes(request))
        .unwrap();
    assert!(matches!(
        event_rx.recv_timeout(Duration::from_secs(1)),
        Ok(NetworkEvent::Error(message))
            if message == "gameplay packet requested before entering the world"
    ));

    command_tx.send(NetworkCommand::Shutdown).unwrap();
    worker.join().unwrap();
}
