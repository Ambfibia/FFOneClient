//! Opt-in Bug 46 replay in the full offline client, using native quest data.
use super::*;

#[derive(Default, Resource)]
struct Probe {
    frame: u32,
    chosen: Option<i32>,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<Probe>()
        .add_systems(Update, discard_offline_events.before(poll_network))
        .add_systems(Last, drive.before(measure));
}

fn discard_offline_events(bridge: Res<NetworkBridge>) {
    let _ = bridge.drain();
}

fn start_task(world: &mut World, task_id: i32) {
    world.resource_scope(|world, mut runtime: Mut<WorldMissionRuntime>| {
        runtime
            .apply_event(
                &WorldMissionServerEvent0104::TaskStartSuccess(
                    ffone_protocol::PcTaskStartSuccess0104 {
                        task_id,
                        remaining_time: 0,
                    },
                ),
                world.resource::<TutorialMissionContent>(),
            )
            .unwrap();
    });
}

fn drive(world: &mut World) {
    if world.resource::<GameplayLoadingState>().visible
        || world.resource::<Capture>().samples.is_empty()
    {
        return;
    }
    let frame = {
        let mut probe = world.resource_mut::<Probe>();
        probe.frame += 1;
        probe.frame
    };
    match frame {
        1 => {
            // World mission binding needs the inventory normally seeded by login.
            let bytes = vec![0; ffone_protocol::PcLoadData0104::SIZE];
            world.resource_mut::<LocalInventoryRuntime>().seed(
                1, &ffone_protocol::PcLoadData0104::decode(&bytes).unwrap());
        }
        5 => {
            start_task(world, 451);
            start_task(world, 2248);
        }
        45 | 165 => {
            world.resource_scope(|world, mut model: Mut<MissionUiModel>| {
                assert!(
                    model.open_journal_from_shortcut(&mut world.resource_mut::<GameplayUiOutbox>())
                );
            });
        }
        85 => {
            let chosen = {
                let mut model = world.resource_mut::<MissionUiModel>();
                assert_eq!(model.nanocom_journal.active_missions.len(), 2);
                let chosen = model.nanocom_journal.active_missions[1].task_id;
                // Categories can sort the visible rows independently of server order.
                for row in 0..2 {
                    assert!(model.select_journal_mission(row));
                    if model.viewed_journal_task_id == Some(chosen) {
                        break;
                    }
                }
                assert_eq!(model.viewed_journal_task_id, Some(chosen));
                model.selected_journal_task_id = Some(chosen);
                chosen
            };
            world.resource_mut::<Probe>().chosen = Some(chosen);
        }
        125 => {
            world.resource_scope(|world, mut model: Mut<MissionUiModel>| {
                assert!(model.close_journal(&mut world.resource_mut::<GameplayUiOutbox>()));
            });
        }
        205 => {
            let model = world.resource::<MissionUiModel>();
            assert_eq!(
                model.viewed_journal_task_id,
                world.resource::<Probe>().chosen
            );
            assert_eq!(
                model.selected_journal_task_id,
                world.resource::<Probe>().chosen
            );
            start_task(world, 451); // Authoritative list refresh/reorder.
        }
        245 => {
            let model = world.resource::<MissionUiModel>();
            assert_eq!(
                model.viewed_journal_task_id,
                world.resource::<Probe>().chosen
            );
            assert_eq!(
                model.selected_journal_task_id,
                world.resource::<Probe>().chosen
            );
            eprintln!(
                "Bug 46 full-client replay passed: tasks 451/2248, chosen {:?}; reopen and refresh",
                world.resource::<Probe>().chosen
            );
        }
        _ => {}
    }
}
