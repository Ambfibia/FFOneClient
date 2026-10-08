//! Bug 8 acceptance through the production cannon, traversal and renderer.
use super::*;
use ffone_runtime_contracts::PlayerRigGender;
use ffone_client::{launcher_ui::LauncherUiModel, world_behaviour::{
    WorldTrigger,WorldTriggerKind,WorldTriggerUseQueue,WorldZiplineTraversal,
    WorldGameplayIntentQueue,update_world_launcher_traversals,
}, tutorial_player_rig_runtime::{TutorialPlayerAnimationApplied,TutorialPlayerWeaponAttachment}};

#[derive(Resource,Default)]
struct Probe { frame:u32, records:Vec<serde_json::Value>, packets:Vec<serde_json::Value>, audio:std::collections::BTreeSet<String>, saw_flight:bool, saw_zipline:bool }

pub(super) fn install(app:&mut App) {
    app.init_resource::<Probe>()
        .add_systems(FixedUpdate, aim.before(super::super::launcher::read_launcher_configured_aim))
        .add_systems(Update, discard_network.before(poll_network))
        .add_systems(Update, keys.before(LegacyMovementSet::ReadInput).before(super::super::launcher::drive_launcher_production))
        .add_systems(Update, collect_packets.after(update_world_launcher_traversals)
            .after(ffone_client::world_behaviour::finish_world_zipline_steps)
            .after(NativeWorldSet::ResolveCollision).before(flush_movement_intents).before(flush_world_gameplay_intents))
        .add_systems(Last, drive.before(super::measure));
}

fn aim(probe:Res<Probe>,mut keys:ResMut<ButtonInput<KeyCode>>) {
    if (90..110).contains(&probe.frame) { keys.press(KeyCode::KeyW);keys.press(KeyCode::KeyD); }
    if (660..715).contains(&probe.frame) { keys.press(KeyCode::KeyW);keys.press(KeyCode::KeyA); }
}
fn keys(probe:Res<Probe>,mut keys:ResMut<ButtonInput<KeyCode>>) {
    // Exercise the actual configurable Space/Escape production readers.
    if matches!(probe.frame,130|350|735) { keys.press(KeyCode::Space); }
    if matches!(probe.frame,165|770) { keys.press(KeyCode::Space); keys.release(KeyCode::Space); }
    if probe.frame==625 { keys.press(KeyCode::Escape); }
}
fn discard_network(bridge:Res<NetworkBridge>) {
    for event in bridge.drain() {
        assert!(matches!(event,ffone_client::network::NetworkEvent::Error(_)),"offline fixture received live event");
    }
}
fn collect_packets(mut probe:ResMut<Probe>,mut gameplay:ResMut<WorldGameplayIntentQueue>,mut movement:ResMut<ffone_client::movement::MovementIntentQueue>) {
    movement.clear();
    for request in gameplay.take_all() {
        let record=match request.packet_type() {
            packet::P_CL2FE_REQ_PC_LAUNCHER => {
                let r=ffone_protocol::PcLauncherRequest0104::decode(request.payload()).unwrap();
                serde_json::json!({"kind":"launcher","position":r.position,"velocity":r.velocity,"speed":r.speed,"angle":r.angle})
            },
            packet::P_CL2FE_REQ_PC_ZIPLINE => {
                let r=ffone_protocol::PcZiplineRequest0104::decode(request.payload()).unwrap();
                serde_json::json!({"kind":"zipline","position":r.position,"velocity":r.velocity,"down":r.down,"distance":r.moved_distance,"max":r.maximum_distance})
            },
            _=>continue,
        };
        probe.packets.push(record);
    }
}
fn seed_weapon(world:&mut World,id:i16) {
    let mut bytes=vec![0;ffone_protocol::PcLoadData0104::SIZE];
    let offset=ffone_protocol::PcLoadData0104::EQUIPMENT_OFFSET;
    bytes[offset+2..offset+4].copy_from_slice(&id.to_le_bytes());
    bytes[offset+4..offset+8].copy_from_slice(&1_i32.to_le_bytes());
    world.resource_mut::<LocalInventoryRuntime>().seed(1,&ffone_protocol::PcLoadData0104::decode(&bytes).unwrap());
}
fn use_trigger(world:&mut World,cne_id:i64) {
    let trigger=world.query::<(Entity,&WorldTrigger)>().iter(world).find(|(_,t)|t.cne_id==cne_id
        && matches!(t.kind,WorldTriggerKind::Launcher|WorldTriggerKind::Zipline)).map(|(e,_)|e).expect("control trigger loaded");
    let actor=world.query_filtered::<Entity,With<LocalPlayer>>().single(world).unwrap();
    world.resource_mut::<WorldTriggerUseQueue>().push(actor,trigger);
}
fn drive(world:&mut World) {
    let initialized=world.resource::<Probe>().frame>0;
    if !initialized && world.resource::<Capture>().ready.is_none() { return; }
    world.resource_mut::<Capture>().samples.clear();
    let mut probe=world.remove_resource::<Probe>().unwrap();
    probe.frame+=1;
    let frame=probe.frame;
    let sounds=world.query::<&AudioPlayer>().iter(world).filter_map(|player|
        world.resource::<AssetServer>().get_path(player.0.id()).map(|path|path.path().to_string_lossy().into_owned())).collect::<Vec<_>>();
    probe.audio.extend(sounds);
    match frame {
        1=>seed_weapon(world,0),
        20=>seed_weapon(world,328),
        60=>use_trigger(world,278),
        310|410=>{
            // Seed a genuine attack before each ride; traversal must discard it.
            let actor=world.query_filtered::<Entity,With<LocalPlayer>>().single(world).unwrap();
            let mut state=world.get_mut::<LegacyAvatarActionState>(actor).unwrap();
            state.upper_action=Some(ffone_client::avatar_action::LegacyVisualClip::AttackUpper(1));
            world.resource_mut::<ffone_client::tutorial_player_presentation::TutorialPlayerPresentationCommandQueue>()
                .push_animation(ffone_client::tutorial_player_presentation::TutorialPlayerAnimationRequest::runtime_cross_fade(
                    PlayerRigGender::Male,TutorialPlayerClip::RifleAttack1Upper));
            use_trigger(world,277);
        },
        330=>seed_weapon(world,43),
        600|650=>use_trigger(world,278),
        _=>{},
    }
    let pose=world.query::<&TutorialPlayerAnimationApplied>().iter(world).next().map(|p|p.clip);
    if pose==Some(TutorialPlayerClip::Launcher) {probe.saw_flight=true;}
    if pose==Some(TutorialPlayerClip::RopeDown) {probe.saw_zipline=true;}
    if frame%5==0 {
        let player=world.query_filtered::<(&Transform,&LegacyPlayerController,&LegacyAvatarActionState,Option<&WorldZiplineTraversal>),With<LocalPlayer>>()
            .iter(world).map(|(t,c,s,z)|serde_json::json!({"position":t.translation.to_array(),"yaw":c.yaw_degrees,"flight":c.launcher_active(),"upper":format!("{:?}",s.upper_action),"zipline":z.is_some()})).collect::<Vec<_>>();
        let weapons=world.query::<(&TutorialPlayerWeaponAttachment,&Visibility)>().iter(world).map(|(w,v)|serde_json::json!({"id":w.item_id,"visibility":format!("{v:?}")})).collect::<Vec<_>>();
        let cannon=world.query::<&WorldTrigger>().iter(world).find(|t|t.cne_id==278)
            .map(|t|t.model_entities.clone()).unwrap_or_default();
        let cannon_visibility=cannon.iter().map(|e|format!("{:?}",world.get::<Visibility>(*e))).collect::<Vec<_>>();
        probe.records.push(serde_json::json!({"frame":frame,"clip":pose.map(|p|p.name()),"player":player,"weapons":weapons,"cannon":cannon_visibility,"aim":world.resource::<LauncherUiModel>().current_rotation_degrees.to_array()}));
    }
    if matches!(frame,55|75|115|155|175|205|270|320|340|360|425|520|555|615|640|725|780|830|900|1000|1100) {
        let output=world.resource::<Capture>().output.join(format!("traversal-{frame}.png"));
        world.spawn(Screenshot::primary_window()).observe(bevy::render::view::screenshot::save_to_disk(output));
    }
    if frame==1150 {
        let output=world.resource::<Capture>().output.join("traversal.json");
        fs::write(output,serde_json::to_vec_pretty(&serde_json::json!({"frames":probe.records,"packets":probe.packets,"audio":probe.audio})).unwrap()).unwrap();
        assert!(probe.saw_flight && probe.saw_zipline,"source traversal clips did not reach renderer");
        assert!(probe.records.iter().find(|r|r["frame"]==55).is_some_and(|r|r["clip"]=="rifleready" && r["weapons"][0]["id"]==328),"rifle equip did not play Ready");
        assert!(probe.records.iter().find(|r|r["frame"]==340).is_some_and(|r|r["weapons"][0]["id"]==43 && r["weapons"][0]["visibility"]=="Hidden"),"mid-ride equip exposed the new weapon");
        assert!(probe.records.iter().find(|r|r["frame"]==555).is_some_and(|r|r["player"][0]["zipline"]==false && r["player"][0]["upper"]=="None" && r["weapons"][0]["visibility"]=="Inherited"),"endpoint did not restore weapon or interrupted attack returned");
        assert!(probe.packets.iter().any(|p|p["kind"]=="zipline" && p["down"]==1),"detachment packet absent");
        assert!(probe.packets.iter().any(|p|p["kind"]=="zipline" && p["down"]==0 && p["distance"]==p["max"]),"endpoint packet absent");
        assert!(probe.records.iter().filter(|r|r["frame"].as_u64().is_some_and(|f|(315..=345).contains(&f)||(415..=530).contains(&f)))
            .all(|r|r["player"][0]["upper"]=="None"), "interrupted attack returned on cable");
        assert!(probe.records.iter().filter(|r|r["frame"].as_u64().is_some_and(|f|(75..=155).contains(&f)))
            .all(|r|r["cannon"].as_array().is_some_and(|models|!models.is_empty() && models.iter().all(|v|v=="Some(Hidden)"))), "cannon renderers became visible while aiming");
        assert!(probe.records.iter().find(|r|r["frame"]==640).is_some_and(|r|r["player"][0]["flight"]==false && r["weapons"][0]["visibility"]=="Inherited"),"Escape did not restore avatar/weapon visibility");
        let first=probe.packets.iter().find(|p|p["kind"]=="launcher" && p["velocity"]!=serde_json::json!([0,0,0])).unwrap();
        assert!(probe.packets.iter().any(|p|p["kind"]=="launcher" && p["velocity"]!=serde_json::json!([0,0,0]) && p["angle"]!=first["angle"]), "opposite cannon aim did not change the shot");
        for cue in ["clickon", "startpower", "powerpulse", "stoppower", "firing"] {
            assert!(probe.audio.iter().any(|path|path.ends_with(&format!("launcher_{cue}.ogg"))), "cannon sound {cue} never reached AudioPlayer");
        }
        assert!(probe.audio.iter().any(|path|path.to_ascii_lowercase().contains("zipline")), "zipline loop never reached AudioPlayer");
    }
    if frame==1170 { world.write_message(AppExit::Success); }
    world.insert_resource(probe);
}
