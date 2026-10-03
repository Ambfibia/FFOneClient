//! Real renderer acceptance of local ability poses and localized stun audio.
use super::*;
use ffone_protocol::PcBuffUpdate0104;
use ffone_client::tutorial_player_rig_runtime::TutorialPlayerAnimationApplied;

#[derive(Resource, Default)]
struct Probe { frame:u32, records:Vec<serde_json::Value>, voices:BTreeSet<String>, saw_dash:bool, saw_stun:bool }

pub(super) fn install(app:&mut App) {
    app.init_resource::<Probe>().add_systems(Update, drive
        .after(poll_network).before(movement_buffs::sync_movement_buffs))
        .add_systems(Last, record.before(super::measure));
}
fn drive(
    mut commands:Commands, capture:Res<Capture>, mut probe:ResMut<Probe>,
    mut players:Query<&mut LegacyPlayerController,With<LocalPlayer>>,
    mut buffs:ResMut<SkillBuffUiModel>, mut movement_buffs:ResMut<movement_buffs::MovementBuffs>,
) {
    if capture.ready.is_none() || capture.samples.is_empty() { return; }
    probe.frame+=1;
    match probe.frame {
        60 => { for mut controller in &mut players { controller.launch_nano_dash(); } },
        180 | 420 => {
            let flags=if probe.frame==180 {0x200}else{0};
            let packet=PcBuffUpdate0104 {buff_id:10,update_kind:if flags==0 {2}else{1},buff_type:0,
                time_buff:ffone_protocol::TimeBuff0104::default(),condition_bit_flag:flags};
            apply_skill_buff_frame(&DecodedFrame {packet_type:packet::P_FE2CL_PC_BUFF_UPDATE,flags:0,checksum:0,payload:packet.encode()}, &mut buffs,&mut movement_buffs).unwrap();
        },
        _=>{},
    }
    if matches!(probe.frame,65|80|185|200|230|430) {
        commands.spawn(Screenshot::primary_window()).observe(bevy::render::view::screenshot::save_to_disk(
            capture.output.join(format!("ability-{}.png",probe.frame))));
    }
}
fn record(
    capture:Res<Capture>,mut probe:ResMut<Probe>,
    poses:Query<&TutorialPlayerAnimationApplied>,players:Query<&AnimationPlayer>,
    voices:Query<&LocalizedVoice>,controllers:Query<(&Transform,&LegacyAvatarActionState),With<LocalPlayer>>,
) {
    if probe.frame==0 || probe.frame>580 {return;}
    for voice in &voices { if voice.true_name.to_ascii_lowercase().contains("stun") {probe.voices.insert(voice.true_name.clone());} }
    let frame=probe.frame;
    for applied in &poses {
        if frame==65 { assert!(matches!(applied.clip,TutorialPlayerClip::StickDash|TutorialPlayerClip::RifleDash|TutorialPlayerClip::RifleTumbling));probe.saw_dash=true; }
        if frame==185 {assert_eq!(applied.clip,TutorialPlayerClip::Stun);probe.saw_stun=true;}
        if frame%5==0 {
            let seek=players.get(applied.animation_player).ok().and_then(|p| p.animation(applied.animation_node)).map(|p|p.seek_time());
            probe.records.push(serde_json::json!({"frame":frame,"clip":applied.clip.name(),"seek":seek,
                "player":controllers.iter().map(|(t,s)|serde_json::json!({"position":t.translation.to_array(),"pose":format!("{:?}",s.locomotion)})).collect::<Vec<_>>()}));
        }
    }
    if frame==580 {
        assert!(probe.saw_dash && probe.saw_stun,"ability clips never reached the rig");
        assert!(!probe.voices.is_empty(),"stun animation never produced localized audio");
        fs::write(capture.output.join("ability-presentation.json"),serde_json::to_vec_pretty(&serde_json::json!({
            "dash":probe.saw_dash,"stun":probe.saw_stun,"voices":probe.voices,"frames":probe.records})).unwrap()).unwrap();
    }
}
