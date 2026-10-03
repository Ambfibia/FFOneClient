use super::components::MissionUiControl;
use super::components::MissionUiView;
use super::components::NpcActionButton;
use super::journal::JournalScrollViewport;
use super::widgets::npc_action_button;
use super::*;
use crate::assets::AssetLocator;
use crate::gameplay_ui::GameplayUiAction;
use crate::tutorial_mission_content::TutorialMissionContent;
use crate::tutorial_mission_content::TutorialWarpTarget;
use bevy::ui::FocusPolicy;
use bevy::window::PrimaryWindow;
use bevy::{
    ecs::system::RunSystemOnce,
    input::touch::Touches,
    math::Affine2,
    reflect::structs::DynamicStruct,
    ui::{ComputedUiTargetCamera, UiStack, ui_focus_system},
};

#[test]
fn warp_presentation_binds_letterbox_without_reopening_npc_windows() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let locator = AssetLocator::open(root).unwrap();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_resource::<MissionUiModel>()
        .init_resource::<GameplayMenuTransition>()
        .insert_resource(TutorialMissionContent::open(&locator).unwrap())
        .init_resource::<NpcMissionRowHeights>()
            .add_systems(Update, bind_mission_ui);
    let assets = MissionUiAssets::load(app.world().resource::<AssetServer>());
    app.insert_resource(assets);
    let bars = app
        .world_mut()
        .spawn((MissionUiView::NpcLetterboxRoot, Node::default()))
        .id();
    let utility = app
        .world_mut()
        .spawn((MissionUiView::NpcWarpRoot, Node::default()))
        .id();
    let quest = app
        .world_mut()
        .spawn((MissionUiView::NpcQuestRoot, Node::default()))
        .id();
    app.world_mut().resource_mut::<MissionUiModel>().enabled = true;
    for pending in [true, false] {
        {
            let mut model = app.world_mut().resource_mut::<MissionUiModel>();
            model.pending_warp = pending.then(WarpUiEntry::default);
            model.warp_transition_active = !pending;
            assert!(!gameplay_chrome_visible(true, Some(&model)));
            assert!(model.gameplay_input_blocked());
            assert!(model.chat_input_blocked());
            assert_eq!(model.npc_subtarget_actor_id(), None);
        }
        app.update();
        assert_eq!(
            app.world().get::<Node>(bars).unwrap().display,
            Display::Flex
        );
        assert_eq!(
            app.world().get::<Node>(utility).unwrap().display,
            Display::None
        );
        assert_eq!(
            app.world().get::<Node>(quest).unwrap().display,
            Display::None
        );
    }
    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .warp_transition_active = false;
    app.update();
    assert_eq!(
        app.world().get::<Node>(bars).unwrap().display,
        Display::None
    );
    assert!(gameplay_chrome_visible(
        true,
        Some(app.world().resource::<MissionUiModel>())
    ));
}

fn spawn_test_warp_button(mut commands: Commands, server: Res<AssetServer>) {
    let assets = MissionUiAssets::load(&server);
    commands.spawn_empty().with_children(|parent| {
        npc_action_button(
            parent,
            MissionUiRect::new(0.0, 0.0, 212.0, 48.0),
            "WARP",
            &assets.npcicon_warp,
            MissionUiControl::NpcUtility(0),
            Some(MissionUiView::NpcUtilityButton(0)),
            Some((
                MissionUiView::NpcUtilityText(0),
                MissionUiView::NpcUtilityIcon(0),
            )),
            &assets,
        );
    });
}

#[test]
fn warp_button_caption_and_icon_pass_real_mouse_presses_in_world_and_tutorial() {
    // Run Bevy's real Interaction producer, not a synthetic Interaction::Pressed.
    // Only layout is supplied here; the button and its children are production UI.
    for tutorial in [false, true] {
        for click_on_caption in [false, true] {
            let mut app = App::new();
            app.add_plugins((MinimalPlugins, AssetPlugin::default()))
                .init_asset::<Image>()
                .init_asset::<Font>()
                .init_resource::<ButtonInput<MouseButton>>()
                .init_resource::<Touches>()
                .init_resource::<TutorialNativeMechanics>()
                .init_resource::<MissionUiModel>()
                .init_resource::<GameplayUiOutbox>()
                .init_resource::<GameplayUiAudioOutbox>()
                .add_systems(Update, (ui_focus_system, handle_mission_ui_buttons).chain());
            app.world_mut()
                .run_system_once(spawn_test_warp_button)
                .unwrap();
            let camera = app.world_mut().spawn(Camera::default()).id();
            let mut target = ComputedUiTargetCamera::default();
            let mut reflected = DynamicStruct::default();
            reflected.insert("camera", camera);
            target.apply(&reflected);
            let mut window = Window::default();
            let cursor = if click_on_caption {
                Vec2::new(150.0, 100.0)
            } else {
                Vec2::new(25.0, 100.0)
            };
            window.set_physical_cursor_position(Some(cursor.as_dvec2()));
            app.world_mut().spawn((window, PrimaryWindow));

            let mut buttons = app
                .world_mut()
                .query_filtered::<Entity, With<NpcActionButton>>();
            let button = buttons.single(app.world()).unwrap();
            let children = app.world().get::<Children>(button).unwrap().to_vec();
            let icon = children[0];
            let caption = children[1];
            for (entity, center, size) in [
                (button, Vec2::new(106.0, 100.0), Vec2::new(212.0, 48.0)),
                (icon, Vec2::new(28.0, 100.0), Vec2::new(38.0, 33.0)),
                (caption, Vec2::new(127.0, 100.0), Vec2::new(154.0, 48.0)),
            ] {
                app.world_mut().entity_mut(entity).insert((
                    ComputedNode { size, ..default() },
                    UiGlobalTransform::from(Affine2::from_translation(center)),
                    InheritedVisibility::VISIBLE,
                    target,
                ));
            }
            app.insert_resource(UiStack {
                uinodes: vec![button, icon, caption],
                partition: vec![0..3],
                ..default()
            });
            if tutorial {
                app.world_mut()
                    .resource_mut::<TutorialNativeMechanics>()
                    .sync_stable_stage(crate::tutorial::TutorialStage::Infection(
                        crate::tutorial::InfectionStage::WarpFromTechSquare,
                    ));
            }
            let warp = WarpUiEntry {
                npc_id: 91,
                npc_type: 682,
                warp_id: 6,
                required_task_id: None,
                target: TutorialWarpTarget {
                    map_id: 3,
                    x: 100,
                    y: 200,
                    z: 300,
                },
                label: "WARP".to_owned(),
            };
            {
                let mut model = app.world_mut().resource_mut::<MissionUiModel>();
                model.enabled = true;
                model.show_npc_interaction(NpcInteractionUi {
                    npc_id: warp.npc_id,
                    npc_type: warp.npc_type,
                    warp: Some(warp.clone()),
                    ..default()
                });
            }
            app.update(); // Arm the visible menu while the mouse is released.

            let overlay = if click_on_caption { caption } else { icon };
            assert_eq!(
                app.world().get::<FocusPolicy>(overlay),
                Some(&FocusPolicy::Pass)
            );
            // Reproduce the old failure even though Pickable::IGNORE is present.
            app.world_mut().entity_mut(overlay).remove::<FocusPolicy>();
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
            app.update();
            assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
            assert_eq!(
                app.world().get::<Interaction>(button),
                Some(&Interaction::None)
            );

            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .release(MouseButton::Left);
            app.update();
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .clear();
            app.world_mut()
                .entity_mut(overlay)
                .insert(FocusPolicy::Pass);
            app.world_mut()
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
            app.update();
            assert_eq!(
                app.world_mut()
                    .resource_mut::<GameplayUiOutbox>()
                    .drain()
                    .collect::<Vec<_>>(),
                vec![GameplayUiAction::NpcWarp {
                    npc_id: warp.npc_id,
                    npc_type: warp.npc_type,
                    warp_id: warp.warp_id,
                    required_task_id: warp.required_task_id,
                    target: warp.target,
                }],
                "tutorial={tutorial}, caption={click_on_caption}"
            );
            app.update();
            assert!(
                app.world().resource::<GameplayUiOutbox>().is_empty(),
                "holding must not repeat warp"
            );
        }
    }
}

fn spawn_test_journal(mut commands: Commands, server: Res<AssetServer>) {
    let assets = MissionUiAssets::load(&server);
    commands
        .spawn_empty()
        .with_children(|root| spawn_journal(root, &assets));
}

#[test]
fn journal_frame_and_row_decorations_pass_real_mouse_presses() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        .init_asset::<Font>()
        .init_resource::<ButtonInput<MouseButton>>()
        .init_resource::<Touches>()
        .init_resource::<TutorialNativeMechanics>()
        .init_resource::<GameplayUiOutbox>()
        .init_resource::<GameplayUiAudioOutbox>()
        .insert_resource(MissionUiModel {
            enabled: true,
            journal: MissionJournalUi::Other(JournalOtherUi {
                active_missions: vec![
                    MissionUiEntry {
                        task_id: 500,
                        mission_type: 3,
                        ..default()
                    },
                    MissionUiEntry {
                        task_id: 501,
                        mission_type: 3,
                        ..default()
                    },
                ],
                ..default()
            }),
            selected_journal_task_id: Some(500),
            ..default()
        })
        .add_systems(Update, (ui_focus_system, handle_mission_ui_buttons).chain());
    app.world_mut().run_system_once(spawn_test_journal).unwrap();
    let camera = app.world_mut().spawn(Camera::default()).id();
    let mut target = ComputedUiTargetCamera::default();
    let mut reflected = DynamicStruct::default();
    reflected.insert("camera", camera);
    target.apply(&reflected);
    let mut window = Window::default();
    window.set_physical_cursor_position(Some(Vec2::splat(100.0).as_dvec2()));
    app.world_mut().spawn((window, PrimaryWindow));
    let mut views = app.world_mut().query::<(Entity, &MissionUiView)>();
    let button = views
        .iter(app.world())
        .find(|(_, view)| **view == MissionUiView::JournalMissionRow(0))
        .unwrap()
        .0;
    let frame = views
        .iter(app.world())
        .find(|(_, view)| **view == MissionUiView::JournalRightFrame)
        .unwrap()
        .0;
    // Rows sit inside the clipped `DoWindowRight` scroll view, so it needs a
    // laid-out rect around the cursor as well.
    let viewport = app
        .world_mut()
        .query_filtered::<Entity, With<JournalScrollViewport>>()
        .single(app.world())
        .unwrap();
    app.world_mut().entity_mut(viewport).insert((
        ComputedNode {
            size: Vec2::new(
                JOURNAL_SCROLL_VIEW_RECT.width,
                JOURNAL_SCROLL_VIEW_RECT.height,
            ),
            ..default()
        },
        UiGlobalTransform::from(Affine2::from_translation(Vec2::splat(100.0))),
    ));
    let mut overlays = app.world().get::<Children>(button).unwrap().to_vec();
    let mut index = 0;
    while index < overlays.len() {
        if let Some(children) = app.world().get::<Children>(overlays[index]) {
            overlays.extend(children.iter());
        }
        index += 1;
    }
    overlays.push(frame);
    for overlay in overlays {
        let checkbox = matches!(
            app.world().get::<MissionUiControl>(overlay),
            Some(MissionUiControl::JournalTrackMission(0))
        );
        // Supply overlapping layout for each production overlay independently;
        // Bevy must deliver both hover and press through it to the mission row.
        for entity in [button, overlay] {
            app.world_mut().entity_mut(entity).insert((
                ComputedNode {
                    size: Vec2::new(330.0, 86.0),
                    ..default()
                },
                UiGlobalTransform::from(Affine2::from_translation(Vec2::splat(100.0))),
                InheritedVisibility::VISIBLE,
                target,
            ));
        }
        app.insert_resource(UiStack {
            uinodes: vec![button, overlay],
            partition: vec![0..2],
            ..default()
        });
        {
            let mut model = app.world_mut().resource_mut::<MissionUiModel>();
            model.viewed_journal_task_id = Some(501);
            model.selected_journal_task_id = Some(501);
        }
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .release(MouseButton::Left);
        app.update();
        assert_eq!(
            app.world().get::<Interaction>(button),
            Some(if checkbox {
                &Interaction::None
            } else {
                &Interaction::Hovered
            }),
            "overlay {overlay:?}"
        );
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .clear();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
        assert_eq!(
            app.world()
                .resource::<MissionUiModel>()
                .viewed_journal_task_id,
            Some(if checkbox { 501 } else { 500 }),
            "overlay {overlay:?}"
        );
        assert_eq!(
            app.world()
                .resource::<MissionUiModel>()
                .selected_journal_task_id,
            Some(if checkbox { 500 } else { 501 }),
            "only the checkbox changes tracking: overlay {overlay:?}"
        );
        assert!(app.world().resource::<GameplayUiOutbox>().is_empty());
    }
}
