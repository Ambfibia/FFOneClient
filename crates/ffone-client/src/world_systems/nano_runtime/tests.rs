use crate::world_nano_runtime::*;

fn skill(target: i32, target_type: i32) -> GameplaySkillUiDefinition {
    GameplaySkillUiDefinition {
        skill_id: 1,
        skill_type: 1,
        values_a: [0; 4],
        icon_number: 10,
        active: true,
        effect_type: 1,
        target_effect: 0,
        target,
        target_type,
        range: 1_000,
        area: 500,
        angle: 90,
        target_number: 4,
        cooldown: 80,
        cool_type: 22,
    }
}

fn npc(actor_id: i32, position: [f32; 3]) -> WorldNanoTarget {
    WorldNanoTarget {
        actor_id,
        kind: WorldNanoTargetKind::Npc { team: 2 },
        position,
        radius: 0.0,
        height: 0.0,
    }
}

fn player(actor_id: i32, position: [f32; 3]) -> WorldNanoTarget {
    WorldNanoTarget {
        actor_id,
        kind: WorldNanoTargetKind::Player,
        position,
        radius: 0.0,
        height: 0.0,
    }
}

fn input<'a>(focus: Option<i32>, targets: &'a [WorldNanoTarget]) -> WorldNanoPlanInput<'a> {
    WorldNanoPlanInput {
        self_id: 99,
        caster_position: [0.0, 0.0, 0.0],
        caster_forward: [0.0, 0.0, 1.0],
        focused_target_id: focus,
        targets,
    }
}

#[test]
fn effect_target_one_uses_the_compatible_focus_and_extent_aware_range() {
    let mut focus = npc(7, [0.0, 0.0, 1.4]);
    focus.radius = 0.5;
    let mut definition = skill(1, 1);
    definition.range = 100;
    definition.target_number = 1;

    assert_eq!(
        plan_world_nano_skill_0104(&definition, input(Some(7), &[focus])),
        Ok(NanoSkillUseRequest0104 {
            bullet_id: -1,
            arg1: 7,
            arg2: 0,
            arg3: 0,
            target_ids: vec![7],
        })
    );

    focus.position[2] = 1.6;
    assert_eq!(
        plan_world_nano_skill_0104(&definition, input(Some(7), &[focus])),
        Err(WorldNanoPlanError::FocusOutsideSelection(7))
    );
}

#[test]
fn effect_target_two_builds_the_exact_self_request() {
    let mut definition = skill(2, 3);
    definition.target_number = 1;
    assert_eq!(
        plan_world_nano_skill_0104(&definition, input(None, &[])),
        Ok(NanoSkillUseRequest0104 {
            bullet_id: -1,
            arg1: 99,
            arg2: 0,
            arg3: 0,
            target_ids: vec![99],
        })
    );
    assert_eq!(
        plan_world_nano_skill_0104(&definition, input(None, &[npc(99, [0.0; 3])]))
            .unwrap()
            .target_ids,
        vec![99],
        "nearby NPC IDs must not block a self-targeted Recall"
    );
}

#[test]
fn effect_target_three_requires_cone_membership_then_focuses_before_order_and_cap() {
    let targets = [
        npc(20, [0.0, 0.0, 2.0]),
        npc(30, [0.0, 0.0, 4.0]),
        npc(10, [0.0, 0.0, 2.0]),
        npc(40, [4.0, 0.0, 0.0]),
    ];
    let mut definition = skill(3, 1);
    definition.angle = 45;
    definition.target_number = 2;

    assert_eq!(
        plan_world_nano_skill_0104(&definition, input(Some(30), &targets)),
        Ok(NanoSkillUseRequest0104 {
            bullet_id: -1,
            arg1: 30,
            arg2: 0,
            arg3: 0,
            target_ids: vec![30, 10],
        })
    );
    assert_eq!(
        plan_world_nano_skill_0104(&definition, input(Some(40), &targets)),
        Err(WorldNanoPlanError::FocusOutsideSelection(40))
    );
}

#[test]
fn effect_target_five_centers_aoe_on_caster_and_preserves_self_player_semantics() {
    let targets = [
        player(20, [1.0, 0.0, 0.0]),
        npc(5, [0.5, 0.0, 0.0]),
        player(10, [-1.0, 0.0, 0.0]),
        player(30, [3.0, 0.0, 0.0]),
    ];
    let mut definition = skill(5, 2);
    definition.area = 200;
    definition.target_number = 3;
    assert_eq!(
        plan_world_nano_skill_0104(&definition, input(None, &targets)),
        Ok(NanoSkillUseRequest0104 {
            bullet_id: -1,
            arg1: 99,
            arg2: 0,
            arg3: 0,
            target_ids: vec![99, 10, 20],
        })
    );

    definition.target_type = 1;
    assert_eq!(
        plan_world_nano_skill_0104(&definition, input(None, &targets))
            .unwrap()
            .target_ids,
        vec![5]
    );

    definition.target_type = 3;
    assert_eq!(
        plan_world_nano_skill_0104(&definition, input(None, &targets))
            .unwrap()
            .target_ids,
        vec![99]
    );
}

#[test]
fn effect_target_six_centers_aoe_on_focus_and_keeps_focus_first() {
    let targets = [
        npc(20, [9.0, 0.0, 0.0]),
        npc(30, [10.0, 0.0, 0.0]),
        npc(10, [11.0, 0.0, 0.0]),
        npc(40, [13.0, 0.0, 0.0]),
    ];
    let mut definition = skill(6, 1);
    definition.range = 1_100;
    definition.area = 150;
    definition.target_number = 3;
    assert_eq!(
        plan_world_nano_skill_0104(&definition, input(Some(30), &targets)),
        Ok(NanoSkillUseRequest0104 {
            bullet_id: -1,
            arg1: 30,
            arg2: 0,
            arg3: 0,
            target_ids: vec![30, 10, 20],
        })
    );
}

#[test]
fn non_target_effect_and_effect_target_four_fail_closed() {
    let mut invalid_effect = skill(2, 3);
    invalid_effect.effect_type = 2;
    assert_eq!(
        plan_world_nano_skill_0104(&invalid_effect, input(None, &[])),
        Err(WorldNanoPlanError::UnsupportedEffectType(2))
    );
    assert_eq!(
        plan_world_nano_skill_0104(&skill(4, 1), input(None, &[])),
        Err(WorldNanoPlanError::UnsupportedEffectTarget(4))
    );
}

#[test]
fn focused_area_rejects_a_distant_focus_before_sending() {
    let targets = [npc(30, [100.0, 0.0, 0.0])];
    assert_eq!(
        plan_world_nano_skill_0104(&skill(6, 1), input(Some(30), &targets)),
        Err(WorldNanoPlanError::FocusOutsideSelection(30))
    );
}
