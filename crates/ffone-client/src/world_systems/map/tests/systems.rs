use super::*;

#[test]
fn open_classifies_zone_queues_last_sync_and_instance_no_map_has_only_callback_exit() {
    let mut model = WorldMapModel::default();
    model.apply_present_npc_types(true, 987, &[10, 20]).unwrap();
    assert_eq!(
        model.try_open(WorldMapOpenContext::gameplay(player(6000.0, 20.0, 1000.0,))),
        Ok(WorldMapOpenDisposition::Open)
    );
    assert_eq!(model.zone(), WorldMapZone::Future);
    assert_eq!(model.zoom(), WorldMapZoom::Type3);
    assert_eq!(
        model.pop_outbox(),
        Some(WorldMapOutboxEvent::RequestPresentNpcTypes {
            last_sync_time: 987
        })
    );
    assert!(model.try_close(WorldMapCloseInput::MapKey22, WorldMapInputGates::default()));
    assert_eq!(model.pop_outbox(), Some(WorldMapOutboxEvent::ExitMode));

    assert_eq!(
        model.try_open(WorldMapOpenContext {
            player: Some(player(4096.0, 0.0, 4096.0)),
            instance_map: true,
            episode_id: 0,
            tutorial_locked: false,
            tutorial_active: false,
        }),
        Ok(WorldMapOpenDisposition::InstanceNoMap)
    );
    model.pop_outbox();
    assert!(!model.try_close(WorldMapCloseInput::MapKey22, WorldMapInputGates::default()));
    assert!(model.try_close(
        WorldMapCloseInput::InstanceNoMapAcknowledged,
        WorldMapInputGates {
            system_popup_open: true,
            escape_close_allowed: false,
        }
    ));
}
