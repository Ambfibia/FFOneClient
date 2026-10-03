use super::*;

pub(in super::super) fn requested() -> bool {
    env::var_os("FFONE_PERF_OUTPUT").is_some()
        || env::var_os("FFONE_CHARACTER_SESSION_PROBE_OUTPUT").is_some()
        || env::var_os("FFONE_NANO_ATTACK_PROBE_OUTPUT").is_some()
        || env::var_os("FFONE_CREATION_PROBE_OUTPUT").is_some()
        || env::var_os("FFONE_CUTSCENE_PROBE_OUTPUT").is_some()
        || env::var_os("FFONE_TUTORIAL_NETWORK_PROBE_OUTPUT").is_some()
        || env::var_os("FFONE_QUIT_MENU_NETWORK_PROBE_OUTPUT").is_some()
}

// Offline authority fixture: exercise the production equipment, movement-buff,
// presentation and audio owners without sending anything to a shard.
pub(super) fn mount_vehicle_fixture(
    fixture: Res<VehicleFixture>,
    state: Res<State<ClientState>>,
    content: Res<TutorialMissionContent>,
    mut inventory: ResMut<LocalInventoryRuntime>,
    mut vehicle: ResMut<LocalVehiclePresentationRuntime>,
) {
    if *state.get() != ClientState::World {
        return;
    }
    if inventory
        .snapshot()
        .is_none_or(|s| s.equipment()[8].item_id != fixture.0)
    {
        let mut bytes = vec![0; ffone_protocol::PcLoadData0104::SIZE];
        let offset = ffone_protocol::PcLoadData0104::EQUIPMENT_OFFSET + 8 * ItemBase0104::SIZE;
        bytes[offset..offset + 2].copy_from_slice(&10_i16.to_le_bytes());
        bytes[offset + 2..offset + 4].copy_from_slice(&fixture.0.to_le_bytes());
        bytes[offset + 4..offset + 8].copy_from_slice(&1_i32.to_le_bytes());
        inventory.seed(1, &ffone_protocol::PcLoadData0104::decode(&bytes).unwrap());
    }
    vehicle.family = LegacyVehiclePresentationFamily::from_legacy_equip_type(
        content
            .gameplay_vehicle_equip_type(fixture.0)
            .expect("vehicle table row"),
    )
    .expect("vehicle family");
}

// Opt-in presentation capture, not an FPS benchmark: hold one reward packet's
// one-second pose so world loading time cannot consume the short-lived notice.
pub(super) fn show_reward_fixture(
    content: Res<TutorialMissionContent>,
    mut notices: ResMut<ffone_client::gameplay_ui::rewards::RewardNotices>,
) {
    let mut mission = WorldMissionRuntime::default();
    mission
        .apply_event(
            &WorldMissionServerEvent0104::TaskStartSuccess(
                ffone_protocol::PcTaskStartSuccess0104 {
                    task_id: 2248,
                    remaining_time: 0,
                },
            ),
            &content,
        )
        .unwrap();
    let mut packet = ffone_protocol::RewardItemReply0104 {
        candy: 100,
        fusion_matter: 200,
        nano_battery: 15,
        weapon_battery: 30,
        pack_padding: [0; 3],
        fatigue: 0,
        fatigue_level: 0,
        npc_type_id: 2676,
        task_id: 2248,
        items: vec![ItemReward0104 {
            inventory_location: 1,
            slot: 0,
            item: ItemBase0104 {
                item_type: 9,
                item_id: 1,
                option: 1,
                time_limit: 0,
            },
        }],
    };
    if env::var("FFONE_PERF_REWARDS").as_deref() == Ok("found") {
        packet.items.push(ItemReward0104 {
            inventory_location: 2,
            slot: 0,
            item: ItemBase0104 {
                item_type: 8,
                item_id: 537,
                option: 1,
                time_limit: 0,
            },
        });
    }
    notices.clear();
    notices.receive(&packet, &content, &mission, 10, 20);
    if env::var("FFONE_PERF_REWARDS").as_deref() == Ok("status") {
        notices.clear();
        notices.receive_currencies(12300, 4500, 12345, 4567);
        notices.set_inventory_full(Some(true));
        for _ in 0..25 {
            notices.advance(0.11);
        }
    } else {
        notices.advance(1.0);
    }
}

pub(super) fn setup(
    mut commands: Commands,
    capture: Res<Capture>,
    assets: Res<AssetServer>,
    catalog: Res<NativeWorldCatalog>,
    mut rigs: ResMut<NativePlayerRigAssetCache>,
    rig_catalog: Res<NativePlayerRigCatalog>,
    weapons: Res<PlayerWeaponAnimationCatalog>,
    data: Res<CharacterCreationDataResource>,
    mut loading: ResMut<GameplayLoadingState>,
    mut runtime: ResMut<RuntimeStatus>,
    mut next_state: ResMut<NextState<ClientState>>,
    waypoints: Res<ClientNpcWaypointCatalog>,
    mut ingress: ResMut<ffone_client::entity_lifecycle::NetworkEntityLifecycleIngress0104>,
    mut tutorial: ResMut<TutorialSession>,
) {
    let tutorial_entry = env::var_os("FFONE_PERF_TUTORIAL").is_some();
    let gender = if env::var_os("FFONE_PERF_PLAYER_FACE").is_some()
        || env::var_os("FFONE_PERF_TUTORIAL_FINALE").is_some() {
        env::var("FFONE_PERF_PLAYER_GENDER")
            .unwrap_or_else(|_| "1".into())
            .parse::<i32>()
            .expect("gender 1 or 2")
    } else {
        1
    };
    assert!((1..=2).contains(&gender));
    let mut appearance = ffone_client::character_creation_ui::CharacterAppearance::default();
    appearance.gender = if gender == 2 {
        ffone_client::character_creation_ui::CharacterGender::Girl
    } else {
        ffone_client::character_creation_ui::CharacterGender::Boy
    };
    if env::var_os("FFONE_PERF_TUTORIAL_FINALE").is_some() {
        // Light skin makes missing face-overlay fragments visible beneath
        // the blue hologram in the reported close-up.
        appearance.skin_color = 9;
    }
    let creator = data
        .0
        .resolve_creator(1, 1, "GPU", "Capture", &appearance)
        .expect("production default creator");
    let character = CharacterSummary {
        slot: 1,
        level: 1,
        pc_uid: 1 + ((capture.entry - 1) % 2) as i64,
        first_name: "GPU".into(),
        last_name: "Capture".into(),
        position: [0; 3],
        style: ffone_protocol::CharacterStyle0104 {
            name_check: 1,
            gender: creator.style.gender,
            face_style: creator.style.face_style,
            hair_style: creator.style.hair_style,
            hair_color: creator.style.hair_color,
            skin_color: creator.style.skin_color,
            eye_color: creator.style.eye_color,
            height: creator.style.height,
            body: creator.style.body,
            class: creator.style.class,
            appearance_flag: 1,
            tutorial_flag: if tutorial_entry { 0 } else { 1 },
            payzone_flag: 0,
        },
        equipment: [ffone_protocol::EquippedItem0104::default();
            ffone_protocol::CHARACTER_EQUIP_SLOT_COUNT_0104],
    };
    spawn_native_world_slice(
        &mut commands,
        &assets,
        &catalog,
        &mut rigs,
        &rig_catalog,
        &weapons,
        &data.0,
        &character,
        character.pc_uid,
        1,
        if tutorial_entry {
            NativeWorldScope::Tutorial
        } else {
            NativeWorldScope::WorldMap
        },
        ProtocolPosition::from_native(capture.position).raw(),
        if tutorial_entry {
            TUTORIAL_START_ANGLE
        } else {
            135
        },
        tutorial_entry,
        &mut loading,
        None,
    )
    .expect("production world capture admission");
    runtime.player_id = Some(1);
    if tutorial_entry {
        tutorial.character = Some(character.clone());
    }
    if capture.entries > 1 || tutorial_entry || env::var_os("FFONE_PERF_PLAYER_FACE").is_some()
        || env::var_os("FFONE_PERF_GAMEPAD").is_some()
    {
        runtime.roster.selected_uid = Some(character.pc_uid);
        runtime.roster.characters = vec![character];
    }
    runtime.player_gender = Some(gender);
    runtime.player_name = "Performance Capture".into();
    runtime.player_level = 1;
    runtime.hp = Some(1000);
    runtime.max_hp = 1000;
    runtime.map_number = Some(0);
    next_state.set(if tutorial_entry {
        ClientState::Tutorial
    } else {
        ClientState::World
    });
    if env::var_os("FFONE_PERF_NPCS").is_some()
        || env::var_os("FFONE_PERF_NPC_SPEECH").is_some()
        || capture.transport_npc_type.is_some()
        || capture.npc_chat_probe
    {
        use ffone_client::entity_lifecycle::NetworkSessionEpoch0104;
        let epoch = NetworkSessionEpoch0104(capture.entry as u64);
        ingress.begin_session(epoch, 1);
        // Deterministic offline fixture at published NPC positions. Server-only
        // movement, combat and player crowds are not synthesized by this probe.
        for row in waypoints.rows().iter().filter(|row| {
            let delta = row.native_position() - capture.position;
            Vec2::new(delta.x, delta.z).length_squared() <= 340.0 * 340.0
        }) {
            let packet = ffone_protocol::NpcEnter0104 {
                appearance: ffone_protocol::NpcAppearance0104 {
                    npc_id: row.row_index as i32 + 100,
                    npc_type: row.npc_type,
                    hp: 1000,
                    condition_bit_flag: 0,
                    position: ProtocolPosition::from_native(row.native_position()).raw(),
                    angle: 0,
                    barker_type: 0,
                },
            };
            ingress.push_frame(
                epoch,
                DecodedFrame {
                    packet_type: packet::P_FE2CL_NPC_ENTER,
                    flags: 0,
                    checksum: 0,
                    payload: packet.encode(),
                },
            );
        }
    }
}

pub(super) fn open_transport_fixture(
    mut capture: ResMut<Capture>,
    state: Res<State<ClientState>>,
    loading: Res<GameplayLoadingState>,
    npcs: Query<
        (&NetworkNpcAppearance0104, &GlobalTransform),
        Or<(With<NetworkNpcVisual0104>, With<NetworkHnpcVisual0104>)>,
    >,
    catalog: Res<TransportationCatalog>,
    mut model: ResMut<TransportationModel>,
    mut production: ResMut<TransportationProductionRuntime>,
) {
    let Some(npc_type) = capture.transport_npc_type else {
        return;
    };
    // The offline bridge may emit its initial disconnected boundary after
    // world assets become ready. Reopen the fixture after that legitimate
    // modal reset; only a continuously visible menu can satisfy the capture.
    if (capture.transport_opened && model.phase() != TransportationPhase::Hidden)
        || *state.get() != ClientState::World
        || loading.visible
    {
        return;
    }
    let Some((npc, transform)) = npcs
        .iter()
        .filter(|(npc, _)| npc.0.npc_type == npc_type)
        .min_by_key(|(npc, _)| npc.0.npc_id)
    else {
        return;
    };
    let player = native_to_unity_vector(capture.position);
    let target = native_to_unity_vector(transform.translation());
    model
        .open(
            &catalog,
            TransportationOpenContext {
                player: TransportationPlayerSnapshot {
                    position: TransportationWorldPoint::new(player.x, player.y, player.z),
                    taros: 10000,
                    unlocks: TransportationUnlocks {
                        warp_location_flags: u32::MAX,
                        wyvern_location_flags: [u64::MAX; 2],
                    },
                    cursor_was_locked: false,
                },
                target: TransportationTarget::Npc {
                    npc_instance_id: npc.0.npc_id,
                    npc_table_id: npc_type,
                    npc_position: TransportationWorldPoint::new(target.x, target.y, target.z),
                    has_move_ok_voice: false,
                },
            },
        )
        .expect("offline transportation presentation");
    production.begin_npc(npc.0.npc_id, false, model.service(), None);
    capture.transport_opened = true;
    capture.ready = None;
    capture.samples.clear();
}

pub(super) fn drive(capture: Res<Capture>, mut cameras: Query<&mut LegacyOrbitCamera>) {
    if capture.orbit && !capture.samples.is_empty() {
        for mut camera in &mut cameras {
            camera.yaw_degrees = 135.0 + capture.samples.len() as f32 * 0.5;
        }
    }
}

pub(super) fn discard_live_fixture_input(world: &mut World) {
    if env::var_os("FFONE_PERF_INTERACTIVE").is_some() { return; }
    if world.resource::<Capture>().focus_probe {
        return;
    }
    // Winit delivers real desktop input even in the offline fixture. It must
    // not move the player/camera or open a window during matched measurements.
    world
        .resource_mut::<Messages<bevy::input::keyboard::KeyboardInput>>()
        .clear();
    world
        .resource_mut::<Messages<bevy::input::mouse::MouseButtonInput>>()
        .clear();
    world
        .resource_mut::<Messages<bevy::input::mouse::MouseMotion>>()
        .clear();
    world
        .resource_mut::<Messages<bevy::input::mouse::MouseWheel>>()
        .clear();
    world.resource_mut::<ButtonInput<KeyCode>>().reset_all();
    // The Croc Pot gesture spans frames. Real desktop messages were discarded
    // above; keep its synthetic held button until the fixture releases it.
    if !enchant_regression::owns_pointer(world) {
        world.resource_mut::<ButtonInput<MouseButton>>().reset_all();
    }
}

pub(super) fn drive_focus_probe(world: &mut World, capture: &mut Capture) {
    use bevy::{
        input::{ButtonState, mouse::MouseButtonInput},
        window::WindowFocused,
    };
    use ffone_client::input_focus::GameInputFocus;
    let window = world
        .query_filtered::<Entity, With<PrimaryWindow>>()
        .single(world)
        .unwrap();
    let mouse = world.resource::<ButtonInput<MouseButton>>();
    match capture.focus_phase {
        1 => assert!(
            mouse.pressed(MouseButton::Left),
            "initial mouse press was lost"
        ),
        2 => {
            assert!(!mouse.pressed(MouseButton::Left));
            assert!(!mouse.just_released(MouseButton::Left));
            assert!(world.resource::<GameInputFocus>().suppressed);
            let cursor = world.get::<CursorOptions>(window).unwrap();
            assert_eq!(cursor.grab_mode, CursorGrabMode::None);
            assert!(cursor.visible);
        }
        3 => {
            assert!(
                mouse.just_pressed(MouseButton::Left),
                "first click after return was lost"
            );
            assert!(!world.resource::<GameInputFocus>().suppressed);
            let cursor = world.get::<CursorOptions>(window).unwrap();
            assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);
            assert!(!cursor.visible);
            fs::write(capture.output.join("input-focus.json"),
                br#"{"simulatedWindowEvents":true,"cancelledWithoutRelease":true,"cursorReleased":true,"cursorRestored":true,"firstReturnClickAccepted":true}"#).unwrap();
        }
        _ => {}
    }
    let focused = capture.focus_phase != 1;
    world.get_mut::<Window>(window).unwrap().focused = focused;
    world.write_message(WindowFocused { window, focused });
    if capture.focus_phase == 0 || capture.focus_phase == 2 {
        world.write_message(MouseButtonInput {
            window,
            button: MouseButton::Left,
            state: ButtonState::Pressed,
        });
    } else if capture.focus_phase == 3 {
        world.write_message(MouseButtonInput {
            window,
            button: MouseButton::Left,
            state: ButtonState::Released,
        });
    }
    capture.focus_phase += 1;
}

pub(super) fn measure(world: &mut World) {
    if env::var_os("FFONE_PERF_GAMEPAD").is_some() { return; }
    if env::var_os("FFONE_PERF_TUTORIAL_FINALE").is_some() { return; }
    if env::var_os("FFONE_PERF_AUDIO").is_some() { return; }
    let mut capture = world.remove_resource::<Capture>().unwrap();
    if capture.restart {
        // Exercise the same slice deletion and state transition as disconnect,
        // including production OnExit/OnEnter systems and streamed-tile teardown.
        let entities: Vec<_> = world
            .query_filtered::<Entity, With<WorldSliceEntity>>()
            .iter(world)
            .collect();
        for entity in entities {
            world.despawn(entity);
        }
        world.resource_mut::<RuntimeStatus>().clear_world();
        world
            .resource_mut::<NetworkEntityLifecycleIngress0104>()
            .disconnect(ffone_client::entity_lifecycle::NetworkSessionEpoch0104(
                capture.entry as u64,
            ));
        world
            .resource_mut::<NextState<ClientState>>()
            .set(ClientState::CharacterSelect);
        capture.restart = false;
        capture.selection_since = Some(Instant::now());
    }
    if let Some(since) = capture.selection_since {
        if since.elapsed() >= Duration::from_millis(if env::var_os("FFONE_PERF_RESOLUTION").is_some() {2500} else {500})
            && *world.resource::<State<ClientState>>().get() == ClientState::CharacterSelect
            && world.get_resource::<resolution_regression::SelectionVerified>().is_none_or(|verified|verified.0)
        {
            capture.entry += 1;
            capture.output = capture.output_root.join(format!("entry-{}", capture.entry));
            fs::create_dir_all(&capture.output).unwrap();
            capture.selection_since = None;
            capture.started = Instant::now();
            capture.previous = capture.started;
            capture.ready = None;
            capture.samples.clear();
            capture.captured = false;
            world.insert_resource(capture);
            use bevy::ecs::system::RunSystemOnce;
            world
                .run_system_once(setup)
                .expect("repeat production world entry");
        } else {
            world.insert_resource(capture);
        }
        return;
    }
    let now = Instant::now();
    let ms = now.duration_since(capture.previous).as_secs_f64() * 1000.0;
    capture.previous = now;
    let transport_ready = if capture.transport_npc_type.is_some() {
        let phase = format!("{:?}", world.resource::<TransportationModel>().phase());
        let assets = format!(
            "{:?}",
            world.resource::<TransportationPresentationAssetStatus>()
        );
        let slots: Vec<_> = world.query_filtered::<(&Node, &ImageNode), With<TransportationPresentationNpcCameraSlot>>()
            .iter(world).map(|(node, image)| serde_json::json!({"display":format!("{:?}",node.display),"bound":image.image != Handle::default()})).collect();
        if !capture.transport_diagnostics_written
            && capture.started.elapsed() > Duration::from_secs(30)
        {
            fs::write(capture.output.join("transport-state.json"), serde_json::to_vec_pretty(&serde_json::json!({"phase":phase,"assets":assets,"slots":slots,"opened":capture.transport_opened})).unwrap()).unwrap();
            capture.transport_diagnostics_written = true;
        }
        phase == "Browsing"
            && assets == "Ready"
            && slots
                .iter()
                .any(|slot| slot["bound"] == true && slot["display"] == "Flex")
    } else {
        true
    };
    let loading = world.resource::<GameplayLoadingState>();
    assert!(
        loading.blocked.is_none(),
        "capture blocked: {:?}",
        loading.blocked
    );
    let streaming = world.resource::<NativeWorldStreamingStatus>().clone();
    assert!(
        streaming.blocker.is_none(),
        "streaming blocked: {:?}",
        streaming.blocker
    );
    assert!(
        capture.started.elapsed() < Duration::from_secs(240),
        "capture timeout: loading={loading:?}, streaming={streaming:?}"
    );
    let ready = !loading.visible
        && transport_ready
        && streaming.resident_tiles > 0
        && streaming.resident_tiles == streaming.target_tiles
        && streaming.loading_colliders == 0
        && world
            .query_filtered::<&NativeWorldPresentationStatus, With<NativeWorldSceneRoot>>()
            .iter(world)
            .all(|s| *s == NativeWorldPresentationStatus::Ready);
    if !ready {
        capture.ready = None;
        capture.samples.clear();
    }
    if ready && !capture.captured {
        if capture.focus_probe && capture.focus_phase < 4 {
            drive_focus_probe(world, &mut capture);
        }
        if capture.npc_chat_probe && !capture.npc_chat_requested {
            let mut candidates = world
                .query::<(Entity, &NetworkNpcAppearance0104)>()
                .iter(world)
                .map(|(owner, npc)| (owner, npc.0.npc_type))
                .collect::<Vec<_>>();
            candidates.sort_by_key(|(_, npc_type)| *npc_type);
            let selected = candidates
                .into_iter()
                .find_map(|(owner, npc_type)| {
                    let npc = world
                        .resource::<TutorialMissionContent>()
                        .gameplay_npc(npc_type)?;
                    (npc.team == 1 && !npc.greeting.is_empty() && npc.greeting != " ")
                        .then(|| (owner, npc.clone()))
                })
                .expect("NPC chat probe requires a loaded friendly NPC with a greeting");
            world
                .resource_mut::<NpcBarkerBubbleRuntime>()
                .request_greeting(selected.0, &selected.1);
            capture.npc_chat_requested = true;
        }
        let ready_at = *capture.ready.get_or_insert(now);
        if now.duration_since(ready_at) >= Duration::from_secs(5) {
            capture.samples.push(ms);
        }
        if capture.samples.len() == 600 {
            vendor_regression::assert_complete(world);
            enchant_regression::assert_complete(world);
            quick_chat::assert_complete(world);
            email_regression::assert_complete(world);
            if let Some(fixture) = world.get_resource::<VehicleFixture>() {
                let item = fixture.0;
                let state = world.resource::<ffone_client::tutorial_player_rig_runtime::PersonalVehiclePresentation>();
                assert_eq!(state.item_id, Some(item));
                let name = format!("Personal vehicle {item}");
                assert_eq!(
                    world
                        .query::<&Name>()
                        .iter(world)
                        .filter(|n| n.as_str() == name)
                        .count(),
                    1
                );
                let trails = world
                    .query::<&Name>()
                    .iter(world)
                    .filter(|n| n.as_str() == "Vehicle exhaust trail")
                    .count();
                assert!(trails > 0, "vehicle fixture must attach its exhaust trails");
                let animations: Vec<_> = world
                    .query::<&TutorialPlayerAnimationApplied>()
                    .iter(world)
                    .map(|a| a.clip.name())
                    .collect();
                assert!(
                    animations
                        .iter()
                        .any(|name| name.starts_with("board_") || name.starts_with("scooter_"))
                );
                fs::write(capture.output.join("vehicle.json"), serde_json::to_vec_pretty(&serde_json::json!({
                    "itemId": item, "attachments": 1, "trails": trails, "animations": animations,
                    "authority": "offline fixture", "networkTested": false,
                })).unwrap()).unwrap();
            }
            if env::var_os("FFONE_PERF_RETROBUTION_MAP").is_some() {
                let map = world.resource::<WorldMapPresentation>();
                let status =
                    world.resource::<ffone_client::world_map::WorldMapPresentationAssetStatus>();
                fs::write(capture.output.join("map-state.json"), serde_json::to_vec_pretty(&serde_json::json!({
                    "phase": format!("{:?}", map.model.phase()), "validation": format!("{:?}", map.validate()),
                    "assets": format!("{status:?}"), "markers": map.markers.len(),
                    "transport": map.transport.len(), "hover": format!("{:?}", map.hover),
                    "runtimeMessage": world.resource::<RuntimeStatus>().message,
                })).unwrap()).unwrap();
                assert_eq!(
                    map.model.phase(),
                    WorldMapPhase::Open,
                    "map fixture must remain open"
                );
                assert!(
                    map.validate().is_ok(),
                    "map fixture has invalid presentation: {:?}",
                    map.validate()
                );
                assert!(matches!(
                    status,
                    ffone_client::world_map::WorldMapPresentationAssetStatus::Ready
                ));
            }
            if capture.focus_probe {
                assert_eq!(capture.focus_phase, 4, "focus probe did not complete");
            }
            if capture.npc_chat_probe {
                let lines = &world.resource::<RuntimeStatus>().chat.world_chat_lines;
                let npc_lines = lines
                    .iter()
                    .filter(|line| line.kind == ChatLineKind::Npc)
                    .map(|line| &line.text)
                    .collect::<Vec<_>>();
                assert!(
                    !npc_lines.is_empty(),
                    "production NPC speech did not reach retained chat"
                );
                fs::write(
                    capture.output.join("npc-chat.json"),
                    serde_json::to_vec_pretty(&npc_lines).unwrap(),
                )
                .unwrap();
            }
            if env::var_os("FFONE_PERF_VENDOR_PORTRAIT").is_some() {
                let diagnostics = service_portrait::vendor_capture_diagnostics(world);
                fs::write(
                    capture.output.join("vendor-portrait.json"),
                    serde_json::to_vec_pretty(&diagnostics).unwrap(),
                )
                .unwrap();
            }
            let mut sorted = capture.samples.clone();
            sorted.sort_by(f64::total_cmp);
            let placements = world
                .query::<&SpawnedNativeWorldVisual>()
                .iter(world)
                .count();
            let visible = world
                .query::<(&Mesh3d, &ViewVisibility)>()
                .iter(world)
                .filter(|(_, v)| v.get())
                .count();
            let meshes = world.query::<&Mesh3d>().iter(world).count();
            // Exclude resource storage entities to retain the pre-0.19 metric.
            let entities = world.entity_count() - world.resource_entities().iter().count() as u32;
            let cameras = world.query::<&Camera>().iter(world).count();
            let world_roots = world.query::<&NativeWorldSceneRoot>().iter(world).count();
            let ui_nodes = world.query::<&Node>().iter(world).count();
            let ui_roots = world
                .query_filtered::<Entity, (With<Node>, Without<ChildOf>)>()
                .iter(world)
                .count();
            let player_position = world
                .query_filtered::<&Transform, With<LocalPlayer>>()
                .single(world)
                .ok()
                .map(|t| t.translation.to_array());
            let camera_pose = world
                .query_filtered::<&Transform, With<LegacyOrbitCamera>>()
                .single(world)
                .ok()
                .map(|t| {
                    serde_json::json!({
                        "translation": t.translation.to_array(), "rotation": t.rotation.to_array()
                    })
                });
            let images = world.resource::<Assets<Image>>();
            let image_count = images.len();
            let image_bytes: usize = images
                .iter()
                .filter_map(|(_, i)| i.data.as_ref().map(Vec::len))
                .sum();
            let adapter = world
                .get_resource::<RenderAdapterInfo>()
                .map(|a| format!("{:?}", a.0));
            let viewport = world
                .query_filtered::<&Window, With<PrimaryWindow>>()
                .iter(world)
                .next()
                .map(|w| [w.physical_width(), w.physical_height()]);
            let material_count = world.resource::<Assets<LegacyModelMaterial>>().len();
            let render_diagnostics: Vec<_> = world.resource::<bevy::diagnostic::DiagnosticsStore>().iter()
                .map(|d| serde_json::json!({"path":d.path().to_string(),"average":d.average(),"unit":d.suffix}))
                .collect();
            let npcs = world
                .query::<&NetworkNpcAppearance0104>()
                .iter(world)
                .count();
            let report = serde_json::json!({"schema":"ffone.production-performance.v1", "position":capture.position.to_array(),
                "frozen":capture.frozen,"orbit":capture.orbit,"adapter":adapter,"frames":sorted.len(),
                "entry":capture.entry,"entities":entities,"cameras":cameras,"worldRoots":world_roots,
                "uiNodes":ui_nodes,"uiRoots":ui_roots,
                "playerPosition":player_position,"cameraPose":camera_pose,
                "viewport":viewport,"far":EXTENDED_WORLD_CAMERA_FAR_NATIVE,"nativeMaterials":material_count,"npcs":npcs,
                "meanMs":sorted.iter().sum::<f64>()/sorted.len() as f64,"p50Ms":sorted[300],"p95Ms":sorted[570],"p99Ms":sorted[594],
                "readySeconds":ready_at.duration_since(capture.started).as_secs_f64(),"placements":placements,
                "meshEntities":meshes,"visibleMeshes":visible,"residentImages":image_count,"imageCpuBytes":image_bytes,
                "sharing":static_asset_sharing_statistics(world),"renderDiagnostics":render_diagnostics,"frameMs":capture.samples});
            fs::write(
                capture.output.join("report.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            println!(
                "PERFORMANCE mean={} p95={} visible={visible}",
                report["meanMs"], report["p95Ms"]
            );
            world.spawn(Screenshot::primary_window()).observe(save);
            capture.captured = true;
        }
    }
    world.insert_resource(capture);
}

// Uses the real streamed HNPC and production service camera; no transaction is emitted.
pub(super) fn open_vendor_portrait_fixture(
    npcs: Query<&NetworkNpcAppearance0104, With<NetworkHnpcVisual0104>>,
    mut state: ResMut<VendorUiState>,
    mut projection: ResMut<VendorModeProjection0104>,
    mut announced: Local<bool>,
) {
    let Some(npc) = npcs.iter().min_by_key(|npc| npc.0.npc_id) else {
        return;
    };
    state.phase = VendorLifecyclePhase::Visible;
    state.opening_elapsed_seconds = 1.;
    projection.session.requested_npc_id = npc.0.npc_id;
    if !*announced {
        eprintln!(
            "VENDOR PORTRAIT fixture NPC {} table {}",
            npc.0.npc_id, npc.0.npc_type
        );
        *announced = true;
    }
}

// Offline acceptance input for the production map projection and presentation.
pub(super) fn open_retrobution_map_fixture(
    state: Res<State<ClientState>>,
    client_npcs: Res<ClientNpcWaypointCatalog>,
    players: Query<
        (&Transform, &LegacyPlayerController),
        (With<LocalPlayer>, Without<LegacyWorldColliderPending>),
    >,
    mut presentation: ResMut<WorldMapPresentation>,
    mut opened: Local<bool>,
    mut capture: ResMut<Capture>,
) {
    // The offline bridge rejects periodic gameplay traffic and resets modal
    // owners. Reapply this fixture's requested open state after that boundary;
    // keep the production reset behavior and never emit a map request packet.
    if *state.get() != ClientState::World
        || (!*opened && capture.ready.is_none())
        || presentation.model.phase() != WorldMapPhase::Closed
    {
        return;
    }
    let Ok((transform, controller)) = players.single() else {
        return;
    };
    let player = world_map_player_from_native(transform, controller);
    presentation
        .model
        .try_open(WorldMapOpenContext::gameplay(player))
        .unwrap();
    while presentation.model.pop_outbox().is_some() {}
    let types = world_map_npc_sources(&client_npcs)
        .iter()
        .map(|npc| npc.npc_type)
        .collect::<BTreeSet<_>>();
    presentation
        .model
        .apply_present_npc_types(true, 1, &types.into_iter().collect::<Vec<_>>())
        .unwrap();
    if !*opened {
        presentation
            .model
            .preferences
            .add_waypoint(player.position.x, player.position.z);
        presentation
            .model
            .preferences
            .add_waypoint(player.position.x + 130.0, player.position.z + 180.0);
        presentation
            .model
            .preferences
            .add_waypoint(player.position.x - 260.0, player.position.z + 200.0);
        *opened = true;
        capture.ready = None;
        capture.samples.clear();
    }
}
