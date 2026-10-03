use super::*;
use ffone_client::mission_ui::{JournalListTab, MissionJournalUi};

fn focus_column(world: &mut World, left: f32, width: f32) -> Entity {
    let entity = world
        .query_filtered::<(Entity, &Node), With<Button>>()
        .iter(world)
        .filter(|(_, node)| {
            node.display != Display::None && node.left == px(left) && node.width == px(width)
        })
        .min_by(|(_, a), (_, b)| match (a.top, b.top) {
            (Val::Px(a), Val::Px(b)) => a.total_cmp(&b),
            _ => std::cmp::Ordering::Equal,
        })
        .expect("visible production journal control")
        .0;
    world
        .resource_mut::<super::super::super::gamepad_ui::PadUiFocus>()
        .entity = Some(entity);
    entity
}

pub(super) fn drive(world: &mut World, probe: &mut Probe) -> bool {
    if probe.frames < 8 {
        return false;
    }
    match probe.phase {
        36 => {
            world.resource_scope(|world, mut runtime: Mut<WorldMissionRuntime>| {
                runtime
                    .apply_event(
                        &WorldMissionServerEvent0104::TaskStartSuccess(
                            ffone_protocol::PcTaskStartSuccess0104 {
                                task_id: 2248,
                                remaining_time: 0,
                            },
                        ),
                        world.resource::<TutorialMissionContent>(),
                    )
                    .unwrap();
            });
        }
        37 => {
            world.resource_scope(|world, mut model: Mut<MissionUiModel>| {
                model.close_nanocom_menu(&mut world.resource_mut::<GameplayUiOutbox>());
                assert!(
                    model.open_journal_from_shortcut(&mut world.resource_mut::<GameplayUiOutbox>())
                );
            });
        }
        38 => {
            probe.first_focus = Some(focus_column(world, 614.0, 330.0));
        }
        39 => {
            assert!(
                world
                    .get::<super::super::super::gamepad_ui::PadUiHighlight>(
                        probe.first_focus.unwrap()
                    )
                    .is_some()
            );
            screenshot(world, "gamepad-journal-mission-focus.png");
            probe.button = Some(GamepadButton::RightTrigger);
        }
        40 => {
            assert_eq!(
                world.resource::<MissionUiModel>().journal_tab,
                JournalListTab::Completed
            );
            probe.button = None;
            probe.first_focus = Some(focus_column(world, 613.0, 22.0));
            assert!(
                world
                    .resource::<MissionUiModel>()
                    .completed_category_expanded[0]
            );
        }
        41 => {
            probe.button = Some(GamepadButton::South);
        }
        42 => {
            assert!(
                !world
                    .resource::<MissionUiModel>()
                    .completed_category_expanded[0],
                "A collapses the completed category"
            );
            probe.button = None;
        }
        43 => {
            probe.button = Some(GamepadButton::South);
        }
        44 => {
            assert!(
                world
                    .resource::<MissionUiModel>()
                    .completed_category_expanded[0],
                "a second A expands the same category"
            );
            screenshot(world, "gamepad-journal-category-focus.png");
            probe.button = Some(GamepadButton::LeftTrigger);
        }
        45 => {
            assert_eq!(
                world.resource::<MissionUiModel>().journal_tab,
                JournalListTab::Active
            );
            probe.button = Some(GamepadButton::East);
        }
        46 => {
            assert!(matches!(
                world.resource::<MissionUiModel>().journal,
                MissionJournalUi::Hidden
            ));
            println!(
                "BUG014 JOURNAL PASS: cyan mission focus, LB/RB Active/Completed, A collapses and reopens the same category, B closes journal"
            );
            probe.button = None;
            world
                .resource_mut::<NextState<ClientState>>()
                .set(ClientState::CharacterSelect);
            probe.phase = 23;
            return true;
        }
        _ => unreachable!(),
    }
    probe.phase += 1;
    true
}
