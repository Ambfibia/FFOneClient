use super::*;

#[test]
fn confirmed_skip_uses_the_same_completion_path_as_finishing_the_tutorial() {
    let content = runtime_test_mission_content();
    let exit = content.warp(2696).unwrap();
    let warp = WarpUiEntry {
        npc_id: 6000,
        npc_type: 2696,
        warp_id: exit.provenance.warp_id,
        required_task_id: exit.required_task_id,
        target: exit.target,
        label: "EXIT".to_owned(),
    };
    let character = CharacterSummary {
        slot: 1,
        level: 1,
        pc_uid: 8,
        first_name: "Test".to_owned(),
        last_name: "Ser".to_owned(),
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
            appearance_flag: 1,
            tutorial_flag: 0,
            payzone_flag: 0,
        },
        equipment: [ffone_protocol::EquippedItem0104::default();
            ffone_protocol::CHARACTER_EQUIP_SLOT_COUNT_0104],
    };

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_state::<ClientState>()
        .insert_resource(NetworkBridge::start())
        .insert_resource(content)
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<MissionUiModel>()
        .init_resource::<TutorialSession>()
        .init_resource::<TutorialMissionRuntime>()
        .init_resource::<TutorialActorCommandQueue>()
        .init_resource::<TutorialAmbientRuntime>()
        .init_resource::<RuntimeStatus>()
        .init_resource::<TutorialLogicRuntime>()
        .add_systems(
            Update,
            (
                consume_tutorial_mission_outbox,
                sync_tutorial_mission_interaction,
            )
                .chain(),
        );
    app.world_mut().resource_mut::<TutorialSession>().character = Some(character);
    {
        let mut runtime = app.world_mut().resource_mut::<RuntimeStatus>();
        runtime.roster.selected_uid = Some(8);
        runtime.map_number = Some(0);
    }
    {
        let mut queued = GameplayUiOutbox::default();
        let mut model = app.world_mut().resource_mut::<MissionUiModel>();
        assert!(model.open_tutorial_exit_dialog());
        assert!(model.confirm_tutorial_exit_dialog(&mut queued));
        model.pending_warp = Some(warp.clone());
        queued.push(GameplayUiAction::NpcWarp {
            npc_id: warp.npc_id,
            npc_type: warp.npc_type,
            warp_id: warp.warp_id,
            required_task_id: warp.required_task_id,
            target: warp.target,
        });
        drop(model);
        *app.world_mut().resource_mut::<GameplayUiOutbox>() = queued;
    }
    app.world_mut().spawn(TutorialActor {
        id: warp.npc_id,
        npc_type: warp.npc_type,
        team: 1,
        hp: 100,
        max_hp: 100,
        damaged: false,
        interacting: true,
        invulnerable: false,
    });
    let initial_position = Vec3::new(1.0, 2.0, 3.0);
    let player = app
        .world_mut()
        .spawn((
            LocalPlayer,
            WorldSliceEntity,
            Transform::from_translation(initial_position),
            LegacyPlayerController::from_baseline_table(),
        ))
        .id();

    app.update();

    let tutorial = app.world().resource::<TutorialSession>();
    assert!(
        tutorial.completion_requested,
        "confirmed skip must request SAVE_CHAR_TUTOR followed by CHAR_SELECT"
    );
    assert_eq!(
        tutorial.character().map(|character| character.pc_uid),
        Some(8)
    );
    assert!(matches!(
        app.world().resource::<NextState<ClientState>>(),
        NextState::Unchanged
    ));
    assert_eq!(
        app.world().resource::<RuntimeStatus>().roster.selected_uid,
        Some(8)
    );
    assert!(!app.world().resource::<MissionUiModel>().enabled);
    assert!(
        app.world()
            .resource::<TutorialMissionRuntime>()
            .completed_warp_ids
            .is_empty()
    );
    assert!(app.world().get_entity(player).is_ok());
    assert_eq!(
        app.world().resource::<RuntimeStatus>().message,
        "Saving tutorial completion and entering the OpenFusion world..."
    );
}

#[test]
fn dexter_ship_cutscenes_resolve_their_recovered_effects_through_the_catalog() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let catalog = NativeAudioCatalog::open(&asset_root, false).unwrap();
    let mut sfx_count = 0;

    for true_name in std::iter::once("CharacterCreation_Loop").chain(
        DEXTER_SHIP_NAME_AUDIO
            .iter()
            .chain(DEXTER_SHIP_TUTORIAL_AUDIO)
            .filter(|cue| !cue.true_name.is_empty())
            .map(|cue| cue.true_name),
    ) {
        let (path, category) = dexter_ship_audio_path(&catalog, "en", true_name)
            .unwrap_or_else(|| panic!("missing Dexter ship cue {true_name:?}"));
        assert!(
            asset_root.join(&path).is_file(),
            "Dexter ship cue {true_name:?} resolved to missing {path}"
        );
        sfx_count += usize::from(category == NativeAudioCategory::Sfx);
    }

    assert!(
        sfx_count >= 10,
        "the ship scenes must retain their recovered SFX"
    );
}

#[test]
fn tutorial_initial_actor_startup_uses_the_shared_production_visual_route() {
    let asset_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(&asset_root).unwrap();
    let catalog = NetworkNpcVisualCatalog0104::open(&locator).unwrap();
    let initial_npc = TUTORIAL_INITIALIZATION.initial_npc;
    let model_path = tutorial_initial_actor_model_path(&catalog).unwrap();

    assert_eq!((initial_npc.runtime_id, initial_npc.npc_type), (2, 2968));
    assert_eq!(model_path, "characters/npcs/npc_building/npc_building.glb");
    assert!(
        asset_root.join(model_path).is_file(),
        "shared tutorial actor route is missing: {model_path}"
    );
}
