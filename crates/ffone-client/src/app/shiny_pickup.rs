//! Proximity pickup is predicted locally; rewards and buffs remain server-owned.
use super::*;
use ffone_client::{
    entity_lifecycle::{NetworkShiny0104, despawn_shiny},
    network_world_runtime::shiny::ShinyPickupEffect,
};
use ffone_protocol::wire_0104::ShinyPickupRequest0104;

pub(super) fn pickup(
    mut commands: Commands,
    bridge: Res<NetworkBridge>,
    state: Res<State<ClientState>>,
    runtime: Res<RuntimeStatus>,
    players: Query<(&Transform, &LegacyAvatarActionContext), With<LocalPlayer>>,
    shinies: Query<(&NetworkShiny0104, &Transform, &ShinyPickupEffect), Without<LocalPlayer>>,
    mut effects: ResMut<TutorialEffectRuntime>,
    mut audio: ResMut<GameplayAudioRuntime>,
) {
    if *state.get() != ClientState::World || !runtime.hp.is_some_and(|hp| hp > 0) {
        return;
    }
    let Ok((player, context)) = players.single() else {
        return;
    };
    // The source checks shinies before modal/input locks. Travel/skill movement
    // and loading still block this branch; ordinary walking/jumping do not.
    if !context.ready_for_play || context.dead || context.move_mode != LegacyMoveMode::None {
        return;
    }
    for (shiny, transform, effect) in &shinies {
        if !in_pickup_range(player.translation, transform.translation) {
            continue;
        }
        let request = ffone_protocol::RegisteredGameplayRequest0104::new(
            packet::P_CL2FE_REQ_SHINY_PICKUP,
            ShinyPickupRequest0104 {
                shiny_id: shiny.shiny_id,
            }
            .encode(),
        );
        let result = request
            .map_err(|error| error.to_string())
            .and_then(|request| bridge.send(NetworkCommand::SendRegisteredGameplay0104(request)));
        if let Err(error) = result {
            warn!("Coco pickup could not be sent: {error}");
            break;
        }
        effects.enqueue(TutorialEffectRuntimeCommand::Add {
            effect_id: effect.0,
            placement: TutorialEffectPlacement::World {
                position: transform.translation,
                rotation: transform.rotation,
            },
            scale: 1.0,
            tracked: false,
            name: None,
            destroy_after_seconds: None,
            source_line: line!(),
        });
        audio.queue_gameplay_ui_sound("Egg_Pickup");
        let id = shiny.shiny_id;
        // Removes the registry entry as well as its scene. Server EXIT is then
        // idempotent and a subsequent NEW/ENTER can respawn the same shiny ID.
        commands.queue(move |world: &mut World| despawn_shiny(world, id));
    }
}

fn in_pickup_range(player: Vec3, egg: Vec3) -> bool {
    // Authored radius 0.5; CheckShiny uses radius * 2, in full 3D.
    player.is_finite() && egg.is_finite() && player.distance_squared(egg) <= 1.0
}

#[cfg(test)]
mod tests;
