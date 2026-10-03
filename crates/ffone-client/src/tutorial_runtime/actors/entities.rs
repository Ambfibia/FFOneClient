use super::*;

pub(super) fn spawn_actor(world: &mut World, spawn: TutorialNpcSpawn) {
    let Some(table_definition) = world
        .resource::<TutorialMissionContent>()
        .gameplay_npc(spawn.npc_type)
        .cloned()
    else {
        world
            .resource_mut::<TutorialActorIssueQueue>()
            .push(TutorialActorIssue::UnknownNpcType {
                id: spawn.id,
                npc_type: spawn.npc_type,
            });
        return;
    };

    if let Some(entity) = world.resource::<TutorialActorRegistry>().entity(spawn.id)
        && world.get_entity(entity).is_ok()
    {
        let next_transform = tutorial_spawn_transform_for_class(spawn, table_definition.npc_class);
        let identical = world
            .get::<TutorialActor>(entity)
            .is_some_and(|actor| actor.npc_type == spawn.npc_type)
            && world
                .get::<Transform>(entity)
                .is_some_and(|transform| *transform == next_transform);
        if identical {
            return;
        }
        let demo_dont_kill = world.resource::<TutorialActorCombatConfig>().demo_dont_kill;
        let old_visual_container = world
            .get::<NetworkNpcVisual0104>(entity)
            .map(|visual| visual.visual_container);
        if let Some(old_visual_container) = old_visual_container {
            let _ = world.despawn(old_visual_container);
        }
        if let Some(mut entity_mut) = world.get_entity_mut(entity).ok() {
            entity_mut.insert((
                TutorialActor {
                    id: spawn.id,
                    npc_type: spawn.npc_type,
                    team: table_definition.team,
                    hp: table_definition.max_hp,
                    max_hp: table_definition.max_hp,
                    damaged: false,
                    interacting: false,
                    invulnerable: spawn.id == DEMO_MONSTER_ID && demo_dont_kill,
                },
                TutorialActorPose::default(),
                TutorialActorAnimationIssueHistory::default(),
                TutorialActorGrounding::default(),
                Name::new(tutorial_actor_root_name(spawn.id)),
                next_transform,
            ));
            entity_mut.remove::<(
                TutorialActorMotion,
                TutorialActorForceUpdate,
                NetworkNpcVisual0104,
                TutorialActorAnimationSource,
                TutorialActorVisualUnavailable0104,
            )>();
        }
        return;
    }

    let demo_dont_kill = world.resource::<TutorialActorCombatConfig>().demo_dont_kill;
    let entity = world
        .spawn((
            TutorialActor {
                id: spawn.id,
                npc_type: spawn.npc_type,
                team: table_definition.team,
                hp: table_definition.max_hp,
                max_hp: table_definition.max_hp,
                damaged: false,
                interacting: false,
                invulnerable: spawn.id == DEMO_MONSTER_ID && demo_dont_kill,
            },
            TutorialActorPose::default(),
            TutorialActorAnimationIssueHistory::default(),
            TutorialActorGrounding::default(),
            Name::new(tutorial_actor_root_name(spawn.id)),
            tutorial_spawn_transform_for_class(spawn, table_definition.npc_class),
            // The authored GLB is attached asynchronously as a child.  Bevy's
            // visibility propagation requires every parent in that hierarchy
            // to carry InheritedVisibility; without a Visibility component on
            // the actor root, tutorial NPCs could disappear on the frame their
            // scene finished loading (B0004).
            Visibility::Inherited,
        ))
        .id();
    world
        .resource_mut::<TutorialActorRegistry>()
        .by_id
        .insert(spawn.id, entity);
}

/// Attaches the same validated XDT-driven visual used by an ordinary network
/// NPC. Tutorial choreography owns the actor root and behavior only.
pub fn spawn_tutorial_actor_visuals(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    content: Res<TutorialMissionContent>,
    catalog_state: Res<NetworkNpcVisualCatalogState0104>,
    mut animation_assets: ResMut<TutorialActorAnimationAssets>,
    actors: Query<
        (Entity, &TutorialActor),
        (
            Without<NetworkNpcVisual0104>,
            Without<TutorialActorVisualUnavailable0104>,
        ),
    >,
) {
    let Some(catalog) = catalog_state.catalog.as_ref() else {
        return;
    };
    for (entity, actor) in &actors {
        let Some(table_definition) = content.gameplay_npc(actor.npc_type) else {
            commands
                .entity(entity)
                .insert(TutorialActorVisualUnavailable0104::MissingRoute(format!(
                    "NPC type {} has no gameplay table row",
                    actor.npc_type
                )));
            continue;
        };
        if table_definition.npc_class >= 100 {
            commands
                .entity(entity)
                .insert(TutorialActorVisualUnavailable0104::HiddenLocation);
            continue;
        }
        let Some(definition) = catalog.get(actor.npc_type) else {
            let detail = format!(
                "no validated shared XDT-to-GLB route for tutorial actor {} type {}",
                actor.id, actor.npc_type
            );
            warn!("{detail}");
            commands
                .entity(entity)
                .insert(TutorialActorVisualUnavailable0104::MissingRoute(detail));
            continue;
        };
        let path = definition.glb.clone();
        let gltf = animation_assets
            .gltfs
            .entry(path.clone())
            .or_insert_with(|| asset_server.load(path.clone()))
            .clone();
        let visual = spawn_network_npc_visual_0104(
            &mut commands,
            &asset_server,
            entity,
            definition,
            format!("tutorial actor {}", actor.id),
        );
        // Keep the exact shared deferred-reveal contract used by ordinary
        // network NPCs and the Editor. Scene0 must not become visible while
        // it still owns glTF StandardMaterial fallbacks or before every
        // source renderer has received its XDT main/sub texture.
        commands.entity(visual.scene).insert(TutorialActorScene);
        commands.entity(entity).insert((
            visual,
            TutorialActorAnimationSource { path, gltf },
            NetworkNpcAppearEffect0104::default(),
        ));
    }
}
