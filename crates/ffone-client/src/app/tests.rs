use crate::app::*;
use ffone_client::combi_runtime::CombiPlayerAuthority0104;
use ffone_client::combi_ui::COMBI_RECIPE_TABLE_PATH;
use ffone_client::combi_ui::CombiModalChoice0104;
use ffone_client::enchant_runtime::EnchantModalChoice0104;
use ffone_client::enchant_runtime::EnchantOperation0104;
use ffone_client::enchant_runtime::EnchantPlayerAuthority0104;
use ffone_client::enchant_ui::ENCHANT_WEAPON_MATERIAL_ID_0104;
use ffone_client::enchant_ui::EnchantItemPresentation0104;
use ffone_client::enchant_ui::EnchantUiCommand0104;
use ffone_client::guide_ui::GuideMentor;
use ffone_client::guide_ui::GuideUiCommand;
use ffone_client::guide_ui::apply_guide_ui_command;
use ffone_client::movement::LegacyInputGate;

mod character;
mod economy;
mod nano_free_tuning;
mod social;
mod tutorial;
mod tutorial_nano;
mod tutorial_warp;
mod warp_departure;
mod world;

fn entry_test_character(pc_uid: i64, appearance_flag: i8, tutorial_flag: i8) -> CharacterSummary {
    CharacterSummary {
        slot: 1,
        level: 1,
        pc_uid,
        first_name: "Test".to_owned(),
        last_name: "Entry".to_owned(),
        position: [0; 3],
        style: ffone_protocol::CharacterStyle0104 {
            name_check: 1,
            gender: 1,
            face_style: 0,
            hair_style: 0,
            hair_color: 0,
            skin_color: 0,
            eye_color: 0,
            height: 0,
            body: 0,
            class: 0,
            appearance_flag,
            tutorial_flag,
            payzone_flag: 0,
        },
        equipment: [ffone_protocol::EquippedItem0104::default();
            ffone_protocol::CHARACTER_EQUIP_SLOT_COUNT_0104],
    }
}

fn launcher_test_trigger() -> ffone_client::launcher_ui::LauncherTriggerSpec {
    ffone_client::launcher_ui::LauncherTriggerSpec {
        trigger_position: Vec3::new(10.0, 20.0, 30.0),
        trigger_euler_degrees: Vec3::new(0.0, 180.0, 0.0),
        min_power: 10.0,
        max_power: 30.0,
        initial_rotation_degrees: Vec3::ZERO,
        maximum_rotation_degrees: Vec3::new(20.0, 45.0, 0.0),
    }
}

fn tutorial_camera_test_app(
    transform: Transform,
    camera: ffone_client::tutorial_choreography_runtime::TutorialCameraPresentation,
) -> (App, Entity) {
    let mut app = App::new();
    let target = app.world_mut().spawn_empty().id();
    let camera_entity = app
        .world_mut()
        .spawn((
            Camera3d::default(),
            LegacyOrbitCamera::new(target),
            transform,
        ))
        .id();
    app.insert_resource(Time::<()>::default());
    app.insert_resource(TutorialChoreographyPresentation {
        camera,
        ..default()
    });
    app.add_systems(Update, apply_tutorial_choreography_camera);
    (app, camera_entity)
}

fn pending_guide_change(
    previous: GuideMentor,
    selected: GuideMentor,
) -> (GuideRuntime, GuideUiModel, GuideUiOutbox) {
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    load.as_bytes_mut()[ffone_protocol::PcLoadData0104::MENTOR_OFFSET
        ..ffone_protocol::PcLoadData0104::MENTOR_OFFSET + 2]
        .copy_from_slice(&previous.wire_id().to_le_bytes());
    load.as_bytes_mut()[ffone_protocol::PcLoadData0104::MENTOR_COUNT_OFFSET
        ..ffone_protocol::PcLoadData0104::MENTOR_COUNT_OFFSET + 2]
        .copy_from_slice(&7_i16.to_le_bytes());
    let mut runtime = GuideRuntime::new(GuideServerProfile::OpenFusion0104);
    runtime.load_pc_state(&load);
    runtime.request_change(selected).unwrap();

    let mut model = GuideUiModel::default();
    let mut outbox = GuideUiOutbox::default();
    let mut audio = GuideUiAudioOutbox::default();
    model.open_change(previous);
    assert!(apply_guide_ui_command(
        &mut model,
        &mut outbox,
        &mut audio,
        GuideUiCommand::SelectMentor(selected),
    ));
    assert!(apply_guide_ui_command(
        &mut model,
        &mut outbox,
        &mut audio,
        GuideUiCommand::OpenConfirmation,
    ));
    assert!(apply_guide_ui_command(
        &mut model,
        &mut outbox,
        &mut audio,
        GuideUiCommand::ConfirmMentor,
    ));
    while outbox.pop_front().is_some() {}
    (runtime, model, outbox)
}

fn runtime_test_mission_content() -> TutorialMissionContent {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("assets")
        .join("game");
    let assets = AssetLocator::open(root).unwrap();
    TutorialMissionContent::open(&assets).unwrap()
}

fn runtime_test_combi_catalog_0104() -> CombiProductionCatalog0104 {
    static CATALOG: std::sync::OnceLock<CombiProductionCatalog0104> = std::sync::OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("assets")
                .join("game");
            let assets = AssetLocator::open(root).unwrap();
            let bytes = assets.read(COMBI_RECIPE_TABLE_PATH).unwrap();
            CombiProductionCatalog0104::from_table_set_bytes(&bytes, |_| None).unwrap()
        })
        .clone()
}

fn runtime_test_email_catalog_0104() -> EmailProductionCatalog0104 {
    static CATALOG: std::sync::OnceLock<EmailProductionCatalog0104> = std::sync::OnceLock::new();
    CATALOG
        .get_or_init(|| {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("..")
                .join("assets")
                .join("game");
            let assets = AssetLocator::open(root).unwrap();
            let content = TutorialMissionContent::open(&assets).unwrap();
            EmailProductionCatalog0104::open(&assets, &content).unwrap()
        })
        .clone()
}

fn tutorial_gate_test_native_mechanics() -> TutorialNativeMechanics {
    let mut native = TutorialNativeMechanics::default();
    native.sync_stable_stage(TutorialStage::Mission(MissionStage::ReturnToNumbuhTwo));
    native
}

fn tutorial_gate_test_actor(interacting: bool) -> TutorialActor {
    TutorialActor {
        id: 1005,
        npc_type: 2671,
        team: 1,
        hp: 100,
        max_hp: 100,
        damaged: false,
        interacting,
        invulnerable: false,
    }
}

fn tutorial_gate_test_interaction() -> NpcInteractionUi {
    NpcInteractionUi {
        npc_id: 1005,
        npc_type: 2671,
        name: "NUMBUH TWO".to_owned(),
        available_missions: Vec::new(),
        completed_missions: Vec::new(),
        services: Vec::new(),
        warp: None,
    }
}

fn begin_test_npc_interaction(mut actors: Query<&mut TutorialActor>) {
    for mut actor in &mut actors {
        actor.interacting = true;
    }
}

#[derive(Resource, Default)]
struct GateObservedAtRead(Option<LegacyInputGate>);

fn observe_test_gate_at_read(gate: Res<LegacyInputGate>, mut observed: ResMut<GateObservedAtRead>) {
    observed.0 = Some(*gate);
}

fn close_test_npc_modal(mut model: ResMut<MissionUiModel>) {
    model.npc_icon_mode_visible = false;
}

fn enchant_test_inventory_0104(entries: &[(usize, ItemBase0104)]) -> InventoryRuntime0104 {
    let mut load = ffone_protocol::PcLoadData0104::zeroed();
    for (slot, item) in entries {
        let offset = ffone_protocol::PcLoadData0104::INVENTORY_OFFSET
            + slot * ffone_protocol::ItemBase0104::SIZE;
        load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&item.item_type.to_le_bytes());
        load.as_bytes_mut()[offset + 2..offset + 4].copy_from_slice(&item.item_id.to_le_bytes());
        load.as_bytes_mut()[offset + 4..offset + 8].copy_from_slice(&item.option.to_le_bytes());
        load.as_bytes_mut()[offset + 8..offset + 12]
            .copy_from_slice(&item.time_limit.to_le_bytes());
    }
    InventoryRuntime0104::from_pc_load(77, &load)
}

fn pending_combi_test_runtime_0104(
    catalog: &CombiProductionCatalog0104,
) -> (
    CombiProductionRuntime0104,
    InventoryRuntime0104,
    ItemBase0104,
    ItemBase0104,
) {
    let style = ItemBase0104 {
        item_type: 0,
        item_id: 1,
        option: 0,
        time_limit: 13,
    };
    let stats = ItemBase0104 {
        item_type: 0,
        item_id: 3,
        option: 0,
        time_limit: 17,
    };
    let inventory = enchant_test_inventory_0104(&[(4, style), (7, stats)]);
    let player = CombiPlayerAuthority0104 {
        owner_pc_id: 77,
        gender: 1,
        level: 100,
        guide: 0,
        taros: 100_000,
    };
    let runtime_catalog = RuntimeCombiItemCatalog0104 {
        source: catalog,
        player,
    };
    let mut runtime = CombiProductionRuntime0104::default();
    runtime
        .open(
            CombiOpenContext0104::clean(9_001, 6_501, player),
            &inventory,
            &runtime_catalog,
            &catalog.recipes,
        )
        .unwrap();
    for command in [
        CombiUiCommand0104::BeginInventoryDrag { inventory_index: 4 },
        CombiUiCommand0104::DropOnSelection {
            slot: CombiSelectionSlot0104::Style,
        },
        CombiUiCommand0104::BeginInventoryDrag { inventory_index: 7 },
        CombiUiCommand0104::DropOnSelection {
            slot: CombiSelectionSlot0104::Stats,
        },
        CombiUiCommand0104::Combine,
    ] {
        runtime
            .apply_ui_command(command, &runtime_catalog, &catalog.recipes)
            .unwrap();
    }
    runtime
        .resolve_modal(
            CombiModalChoice0104::Continue,
            &runtime_catalog,
            &catalog.recipes,
        )
        .unwrap();
    let request = runtime.tick(4.01).unwrap();
    assert_eq!(
        request
            .request
            .as_ref()
            .map(|request| request.packet_type()),
        Some(ffone_client::combi_ui::COMBI_REQUEST_PACKET_ID_0104)
    );
    assert!(runtime.request_pending());
    (runtime, inventory, style, stats)
}

fn open_enchant_test_runtime_0104(
    inventory: &InventoryRuntime0104,
) -> EnchantProductionRuntime0104 {
    let mut runtime = EnchantProductionRuntime0104::default();
    runtime
        .open(
            EnchantOpenContext0104::clean(
                9_001,
                6_501,
                EnchantPlayerAuthority0104 {
                    owner_pc_id: 77,
                    taros: 10_000,
                    weapon_battery: 17,
                    nano_battery: 23,
                },
                true,
            ),
            inventory,
        )
        .unwrap();
    runtime
}

fn pending_enchant_delete_test_runtime_0104(
    item: ItemBase0104,
) -> (EnchantProductionRuntime0104, InventoryRuntime0104) {
    let inventory = enchant_test_inventory_0104(&[(4, item)]);
    let mut runtime = open_enchant_test_runtime_0104(&inventory);
    let catalog = |_: ItemBase0104| Some(EnchantItemPresentation0104::default());
    runtime
        .apply_ui_command(EnchantUiCommand0104::BeginInventoryDrag(4), &catalog)
        .unwrap();
    runtime
        .apply_ui_command(EnchantUiCommand0104::DropOnTrash, &catalog)
        .unwrap();
    let output = runtime
        .resolve_modal(EnchantModalChoice0104::Accept)
        .unwrap();
    assert_eq!(
        output.request.as_ref().map(|request| request.packet_type()),
        Some(ffone_client::enchant_ui::ENCHANT_DELETE_REQUEST_PACKET_ID_0104)
    );
    assert_eq!(
        runtime
            .session()
            .and_then(|session| session.pending_operation()),
        Some(EnchantOperation0104::Delete)
    );
    (runtime, inventory)
}

fn enchant_confirmation_test_runtime_0104() -> EnchantProductionRuntime0104 {
    let target = ItemBase0104 {
        item_type: 0,
        item_id: 501,
        option: 1,
        time_limit: 0,
    };
    let weapon_material = ItemBase0104 {
        item_type: 7,
        item_id: ENCHANT_WEAPON_MATERIAL_ID_0104,
        option: 40,
        time_limit: 0,
    };
    let inventory = enchant_test_inventory_0104(&[(7, target), (8, weapon_material)]);
    let mut runtime = open_enchant_test_runtime_0104(&inventory);
    let catalog = |_: ItemBase0104| Some(EnchantItemPresentation0104::default());
    for command in [
        EnchantUiCommand0104::BeginInventoryDrag(7),
        EnchantUiCommand0104::DropOnAttachment(EnchantAttachmentSlot0104::Target),
        EnchantUiCommand0104::BeginInventoryDrag(8),
        EnchantUiCommand0104::DropOnAttachment(EnchantAttachmentSlot0104::WeaponMaterial),
        EnchantUiCommand0104::Enchant,
    ] {
        runtime.apply_ui_command(command, &catalog).unwrap();
    }
    assert!(matches!(
        runtime.session().unwrap().model().phase(),
        EnchantPhase0104::SystemMessage { .. }
    ));
    runtime
}
