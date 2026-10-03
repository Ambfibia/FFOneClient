//! Deterministic full-renderer recipient effects and radar expiry acceptance.
use super::*;
use ffone_client::entity_lifecycle::{
    NetworkHealingTickEffects0104, NetworkNanoEffectEvents0104, NetworkNpcResultEffectEvents0104,
};
use ffone_client::world_nano_authority::*;

pub(super) fn install(app: &mut App) {
    app.add_systems(
        Update,
        drive
            .after(poll_network)
            .after(NetworkSessionLifecycleSet::Apply)
            .after(consume_network_entity_lifecycle_0104)
            .before(world_skill_effects::sync_world_skill_effects)
            .before(world_skill_effects::spawn_world_instant_skill_effects)
            .before(world_skill_effects::spawn_healing_tick_effects),
    );
}
fn drive(
    mut commands: Commands,
    capture: Res<Capture>,
    mut frame: Local<u32>,
    runtime: Res<RuntimeStatus>,
    mut buffs: ResMut<SkillBuffUiModel>,
    mut nano: ResMut<NetworkNanoEffectEvents0104>,
    mut npc: ResMut<NetworkNpcResultEffectEvents0104>,
    mut heal: ResMut<NetworkHealingTickEffects0104>,
) {
    if capture.ready.is_none() || capture.samples.is_empty() {
        return;
    }
    *frame += 1;
    let pc = runtime.player_id.unwrap();
    // The offline bridge rejects periodic requests and resets session-owned UI.
    // Reapply this fixture's authority before exercising the production effects.
    buffs.local_character_id = Some(pc);
    buffs.local_condition_bit_flag = match *frame {
        30..=99 | 151..=519 => 1,
        100..=150 => 0x3001,
        _ => 0,
    };
    match *frame {
        30 => {buffs.local_character_id=Some(pc);buffs.local_condition_bit_flag=1;},
        100 => buffs.local_condition_bit_flag=0x3001,
        150 => {
            assert!(buffs.reveals_mobs() && buffs.reveals_shinies());
            buffs.apply_timeout(ffone_protocol::CharTimeBuffTimeout0104 {character_type:1,character_id:pc,condition_bit_flag:1});
            assert!(!buffs.reveals_mobs() && !buffs.reveals_shinies());
        },
        200 => nano.0.push_back(WorldNanoAuthoritativeProjection0104 {
            delivery:ffone_protocol::NanoSkillUseDelivery0104::LocalSuccess,skill_type:2,
            caster:WorldNanoCasterPostState0104 {pc_id:pc,nano_id:1,skill_id:2,nano_stamina:100,nano_deactivated:false,absolute_hp:None},
            targets:vec![WorldNanoTargetPostState0104 {target:WorldNanoEntity0104 {kind:WorldNanoEntityKind0104::Player,id:pc},
                absolute_hp:Some(500),absolute_condition_bit_flag:None,absolute_weapon_battery:None,absolute_nano_battery:None,
                absolute_nano_stamina:None,nano_deactivated:None,movement:None,
                source_result:ffone_protocol::NanoSkillResult0104::HealHp(ffone_protocol::NanoSkillHealHpResult0104 {
                    target:ffone_protocol::NanoSkillTarget0104 {entity_type:1,id:pc},healed_hp:100,hp:500})}],
        }),
        320 => {
            let mut payload=vec![0u8;48];
            for (offset,value) in [(0,991),(20,1),(24,1),(28,1),(32,pc),(36,0),(40,50),(44,450)] {
                payload[offset..offset+4].copy_from_slice(&i32::to_le_bytes(value));
            }
            payload[4..6].copy_from_slice(&184i16.to_le_bytes());
            npc.0.push_back(ffone_client::world_npc_skill_authority::decode_world_npc_skill_authority_0104(
                packet::P_FE2CL_NPC_SKILL_HIT,&payload).unwrap().unwrap());
        },
        440 => heal.0.push_back(ffone_protocol::TimeBuffHealTick0104 {character_type:1,character_id:pc,healed_hp:50,hp:500}),
        520 => buffs.local_condition_bit_flag=0,
        580 => fs::write(capture.output.join("skill-effects.json"),
            serde_json::to_vec_pretty(&serde_json::json!({"radarExpiry":true,"nanoHealing":2,"eggDamage":184,"healingTick":24,"finalMask":buffs.local_condition_bit_flag})).unwrap()).unwrap(),
        _=>{},
    }
    if matches!(*frame, 40 | 220 | 240 | 350 | 370 | 460 | 540) {
        commands.spawn(Screenshot::primary_window()).observe(
            bevy::render::view::screenshot::save_to_disk(
                capture.output.join(format!("skill-{}.png", *frame)),
            ),
        );
    }
}
