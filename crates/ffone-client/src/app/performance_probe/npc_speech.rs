//! Bug 43: greeting -> mandatory quest through the full client bubble renderer.
use super::*;
use ffone_client::localization::LocalizedText;

#[derive(Default, Resource)]
struct Replay {
    frame: u32,
    owner: Option<Entity>,
    npc_type: i32,
    quest: Option<LocalizedText>,
    greeting: Option<LocalizedText>,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<Replay>()
        .add_systems(Update, discard_offline_events.before(poll_network))
        .add_systems(
            Update,
            pose.after(LegacyMovementSet::CameraPose)
                .before(ffone_client::gameplay_ui::GameplayUiSet::NpcSpeech),
        )
        .add_systems(Last, drive.before(measure));
}

fn discard_offline_events(bridge: Res<NetworkBridge>) {
    let _ = bridge.drain();
}

fn pose(world: &mut World) {
    let Some(owner) = world.resource::<Replay>().owner else {
        return;
    };
    let center = world.get::<GlobalTransform>(owner).unwrap().translation() + Vec3::Y * 2.0;
    let camera_pose =
        Transform::from_translation(center + Vec3::new(0.0, 3.0, 12.0)).looking_at(center, Vec3::Y);
    let cameras: Vec<_> = world
        .query_filtered::<(Entity, &Camera), With<Camera3d>>()
        .iter(world)
        .filter(|(_, camera)| camera.is_active)
        .map(|(entity, _)| entity)
        .collect();
    for entity in cameras {
        world
            .entity_mut(entity)
            .insert((camera_pose, GlobalTransform::from(camera_pose)));
    }
}

fn drive(world: &mut World) {
    if world.resource::<GameplayLoadingState>().visible
        || world.resource::<Capture>().ready.is_none()
    {
        return;
    }
    let mut replay = world.remove_resource::<Replay>().unwrap();
    replay.frame += 1;
    if replay.frame == 1 {
        let npc_type = 692; // Larry 3000, Goat's Junkyard, waypoint row 2876.
        let owner = world
            .query::<(Entity, &NetworkNpcAppearance0104)>()
            .iter(world)
            .find(|(_, appearance)| appearance.0.npc_type == npc_type)
            .map(|(entity, _)| entity)
            .expect("published Larry 3000 fixture");
        replay.owner = Some(owner);
        replay.npc_type = npc_type;
        let content = world.resource::<TutorialMissionContent>();
        let npc = content.gameplay_npc(npc_type).unwrap().clone();
        replay.greeting = Some(
            ffone_client::localization::localized_tabledata_npc_greeting(
                npc.greeting_string_id,
                &npc.greeting,
            ),
        );
        let quest = content
            .missions()
            .flat_map(|task| {
                [
                    task.start_dialogue.as_ref(),
                    task.success_dialogue.as_ref(),
                    task.failure_dialogue.as_ref(),
                ]
            })
            .flatten()
            .find(|line| line.npc_type == npc_type)
            .unwrap();
        replay.quest = Some(LocalizedText::new(
            format!(
                "content.tabledata.mission.mission_string.{}.str_name_string",
                quest.string_id
            ),
            &quest.text,
        ));
        world
            .resource_mut::<NpcBarkerBubbleRuntime>()
            .request_greeting(owner, &npc);
    }
    // Keep the real window camera on the actual NPC for screenshot acceptance.
    if let Some(owner) = replay.owner {
        let center = world.get::<GlobalTransform>(owner).unwrap().translation() + Vec3::Y * 2.0;
        let nearby = center - Vec3::Y * 2.0 + Vec3::Z * 5.0;
        let players: Vec<_> = world
            .query_filtered::<Entity, With<LocalPlayer>>()
            .iter(world)
            .collect();
        for player in players {
            world.get_mut::<Transform>(player).unwrap().translation = nearby;
            world
                .entity_mut(player)
                .insert(GlobalTransform::from_translation(nearby));
        }
        let pose = Transform::from_translation(center + Vec3::new(0.0, 3.0, 12.0))
            .looking_at(center, Vec3::Y);
        let cameras: Vec<_> = world
            .query_filtered::<(Entity, &Camera), With<Camera3d>>()
            .iter(world)
            .filter(|(_, camera)| camera.is_active)
            .map(|(entity, _)| entity)
            .collect();
        for entity in cameras {
            world
                .entity_mut(entity)
                .insert((pose, GlobalTransform::from(pose)));
        }
    }
    if replay.frame == 20 {
        let greeting = replay.greeting.as_ref().unwrap();
        assert!(
            world
                .query::<(&Text, &LocalizedText)>()
                .iter(world)
                .any(|(_, localized)| localized == greeting),
            "greeting must be rendered before quest"
        );
        let output = world.resource::<Capture>().output.join("greeting.png");
        world
            .spawn(Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(output));
    }
    if replay.frame == 21 {
        world
            .resource_mut::<NpcBarkerBubbleRuntime>()
            .request_quest_dialogue(
                replay.owner.unwrap(),
                replay.npc_type,
                replay.quest.clone().unwrap(),
            );
    }
    if replay.frame == 25 {
        let quest = replay.quest.as_ref().unwrap();
        let text = world
            .resource::<Localization>()
            .text(world.resource::<Language>(), quest);
        let parent = world
            .query::<(&Text, &LocalizedText, &ChildOf)>()
            .iter(world)
            .find_map(|(rendered, localized, parent)| {
                (localized == quest && rendered.0 == text).then_some(parent.parent())
            });
        assert!(
            parent.is_some(),
            "quest must replace greeting in renderer within four frames, before its seven-second lifetime"
        );
        let parent = parent.unwrap();
        assert_ne!(
            world.get::<Node>(parent).unwrap().display,
            Display::None,
            "quest bubble must be visible"
        );
        assert!(
            world
                .get::<ComputedNode>(parent)
                .unwrap()
                .size()
                .min_element()
                > 0.0,
            "quest bubble must have a rendered area"
        );
        let greeting = replay.greeting.as_ref().unwrap();
        assert!(
            !world
                .query::<(&Text, &LocalizedText)>()
                .iter(world)
                .any(|(_, localized)| localized == greeting),
            "old greeting bubble must disappear"
        );
        fs::write(
            world
                .resource::<Capture>()
                .output
                .join("npc-speech-pass.txt"),
            format!(
                "PASS npc_type={} key={} locale={} quest rendered within four frames (<7 s)\n",
                replay.npc_type,
                quest.key,
                world.resource::<Language>().effective
            ),
        )
        .unwrap();
        let output = world.resource::<Capture>().output.join("quest.png");
        world
            .spawn(Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(output));
    }
    if replay.frame == 40 {
        world
            .resource_mut::<Messages<AppExit>>()
            .write(AppExit::Success);
    }
    world.insert_resource(replay);
}
