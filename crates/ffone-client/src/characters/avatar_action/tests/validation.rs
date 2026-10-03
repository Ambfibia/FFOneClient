use super::*;

#[test]
fn caster_area_nano_does_not_require_a_visible_focus() {
    let ids = entities(2);
    let feed = LegacyAvatarTargetFeed {
        source_connected: true,
        samples: vec![
            LegacyTargetSample {
                entity: ids[0],
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 2.0,
                in_view: false,
                in_attack_arc: false,
                in_nano_arc: false,
                in_attack_cone: false,
                talk_enabled: false,
                position: [2.0, 0.0, 0.0],
                radius: 0.5,
                height: 1.0,
            },
            LegacyTargetSample {
                entity: ids[1],
                kind: LegacyTargetKind::Npc { team: 2 },
                distance: 6.0,
                in_view: false,
                in_attack_arc: false,
                in_nano_arc: false,
                in_attack_cone: false,
                talk_enabled: false,
                position: [6.0, 0.0, 0.0],
                radius: 0.5,
                height: 1.0,
            },
        ],
        trigger: None,
    };
    let selected = select_legacy_targets(
        &feed,
        &LegacyAvatarActionContext {
            nano_skill_usable: true,
            nano_target_policy: Some(LegacyNanoTargetPolicy {
                effect_target: 5,
                target_type: 1,
                range: 0.0,
                area: 4.0,
                half_angle_degrees: 0.0,
                capacity: 4,
            }),
            ..default()
        },
    );
    assert_eq!(selected.nano_targets.len(), 1);
    assert_eq!(selected.nano_targets[0].entity, ids[0]);
}
