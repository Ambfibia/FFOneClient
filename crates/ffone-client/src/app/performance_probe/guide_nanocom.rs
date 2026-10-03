//! Offline regression through the production schedule and UI. Opt in with
//! FFONE_PERF_GUIDE_NANOCOM alongside FFONE_PERF_OUTPUT.
use super::*;

#[derive(Default, Resource)]
struct Probe {
    frame: u32,
    paused_remaining: f32,
    request_id: u64,
}

pub(super) fn install(app: &mut App) {
    app.init_resource::<Probe>()
        .add_systems(
            Update,
            drive
                .after(consume_world_gameplay_ui_outbox)
                .before(social_ingress::sync_buddy_nanocom_context),
        )
        .add_systems(Last, verify.before(measure));
}

fn drive(
    capture: Res<Capture>,
    mut probe: ResMut<Probe>,
    content: Res<TutorialMissionContent>,
    mut guide: ResMut<GuideRuntime>,
    mut messages: ResMut<NanocomMessageUiModel>,
    mut mission: ResMut<MissionUiModel>,
    mut outbox: ResMut<GameplayUiOutbox>,
) {
    if capture.ready.is_none() || capture.samples.is_empty() {
        return;
    }
    probe.frame += 1;
    match probe.frame {
        1 => {
            let mut load = ffone_protocol::PcLoadData0104::zeroed();
            let offset = ffone_protocol::PcLoadData0104::MENTOR_OFFSET;
            load.as_bytes_mut()[offset..offset + 2].copy_from_slice(&5_i16.to_le_bytes());
            guide.load_pc_state(&load);
            messages.clear();
            assert!(crate::app::guide_nanocom::enqueue_level_up(
                &content,
                &guide,
                4,
                5,
                &mut messages
            ));
            probe.request_id = messages.active().unwrap().request.request_id;
        }
        60 => {
            probe.paused_remaining = messages.active().unwrap().remaining_seconds;
            let npc = content.gameplay_npc(730).unwrap();
            mission.show_npc_interaction(NpcInteractionUi {
                npc_id: 730,
                npc_type: 730,
                name: npc.name.clone(),
                available_missions: vec![],
                completed_missions: vec![],
                services: vec![],
                warp: None,
            });
        }
        180 => {
            mission.close_npc_interaction(&mut outbox);
        }
        _ => {}
    }
}

fn verify(
    mut commands: Commands,
    capture: Res<Capture>,
    probe: Res<Probe>,
    messages: Res<NanocomMessageUiModel>,
) {
    if capture.ready.is_none() || probe.frame == 0 || probe.frame > 210 {
        return;
    }
    let active = messages.active().unwrap_or_else(|| {
        panic!("guide notice missing at fixture frame {}, suspended={}", probe.frame, messages.scene_event_active())
    });
    assert_eq!(active.request.request_id, probe.request_id);
    if (60..180).contains(&probe.frame) {
        assert!(messages.scene_event_active());
        assert!(!messages.compact_visible());
        assert_eq!(active.remaining_seconds, probe.paused_remaining);
    } else {
        assert!(messages.compact_visible());
    }
    let name = match probe.frame {
        30 => "guide-before-dialogue",
        90 => "guide-dialogue-open",
        210 => "guide-after-dialogue",
        _ => return,
    };
    commands.spawn(Screenshot::primary_window()).observe(
        bevy::render::view::screenshot::save_to_disk(capture.output.join(format!("{name}.png"))),
    );
    fs::write(
        capture.output.join(format!("{name}.json")),
        serde_json::to_vec_pretty(&serde_json::json!({
            "requestId": probe.request_id,
            "remainingSeconds": active.remaining_seconds,
            "suspended": messages.scene_event_active(),
            "visible": messages.compact_visible(),
            "bodyKey": active.request.compact_body_localized(active.remaining_seconds).key,
        }))
        .unwrap(),
    )
    .unwrap();
}
