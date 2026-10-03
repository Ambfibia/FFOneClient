use super::*;
use crate::tutorial_mission_content::TutorialMissionContent;
use ffone_protocol::Nano0104;
use std::array;

fn fixture() -> (UserEquipUiState, UserEquipNanoModeProjection) {
    let mut state = UserEquipUiState::default();
    state.open_nano_station(42);
    state.tick(USER_EQUIP_OPEN_SECONDS);
    let projection = UserEquipNanoModeProjection {
        gallery: vec![UserEquipNanoGalleryEntryProjection {
            nano_id: 50,
            name: "Fixture".into(),
            attribute: String::new(),
            description: String::new(),
            skills: Default::default(),
            sort_number: 50,
            column: 0,
            row: 0,
            owned: true,
            equipped: false,
            current_power: Some(258),
            icon: UserEquipPresentationIcon::Empty,
            viewer_icon: UserEquipPresentationIcon::Empty,
        }],
        status: Default::default(),
    };
    (state, projection)
}

#[test]
fn station_rejects_book_unowned_active_and_invalid_targets() {
    let (mut state, mut projection) = fixture();
    let modal = UserEquipModalState::default();
    let action = UserEquipNanoStationAction::Equip {
        nano_id: 50,
        slot: 1,
    };
    assert!(nano_station_action_allowed(
        &state,
        modal,
        &projection,
        action
    ));
    projection.status[1].active = true;
    assert!(!nano_station_action_allowed(
        &state,
        modal,
        &projection,
        action
    ));
    projection.status[1].active = false;
    projection.gallery[0].owned = false;
    assert!(!nano_station_action_allowed(
        &state,
        modal,
        &projection,
        action
    ));
    projection.gallery[0].owned = true;
    assert!(!nano_station_action_allowed(
        &state,
        modal,
        &projection,
        UserEquipNanoStationAction::Equip {
            nano_id: 50,
            slot: 3
        }
    ));
    state.open_item_mode();
    state.select_nano_tab();
    state.tick(USER_EQUIP_OPEN_SECONDS);
    assert!(!nano_station_action_allowed(
        &state,
        modal,
        &projection,
        action
    ));
}

#[test]
fn station_pending_reply_locks_input_and_only_matching_reply_unlocks() {
    let (mut state, projection) = fixture();
    let action = UserEquipNanoStationAction::Equip {
        nano_id: 50,
        slot: 2,
    };
    state.begin_nano_station_request(action);
    assert!(!nano_station_action_allowed(
        &state,
        Default::default(),
        &projection,
        action
    ));
    assert_eq!(
        state.close_disposition(UserEquipCloseSource::Escape, Default::default()),
        UserEquipCloseDisposition::Blocked(UserEquipCloseBlockedReason::SendPending)
    );
    state.acknowledge_nano_station_request(UserEquipNanoStationAction::Equip {
        nano_id: 1,
        slot: 2,
    });
    assert!(state.nano_station_send_pending());
    state.acknowledge_nano_station_request(action);
    assert!(!state.nano_station_send_pending());
    state.begin_nano_station_request(action);
    state.tick(10.0);
    assert!(!state.nano_station_send_pending());
    assert_eq!(projection.status[2].nano_id, None);
}

#[test]
fn station_unequip_uses_actual_slot_and_allows_active_nano() {
    let (state, mut projection) = fixture();
    projection.gallery[0].equipped = true;
    projection.status[2].nano_id = Some(50);
    projection.status[2].active = true;
    let action = action_for(3, &projection.gallery[0], &projection).unwrap();
    assert_eq!(
        action,
        UserEquipNanoStationAction::Unequip {
            nano_id: 50,
            slot: 2
        }
    );
    assert!(nano_station_action_allowed(
        &state,
        Default::default(),
        &projection,
        action
    ));
    assert!(action_for(0, &projection.gallery[0], &projection).is_none());
}

#[test]
fn station_entities_keep_absolute_rects_and_hide_inapplicable_controls() {
    let (state, projection) = fixture();
    let mut app = App::new();
    app.insert_resource(state)
        .insert_resource(projection)
        .insert_resource(UserEquipUiRuntimeAssets(UserEquipUiAssets {
            images: array::from_fn(|_| Handle::default()),
            font: Handle::default(),
            missing_checker: Handle::default(),
        }))
        .init_resource::<UserEquipModalState>()
        .init_resource::<UserEquipNanoViewerState>()
        .add_systems(
            Startup,
            |mut commands: Commands, assets: Res<UserEquipUiRuntimeAssets>| {
                commands
                    .spawn(Node::default())
                    .with_children(|parent| spawn_controls(parent, &assets.0));
            },
        )
        .add_systems(Update, bind_controls);
    app.world_mut()
        .resource_mut::<UserEquipNanoViewerState>()
        .open(0);
    app.update();
    let mut query = app
        .world_mut()
        .query::<(&StationControl, &Node, &ImageNode)>();
    assert_eq!(query.iter(app.world()).count(), 4);
    for (control, node, image) in query.iter(app.world()) {
        assert_eq!(node.position_type, PositionType::Absolute);
        if control.0 < 3 {
            assert_eq!(node.display, Display::Flex);
            assert_eq!(node.left, px(25.0 + control.0 as f32 * 105.0));
            assert_eq!(node.top, px(574.0));
            assert!(matches!(image.image_mode, NodeImageMode::Sliced(_)));
        } else {
            assert_eq!(node.display, Display::None);
        }
    }
    {
        let mut projection = app
            .world_mut()
            .resource_mut::<UserEquipNanoModeProjection>();
        projection.gallery[0].equipped = true;
        projection.status[2].nano_id = Some(50);
    }
    app.update();
    for (control, node, _) in query.iter(app.world()) {
        if control.0 == 3 {
            assert_eq!(node.display, Display::Flex);
            assert_eq!(node.left, px(222.5));
            assert_eq!(node.top, px(592.0));
        } else {
            assert_eq!(node.display, Display::None);
        }
    }
    app.world_mut()
        .resource_mut::<UserEquipUiState>()
        .open_item_mode();
    app.update();
    assert!(
        query
            .iter(app.world())
            .all(|(_, node, _)| node.display == Display::None)
    );
}

#[test]
fn station_pointer_emits_request_without_changing_authority() {
    let (state, projection) = fixture();
    let mut app = App::new();
    app.insert_resource(state)
        .insert_resource(projection.clone())
        .init_resource::<UserEquipModalState>()
        .init_resource::<UserEquipNanoViewerState>()
        .init_resource::<UserEquipUiOutbox>()
        .init_resource::<UserEquipUiAudioOutbox>()
        .add_systems(Update, collect_actions);
    app.world_mut()
        .resource_mut::<UserEquipNanoViewerState>()
        .open(0);
    app.world_mut()
        .spawn((Interaction::Pressed, StationControl(2)));
    app.update();
    assert_eq!(
        app.world_mut()
            .resource_mut::<UserEquipUiOutbox>()
            .drain()
            .collect::<Vec<_>>(),
        vec![UserEquipUiAction::NanoStation(
            UserEquipNanoStationAction::Equip {
                nano_id: 50,
                slot: 2
            }
        )]
    );
    assert_eq!(
        app.world().resource::<UserEquipNanoModeProjection>(),
        &projection
    );
    app.update();
    assert!(app.world().resource::<UserEquipUiOutbox>().is_empty());
}

#[test]
fn every_added_nano_uses_its_owned_identity_for_station_slot_replacement() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let assets = crate::assets::AssetLocator::open(root).unwrap();
    let content = TutorialMissionContent::open(&assets).unwrap();
    let mut bank: Vec<_> = content
        .gameplay_nanos()
        .map(|nano| Nano0104 {
            id: nano.nano_id,
            skill_id: content
                .journal_nano(i32::from(nano.nano_id))
                .unwrap()
                .skills[0]
                .skill_id as i16,
            stamina: 150,
        })
        .collect();
    bank.reverse();
    let equipped = [UserEquipNanoEquippedAuthority {
        nano_id: Some(1),
        skill_id: 1,
        stamina: 150,
        active: false,
    }; 3];
    let projection = UserEquipNanoModeProjection::from_authoritative(&bank, equipped, &content);
    let mut state = UserEquipUiState::default();
    state.open_nano_station(42);
    state.tick(USER_EQUIP_OPEN_SECONDS);
    for entry in projection
        .gallery
        .iter()
        .filter(|entry| entry.nano_id >= 48)
    {
        assert!(entry.owned, "Nano {}", entry.nano_id);
        for slot in 0..3 {
            let action = action_for(slot, entry, &projection).unwrap();
            assert_eq!(
                action,
                UserEquipNanoStationAction::Equip {
                    nano_id: entry.nano_id,
                    slot
                }
            );
            assert!(
                nano_station_action_allowed(
                    &state,
                    UserEquipModalState::default(),
                    &projection,
                    action
                ),
                "Nano {} slot {slot}",
                entry.nano_id
            );
        }
    }
}
