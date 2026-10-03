//! Offline remote-PC ingress for real pointer/menu acceptance in the full client.
use super::*;
use ffone_client::entity_lifecycle::NetworkSessionEpoch0104;

pub(super) fn install(app: &mut App) {
    app.add_systems(Update, (|bridge: Res<NetworkBridge>| { let _ = bridge.drain(); }).before(poll_network))
        .add_systems(PostUpdate, pose.before(bevy::transform::TransformSystems::Propagate))
        .add_systems(Last, drive.before(measure));
}

fn pose(players: Query<&Transform, With<LocalPlayer>>, mut cameras: Query<&mut Transform, (With<Camera3d>, Without<LocalPlayer>)>) {
    let Ok(player) = players.single() else { return; };
    for mut camera in &mut cameras {
        *camera = Transform::from_translation(player.translation + Vec3::new(0., 3., 7.))
            .looking_at(player.translation + Vec3::new(0., 1., -1.), Vec3::Y);
    }
}

fn drive(world: &mut World, mut opened: Local<bool>) {
    if world.resource::<Capture>().ready.is_none() || world.resource::<GameplayLoadingState>().visible { return; }
    world.resource_mut::<Capture>().samples.clear();
    let player = world.query_filtered::<&Transform, With<LocalPlayer>>().single(world).unwrap().translation;
    if !*opened {
        let mut appearance = ffone_protocol::PcAppearance0104::decode(&vec![0; ffone_protocol::PcAppearance0104::SIZE]).unwrap();
        appearance.id = 82;
        appearance.style = ffone_protocol::PcStyle0104 {
            pc_uid: 820, name_check: 1,
            first_name: ffone_protocol::FixedUtf16::from_str("Remote").unwrap(),
            last_name: ffone_protocol::FixedUtf16::from_str("Player").unwrap(),
            gender: 1, face_style: 1, hair_style: 1, hair_color: 1, skin_color: 1,
            eye_color: 1, height: 1, body: 1, class: 0,
        };
        appearance.level = 12;
        appearance.hp = 700;
        appearance.pc_state = 1;
        appearance.render_type = 1;
        appearance.map_number = 1;
        appearance.position = ProtocolPosition::from_native(player + Vec3::new(2., 0., -2.)).raw();
        let epoch = NetworkSessionEpoch0104(1);
        let mut ingress = world.resource_mut::<NetworkEntityLifecycleIngress0104>();
        ingress.begin_session(epoch, 1);
        ingress.push_frame(epoch, DecodedFrame { packet_type: packet::P_FE2CL_PC_NEW,
            flags: 0, checksum: 0, payload: appearance.encode() });
        *opened = true;
        println!("PLAYER MENU: remote 82, level 12, HP 700; click the remote player, then choose an action");
    }
}
