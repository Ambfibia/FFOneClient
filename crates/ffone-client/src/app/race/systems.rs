use super::*;

pub(in super::super) fn sync_race_hud(
    time: Res<Time>,
    race: Res<RaceProductionRuntime>,
    mode: Res<RaceModeModel>,
    mut hud: ResMut<RaceHudState>,
) {
    let running = race.player.ring_race_active;
    let elapsed = (time.elapsed_secs() - race.player.local_start_time).max(0.0);
    let next = RaceHudState {
        awaiting_start: mode.phase() == RaceModePhase::AwaitingStart && !running,
        running,
        elapsed_seconds: if running { elapsed.floor() as i32 } else { 0 },
        remaining_seconds: if running {
            (race.player.race_limit_time as f32 - elapsed).ceil().max(0.0) as i32
        } else { 0 },
        pods: if running { race.player.ring_count } else { 0 },
        confirmed_pickups: if running { race.collected_rings.len() } else { 0 },
    };
    if *hud != next { *hud = next; }
}

pub(in super::super) fn cancel_expired_race(
    hud: Res<RaceHudState>,
    mut model: ResMut<RaceModeModel>,
    mut race: ResMut<RaceProductionRuntime>,
    mut status: ResMut<RuntimeStatus>,
) {
    if !hud.running || hud.remaining_seconds > 0
        || model.phase() != RaceModePhase::Hidden || race.active_game_mode.is_some() {
        return;
    }
    let result = model.open(RaceModeOpenContext {
        ecom_type: RaceEcomType::Start,
        npc: None,
        player: race.player,
        current_ep_instance_exists: false,
    });
    if let Err(error) = result {
        status.message = format!("Race timeout cancel rejected: {error:?}");
        return;
    }
    if let Err(error) = race.begin_mode(RACE_MODE_GAME_MODE_ID, 0, 0, String::new()) {
        *model = RaceModeModel::default();
        status.message = format!("Race timeout cancel lifecycle rejected: {error}");
    }
}

pub(in super::super) fn sync_race_presentation_input(
    system_messages: Res<SystemMessageUiModel>,
    mission_ui: Res<MissionUiModel>,
    mut mode_input: ResMut<RaceModePresentationInput>,
    mut rank_input: ResMut<RaceRankPresentationInput>,
) {
    let system_popup_active = system_messages.is_popup() || mission_ui.system_popup_active();
    mode_input.system_popup_active = system_popup_active;
    rank_input.system_popup_active = system_popup_active;
}

pub(in super::super) fn sync_race_world_rings(
    mut race: ResMut<RaceProductionRuntime>,
    triggers: Query<(&ffone_client::world_behaviour::WorldTrigger, &GlobalTransform, &Name)>,
    players: Query<&Transform, With<LocalPlayer>>,
    mut visibility: Query<&mut Visibility>,
    mut intents: ResMut<WorldGameplayIntentQueue>,
) {
    let player = players.single().ok().map(|t| t.translation);
    let active = race.player.ring_race_active;
    for (trigger, global, name) in &triggers {
        if trigger.kind != ffone_client::world_behaviour::WorldTriggerKind::Ring { continue; }
        let position = global.translation();
        let raw = ProtocolPosition::from_native(position).raw();
        let in_course = race.course_bounds.is_some_and(|[xmin,xmax,ymin,ymax]|
            raw[0] >= xmin && raw[0] < xmax && raw[1] >= ymin && raw[1] < ymax);
        let next_id = race.ring_ids.len() as i32 + 1;
        let id = *race.ring_ids.entry((raw, name.as_str().to_owned())).or_insert(next_id);
        let mut shown = active && in_course && trigger.server_id > 0
            && !race.collected_rings.contains(&id) && !race.pending_rings.contains(&id);
        let center = global.transform_point(trigger.start_position);
        if shown && player.is_some_and(|p| {
            let radius = ffone_client::world::AUTHORED_CHARACTER_CONTROLLER_RADIUS;
            let height = ffone_client::world::AUTHORED_CHARACTER_CONTROLLER_HEIGHT;
            let capsule_point = Vec3::new(p.x, center.y.clamp(p.y + radius, p.y + height - radius), p.z);
            capsule_point.distance_squared(center) <= (trigger.radius + radius).powi(2)
        }) {
            let payload = id.to_le_bytes().to_vec();
            if intents.push_payload(ffone_protocol::packet::P_CL2FE_REQ_EP_GET_RING, payload) {
                race.pending_rings.insert(id);
                shown = false;
            }
        }
        for &entity in &trigger.model_entities {
            if let Ok(mut v) = visibility.get_mut(entity) {
                let next = if shown { Visibility::Inherited } else { Visibility::Hidden };
                if *v != next { *v = next; }
            }
        }
    }
    race.ring_activation_requested = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn course_info_alone_does_not_show_ready_and_race_time_increases() {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<RaceProductionRuntime>()
            .init_resource::<RaceModeModel>()
            .init_resource::<RaceHudState>()
            .add_systems(Update, sync_race_hud);
        app.world_mut().resource_mut::<RaceProductionRuntime>().course_bounds = Some([0, 100, 0, 100]);
        app.update();
        assert!(!app.world().resource::<RaceHudState>().awaiting_start);
        app.world_mut().resource_mut::<RaceModeModel>().open(RaceModeOpenContext {
            ecom_type: RaceEcomType::Start,
            npc: Some(RaceNpcContext { instance_id: 9001, has_race_start_voice: false }),
            player: RacePlayerState::default(),
            current_ep_instance_exists: true,
        }).unwrap();
        app.update();
        assert!(app.world().resource::<RaceHudState>().awaiting_start);
        *app.world_mut().resource_mut::<RaceModeModel>() = RaceModeModel::default();
        {
            let mut race = app.world_mut().resource_mut::<RaceProductionRuntime>();
            race.player.ring_race_active = true;
            race.player.local_start_time = 0.0;
            race.player.race_limit_time = 120;
        }
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(10));
        app.update();
        assert_eq!(app.world().resource::<RaceHudState>().elapsed_seconds, 10);
        assert_eq!(app.world().resource::<RaceHudState>().remaining_seconds, 110);
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs(5));
        app.update();
        assert_eq!(app.world().resource::<RaceHudState>().elapsed_seconds, 15);
        app.world_mut().resource_mut::<RaceProductionRuntime>().player.ring_race_active = false;
        app.update();
        let hud = app.world().resource::<RaceHudState>();
        assert!(!hud.running && !hud.awaiting_start);
    }
}
