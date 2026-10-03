use crate::world_combat::*;

#[test]
fn hitscan_preserves_selected_order_and_protocol_limit() {
    let entities = (1..=4).map(Entity::from_bits).collect::<Vec<_>>();
    let targets = entities
        .iter()
        .copied()
        .enumerate()
        .map(|(index, entity)| LegacyAttackTarget {
            entity,
            kind: LegacyTargetKind::Npc { team: 2 },
            distance: index as f32,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        build_world_primary_attack_0104(
            LegacyWeaponTargetMode::Normal,
            &Transform::IDENTITY,
            10.0,
            &targets,
            |entity| Some(entity.to_bits() as i32),
        ),
        Some(WorldPrimaryAttack0104::Hitscan(PcAttackNpcsRequest0104 {
            npc_ids: vec![1, 2, 3],
        }))
    );
}

#[test]
fn special_weapon_destinations_use_native_to_server_axis_conversion() {
    let transform = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(
        build_world_primary_attack_0104(
            LegacyWeaponTargetMode::Rocket,
            &transform,
            4.0,
            &[],
            |_| None,
        ),
        Some(WorldPrimaryAttack0104::Rocket(
            PcRocketStyleFireRequest0104 {
                skill_id: 0,
                position: [-100, 300, 200],
                destination: [-100, -100, 200],
            }
        ))
    );
    assert_eq!(
        build_world_primary_attack_0104(
            LegacyWeaponTargetMode::Grenade,
            &transform,
            4.0,
            &[],
            |_| None,
        ),
        Some(WorldPrimaryAttack0104::Grenade(
            PcGrenadeStyleFireRequest0104 {
                skill_id: 0,
                destination: [-100, -100, 200],
            }
        ))
    );
}

#[test]
fn special_weapon_invalid_geometry_fails_closed() {
    assert_eq!(
        build_world_primary_attack_0104(
            LegacyWeaponTargetMode::Rocket,
            &Transform::IDENTITY,
            f32::NAN,
            &[],
            |_| None,
        ),
        None
    );
}
