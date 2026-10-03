//! Offline captures of both production character-creation cutscenes.
use super::*;

#[derive(Resource)]
struct CutsceneProbe {
    output: PathBuf,
    started: Instant,
    tutorial: bool,
    sample: usize,
    pending: usize,
    last_status: String,
}

pub(super) fn install(app: &mut App, output: PathBuf) {
    fs::create_dir_all(&output).unwrap();
    app.insert_resource(CutsceneProbe {
        output,
        started: Instant::now(),
        tutorial: false,
        sample: 0,
        pending: 0,
        last_status: String::new(),
    })
    .insert_resource(bevy::winit::WinitSettings::continuous())
    .add_systems(PostStartup, |mut next: ResMut<NextState<ClientState>>| {
        next.set(ClientState::CharacterCreateIntro);
    })
    .add_systems(Last, capture);
}

fn capture(world: &mut World) {
    let actors = world
        .query::<&DexterShipActor>()
        .iter(world)
        .map(|actor| format!("{:?}:{}", actor.role, actor.clip))
        .collect::<Vec<_>>();
    let surfaces = world
        .query_filtered::<Entity, With<LegacyMaterialApplied>>()
        .iter(world)
        .count();
    let bound = world
        .query_filtered::<Entity, With<NetworkNpcTextureVariantBound0104>>()
        .iter(world)
        .count();
    let fallback = world
        .query_filtered::<Entity, With<MeshMaterial3d<StandardMaterial>>>()
        .iter(world)
        .count();
    let ready_scenes = world
        .query_filtered::<Entity, With<DexterShipSceneReady>>()
        .iter(world)
        .count();
    let loading = world.resource::<GameplayLoadingState>();
    let status = format!(
        "{:?} {:?}/{:?} blocked={:?} scenes={ready_scenes} materials={surfaces}/{bound} fallback={fallback} actors={actors:?}",
        world.resource::<State<ClientState>>().get(),
        loading.scope,
        loading.phase,
        loading.blocked
    );
    if world.resource::<CutsceneProbe>().last_status != status {
        println!("cutscene-probe {status}");
        let mut probe = world.resource_mut::<CutsceneProbe>();
        fs::write(probe.output.join("status.txt"), &status).unwrap();
        probe.last_status = status;
    }
    let probe = world.resource::<CutsceneProbe>();
    assert!(
        probe.started.elapsed() < Duration::from_secs(180),
        "cutscene capture timed out"
    );
    let tutorial = probe.tutorial;
    let sample = probe.sample;
    let pending = probe.pending;
    let output = probe.output.clone();
    let times: &[f32] = if tutorial {
        &[1.0, 19.0, 26.9, 27.8, 29.5]
    } else {
        &[3.0, 8.0, 16.4, 16.8]
    };
    if sample == times.len() {
        if pending != 0 {
            return;
        }
        if tutorial {
            fs::write(output.join("passed.txt"), "Both production cutscenes: table-selected actors, textures and event clips loaded; captures complete.\n").unwrap();
            world.write_message(AppExit::Success);
        } else {
            let mut probe = world.resource_mut::<CutsceneProbe>();
            probe.tutorial = true;
            probe.sample = 0;
            world
                .resource_mut::<NextState<ClientState>>()
                .set(ClientState::TutorialIntro);
        }
        return;
    }
    let expected_state = if tutorial {
        ClientState::TutorialIntro
    } else {
        ClientState::CharacterCreateIntro
    };
    if *world.resource::<State<ClientState>>().get() != expected_state {
        return;
    }
    let cutscene = world.resource::<DexterShipCutsceneRuntime>();
    if !cutscene.presentation_ready || cutscene.elapsed < times[sample] {
        return;
    }
    let elapsed = cutscene.elapsed;
    let mut playback = Vec::new();
    for (player, graph) in world
        .query::<(&AnimationPlayer, &AnimationGraphHandle)>()
        .iter(world)
    {
        for (node, active) in player.playing_animations() {
            let graphs = world.resource::<Assets<AnimationGraph>>();
            let clips = world.resource::<Assets<AnimationClip>>();
            let duration = graphs.get(graph).and_then(|graph| {
                if let bevy::animation::graph::AnimationNodeType::Clip(handle) =
                    &graph[*node].node_type
                {
                    clips.get(handle).map(AnimationClip::duration)
                } else {
                    None
                }
            });
            playback.push(serde_json::json!({
                "seek": active.seek_time(), "duration": duration,
                "repeat": format!("{:?}", active.repeat_mode()),
                "completions": active.completions(), "finished": active.is_finished(),
            }));
        }
    }
    let mut actors = world.query::<(&DexterShipActor, &NpcSceneTextureOverrides0104)>();
    let diagnostics: Vec<_> = actors
        .iter(world)
        .map(|(actor, textures)| {
            let asset_server = world.resource::<AssetServer>();
            if actor.role == DexterShipActorRole::Dexter {
                assert_eq!(actor.clip, dexter_ship_dexter_clip(!tutorial, elapsed));
            }
            serde_json::json!({
                "role": format!("{:?}", actor.role),
                "npcType": textures.npc_type,
                "model": asset_server.get_path(actor.gltf.id()).unwrap().to_string(),
                "clip": actor.clip,
                "mainTexture": textures.main_texture.as_ref().map(|texture| &texture.path),
                "subTexture": textures.sub_texture.as_ref().map(|texture| &texture.path),
            })
        })
        .collect();
    assert_eq!(diagnostics.len(), 3);
    let stem = format!(
        "{}-{sample}",
        if tutorial {
            "after-creation"
        } else {
            "before-creation"
        }
    );
    fs::write(
        output.join(format!("{stem}.json")),
        serde_json::to_vec_pretty(
            &serde_json::json!({"elapsed": elapsed, "actors": diagnostics, "playback": playback}),
        )
        .unwrap(),
    )
    .unwrap();
    println!("cutscene-probe {stem} elapsed={elapsed:.3}");
    // The 512px cinematic camera target remains valid when Windows minimizes
    // the main window (whose screenshot can otherwise be only one pixel).
    let hologram = world
        .query_filtered::<&MaterialNode<DexterHologramMaterial>, With<DexterShipHologram>>()
        .single(world)
        .unwrap()
        .0
        .clone();
    let camera_target = world
        .resource::<Assets<DexterHologramMaterial>>()
        .get(&hologram)
        .unwrap()
        .scene_texture
        .clone();
    {
        let mut probe = world.resource_mut::<CutsceneProbe>();
        probe.sample += 1;
        probe.pending += 2;
    }
    world
        .spawn(Screenshot::image(camera_target))
        .observe(bevy::render::view::screenshot::save_to_disk(
            output.join(format!("{stem}-actors.png")),
        ))
        .observe(
            |event: On<ScreenshotCaptured>, mut probe: ResMut<CutsceneProbe>| {
                assert_eq!(event.image.width(), DEXTER_SHIP_RENDER_TEXTURE_SIZE);
                assert_eq!(event.image.height(), DEXTER_SHIP_RENDER_TEXTURE_SIZE);
                probe.pending -= 1;
            },
        );
    world
        .spawn(Screenshot::primary_window())
        .observe(bevy::render::view::screenshot::save_to_disk(
            output.join(format!("{stem}.png")),
        ))
        .observe(
            |_: On<ScreenshotCaptured>, mut probe: ResMut<CutsceneProbe>| {
                probe.pending -= 1;
            },
        );
}
