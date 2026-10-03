use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IgnoredLifecycleFrameReason0104 {
    StaleSession {
        active_epoch: Option<NetworkSessionEpoch0104>,
    },
    LocalPlayer {
        pc_id: i32,
    },
    MissingAppearance {
        kind: LifecycleEntityKind0104,
        id: i32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntityLifecycleDecodeError0104 {
    Fixed(PayloadError),
    Counted(CountedPayloadError0104),
    Around(AroundDecodeError),
    NanoSkillAuthority(WorldNanoProjectionError0104),
    NpcSkillAuthority(WorldNpcSkillProjectionError0104),
    NonFinitePcMoveVelocity,
}

impl fmt::Display for EntityLifecycleDecodeError0104 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fixed(error) => write!(formatter, "fixed lifecycle payload: {error}"),
            Self::Counted(error) => write!(formatter, "counted lifecycle payload: {error}"),
            Self::Around(error) => write!(formatter, "AROUND lifecycle payload: {error}"),
            Self::NanoSkillAuthority(error) => {
                write!(formatter, "Nano skill authority payload: {error}")
            }
            Self::NpcSkillAuthority(error) => {
                write!(formatter, "NPC skill authority payload: {error}")
            }
            Self::NonFinitePcMoveVelocity => {
                formatter.write_str("PC_MOVE contains a non-finite velocity")
            }
        }
    }
}

impl std::error::Error for EntityLifecycleDecodeError0104 {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Fixed(error) => Some(error),
            Self::Counted(error) => Some(error),
            Self::Around(error) => Some(error),
            Self::NanoSkillAuthority(error) => Some(error),
            Self::NpcSkillAuthority(error) => Some(error),
            Self::NonFinitePcMoveVelocity => None,
        }
    }
}

impl From<PayloadError> for EntityLifecycleDecodeError0104 {
    fn from(error: PayloadError) -> Self {
        Self::Fixed(error)
    }
}

impl From<CountedPayloadError0104> for EntityLifecycleDecodeError0104 {
    fn from(error: CountedPayloadError0104) -> Self {
        Self::Counted(error)
    }
}

impl From<AroundDecodeError> for EntityLifecycleDecodeError0104 {
    fn from(error: AroundDecodeError) -> Self {
        Self::Around(error)
    }
}

impl From<WorldNanoProjectionError0104> for EntityLifecycleDecodeError0104 {
    fn from(error: WorldNanoProjectionError0104) -> Self {
        Self::NanoSkillAuthority(error)
    }
}

impl From<WorldNpcSkillProjectionError0104> for EntityLifecycleDecodeError0104 {
    fn from(error: WorldNpcSkillProjectionError0104) -> Self {
        Self::NpcSkillAuthority(error)
    }
}

pub fn decode_entity_lifecycle_frame_0104(
    frame: &DecodedFrame,
) -> Result<Option<DecodedEntityLifecyclePacket0104>, EntityLifecycleDecodeError0104> {
    let packet = match frame.packet_type {
        ffone_protocol::packet::P_FE2CL_PC_EQUIP_CHANGE => {
            let packet = ffone_protocol::EquipChangePacket0104::decode(&frame.payload)?;
            if !(0..ffone_protocol::CHARACTER_EQUIP_SLOT_COUNT_0104 as i32)
                .contains(&packet.equip_slot_num)
            {
                return Err(PayloadError::ValueOutOfRange {
                    field: "equip_slot_num",
                    value: packet.equip_slot_num,
                    minimum: 0,
                    maximum: ffone_protocol::CHARACTER_EQUIP_SLOT_COUNT_0104 as i32 - 1,
                }
                .into());
            }
            DecodedEntityLifecyclePacket0104::PcEquipmentChange(packet)
        }
        0x3100013a => {
            let packet=ffone_protocol::wire_0104::PcStyleChange0104::decode(&frame.payload)?;
            DecodedEntityLifecyclePacket0104::PcStyleChange {
                pc_id:packet.pc_id, style:PcStyle0104::decode(&frame.payload[4..])?,
            }
        }
        ffone_protocol::packet::P_FE2CL_CHAR_TIME_BUFF_TIME_TICK
            if frame.payload.get(8..10)==Some(&24i16.to_le_bytes()) => {
            DecodedEntityLifecyclePacket0104::HealingTick(ffone_protocol::TimeBuffHealTick0104::decode(&frame.payload)?)
        }
        ffone_protocol::packet::P_FE2CL_CHAR_TIME_BUFF_TIME_OUT => {
            DecodedEntityLifecyclePacket0104::BuffTimeout(CharTimeBuffTimeout0104::decode(
                &frame.payload,
            )?)
        }
        0x3100_007a => {
            let state = ffone_protocol::wire_0104::PcStateChange0104::decode(&frame.payload)?;
            DecodedEntityLifecyclePacket0104::PcStateChange {
                pc_id: state.pc_id,
                state: state.state,
            }
        }
        P_FE2CL_PC_AROUND => {
            DecodedEntityLifecyclePacket0104::PcAround(decode_pc_around_0104(&frame.payload)?)
        }
        P_FE2CL_PC_NEW => {
            DecodedEntityLifecyclePacket0104::PcNew(PcNew0104::decode(&frame.payload)?)
        }
        P_FE2CL_PC_EXIT => {
            DecodedEntityLifecyclePacket0104::PcExit(PcExit0104::decode(&frame.payload)?)
        }
        P_FE2CL_AROUND_DEL_PC => {
            DecodedEntityLifecyclePacket0104::AroundDelPc(AroundDelPc0104::decode(&frame.payload)?)
        }
        P_FE2CL_PC_MOVE => {
            let packet = PcMove0104::decode(&frame.payload)?;
            if packet
                .movement
                .velocity
                .iter()
                .any(|value| !value.is_finite())
            {
                return Err(EntityLifecycleDecodeError0104::NonFinitePcMoveVelocity);
            }
            DecodedEntityLifecyclePacket0104::PcMotion(DecodedRemotePacket::Move(packet))
        }
        P_FE2CL_PC_STOP => DecodedEntityLifecyclePacket0104::PcMotion(DecodedRemotePacket::Stop(
            PcStop0104::decode(&frame.payload)?,
        )),
        P_FE2CL_PC_JUMP => DecodedEntityLifecyclePacket0104::PcMotion(DecodedRemotePacket::Jump(
            PcJump0104::decode(&frame.payload)?,
        )),
        P_FE2CL_PC_REGEN => {
            let Some(PcRegenPacket0104::Broadcast(packet)) =
                decode_pc_regen_packet_0104(frame.packet_type, &frame.payload)?
            else {
                unreachable!("PC_REGEN packet ID must decode as its broadcast variant");
            };
            DecodedEntityLifecyclePacket0104::PcRegen(packet)
        }
        P_FE2CL_PC_SUDDEN_DEAD => {
            let Some(PcRegenPacket0104::SuddenDead(packet)) =
                decode_pc_regen_packet_0104(frame.packet_type, &frame.payload)?
            else {
                unreachable!("PC_SUDDEN_DEAD packet ID must decode as its sudden-dead variant");
            };
            DecodedEntityLifecyclePacket0104::PcSuddenDead(packet)
        }
        P_FE2CL_NPC_AROUND => {
            DecodedEntityLifecyclePacket0104::NpcAround(decode_npc_around_0104(&frame.payload)?)
        }
        P_FE2CL_NPC_ENTER => {
            DecodedEntityLifecyclePacket0104::NpcEnter(NpcEnter0104::decode(&frame.payload)?)
        }
        P_FE2CL_NPC_EXIT => {
            DecodedEntityLifecyclePacket0104::NpcExit(NpcExit0104::decode(&frame.payload)?)
        }
        P_FE2CL_NPC_MOVE => {
            DecodedEntityLifecyclePacket0104::NpcMove(NpcMove0104::decode(&frame.payload)?)
        }
        P_FE2CL_NPC_NEW => {
            DecodedEntityLifecyclePacket0104::NpcNew(NpcNew0104::decode(&frame.payload)?)
        }
        P_FE2CL_AROUND_DEL_NPC => DecodedEntityLifecyclePacket0104::AroundDelNpc(
            AroundDelNpc0104::decode(&frame.payload)?,
        ),
        ffone_protocol::packet::P_FE2CL_PC_ROCKET_STYLE_HIT | ffone_protocol::packet::P_FE2CL_PC_GRENADE_STYLE_HIT => {
            DecodedEntityLifecyclePacket0104::NpcAttackResults(ffone_protocol::decode_pc_warhead_hit_0104(&frame.payload)?.results)
        }
        P_FE2CL_PC_ATTACK_NPCS_SUCC => DecodedEntityLifecyclePacket0104::NpcAttackResults(
            PcAttackNpcsSuccess0104::decode(&frame.payload)?.results,
        ),
        P_FE2CL_PC_ATTACK_NPCS => {
            let packet = PcAttackNpcs0104::decode(&frame.payload)?;
            DecodedEntityLifecyclePacket0104::RemotePcAttack { pc_id: packet.pc_id, results: packet.results, npc_only: true }
        },
        P_FE2CL_NPC_ATTACK_PCS => DecodedEntityLifecyclePacket0104::NpcAttackPcs(
            NpcAttackPcs0104::decode(&frame.payload)?,
        ),
        P_FE2CL_PC_ATTACK_CHARS_SUCC => DecodedEntityLifecyclePacket0104::MixedAttackResults(
            PcAttackCharsSuccess0104::decode(&frame.payload)?.results,
        ),
        P_FE2CL_PC_ATTACK_CHARS => {
            let packet = PcAttackChars0104::decode(&frame.payload)?;
            DecodedEntityLifecyclePacket0104::RemotePcAttack { pc_id: packet.pc_id, results: packet.results, npc_only: false }
        },
        P_FE2CL_NPC_ATTACK_CHARS => DecodedEntityLifecyclePacket0104::NpcAttackChars(
            NpcAttackChars0104::decode(&frame.payload)?,
        ),
        P_FE2CL_CHARACTER_ATTACK_CHARACTERS => {
            DecodedEntityLifecyclePacket0104::CharacterAttackCharacters(
                CharacterAttackCharacters0104::decode(&frame.payload)?,
            )
        }
        P_FE2CL_REP_BARKER => {
            DecodedEntityLifecyclePacket0104::NpcBarker(NpcBarker0104::decode(&frame.payload)?)
        }
        P_FE2CL_NANO_SKILL_USE_SUCC | P_FE2CL_NANO_SKILL_USE => {
            DecodedEntityLifecyclePacket0104::NanoSkillAuthority(
                decode_world_nano_authority_0104(frame.packet_type, &frame.payload)?
                    .expect("matched Nano skill packet must decode as authority"),
            )
        }
        P_FE2CL_NPC_SKILL_READY
        | P_FE2CL_NPC_SKILL_FIRE
        | P_FE2CL_NPC_SKILL_CORRUPTION_READY
        | P_FE2CL_NPC_SKILL_CANCEL => DecodedEntityLifecyclePacket0104::NpcSkill {
            signal: decode_npc_skill_signal_0104(frame.packet_type, &frame.payload)?
                .expect("matched NPC skill packet must decode as a skill signal"),
            authority: None,
        },
        P_FE2CL_NPC_SKILL_HIT | P_FE2CL_NPC_SKILL_CORRUPTION_HIT => {
            // Decode the strict tail before publishing even the animation
            // transition, keeping a malformed hit frame wholly atomic.
            let authority =
                decode_world_npc_skill_authority_0104(frame.packet_type, &frame.payload)?
                    .expect("matched NPC skill hit must decode as authority");
            DecodedEntityLifecyclePacket0104::NpcSkill {
                signal: decode_npc_skill_signal_0104(frame.packet_type, &frame.payload)?
                    .expect("matched NPC skill packet must decode as a skill signal"),
                authority: Some(authority),
            }
        }
        P_FE2CL_TRANSPORTATION_AROUND => DecodedEntityLifecyclePacket0104::TransportationAround(
            decode_transportation_around_0104(&frame.payload)?,
        ),
        P_FE2CL_TRANSPORTATION_ENTER | P_FE2CL_TRANSPORTATION_NEW => {
            DecodedEntityLifecyclePacket0104::TransportationUpsert(
                TransportationAppearance0104::decode(&frame.payload)?,
            )
        }
        P_FE2CL_TRANSPORTATION_EXIT => DecodedEntityLifecyclePacket0104::TransportationExit(
            TransportationExit0104::decode(&frame.payload)?,
        ),
        P_FE2CL_TRANSPORTATION_MOVE => DecodedEntityLifecyclePacket0104::TransportationMove(
            TransportationMove0104::decode(&frame.payload)?,
        ),
        P_FE2CL_AROUND_DEL_TRANSPORTATION => {
            DecodedEntityLifecyclePacket0104::AroundDelTransportation(
                AroundDelTransportation0104::decode(&frame.payload)?,
            )
        }
        P_FE2CL_SHINY_AROUND => {
            DecodedEntityLifecyclePacket0104::ShinyAround(decode_shiny_around_0104(&frame.payload)?)
        }
        P_FE2CL_SHINY_ENTER | P_FE2CL_SHINY_NEW => DecodedEntityLifecyclePacket0104::ShinyUpsert(
            ShinyAppearance0104::decode(&frame.payload)?,
        ),
        P_FE2CL_SHINY_EXIT => {
            DecodedEntityLifecyclePacket0104::ShinyExit(ShinyExit0104::decode(&frame.payload)?)
        }
        P_FE2CL_AROUND_DEL_SHINY => DecodedEntityLifecyclePacket0104::AroundDelShiny(
            AroundDelShiny0104::decode(&frame.payload)?,
        ),
        _ => return Ok(None),
    };
    Ok(Some(packet))
}

pub(super) fn consume_live_frame(world: &mut World, epoch: NetworkSessionEpoch0104, frame: DecodedFrame) {
    let active = *world.resource::<ActiveNetworkEntitySession0104>();
    if active.epoch != Some(epoch) {
        world
            .resource_mut::<IgnoredLifecycleFrames0104>()
            .frames
            .push(IgnoredLifecycleFrame0104 {
                epoch,
                frame,
                reason: IgnoredLifecycleFrameReason0104::StaleSession {
                    active_epoch: active.epoch,
                },
            });
        return;
    }

    world
        .resource_mut::<NetworkEntityLifecycleStats0104>()
        .live_frames += 1;
    let decoded = match decode_entity_lifecycle_frame_0104(&frame) {
        Ok(Some(decoded)) => decoded,
        Ok(None) => {
            world
                .resource_mut::<PassthroughLifecycleFrames0104>()
                .frames
                .push(PassthroughLifecycleFrame0104 { epoch, frame });
            return;
        }
        Err(error) => {
            world
                .resource_mut::<MalformedLifecycleFrames0104>()
                .frames
                .push(MalformedLifecycleFrame0104 {
                    epoch,
                    frame,
                    error,
                });
            return;
        }
    };

    match decoded {
        DecodedEntityLifecyclePacket0104::PcEquipmentChange(packet) => {
            if let Some(entity) =
                resolve_remote_player(world, epoch, active.local_player_id, packet.pc_id, frame)
                && let Some(mut appearance) = world.get::<NetworkPcAppearance0104>(entity).cloned()
            {
                let slot = packet.equip_slot_num as usize;
                if appearance.0.equipment[slot] != packet.equip_slot_item {
                    appearance.0.equipment[slot] = packet.equip_slot_item;
                    let pending = PendingPcVisual0104::from(&appearance.0);
                    world.entity_mut(entity).insert((appearance, pending));
                }
            }
        }
        DecodedEntityLifecyclePacket0104::RemotePcAttack { pc_id, results, npc_only } => {
            if npc_only { apply_npc_attack_results(world, results); }
            else { apply_mixed_attack_results(world, &results); }
            if active.local_player_id != Some(pc_id) {
                if let Some(entity) = world.resource::<RemotePcRegistry0104>().get(pc_id) {
                    world.entity_mut(entity).insert(RemoteAnimation { state: crate::remote::RemoteAnimationState::Attacking });
                }
            }
        }
        DecodedEntityLifecyclePacket0104::PcStyleChange { pc_id, style } => {
            if let Some(entity)=resolve_remote_player(world,epoch,active.local_player_id,pc_id,frame) {
                if let Some(mut appearance)=world.get::<NetworkPcAppearance0104>(entity).cloned() {
                    if appearance.0.style.pc_uid==style.pc_uid {
                        appearance.0.style=style;
                        let pending=PendingPcVisual0104::from(&appearance.0);
                        world.entity_mut(entity).insert((appearance,pending));
                    }
                }
            }
        }
        DecodedEntityLifecyclePacket0104::PcAround(players) => {
            for appearance in players {
                upsert_player(world, epoch, active.local_player_id, appearance);
            }
        }
        DecodedEntityLifecyclePacket0104::PcNew(packet) => {
            upsert_player(world, epoch, active.local_player_id, packet.appearance);
        }
        DecodedEntityLifecyclePacket0104::PcExit(packet) => {
            despawn_player(world, active.local_player_id, packet.pc_id);
        }
        DecodedEntityLifecyclePacket0104::AroundDelPc(packet) => {
            for pc_id in packet.pc_ids {
                despawn_player(world, active.local_player_id, pc_id);
            }
        }
        DecodedEntityLifecyclePacket0104::PcMotion(packet) => {
            apply_player_motion(world, epoch, active.local_player_id, packet, frame);
        }
        DecodedEntityLifecyclePacket0104::PcRegen(packet) => {
            apply_player_regen(world, epoch, active.local_player_id, packet, frame);
        }
        DecodedEntityLifecyclePacket0104::PcStateChange { pc_id, state } => {
            if let Some(entity) =
                resolve_remote_player(world, epoch, active.local_player_id, pc_id, frame)
            {
                if let Some(mut appearance) = world.get_mut::<NetworkPcAppearance0104>(entity) {
                    if appearance.0.pc_state != state {
                        appearance.0.pc_state = state;
                    }
                }
            }
        }
        DecodedEntityLifecyclePacket0104::PcSuddenDead(packet) => {
            apply_player_sudden_dead(world, epoch, active.local_player_id, packet, frame);
        }
        DecodedEntityLifecyclePacket0104::NpcAround(npcs) => {
            for appearance in npcs {
                upsert_npc(world, epoch, appearance);
            }
        }
        DecodedEntityLifecyclePacket0104::NpcEnter(packet) => {
            upsert_npc(world, epoch, packet.appearance);
        }
        DecodedEntityLifecyclePacket0104::NpcExit(packet) => {
            despawn_npc(world, packet.npc_id);
        }
        DecodedEntityLifecyclePacket0104::NpcMove(packet) => {
            apply_npc_motion(world, epoch, packet, frame);
        }
        DecodedEntityLifecyclePacket0104::NpcNew(packet) => {
            upsert_npc(world, epoch, packet.appearance);
        }
        DecodedEntityLifecyclePacket0104::AroundDelNpc(packet) => {
            for npc_id in packet.npc_ids {
                despawn_npc(world, npc_id);
            }
        }
        DecodedEntityLifecyclePacket0104::NpcAttackResults(results) => {
            apply_npc_attack_results(world, results);
        }
        DecodedEntityLifecyclePacket0104::NpcAttackPcs(packet) => {
            if let Some(target) = packet.results.first() {
                face_network_npc_toward_pc(world, packet.npc_id, target.id, active.local_player_id);
            }
            request_network_npc_combat_animation(
                world,
                packet.npc_id,
                NetworkNpcCombatClip0104::Melee,
            );
            apply_pc_attack_results(world, &packet.results);
            world
                .resource_mut::<NetworkNpcAttackEventQueue0104>()
                .events
                .push_back(packet);
        }
        DecodedEntityLifecyclePacket0104::MixedAttackResults(results) => {
            apply_mixed_attack_results(world, &results);
        }
        DecodedEntityLifecyclePacket0104::NpcAttackChars(packet) => {
            // GameFrame case 822083844: the NPC plays its attack toward the
            // first result, player results (`eCT == 1`) take the ordinary
            // NPC_ATTACK_PCS presentation path and every other result is an NPC.
            if let Some(target) = packet
                .results
                .iter()
                .find(|result| result.entity_type == PC_ATTACK_ENTITY_TYPE)
            {
                face_network_npc_toward_pc(world, packet.npc_id, target.id, active.local_player_id);
            } else if let Some(target) = packet.results.first() {
                face_network_npc_toward_npc(world, packet.npc_id, target.id);
            }
            request_network_npc_combat_animation(
                world,
                packet.npc_id,
                NetworkNpcCombatClip0104::Melee,
            );
            apply_mixed_attack_results(world, &packet.results);
            let pc_results: Vec<AttackResult0104> = packet
                .results
                .iter()
                .copied()
                .filter(|result| result.entity_type == PC_ATTACK_ENTITY_TYPE)
                .collect();
            if !pc_results.is_empty() {
                world
                    .resource_mut::<NetworkNpcAttackEventQueue0104>()
                    .events
                    .push_back(NpcAttackPcs0104 {
                        npc_id: packet.npc_id,
                        results: pc_results,
                    });
            }
        }
        DecodedEntityLifecyclePacket0104::CharacterAttackCharacters(packet) => {
            // GameFrame case 822083717 resolves both the attacker and every
            // target through NpcContainer only.
            if packet.entity_type == NPC_ATTACK_ENTITY_TYPE {
                request_network_npc_combat_animation(
                    world,
                    packet.character_id,
                    NetworkNpcCombatClip0104::Melee,
                );
            }
            apply_npc_attack_results(world, packet.results);
        }
        DecodedEntityLifecyclePacket0104::NpcBarker(packet) => {
            world
                .resource_mut::<NetworkNpcBarkerEventQueue0104>()
                .events
                .push_back(NetworkNpcBarkerEvent0104::Mission(packet));
        }
        DecodedEntityLifecyclePacket0104::NanoSkillAuthority(projection) => {
            apply_world_nano_authority(world, epoch, active.local_player_id, &projection, &frame);
            world
                .resource_mut::<NetworkNanoEffectEvents0104>()
                .0
                .push_back(projection);
        }
        DecodedEntityLifecyclePacket0104::HealingTick(tick) => {
            match tick.character_type {
                1 => apply_remote_pc_authority(world, epoch, active.local_player_id, tick.character_id,
                    RemotePcAuthorityUpdate0104 {absolute_hp:Some(tick.hp),..Default::default()}, &frame),
                2 | 4 => apply_npc_authority(world, epoch, tick.character_id,
                    NpcAuthorityUpdate0104 {absolute_hp:Some(tick.hp),..Default::default()}, &frame),
                _ => return,
            }
            world.resource_mut::<NetworkHealingTickEffects0104>().0.push_back(tick);
        }
        DecodedEntityLifecyclePacket0104::BuffTimeout(packet) => {
            // OpenFusion uses both NPC (2) and mob (4) character categories.
            // Replace the absolute mask even when it is zero; HP and movement
            // remain owned by their respective result packets.
            match packet.character_type {
                1 => apply_remote_pc_authority(
                    world,
                    epoch,
                    active.local_player_id,
                    packet.character_id,
                    RemotePcAuthorityUpdate0104 {
                        absolute_condition_bit_flag: Some(packet.condition_bit_flag),
                        ..Default::default()
                    },
                    &frame,
                ),
                2 | 4 => apply_npc_authority(
                    world,
                    epoch,
                    packet.character_id,
                    NpcAuthorityUpdate0104 {
                        absolute_condition_bit_flag: Some(packet.condition_bit_flag),
                        ..Default::default()
                    },
                    &frame,
                ),
                _ => {}
            }
        }
        DecodedEntityLifecyclePacket0104::NpcSkill { signal, authority } => {
            let mega = world.resource::<NetworkNpcRegistry0104>().get(signal.npc_id)
                .and_then(|entity| world.get::<NetworkNpc0104>(entity))
                .and_then(|npc| world.get_resource::<crate::network_world_runtime::NetworkNpcVisualCatalogState0104>()?
                    .catalog.as_ref()?.skill_animations(npc.npc_type))
                .zip(signal.skill_id)
                .is_some_and(|(set, id)| set.ready(id) == NetworkNpcCombatClip0104::MegaReady);
            let offset = match signal.kind {
                NpcSkillSignalKind0104::Ready
                | NpcSkillSignalKind0104::Fire
                | NpcSkillSignalKind0104::CorruptionReady
                | NpcSkillSignalKind0104::CorruptionHit => Some(8),
                NpcSkillSignalKind0104::Hit => Some(8),
                NpcSkillSignalKind0104::Cancel => None,
            };
            let position = offset.map(|offset| {
                std::array::from_fn(|i| {
                    i32::from_le_bytes(
                        frame.payload[offset + i * 4..offset + i * 4 + 4]
                            .try_into()
                            .unwrap(),
                    )
                })
            });
            let style = matches!(
                signal.kind,
                NpcSkillSignalKind0104::CorruptionReady | NpcSkillSignalKind0104::CorruptionHit
            )
            .then(|| i16::from_le_bytes(frame.payload[6..8].try_into().unwrap()));
            world
                .resource_mut::<NetworkNpcSkillEffectEvents0104>()
                .0
                .push_back(NetworkNpcSkillEffect0104 {
                    signal,
                    position,
                    style,
                    mega,
                });
            if matches!(
                signal.kind,
                NpcSkillSignalKind0104::Ready | NpcSkillSignalKind0104::CorruptionReady
            ) {
                world
                    .resource_mut::<NetworkNpcBarkerEventQueue0104>()
                    .events
                    .push_back(NetworkNpcBarkerEvent0104::SkillReady(signal));
            }
            apply_network_npc_skill_animation(world, signal);
            if let Some(authority) = authority {
                apply_world_npc_skill_authority(
                    world,
                    epoch,
                    active.local_player_id,
                    &authority,
                    &frame,
                );
                world.resource_mut::<NetworkNpcResultEffectEvents0104>().0.push_back(authority);
            }
        }
        DecodedEntityLifecyclePacket0104::TransportationAround(entries) => {
            for appearance in entries {
                upsert_transportation(world, epoch, appearance);
            }
        }
        DecodedEntityLifecyclePacket0104::TransportationUpsert(appearance) => {
            upsert_transportation(world, epoch, appearance);
        }
        DecodedEntityLifecyclePacket0104::TransportationExit(packet) => {
            despawn_transportation(world, packet.transportation_kind, packet.id);
        }
        DecodedEntityLifecyclePacket0104::TransportationMove(packet) => {
            apply_transportation_motion(world, epoch, packet, frame);
        }
        DecodedEntityLifecyclePacket0104::AroundDelTransportation(packet) => {
            for id in packet.ids {
                despawn_transportation(world, packet.transportation_kind, id);
            }
        }
        DecodedEntityLifecyclePacket0104::ShinyAround(entries) => {
            for appearance in entries {
                upsert_shiny(world, epoch, appearance);
            }
        }
        DecodedEntityLifecyclePacket0104::ShinyUpsert(appearance) => {
            upsert_shiny(world, epoch, appearance);
        }
        DecodedEntityLifecyclePacket0104::ShinyExit(packet) => {
            despawn_shiny(world, packet.shiny_id);
        }
        DecodedEntityLifecyclePacket0104::AroundDelShiny(packet) => {
            for shiny_id in packet.shiny_ids {
                despawn_shiny(world, shiny_id);
            }
        }
    }
}

pub(super) fn push_ignored_frame(
    world: &mut World,
    epoch: NetworkSessionEpoch0104,
    frame: DecodedFrame,
    reason: IgnoredLifecycleFrameReason0104,
) {
    world
        .resource_mut::<IgnoredLifecycleFrames0104>()
        .frames
        .push(IgnoredLifecycleFrame0104 {
            epoch,
            frame,
            reason,
        });
}
