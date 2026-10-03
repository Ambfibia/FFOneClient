//! Offline acceptance of the real client's egg lifecycle and proximity pickup.
use super::*;
use ffone_client::entity_lifecycle::{NetworkSessionEpoch0104, NetworkShinyRegistry0104};

pub(super) fn install(app: &mut App) {
    app.add_systems(Update, drive.before(consume_network_entity_lifecycle_0104));
}

fn drive(
    mut commands: Commands,
    capture: Res<Capture>,
    mut frame: Local<u32>,
    players: Query<&Transform, With<LocalPlayer>>,
    registry: Res<NetworkShinyRegistry0104>,
    mut ingress: ResMut<NetworkEntityLifecycleIngress0104>,
    mut notices: ResMut<ffone_client::gameplay_ui::rewards::RewardNotices>,
    content: Res<TutorialMissionContent>,
    localization: Res<Localization>,
    language: Res<Language>,
) {
    if capture.ready.is_none() || capture.samples.is_empty() {
        return;
    }
    let Ok(player) = players.single() else {
        return;
    };
    *frame += 1;
    let epoch = NetworkSessionEpoch0104(capture.entry as u64);
    let mut egg = |id, kind, position| {
        ingress.push_frame(
            epoch,
            DecodedFrame {
                packet_type: packet::P_FE2CL_SHINY_ENTER,
                flags: 0,
                checksum: 0,
                payload: ffone_protocol::ShinyAppearance0104 {
                    shiny_id: id,
                    shiny_type: kind,
                    map_number: 0,
                    position: ProtocolPosition::from_native(position).raw(),
                }
                .encode(),
            },
        )
    };
    match *frame {
        1 => {
            // The ordinary performance fixture is deliberately disconnected.
            drop(egg);
            ingress.begin_session(epoch, 1);
            for (id, kind, offset) in [
                (-1_900_001, 17, Vec3::new(2.0, 0.0, 2.0)),
                (-1_900_002, 9, Vec3::new(-2.0, 0.0, 2.0)),
            ] {
                ingress.push_frame(
                    epoch,
                    DecodedFrame {
                        packet_type: packet::P_FE2CL_SHINY_ENTER,
                        flags: 0,
                        checksum: 0,
                        payload: ffone_protocol::ShinyAppearance0104 {
                            shiny_id: id,
                            shiny_type: kind,
                            map_number: 0,
                            position: ProtocolPosition::from_native(player.translation + offset)
                                .raw(),
                        }
                        .encode(),
                    },
                );
            }
        }
        120 => {
            assert_eq!(
                registry.len(),
                2,
                "both native egg types must survive outside pickup range"
            );
            egg(-1_900_001, 17, player.translation + Vec3::X * 0.5);
        }
        150 => {
            assert!(
                registry.get(-1_900_001).is_none(),
                "production proximity pickup did not remove the egg"
            );
            assert!(
                registry.get(-1_900_002).is_some(),
                "pickup removed the other egg"
            );
            notices
                .receive_shiny(
                    &ffone_protocol::wire_0104::ShinyPickupSuccess0104 {
                        skill_id: 183,
                        cstb: 0,
                    },
                    &content,
                    &localization,
                    &language,
                )
                .expect("server reward projection");
        }
        210 => egg(
            -1_900_001,
            17,
            player.translation + Vec3::new(2.0, 0.0, 2.0),
        ),
        240 => {
            assert_eq!(
                registry.len(),
                2,
                "authoritative respawn must restore the picked-up egg"
            );
            fs::write(capture.output.join("coco-lifecycle.json"),serde_json::to_vec_pretty(&serde_json::json!({
                "nativeTypes":[17,9],"proximityPickup":true,"otherEggRetained":true,"serverRespawn":true,
                "rewardProjection":true,"authority":"offline fixture","networkTested":false,
            })).unwrap()).unwrap();
        }
        _ => {}
    }
    if matches!(*frame, 100 | 135 | 180 | 260) {
        commands.spawn(Screenshot::primary_window()).observe(
            bevy::render::view::screenshot::save_to_disk(
                capture.output.join(format!("coco-{}.png", *frame)),
            ),
        );
    }
}
