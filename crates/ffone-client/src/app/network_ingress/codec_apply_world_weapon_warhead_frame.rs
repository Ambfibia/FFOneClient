use super::*;

pub(super) fn apply_world_weapon_warhead_frame(
    frame: &DecodedFrame,
    runtime: &mut RuntimeStatus,
    weapon_catalog: &PlayerWeaponAnimationCatalog,
    effects: &mut TutorialEffectRuntime,
    queries: &mut WorldIngressQueries<'_, '_>,
) -> Result<bool, String> {
    let Some(packet) = decode_pc_warhead_fire_packet_0104(frame.packet_type, &frame.payload)
        .map_err(|error| format!("malformed protocol-0104 weapon warhead packet: {error}"))?
    else {
        return Ok(false);
    };

    let (local, rocket_route, pc_id, skill_id, packet_position, destination, bullet, battery, bullet_id) =
        match packet {
            PcWarheadFirePacket0104::LocalRocket(packet) => (
                true,
                true,
                runtime.player_id,
                packet.skill_id,
                Some(packet.position),
                packet.destination,
                packet.bullet,
                Some(packet.weapon_battery),
                packet.bullet_id,
            ),
            PcWarheadFirePacket0104::RemoteRocket(packet) => (
                false,
                true,
                Some(packet.pc_id),
                0,
                Some(packet.position),
                packet.destination,
                packet.bullet,
                None,
                packet.bullet_id,
            ),
            PcWarheadFirePacket0104::LocalGrenade(packet) => (
                true,
                false,
                runtime.player_id,
                packet.skill_id,
                None,
                packet.destination,
                packet.bullet,
                Some(packet.weapon_battery),
                packet.bullet_id,
            ),
            PcWarheadFirePacket0104::RemoteGrenade(packet) => (
                false,
                false,
                Some(packet.pc_id),
                0,
                None,
                packet.destination,
                packet.bullet,
                None,
                packet.bullet_id,
            ),
        };

    // Nano rockets/grenades are owned by the Nano authority path. `sPCBullet`
    // attack type 2 changes `id` from WeaponItemTable to NanoSkillTable.
    if skill_id != 0 || bullet.attack_type == 2 {
        return Ok(false);
    }
    if battery.is_some_and(|weapon_battery| weapon_battery < 0) {
        return Err("local weapon warhead returned a negative weapon battery".to_owned());
    }
    let item_id = i16::try_from(bullet.id)
        .map_err(|_| format!("weapon warhead item {} is outside protocol i16", bullet.id))?;
    let profile = weapon_catalog
        .combat_profile_for_item(item_id)
        .ok_or_else(|| format!("weapon warhead item {item_id} has no exact WeaponItemTable row"))?;
    if !matches!(
        profile.target_mode,
        LegacyWeaponTargetMode::Rocket | LegacyWeaponTargetMode::Grenade
    ) {
        return Err(format!(
            "weapon warhead item {item_id} has non-warhead target mode {:?}",
            profile.target_mode
        ));
    }

    let old_weapon_battery = runtime.weapon_battery;
    let bullet_type =
        profile.native_presentation_bullet_type(if local { old_weapon_battery } else { 0 });

    let (source_entity, source_root) = if local {
        let (entity, transform, ..) = queries
            .local_player
            .single_mut()
            .map_err(|_| "local weapon warhead has no unique player transform".to_owned())?;
        (entity, transform.translation)
    } else {
        let pc_id = pc_id.ok_or_else(|| "remote weapon warhead has no PC id".to_owned())?;
        let (entity, _, transform) = queries
            .remote_players
            .iter()
            .find(|(_, remote, _)| remote.pc_id == pc_id)
            .ok_or_else(|| format!("remote weapon warhead PC {pc_id} is not materialized"))?;
        (entity, transform.translation())
    };
    let mut target = ProtocolPosition::new(destination).to_native();
    let source = if rocket_route {
        let packet_source = packet_position
            .map(ProtocolPosition::new)
            .map(ProtocolPosition::to_native)
            .unwrap_or(source_root);
        let source = tutorial_named_descendant_position(
            source_entity,
            "Gtag01",
            &queries.parents,
            &queries.named_transforms,
        )
        .unwrap_or(packet_source);
        target.y = source.y;
        source
    } else {
        source_root + Vec3::Y * 0.8
    };
    let initial_vertical_speed = match profile.target_mode {
        LegacyWeaponTargetMode::Rocket => None,
        LegacyWeaponTargetMode::Grenade => Some(profile.grenade_initial_vertical_speed),
        LegacyWeaponTargetMode::Normal => unreachable!("warhead mode validated above"),
    };
    // The clean local handler captures the charged effect choice from the old
    // battery, then applies the authoritative post-shot battery in the reply.
    if local && let Some(weapon_battery) = battery {
        runtime.weapon_battery = weapon_battery;
    }
    effects.enqueue(TutorialEffectRuntimeCommand::Projectile {
        bullet_type,
        source,
        target,
        target_exists: true,
        source_style: -1,
        target_style: -1,
        motion: TutorialProjectileMotion::Warhead {
            speed: profile.attack_range,
            initial_vertical_speed,
            duration_seconds: profile.warhead_duration_seconds,
            authority: local.then_some(ffone_client::tutorial_effects_runtime::WarheadAuthority {
                bullet_id, blast_radius: profile.blast_radius, target_capacity: profile.target_capacity,
            }),
        },
        source_line: line!(),
    });
    Ok(true)
}

/// Nano grants in the tutorial belong to the local virtual server. These
/// replies describe the shard's pre-tutorial bank and must not tune, unequip,
/// deactivate, or open free tuning over the scripted Buttercup grant.
/// NPC corruption and regeneration also carry complete Nano post-state;
/// local tutorial combat owns those effects and never enters shard death.
pub(in super::super) fn tutorial_owns_local_gameplay_frame(local_tutorial: bool, packet_type: u32) -> bool {
    local_tutorial
        && matches!(
            packet_type,
            packet::P_FE2CL_REP_NANO_ACTIVE_SUCC
                | packet::P_FE2CL_NANO_SKILL_USE_SUCC
                | packet::P_FE2CL_NANO_SKILL_USE
                | packet::P_FE2CL_REP_NANO_BOOK_SUBSET
                | packet::P_FE2CL_REP_NANO_EQUIP_SUCC
                | packet::P_FE2CL_REP_NANO_UNEQUIP_SUCC
                | packet::P_FE2CL_REP_NANO_TUNE_SUCC
                | packet::P_FE2CL_REP_NANO_TUNE_FAIL
                | packet::P_FE2CL_REP_PC_NANO_CREATE_SUCC
                | packet::P_FE2CL_REP_PC_NANO_CREATE_FAIL
                | packet::P_FE2CL_NPC_SKILL_HIT
                | packet::P_FE2CL_NPC_SKILL_CORRUPTION_HIT
                | packet::P_FE2CL_REP_PC_REGEN_SUCC
                | packet::P_FE2CL_PC_SUDDEN_DEAD
        )
}
