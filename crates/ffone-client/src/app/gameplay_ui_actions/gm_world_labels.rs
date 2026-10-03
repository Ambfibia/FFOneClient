//! Project opt-in GM identities from gameplay roots, never individual mesh children.
use super::*;
use ffone_client::{
    entity_lifecycle::{
        NetworkNpc0104, NetworkRemotePc0104, NetworkShiny0104, NetworkTransportation0104,
    },
    tutorial_actors::TutorialActor,
    world::SpawnedNativeWorldVisual,
    world_behaviour::WorldTrigger,
};

#[derive(Component)]
pub(crate) struct WorldIdLabel(Entity);

type IdentityRoots<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GlobalTransform,
        Option<&'static NetworkNpc0104>,
        Option<&'static NetworkShiny0104>,
        Option<&'static NetworkTransportation0104>,
        Option<&'static NetworkRemotePc0104>,
        Option<&'static TutorialActor>,
        Option<&'static WorldTrigger>,
        Option<&'static SpawnedNativeWorldVisual>,
        Option<&'static InheritedVisibility>,
        Has<LocalPlayer>,
    ),
    Or<(
        With<NetworkNpc0104>,
        With<NetworkShiny0104>,
        With<NetworkTransportation0104>,
        With<NetworkRemotePc0104>,
        With<TutorialActor>,
        With<WorldTrigger>,
        With<SpawnedNativeWorldVisual>,
        With<LocalPlayer>,
    )>,
>;

pub(crate) fn presentation(
    mut commands: Commands,
    runtime: Res<RuntimeStatus>,
    assets: Res<AssetServer>,
    content: Option<Res<TutorialMissionContent>>,
    cameras: Query<(&Camera, &GlobalTransform, &LegacyOrbitCamera), With<Camera3d>>,
    players: Query<Entity, With<LocalPlayer>>,
    roots: IdentityRoots,
    mut labels: Query<(Entity, &WorldIdLabel, &mut Node, &mut Text)>,
) {
    let projection = players.single().ok().and_then(|player| {
        cameras
            .iter()
            .find(|(camera, _, orbit)| camera.is_active && orbit.target == player)
    });
    let enabled =
        runtime.player_id.is_some() && runtime.user_level <= 50 && runtime.chat.gm.view_location;
    let mut wanted = BTreeMap::new();
    if let (true, Some((camera, camera_transform, _))) = (enabled, projection) {
        for (
            entity,
            transform,
            npc,
            shiny,
            transport,
            pc,
            tutorial,
            trigger,
            visual,
            visibility,
            local,
        ) in &roots
        {
            let mut position = transform.translation();
            let identity = npc
                .map(|n| (n.npc_id, n.npc_type))
                .or_else(|| tutorial.map(|n| (n.id, n.npc_type)));
            let text = if let Some((id, kind)) = identity {
                let definition = content.as_ref().and_then(|c| c.gameplay_npc(kind));
                position.y += definition.map_or(2.0, |d| d.height().max(0.5));
                let category = if definition.is_some_and(|d| d.team != 1) {
                    "Mob"
                } else {
                    "NPC"
                };
                let mut text = format!("{category} ID={id} type={kind}");
                if let Some(content) = &content {
                    for warp in content.gameplay_warps().filter(|w| w.npc_type == kind) {
                        text.push_str(&format!("\nWarp ID={}", warp.warp_id));
                    }
                }
                text
            } else if let Some(shiny) = shiny {
                position.y += 0.6;
                format!("Shiny ID={} type={}", shiny.shiny_id, shiny.shiny_type)
            } else if let Some(transport) = transport {
                position.y += 2.5;
                format!(
                    "Transport ID={} type={} kind={}",
                    transport.id, transport.transportation_type, transport.transportation_kind
                )
            } else if local || pc.is_some() {
                position.y += ffone_client::world::AUTHORED_CHARACTER_CONTROLLER_HEIGHT;
                format!(
                    "PC ID={}",
                    pc.map_or(runtime.player_id.unwrap_or_default(), |p| p.pc_id)
                )
            } else if let Some(trigger) = trigger {
                position = transform.transform_point(trigger.start_position) + Vec3::Y;
                format!(
                    "{:?} ID={} object={} CNE={}",
                    trigger.kind, trigger.server_id, trigger.object_id, trigger.cne_id
                )
            } else if visual.is_some() {
                // Static scenery has no server ID. Explicitly identify the runtime
                // entity instead of inventing a server or TableData identity.
                if visibility.is_some_and(|v| !v.get()) {
                    continue;
                }
                position.y += 1.0;
                format!("Entity={entity}")
            } else {
                continue;
            };
            let Ok(viewport) = camera.world_to_viewport(camera_transform, position) else {
                continue;
            };
            let Some(rect) = camera.logical_viewport_rect() else {
                continue;
            };
            if !rect.contains(viewport) {
                continue;
            }
            wanted.insert(entity, (viewport, text));
        }
    }
    for (entity, label, mut node, mut text) in &mut labels {
        let Some((position, value)) = wanted.remove(&label.0) else {
            commands.entity(entity).despawn();
            continue;
        };
        let (left, top, width) = label_rect(position, &value);
        node.left = px(left);
        node.top = px(top);
        node.width = px(width);
        if text.0 != value {
            text.0 = value;
        }
    }
    for (owner, (position, text)) in wanted {
        let (left, top, width) = label_rect(position, &text);
        commands.spawn((
            WorldIdLabel(owner),
            Node {
                position_type: PositionType::Absolute,
                left: px(left),
                top: px(top),
                width: px(width),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Text::new(text),
            TextFont {
                font: assets.load("fonts/chaletbook-regular.ttf").into(),
                font_size: 12.0.into(),
                ..default()
            },
            TextColor(Color::srgb(1.0, 0.95, 0.4)),
            TextLayout::justify(Justify::Center),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.45)),
            GlobalZIndex(1999),
            bevy::ui::FocusPolicy::Pass,
            Pickable::IGNORE,
        ));
    }
}

fn label_rect(position: Vec2, text: &str) -> (f32, f32, f32) {
    let width =
        (text.lines().map(str::len).max().unwrap_or(0) as f32 * 7.5 + 8.0).clamp(100.0, 480.0);
    let height = text.lines().count().max(1) as f32 * 16.0;
    (position.x - width * 0.5, position.y - height, width)
}
