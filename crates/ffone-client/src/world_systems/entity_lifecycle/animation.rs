use super::*;

/// Persistent request identities and independent native animation lanes. Packet
/// batches may update both high layers before the renderer observes either one.
#[derive(Debug, Default, Component)]
pub struct NetworkNpcAnimationLayers0104 {
    pub(super) revision: u64,
    pub(crate) reset_revision: u64,
    pub low: Option<NetworkNpcCombatAnimation0104>,
    pub high: [Option<NetworkNpcCombatAnimation0104>; 2],
    pub high_owner: Option<u64>,
    pub stand_attack: bool,
}

impl NetworkNpcAnimationLayers0104 {
    pub(crate) fn reset(&mut self) {
        self.revision = self.revision.wrapping_add(1).max(1);
        self.reset_revision = self.revision;
        self.low = None;
        self.high = [None; 2];
        self.high_owner = None;
        self.stand_attack = false;
    }
    pub(crate) fn request(
        &mut self,
        clip: NetworkNpcCombatClip0104,
    ) -> NetworkNpcCombatAnimation0104 {
        self.revision = self.revision.wrapping_add(1).max(1);
        let request = NetworkNpcCombatAnimation0104 {
            revision: self.revision,
            clip,
        };
        if let Some(layer) = clip.high_layer() {
            self.high[layer] = Some(request);
            self.high_owner = Some(request.revision);
            self.stand_attack |= clip == NetworkNpcCombatClip0104::Melee;
        } else {
            self.low = Some(request);
        }
        request
    }
}

pub(super) fn request_network_npc_combat_animation(
    world: &mut World,
    npc_id: i32,
    clip: NetworkNpcCombatClip0104,
) {
    let Some(entity) = world.resource::<NetworkNpcRegistry0104>().get(npc_id) else {
        return;
    };
    if world.get_entity(entity).is_err()
        || world
            .get::<NetworkNpcAppearance0104>(entity)
            .is_some_and(|appearance| appearance.0.hp <= 0)
    {
        return;
    }
    let request = world
        .entity_mut(entity)
        .entry::<NetworkNpcAnimationLayers0104>()
        .or_default()
        .into_mut()
        .request(clip);
    world.entity_mut(entity).insert(request);
    if clip.high_layer().is_none() {
        world
            .entity_mut(entity)
            .remove::<NetworkNpcReadyAnimation0104>();
    }
}

pub(super) fn apply_network_npc_hp_animation(
    world: &mut World,
    entity: Entity,
    npc_id: i32,
    previous_hp: i32,
    hp: i32,
) {
    if hp <= 0 {
        let mut owner = world.entity_mut(entity);
        let mut layers = owner
            .entry::<NetworkNpcAnimationLayers0104>()
            .or_default()
            .into_mut();
        layers.low = None;
        layers.stand_attack = false;
        if previous_hp > 0 {
            layers.request(NetworkNpcCombatClip0104::Wound);
        }
        layers.high_owner = None;
        world
            .entity_mut(entity)
            .remove::<NetworkNpcMotion0104>()
            .remove::<NetworkNpcCombatAnimation0104>()
            .remove::<NetworkNpcReadyAnimation0104>()
            .remove::<NetworkNpcSkillPhase0104>();
    } else if hp < previous_hp {
        request_network_npc_combat_animation(world, npc_id, NetworkNpcCombatClip0104::Wound);
    }
}

pub(super) fn apply_network_npc_skill_animation(world: &mut World, signal: NpcSkillSignal0104) {
    let Some(entity) = world
        .resource::<NetworkNpcRegistry0104>()
        .get(signal.npc_id)
    else {
        return;
    };
    if world.get_entity(entity).is_err()
        || world
            .get::<NetworkNpcAppearance0104>(entity)
            .is_some_and(|appearance| appearance.0.hp <= 0)
    {
        return;
    }
    let selection = world.get::<NetworkNpc0104>(entity).and_then(|npc| {
        world
            .get_resource::<crate::network_world_runtime::NetworkNpcVisualCatalogState0104>()?
            .catalog
            .as_ref()?
            .skill_animations(npc.npc_type)
            .cloned()
    });
    match signal.kind {
        NpcSkillSignalKind0104::Ready => {
            let clip = selection
                .as_ref()
                .zip(signal.skill_id)
                .map_or(NetworkNpcCombatClip0104::MegaReady, |(set, id)| {
                    set.ready(id)
                });
            world
                .entity_mut(entity)
                .insert(NetworkNpcSkillPhase0104::MegaReady);
            request_network_npc_combat_animation(world, signal.npc_id, clip);
        }
        NpcSkillSignalKind0104::Fire => {
            // The primary GameFrame dispatches READY and HIT. FIRE has no
            // animation callback; starting skill0 here interrupts preparation.
        }
        NpcSkillSignalKind0104::Hit => {
            let fallback = if world.get::<NetworkNpcSkillPhase0104>(entity)
                == Some(&NetworkNpcSkillPhase0104::MegaReady)
            {
                NetworkNpcCombatClip0104::Mega
            } else {
                NetworkNpcCombatClip0104::Skill
            };
            world
                .entity_mut(entity)
                .remove::<NetworkNpcSkillPhase0104>();
            let clip = match selection.as_ref().zip(signal.skill_id) {
                Some((set, id)) => set.hit(id),
                None => Some(fallback),
            };
            if let Some(clip) = clip {
                request_network_npc_combat_animation(world, signal.npc_id, clip);
            }
        }
        NpcSkillSignalKind0104::CorruptionReady => {
            world
                .entity_mut(entity)
                .insert(NetworkNpcSkillPhase0104::CorruptionReady);
            request_network_npc_combat_animation(
                world,
                signal.npc_id,
                NetworkNpcCombatClip0104::CorruptionReady,
            );
        }
        NpcSkillSignalKind0104::CorruptionHit => {
            world
                .entity_mut(entity)
                .remove::<NetworkNpcSkillPhase0104>();
            request_network_npc_combat_animation(
                world,
                signal.npc_id,
                NetworkNpcCombatClip0104::Corruption,
            );
        }
        NpcSkillSignalKind0104::Cancel => {
            if let Some(mut layers) = world.get_mut::<NetworkNpcAnimationLayers0104>(entity) {
                layers.low = None;
            }
            world
                .entity_mut(entity)
                .remove::<NetworkNpcSkillPhase0104>()
                .remove::<NetworkNpcCombatAnimation0104>()
                .remove::<NetworkNpcReadyAnimation0104>();
        }
    }
}
