use super::*;

#[derive(Resource)]
struct BackgroundAttempt(Entity, LocalizedText, f64, bool);

fn background(
    attempt: Res<BackgroundAttempt>,
    mut runtime: ResMut<NpcBarkerBubbleRuntime>,
    mut chat: MessageWriter<NpcChatEvent>,
) {
    runtime.publish_background_speech(
        attempt.0,
        attempt.1.clone(),
        attempt.2,
        attempt.3,
        &mut chat,
    );
}

fn app_with_actors() -> (App, Entity, Entity) {
    let mut app = App::new();
    app.init_resource::<NpcBarkerBubbleRuntime>()
        .add_message::<NpcChatEvent>();
    let a = app.world_mut().spawn_empty().id();
    let b = app.world_mut().spawn_empty().id();
    for (owner, npc_type) in [(a, 2555), (b, 2672)] {
        app.world_mut()
            .resource_mut::<NpcBarkerBubbleRuntime>()
            .actors
            .insert(
                owner,
                NpcBarkerActorState {
                    npc_type,
                    f_barker_time: 0.0,
                    lines: Default::default(),
                },
            );
    }
    (app, a, b)
}

#[test]
fn background_cooldown_is_shared_by_text_across_owners_rows_and_locales() {
    let (mut app, a, b) = app_with_actors();
    app.insert_resource(BackgroundAttempt(
        a,
        LocalizedText::new("row.a", "Same line"),
        10.0,
        true,
    ))
    .add_systems(Update, background);
    let mut cursor = app
        .world()
        .resource::<Messages<NpcChatEvent>>()
        .get_cursor();
    app.update();
    assert_eq!(
        cursor
            .read(app.world().resource::<Messages<NpcChatEvent>>())
            .count(),
        1
    );
    assert_eq!(
        app.world().resource::<NpcBarkerBubbleRuntime>().actors[&a].lines[0].kind,
        NpcBubbleKind::Ordinary
    );
    // Different table row / speaker, whitespace variation, just before 10 min.
    app.insert_resource(BackgroundAttempt(
        b,
        LocalizedText::new("row.b", " Same   line "),
        609.999,
        true,
    ));
    app.update();
    assert_eq!(
        cursor
            .read(app.world().resource::<Messages<NpcChatEvent>>())
            .count(),
        0
    );
    assert!(
        app.world().resource::<NpcBarkerBubbleRuntime>().actors[&b]
            .lines
            .is_empty()
    );
    // The source-text key is independent of the selected EN/RU translation.
    app.insert_resource(BackgroundAttempt(
        b,
        LocalizedText::new("row.b", "Same line"),
        610.0,
        true,
    ));
    app.update();
    assert_eq!(
        cursor
            .read(app.world().resource::<Messages<NpcChatEvent>>())
            .count(),
        1
    );
}

#[test]
fn background_history_cooldown_applies_even_when_balloons_are_disabled() {
    let (mut app, a, b) = app_with_actors();
    app.insert_resource(BackgroundAttempt(
        a,
        LocalizedText::new("a", "Shared"),
        1.0,
        false,
    ))
    .add_systems(Update, background);
    app.update();
    app.insert_resource(BackgroundAttempt(
        b,
        LocalizedText::new("b", "Shared"),
        2.0,
        true,
    ));
    app.update();
    let runtime = app.world().resource::<NpcBarkerBubbleRuntime>();
    assert!(runtime.actors[&a].lines.is_empty());
    assert!(runtime.actors[&b].lines.is_empty());
    assert_eq!(runtime.background_last_spoken.len(), 1);
}

#[test]
fn quest_replaces_greeting_and_queued_background_without_waiting_and_can_repeat() {
    let (mut app, owner, _) = app_with_actors();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let content =
        TutorialMissionContent::open(&crate::assets::AssetLocator::open(&root).unwrap()).unwrap();
    let definition = content.gameplay_npc(2555).unwrap().clone();
    app.insert_resource(Time::<()>::default())
        .insert_resource(content)
        .init_resource::<GameplayUiModel>()
        .init_resource::<LegacyNanoStandRandomStream>()
        .add_systems(Update, advance_npc_barker_bubbles);
    app.world_mut().entity_mut(owner).insert((
        GlobalTransform::IDENTITY,
        normal_world_npc_appearance(2555, 100),
    ));
    let quest = LocalizedText::new("quest", "Mandatory quest");
    {
        let mut runtime = app.world_mut().resource_mut::<NpcBarkerBubbleRuntime>();
        runtime.enqueue(owner, LocalizedText::new("background", "Old ambient"), 0.0);
        runtime.request_greeting(owner, &definition);
        runtime.request_quest_dialogue(owner, 2555, quest.clone());
        runtime
            .background_last_spoken
            .insert("Mandatory quest".into(), 0.0);
    }
    let mut cursor = app
        .world()
        .resource::<Messages<NpcChatEvent>>()
        .get_cursor();
    app.update();
    let runtime = app.world().resource::<NpcBarkerBubbleRuntime>();
    assert_eq!(runtime.actors[&owner].lines.len(), 1);
    assert_eq!(runtime.actors[&owner].lines[0].localized, quest);
    assert_eq!(runtime.actors[&owner].lines[0].kind, NpcBubbleKind::Quest);
    let first_serial = runtime.actors[&owner].lines[0].serial;
    assert_eq!(
        cursor
            .read(app.world().resource::<Messages<NpcChatEvent>>())
            .count(),
        2
    );
    app.world_mut()
        .resource_mut::<NpcBarkerBubbleRuntime>()
        .request_quest_dialogue(owner, 2555, quest.clone());
    app.update();
    let line = &app.world().resource::<NpcBarkerBubbleRuntime>().actors[&owner].lines[0];
    assert_eq!(line.localized, quest);
    assert_eq!(line.kind, NpcBubbleKind::Quest);
    assert_ne!(line.serial, first_serial);
    assert_eq!(
        cursor
            .read(app.world().resource::<Messages<NpcChatEvent>>())
            .count(),
        1
    );
}
