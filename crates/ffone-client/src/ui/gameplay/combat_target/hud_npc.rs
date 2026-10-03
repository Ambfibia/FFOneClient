//! HUD NPC naming and the NPC lookup query shared by target and minimap binders.

use super::target_icons::combat_target_icon_path;
use crate::{
    avatar_action::LegacyAvatarActionState,
    entity_lifecycle::{NetworkNpcAppearance0104, NetworkPcAppearance0104},
    localization::{LocalizedText, localized_tabledata_npc_name},
    movement::SERVER_TO_CLIENT_SCALE,
    tutorial_actors::TutorialActor,
    tutorial_mission_content::{GameplayNpcUiDefinition, TutorialMissionContent},
};
use bevy::{ecs::system::SystemParam, prelude::*};

pub(in super::super) fn localized_tutorial_npc_name(
    npc_type: i32,
    fallback: &str,
) -> LocalizedText {
    LocalizedText::new(format!("content.npc.{npc_type}.name"), fallback)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in super::super) enum GameplayHudNpcName<'a> {
    /// Tutorial actors own curated EN/RU semantic keys.
    Tutorial { npc_type: i32, fallback: &'a str },
    /// Normal-world actors use the same stable TableData NPC identity as the
    /// tutorial actors. Passing the raw table string through here left every
    /// network NPC name in English even though `content.npc.<id>.name` was
    /// already present in both authored bundles.
    Table { npc_type: i32, fallback: &'a str },
}

impl<'a> GameplayHudNpcName<'a> {
    pub(in super::super) fn localized(self) -> LocalizedText {
        match self {
            Self::Tutorial { npc_type, fallback } => {
                localized_tutorial_npc_name(npc_type, fallback)
            }
            Self::Table { npc_type, fallback } => localized_tabledata_npc_name(npc_type, fallback),
        }
    }

    #[cfg(test)]
    pub(in super::super) const fn fallback(self) -> &'a str {
        match self {
            Self::Tutorial { fallback, .. } | Self::Table { fallback, .. } => fallback,
        }
    }
}

/// Source-independent data consumed by the shared gameplay target HUD.
///
/// Normal-world level and affinity come only from their exact TableData row;
/// no tutorial default is shared across unrelated world NPCs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in super::super) struct GameplayHudNpc<'a> {
    pub(in super::super) name: GameplayHudNpcName<'a>,
    pub(in super::super) team: i32,
    pub(in super::super) hp_fraction: Option<f32>,
    pub(in super::super) height: f32,
    pub(in super::super) projection_origin_y: f32,
    pub(in super::super) level: Option<i32>,
    pub(in super::super) affinity_style: Option<i32>,
    pub(in super::super) portrait_icon_path: Option<&'a str>,
}

impl<'a> GameplayHudNpc<'a> {
    pub(in super::super) fn tutorial(
        actor: &TutorialActor,
        table_definition: Option<&'a GameplayNpcUiDefinition>,
    ) -> Option<Self> {
        let definition = table_definition.filter(|table| table.npc_type == actor.npc_type)?;
        let hp_fraction = (actor.max_hp > 0)
            .then(|| (actor.hp.max(0) as f32 / actor.max_hp as f32).clamp(0.0, 1.0));
        let affinity_style = (actor.team != 1 && (0..=2).contains(&definition.npc_style))
            .then_some(definition.npc_style);
        Some(Self {
            name: GameplayHudNpcName::Tutorial {
                npc_type: actor.npc_type,
                fallback: definition.name.as_str(),
            },
            team: actor.team,
            hp_fraction,
            height: definition.height(),
            // Tutorial spawn transforms already apply `NpcContainer.Add`'s
            // ordinary-NPC one-unit lift.
            projection_origin_y: 0.0,
            // Every hostile emitted by cntutorialscript owns exact table
            // level; friendly tutorial actors never draw this field.
            level: (actor.team != 1).then_some(definition.npc_level),
            affinity_style,
            portrait_icon_path: combat_target_icon_path(actor.npc_type),
        })
    }

    pub(in super::super) fn network(
        appearance: &NetworkNpcAppearance0104,
        definition: &'a GameplayNpcUiDefinition,
        portrait_icon_path: Option<&'a str>,
    ) -> Option<Self> {
        if definition.npc_type != appearance.0.npc_type {
            return None;
        }
        Some(Self {
            name: GameplayHudNpcName::Table {
                npc_type: definition.npc_type,
                fallback: &definition.name,
            },
            team: definition.team,
            hp_fraction: definition.hp_fraction(appearance.0.hp),
            height: definition.height_server_units as f32 * SERVER_TO_CLIENT_SCALE,
            // Native network roots are grounded by the world presentation
            // pass. Clean's one-unit `NpcContainer.Add` lift is only an
            // initial spawn position and is consumed by movement grounding
            // before PrintName and sub-target projection.
            projection_origin_y: 0.0,
            level: Some(definition.npc_level),
            affinity_style: Some(definition.npc_style),
            // The normal-world route comes from the exact Npc TableData icon
            // row and the installed semantic icon catalog. Missing routes
            // remain hidden instead of receiving a guessed portrait.
            portrait_icon_path,
        })
    }

    pub(in super::super) fn projected_world_position(self, root: Vec3, height_factor: f32) -> Vec3 {
        root + Vec3::Y * (self.projection_origin_y + self.height * height_factor)
    }
}

#[derive(SystemParam)]
pub(in super::super) struct GameplayHudNpcQuery<'w, 's> {
    pub(in super::super) tutorial_actors:
        Query<'w, 's, (&'static GlobalTransform, &'static TutorialActor)>,
    pub(in super::super) network_npcs:
        Query<'w, 's, (&'static GlobalTransform, &'static NetworkNpcAppearance0104)>,
    pub(in super::super) network_players:
        Query<'w, 's, (&'static GlobalTransform, &'static NetworkPcAppearance0104)>,
    pub(in super::super) player_roots:
        Query<'w, 's, &'static GlobalTransform, With<LegacyAvatarActionState>>,
    pub(in super::super) content: Option<Res<'w, TutorialMissionContent>>,
}

impl GameplayHudNpcQuery<'_, '_> {
    pub(in super::super) fn get(
        &self,
        entity: Entity,
    ) -> Option<(&GlobalTransform, GameplayHudNpc<'_>)> {
        if let Ok((transform, actor)) = self.tutorial_actors.get(entity) {
            let table_definition = self
                .content
                .as_deref()
                .and_then(|content| content.gameplay_npc(actor.npc_type));
            return GameplayHudNpc::tutorial(actor, table_definition)
                .map(|target| (transform, target));
        }
        let (transform, appearance) = self.network_npcs.get(entity).ok()?;
        let content = self.content.as_deref()?;
        let definition = content.gameplay_npc(appearance.0.npc_type)?;
        let portrait_icon_path = content.gameplay_npc_portrait_icon_path(appearance.0.npc_type);
        GameplayHudNpc::network(appearance, definition, portrait_icon_path)
            .map(|target| (transform, target))
    }

    pub(in super::super) fn projected_world_position(
        &self,
        entity: Entity,
        height_factor: f32,
    ) -> Option<Vec3> {
        if let Some((transform, target)) = self.get(entity) {
            return Some(target.projected_world_position(transform.translation(), height_factor));
        }
        let transform = self
            .network_players
            .get(entity)
            .map(|(transform, _)| transform)
            .or_else(|_| self.player_roots.get(entity))
            .ok()?;
        Some(
            transform.translation()
                + Vec3::Y * (crate::world::AUTHORED_CHARACTER_CONTROLLER_HEIGHT * height_factor),
        )
    }

    pub(in super::super) fn player_health(&self, entity: Entity) -> Option<(i16, f32)> {
        let (_, appearance) = self.network_players.get(entity).ok()?;
        player_health(&appearance.0)
    }

    pub(in super::super) fn localized_player_name(&self, entity: Entity) -> Option<LocalizedText> {
        let (_, appearance) = self.network_players.get(entity).ok()?;
        let first_name = appearance.0.style.first_name.to_string_lossy();
        let last_name = appearance.0.style.last_name.to_string_lossy();
        let display_name = match (first_name.trim(), last_name.trim()) {
            ("", "") => return None,
            (first, "") => first.to_owned(),
            ("", last) => last.to_owned(),
            (first, last) => format!("{first} {last}"),
        };
        Some(LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", display_name))
    }
}

// Published avatar growth contract, also used by the local HUD. Missing levels
// deliberately have no health strip; current HP cannot stand in for maximum HP.
fn player_health(pc: &ffone_protocol::PcAppearance0104) -> Option<(i16, f32)> {
    if !(1..=40).contains(&pc.level) {
        return None;
    }
    let bonus = match pc.style.class {
        2 => 50,
        3 => 20,
        _ => 0,
    };
    let maximum = (925 + i32::from(pc.level) * 75) * (100 + bonus) / 100;
    Some((
        pc.level,
        (pc.hp.max(0) as f32 / maximum as f32).clamp(0.0, 1.0),
    ))
}

#[cfg(test)]
mod player_health_tests {
    use super::*;
    #[test]
    fn remote_health_uses_authoritative_level_and_class_and_rejects_missing_level() {
        let mut pc =
            ffone_protocol::PcAppearance0104::decode(&[0; ffone_protocol::PcAppearance0104::SIZE])
                .unwrap();
        assert_eq!(player_health(&pc), None);
        pc.level = 1;
        pc.hp = 500;
        assert_eq!(player_health(&pc), Some((1, 0.5)));
        pc.style.class = 2;
        pc.hp = 750;
        assert_eq!(player_health(&pc), Some((1, 0.5)));
        pc.hp = -1;
        assert_eq!(player_health(&pc), Some((1, 0.0)));
        pc.level = 41;
        assert_eq!(player_health(&pc), None);
    }
}
