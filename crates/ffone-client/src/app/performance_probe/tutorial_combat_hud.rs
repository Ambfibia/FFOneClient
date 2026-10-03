//! Full-client replay of tutorial rewards and the combat HUD timeout.
use super::*;
use ffone_client::tutorial_logic::{LegacySpawnPosition, TutorialNpcSpawn};

#[derive(Resource, Default)]
struct CombatHudProbe {
    frame: u32,
    pending: usize,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<CombatHudProbe>()
        .add_systems(Last, drive.before(super::measure));
}

fn capture(world: &mut World, name: &str, fm: i32, combat: bool, event: bool) {
    let hud = world.resource::<GameplayUiModel>();
    assert_eq!(hud.minimap.fusion_matter, fm, "{name}: FM");
    assert_eq!(hud.minimap.max_fusion_matter, 220);
    let context = world
        .query_filtered::<&LegacyAvatarActionContext, With<LocalPlayer>>()
        .single(world)
        .unwrap();
    assert_eq!(context.combat_condition, combat, "{name}: combat");
    assert_eq!(context.tutorial_event, event, "{name}: event scene");
    let images: Vec<_> = world
        .query::<(&Node, &ImageNode)>()
        .iter(world)
        .map(|(node, image)| (image.image.id(), node.display))
        .collect();
    let assets = world.resource::<AssetServer>();
    for (path, visible) in [
        ("ui/en/gameplay/shared/combat-frame.png", combat && !event),
        ("ui/en/gameplay/shared/danger.png", combat),
    ] {
        let matches: Vec<_> = images
            .iter()
            .filter(|(id, _)| {
                assets
                    .get_path(*id)
                    .is_some_and(|asset| asset.path().ends_with(path.trim_start_matches("ui/en/")))
            })
            .collect();
        assert_eq!(matches.len(), 1, "{name}: {path}");
        // The monster-info panel replaces DANGER while a valid target owns
        // that slot. The full-screen frame is independent of that panel.
        if path.ends_with("combat-frame.png") || !combat {
            assert_eq!(
                matches[0].1,
                if visible {
                    Display::Flex
                } else {
                    Display::None
                },
                "{name}: {path}"
            );
        }
    }
    println!("tutorial-combat-hud {name}: FM={fm}/220, combat={combat}, event={event}");
    let path = world
        .resource::<Capture>()
        .output
        .join(format!("{name}.png"));
    world.resource_mut::<CombatHudProbe>().pending += 1;
    world.spawn(Screenshot::primary_window()).observe(
        move |event: On<ScreenshotCaptured>, mut probe: ResMut<CombatHudProbe>| {
            event
                .image
                .clone()
                .try_into_dynamic()
                .unwrap()
                .save(&path)
                .unwrap();
            probe.pending -= 1;
        },
    );
}

fn drive(world: &mut World) {
    if *world.resource::<State<ClientState>>().get() != ClientState::Tutorial
        || world.resource::<GameplayLoadingState>().visible
    {
        return;
    }
    let frame = {
        let mut probe = world.resource_mut::<CombatHudProbe>();
        probe.frame += 1;
        probe.frame
    };
    if frame == 1 {
        // Enter the authored combat area through the ordinary production player,
        // rendering and actor pipeline. This fixture owns its screenshots/exit.
        world.resource_mut::<Capture>().captured = true;
        world.resource_mut::<TutorialChoreographyPlayer>().stop();
        world
            .resource_mut::<TutorialChoreographyPresentation>()
            .event_scene = false;
        let mut tutorial = world.resource_mut::<TutorialSession>();
        tutorial.init_chapter(1).unwrap();
        tutorial.init_step(3);
        tutorial.scene = TutorialScene::None;
        let mut commands = world.resource_mut::<TutorialActorCommandQueue>();
        // BasicMove's two companions are also camera/animation owners in
        // BasicCombatC. Retain their authored identities for that handoff.
        for (id, npc_type, y) in [(100, 2669, 65_700), (101, 2670, 65_300)] {
            commands.spawn(TutorialNpcSpawn::new(
                id,
                npc_type,
                LegacySpawnPosition::centiunits(54_100, y, -10_400),
                None,
            ));
        }
        for (id, npc_type, x) in [
            (1001, 2674, 54_900),
            (1002, 2674, 55_100),
            (1003, 2674, 55_300),
            (1005, 2675, 55_500),
        ] {
            commands.spawn(TutorialNpcSpawn::new(
                id,
                npc_type,
                LegacySpawnPosition::centiunits(x, 65_500, -10_400),
                None,
            ));
        }
    }
    match frame {
        90 => capture(world, "01-idle", 0, false, false),
        100 => world
            .resource_mut::<TutorialActorCommandQueue>()
            .damage(1001, i32::MAX),
        110 => capture(world, "02-first-spawn", 30, true, false),
        120 => {
            world
                .resource_mut::<TutorialChoreographyPresentation>()
                .event_scene = true
        }
        125 => capture(world, "03-event-scene", 30, true, true),
        130 => {
            world
                .resource_mut::<TutorialChoreographyPresentation>()
                .event_scene = false
        }
        135 => capture(world, "04-combat-resumed", 30, true, false),
        405 => capture(world, "05-combat-timeout", 30, false, false),
        410 => world
            .resource_mut::<TutorialActorCommandQueue>()
            .damage(1002, i32::MAX),
        420 => capture(world, "06-second-spawn", 60, true, false),
        430 => world
            .resource_mut::<TutorialActorCommandQueue>()
            .damage(1003, i32::MAX),
        440 => capture(world, "07-third-spawn", 90, true, false),
        450 => {
            // Continue at the actual Kerber fight so its kill takes the normal
            // BasicCombatC handoff and the scene suppresses the green frame.
            world.resource_mut::<TutorialSession>().init_step(7);
            world
                .resource_mut::<TutorialActorCommandQueue>()
                .damage(1005, i32::MAX);
        }
        455 => capture(world, "08-kerber", 120, true, true),
        _ => {}
    }
    if frame > 455 && world.resource::<CombatHudProbe>().pending == 0 {
        assert!(
            world
                .resource::<RuntimeStatus>()
                .diagnostics
                .tutorial_issue_history
                .is_empty(),
            "combat HUD replay must not leave unresolved tutorial actors or choreography"
        );
        fs::write(world.resource::<Capture>().output.join("passed.txt"),
            "Tutorial FM rewards, combat activity/timeout and cutscene overlay passed in the full client.\n").unwrap();
        world.write_message(AppExit::Success);
    }
}
