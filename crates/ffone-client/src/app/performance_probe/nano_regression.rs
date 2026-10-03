//! Full-client acquisition fixture: real scene loading, animation, camera and audio.
use super::*;
use ffone_client::nano_free_tuning_ui::*;

#[derive(Resource, Default)]
struct Probe {
    opened: bool,
    selected: bool,
    phase: String,
    frames: u32,
    records: Vec<serde_json::Value>,
    voices: BTreeSet<String>,
    captured: BTreeSet<String>,
}

pub(super) fn install(app: &mut App) {
    app.add_systems(
        Update,
        (|mut resets: ResMut<Messages<NetworkSessionReset>>| {
            // This fixture has no shard: periodic world traffic is deliberately
            // rejected by the offline worker. Keep those synthetic errors from
            // cancelling the scene under observation; real sessions are unaffected.
            resets.clear();
        })
        .after(poll_network)
        .before(NetworkSessionLifecycleSet::Apply),
    );
    app.init_resource::<Probe>().add_systems(
        PostUpdate,
        drive.after(bevy::transform::TransformSystems::Propagate),
    );
}

fn drive(world: &mut World) {
    if world.resource::<Capture>().ready.is_none() {
        return;
    }
    let mut probe = world.remove_resource::<Probe>().unwrap();
    let id: i16 = env::var("FFONE_PERF_NANO_ACQUISITION")
        .unwrap()
        .parse()
        .unwrap();
    if !probe.opened {
        let pc = world.resource::<RuntimeStatus>().player_id.unwrap();
        let load = ffone_protocol::PcLoadData0104::zeroed();
        world
            .resource_mut::<LocalInventoryRuntime>()
            .seed(pc, &load);
        let page = ffone_protocol::wire_0104::NanoBookSubsetReply0104 {
            pcuid: i64::from(pc),
            book_size: 71,
            element_offset: 0,
            element: std::array::from_fn(|_| ffone_protocol::wire_0104::Nano0104 {
                id: 0,
                skill_id: 0,
                stamina: 0,
            }),
        };
        let skill = world
            .resource::<TutorialMissionContent>()
            .journal_nano(i32::from(id))
            .unwrap()
            .skills[0]
            .skill_id as i16;
        let mut bank = world.resource_mut::<NanoFreeTuningBank0104>();
        bank.apply_book_subset(&page, pc).unwrap();
        bank.apply_create_success(ffone_protocol::PcNanoCreateSuccess0104 {
            nano: ffone_protocol::Nano0104 {
                id,
                skill_id: skill,
                stamina: 150,
            },
            fusion_matter: 0,
            quest_item_slot: 0,
            quest_item: ffone_protocol::ItemBase0104 {
                item_type: 0,
                item_id: 0,
                option: 0,
                time_limit: 0,
            },
            player_level: 1,
        })
        .unwrap();
        world
            .resource_mut::<NanoFreeTuningProductionRuntime>()
            .pending_open = Some(NanoFreeTuningOpenTrigger {
            nano_id: id,
            killed_fusion: true,
        });
        probe.opened = true;
    }
    let phase = world.resource::<NanoFreeTuningModel>().phase();
    let phase_name = format!("{phase:?}");
    if probe.phase != phase_name {
        probe.phase = phase_name.clone();
        probe.frames = 0;
    }
    probe.frames += 1;
    for name in world
        .query_filtered::<&Name, With<LocalizedVoice>>()
        .iter(world)
    {
        probe.voices.insert(name.as_str().to_owned());
    }
    if probe.frames == 10 && !probe.captured.contains(&phase_name) {
        let root = world
            .resource::<NanoFreeTuningProductionRuntime>()
            .preview_entity;
        let preview =
            root.and_then(|root| world.get::<GlobalTransform>(root).map(|t| t.translation()));
        let visibility = root.and_then(|root| world.get::<Visibility>(root).copied());
        let player: Vec<_> = world
            .query_filtered::<&Visibility, With<LocalCharacterScene>>()
            .iter(world)
            .map(|v| format!("{v:?}"))
            .collect();
        let animations: Vec<_> = world.query::<(&AnimationPlayer, &NanoFreeTuningPreviewAnimation)>().iter(world).map(|(p,a)| serde_json::json!({"clip":a.clip_name,"time":p.animation(a.node).map(|a|a.seek_time())})).collect();
        probe.records.push(serde_json::json!({"phase":phase_name,"preview":preview,"visibility":format!("{visibility:?}"),"player":player,"animations":animations,"message":world.resource::<RuntimeStatus>().message}));
        if matches!(
            phase,
            NanoFreeTuningPhase::CameraApproach
                | NanoFreeTuningPhase::PowerSelection
                | NanoFreeTuningPhase::ResultSkill
        ) {
            let path = world
                .resource::<Capture>()
                .output
                .join(format!("nano-{id}-{phase_name}.png"));
            world.spawn(Screenshot::primary_window()).observe(
                move |event: On<ScreenshotCaptured>| {
                    event
                        .image
                        .clone()
                        .try_into_dynamic()
                        .unwrap()
                        .save(&path)
                        .unwrap();
                },
            );
        }
        probe.captured.insert(phase_name);
    }
    if phase == NanoFreeTuningPhase::PowerSelection && probe.frames == 40 && !probe.selected {
        let skill = world
            .resource::<TutorialMissionContent>()
            .journal_nano(i32::from(id))
            .unwrap()
            .skills[0]
            .skill_id as i16;
        let mut model = world.resource_mut::<NanoFreeTuningModel>();
        let token = model.click_power(0).unwrap();
        // Offline fixture supplies a server reply; no socket mutation is claimed.
        model.drain_intents().for_each(drop);
        model
            .apply_reply(NanoFreeTuningReplyEnvelope {
                request_token: token,
                packet_id: NANO_TUNE_SUCCESS_PACKET_ID,
                payload_size: NANO_TUNE_SUCCESS_SIZE,
                body: NanoFreeTuningReplyBody::Success(NanoTuneSuccess {
                    nano_id: id,
                    skill_id: skill,
                    fusion_matter: 0,
                    item_slots: [0; NANO_TUNE_ITEM_SLOT_COUNT],
                    items: [NanoTuneItemBase::default(); NANO_TUNE_ITEM_SLOT_COUNT],
                }),
            })
            .unwrap();
        probe.selected = true;
    }
    let path = world
        .resource::<Capture>()
        .output
        .join("nano-acquisition.json");
    fs::write(path, serde_json::to_vec_pretty(&serde_json::json!({"states":probe.records,"voices":probe.voices,"selected":probe.selected})).unwrap()).unwrap();
    world.insert_resource(probe);
}
