use super::*;

#[test]
fn gameplay_sender_barber_registered_requests_reach_the_encrypted_socket() {
    use ffone_protocol::{
        RegisteredGameplayRequest0104,
        wire_0104::{PcBarberConfirmRequest0104, PcBarberOpenRequest0104},
    };

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let e_key = 0x1020_3040_5060_7080;
    let open = PcBarberOpenRequest0104 { npc_id: 900_347 };
    let confirm = PcBarberConfirmRequest0104::decode(&[0; 76]).unwrap();
    let requests = [
        RegisteredGameplayRequest0104::new(packet::P_CL2FE_REQ_PC_BARBER_OPEN, open.encode())
            .unwrap(),
        RegisteredGameplayRequest0104::new(packet::P_CL2FE_REQ_PC_BARBER_CONFIRM, confirm.encode())
            .unwrap(),
    ];
    let expected = requests.clone();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        for (sequence, request) in expected.into_iter().enumerate() {
            let frame = read_client(&mut stream, e_key, sequence as u16);
            assert_eq!(frame.packet_type, request.packet_type());
            assert_eq!(frame.payload, request.payload());
        }
    });
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(TcpStream::connect(endpoint).unwrap()).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    for request in requests {
        sender.send_registered_request(&request).unwrap();
    }
    server.join().unwrap();
}

pub(super) fn read_wire(stream: &mut TcpStream) -> Vec<u8> {
    let mut length = [0u8; 4];
    stream.read_exact(&mut length).unwrap();
    let body_len = u32::from_le_bytes(length) as usize;
    assert!((4..=MAX_BODY_SIZE_0104).contains(&body_len));
    let mut frame = vec![0u8; body_len + 4];
    frame[..4].copy_from_slice(&length);
    stream.read_exact(&mut frame[4..]).unwrap();
    frame
}

#[test]
fn gameplay_sender_guide_npc_routes_use_exact_0104_wire_packets() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let e_key = 0x1020_3040_5060_7080;
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();

        let mentor = read_client(&mut stream, e_key, 0);
        assert_eq!(mentor.packet_type, packet::P_CL2FE_REQ_PC_CHANGE_MENTOR);
        assert_eq!(mentor.payload, vec![4, 0]);
        assert_eq!(
            PcChangeMentorRequest0104::decode(&mentor.payload),
            Ok(PcChangeMentorRequest0104 { mentor: 4 })
        );

        let warp = read_client(&mut stream, e_key, 1);
        assert_eq!(warp.packet_type, packet::P_CL2FE_REQ_PC_WARP_USE_NPC);
        assert_eq!(
            PcWarpUseNpcRequest0104::decode(&warp.payload),
            Ok(PcWarpUseNpcRequest0104 {
                npc_id: 2_661,
                warp_id: 76,
                e_il_1: 4,
                item_slot_1: 0,
                e_il_2: 4,
                item_slot_2: 0,
            })
        );

        let task = read_client(&mut stream, e_key, 2);
        assert_eq!(task.packet_type, packet::P_CL2FE_REQ_PC_TASK_STOP);
        assert_eq!(task.payload, 0x1234_5678i32.to_le_bytes());

        let special_state = read_client(&mut stream, e_key, 3);
        assert_eq!(
            special_state.packet_type,
            packet::P_CL2FE_REQ_PC_SPECIAL_STATE_SWITCH
        );
        assert_eq!(
            special_state.payload,
            vec![0x04, 0x03, 0x02, 0x01, 16, 0, 0, 0]
        );

        let interaction = read_client(&mut stream, e_key, 4);
        assert_eq!(interaction.packet_type, packet::P_CL2FE_REQ_NPC_INTERACTION);
        assert_eq!(interaction.payload, vec![0x65, 0x0a, 0, 0, 1, 0, 0, 0]);
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    sender
        .send_change_mentor(&PcChangeMentorRequest0104 { mentor: 4 })
        .unwrap();
    sender
        .send_npc_warp(&PcWarpUseNpcRequest0104 {
            npc_id: 2_661,
            warp_id: 76,
            e_il_1: 4,
            item_slot_1: 0,
            e_il_2: 4,
            item_slot_2: 0,
        })
        .unwrap();
    sender
        .send_task_stop(&PcTaskStopRequest0104 {
            task_id: 0x1234_5678,
        })
        .unwrap();
    sender
        .send_special_state_switch(&PcSpecialStateSwitchRequest0104 {
            pc_id: 0x0102_0304,
            special_state_flag: 16,
        })
        .unwrap();
    sender
        .send_npc_interaction(&NpcInteractionRequest0104 {
            npc_id: 2_661,
            flag: 1,
        })
        .unwrap();

    server.join().unwrap();
}

#[test]
fn gameplay_sender_vendor_routes_use_exact_0104_wire_packets() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let e_key = 0x1203_4506_7809_abcd;
    let item = ItemBase0104 {
        item_type: 7,
        item_id: 42,
        option: 3,
        time_limit: 0,
    };
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();

        let start = read_client(&mut stream, e_key, 0);
        assert_eq!(start.packet_type, packet::P_CL2FE_REQ_PC_VENDOR_START);
        assert_eq!(
            VendorStartRequest0104::decode(&start.payload),
            Ok(VendorStartRequest0104 {
                npc_id: 9_001,
                vendor_id: 650,
            })
        );

        let table = read_client(&mut stream, e_key, 1);
        assert_eq!(
            table.packet_type,
            packet::P_CL2FE_REQ_PC_VENDOR_TABLE_UPDATE
        );
        assert_eq!(
            VendorTableUpdateRequest0104::decode(&table.payload),
            Ok(VendorTableUpdateRequest0104 {
                npc_id: 9_001,
                vendor_id: 650,
            })
        );

        let buy = read_client(&mut stream, e_key, 2);
        assert_eq!(buy.packet_type, packet::P_CL2FE_REQ_PC_VENDOR_ITEM_BUY);
        assert_eq!(
            VendorItemBuyRequest0104::decode(&buy.payload),
            Ok(VendorItemBuyRequest0104 {
                npc_id: 9_001,
                vendor_id: 9_001,
                list_id: 41,
                item,
                inventory_slot: 4,
            })
        );

        let sell = read_client(&mut stream, e_key, 3);
        assert_eq!(sell.packet_type, packet::P_CL2FE_REQ_PC_VENDOR_ITEM_SELL);
        assert_eq!(
            VendorItemSellRequest0104::decode(&sell.payload),
            Ok(VendorItemSellRequest0104 {
                inventory_slot: 5,
                item_count: 2,
            })
        );

        let restore = read_client(&mut stream, e_key, 4);
        assert_eq!(
            restore.packet_type,
            packet::P_CL2FE_REQ_PC_VENDOR_ITEM_RESTORE_BUY
        );
        assert_eq!(
            VendorItemRestoreBuyRequest0104::decode(&restore.payload),
            Ok(VendorItemRestoreBuyRequest0104 {
                npc_id: 9_001,
                vendor_id: 9_001,
                list_id: 3,
                item,
                inventory_slot: 6,
            })
        );

        let battery = read_client(&mut stream, e_key, 5);
        assert_eq!(
            battery.packet_type,
            packet::P_CL2FE_REQ_PC_VENDOR_BATTERY_BUY
        );
        assert_eq!(
            VendorBatteryBuyRequest0104::decode(&battery.payload),
            Ok(VendorBatteryBuyRequest0104 {
                npc_id: 9_001,
                vendor_id: 9_001,
                list_id: 41,
                item,
            })
        );

        let delete = read_client(&mut stream, e_key, 6);
        assert_eq!(delete.packet_type, packet::P_CL2FE_REQ_PC_ITEM_DELETE);
        assert_eq!(
            PcItemDeleteRequest0104::decode(&delete.payload),
            Ok(PcItemDeleteRequest0104 {
                item_location: 1,
                slot_num: 7,
            })
        );

        let disassemble = read_client(&mut stream, e_key, 7);
        assert_eq!(
            disassemble.packet_type,
            packet::P_CL2FE_REQ_PC_DISASSEMBLE_ITEM
        );
        assert_eq!(
            PcDisassembleItemRequest0104::decode(&disassemble.payload),
            Ok(PcDisassembleItemRequest0104 { item_slot: 8 })
        );
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    sender
        .send_vendor_start(&VendorStartRequest0104 {
            npc_id: 9_001,
            vendor_id: 650,
        })
        .unwrap();
    sender
        .send_vendor_table_update(&VendorTableUpdateRequest0104 {
            npc_id: 9_001,
            vendor_id: 650,
        })
        .unwrap();
    sender
        .send_vendor_item_buy(&VendorItemBuyRequest0104 {
            npc_id: 9_001,
            vendor_id: 9_001,
            list_id: 41,
            item,
            inventory_slot: 4,
        })
        .unwrap();
    sender
        .send_vendor_item_sell(&VendorItemSellRequest0104 {
            inventory_slot: 5,
            item_count: 2,
        })
        .unwrap();
    sender
        .send_vendor_item_restore(&VendorItemRestoreBuyRequest0104 {
            npc_id: 9_001,
            vendor_id: 9_001,
            list_id: 3,
            item,
            inventory_slot: 6,
        })
        .unwrap();
    sender
        .send_vendor_battery_buy(&VendorBatteryBuyRequest0104 {
            npc_id: 9_001,
            vendor_id: 9_001,
            list_id: 41,
            item,
        })
        .unwrap();
    sender
        .send_item_delete(&PcItemDeleteRequest0104 {
            item_location: 1,
            slot_num: 7,
        })
        .unwrap();
    sender
        .send_disassemble_item(&PcDisassembleItemRequest0104 { item_slot: 8 })
        .unwrap();

    server.join().unwrap();
}

#[test]
fn gameplay_sender_pc_exit_uses_the_exact_four_byte_0104_wire_packet() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let e_key = 0x1020_3040_5060_7080;
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let frame = read_client(&mut stream, e_key, 0);
        assert_eq!(frame.packet_type, packet::P_CL2FE_REQ_PC_EXIT);
        assert_eq!(frame.payload, 0x0102_0304i32.to_le_bytes());
        assert_eq!(
            PcExitRequest0104::decode(&frame.payload),
            Ok(PcExitRequest0104 { pc_id: 0x0102_0304 })
        );
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    sender.send_pc_exit(0x0102_0304).unwrap();

    server.join().unwrap();
}

#[test]
fn gameplay_sender_pc_regen_uses_the_exact_twelve_byte_0104_wire_packet() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let e_key = 0x1122_3344_5566_7788;
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let frame = read_client(&mut stream, e_key, 0);
        assert_eq!(frame.packet_type, packet::P_CL2FE_REQ_PC_REGEN);
        assert_eq!(frame.payload, vec![6, 0, 0, 0, 1, 0, 0, 0, 4, 3, 2, 1]);
        assert_eq!(
            PcRegenRequest0104::decode(&frame.payload),
            Ok(PcRegenRequest0104 {
                regen_type: 6,
                e_il: 1,
                index: 0x0102_0304,
            })
        );
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    sender
        .send_pc_regen(&PcRegenRequest0104 {
            regen_type: 6,
            e_il: 1,
            index: 0x0102_0304,
        })
        .unwrap();

    server.join().unwrap();
}

#[test]
fn gameplay_sender_group_leave_uses_the_exact_one_byte_0104_wire_packet() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let e_key = 0x7766_5544_3322_1100;
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let frame = read_client(&mut stream, e_key, 0);
        assert_eq!(frame.packet_type, packet::P_CL2FE_REQ_PC_GROUP_LEAVE);
        assert_eq!(frame.payload, vec![0]);
        assert_eq!(
            GroupLeaveRequest0104::decode(&frame.payload),
            Ok(GroupLeaveRequest0104 { unused: 0 })
        );
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    sender
        .send_group_leave(&GroupLeaveRequest0104 { unused: 0 })
        .unwrap();

    server.join().unwrap();
}

#[test]
fn gameplay_sender_freechat_uses_the_exact_0104_wire_packet() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = listener.local_addr().unwrap();
    let e_key = 0x8877_6655_4433_2211;
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let frame = read_client(&mut stream, e_key, 0);
        assert_eq!(frame.packet_type, packet::P_CL2FE_REQ_PC_FREECHAT);
        assert_eq!(frame.payload.len(), FreeChatRequest0104::SIZE);
        let request = FreeChatRequest0104::decode(&frame.payload).unwrap();
        assert_eq!(request.message.to_string_lossy(), "Привет, FusionFall 🌌");
        assert_eq!(request.emote_code, 23);
    });

    let stream = TcpStream::connect(endpoint).unwrap();
    let sender = GameplaySender {
        inner: Arc::new(Mutex::new(GameplayWriter {
            io: TcpFrameIo::from_stream(stream).unwrap(),
            outbound_e_key: e_key,
        })),
    };
    sender
        .send_freechat(&FreeChatRequest0104 {
            message: FixedUtf16::from_str("Привет, FusionFall 🌌").unwrap(),
            emote_code: 23,
        })
        .unwrap();

    server.join().unwrap();
}
