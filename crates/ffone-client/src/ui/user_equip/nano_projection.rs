//! Read-only projection of authoritative Nano data into the Nano gallery and status panels.

use super::geometry::{
    USER_EQUIP_MERGED_NANO_ORDER, USER_EQUIP_NANO_COLUMNS, USER_EQUIP_NANO_GALLERY_COUNT,
};
use super::item_projection::semantic_presentation_icon;
use super::view_model::UserEquipPresentationIcon;
use crate::tutorial_mission_content::TutorialMissionContent;
use bevy::prelude::*;
use ffone_protocol::Nano0104;
use std::array;

/// Read-only authority supplied by the world runtime for one of the clean
/// three equipped-Nano slots. It deliberately carries no mutation endpoint.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UserEquipNanoEquippedAuthority {
    pub nano_id: Option<i16>,
    pub skill_id: i16,
    pub stamina: i16,
    pub active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserEquipNanoGalleryEntryProjection {
    pub nano_id: i16,
    pub name: String,
    pub attribute: String,
    pub description: String,
    pub skills: [UserEquipNanoSkillProjection; 3],
    pub sort_number: i32,
    pub column: usize,
    pub row: usize,
    pub owned: bool,
    pub equipped: bool,
    pub current_power: Option<i16>,
    /// Grid presentation: clean uses `nanoready` while the Nano is unowned.
    pub icon: UserEquipPresentationIcon,
    /// Detail-window portrait: clean always draws the full-color `nanoicon`.
    pub viewer_icon: UserEquipPresentationIcon,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UserEquipNanoSkillProjection {
    pub tune_id: i32,
    pub skill_id: i32,
    pub name: String,
    pub type_label: String,
    pub description: String,
    pub icon: UserEquipPresentationIcon,
    pub required_item_id: i32,
    pub required_item_name: String,
    pub required_item_count: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UserEquipNanoStatusProjection {
    pub slot_index: usize,
    pub nano_id: Option<i16>,
    pub name: String,
    pub attribute: String,
    pub style: Option<u8>,
    pub stamina: i16,
    pub max_stamina: i16,
    pub current_power: Option<i16>,
    pub nano_icon: UserEquipPresentationIcon,
    pub skill_icon: UserEquipPresentationIcon,
    pub active: bool,
}

impl UserEquipNanoStatusProjection {
    #[must_use]
    pub fn stamina_fraction(&self) -> f32 {
        if self.nano_id.is_none() || self.max_stamina <= 0 {
            0.0
        } else {
            (f32::from(self.stamina.max(0)) / f32::from(self.max_stamina)).min(1.0)
        }
    }
}

/// Clean UserEquip presentation extended over every Nano explicitly published
/// by the patched TableData. Bank entries remain the sole ownership authority.
#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct UserEquipNanoModeProjection {
    pub gallery: Vec<UserEquipNanoGalleryEntryProjection>,
    pub status: [UserEquipNanoStatusProjection; 3],
}

pub(super) fn user_equip_merged_nano_rank(nano_id: i16) -> Option<usize> {
    USER_EQUIP_MERGED_NANO_ORDER
        .iter()
        .position(|candidate| *candidate == nano_id)
}

impl UserEquipNanoModeProjection {
    #[must_use]
    pub fn from_authoritative(
        bank: &[Nano0104],
        equipped: [UserEquipNanoEquippedAuthority; 3],
        content: &TutorialMissionContent,
    ) -> Self {
        let status = array::from_fn(|slot_index| {
            project_user_equip_nano_status(slot_index, equipped[slot_index], content)
        });
        let equipped_ids = status
            .iter()
            .filter_map(|slot| slot.nano_id)
            .collect::<Vec<_>>();
        let mut gallery = content
            .gameplay_nanos()
            .filter_map(|definition| {
                let nano_id = definition.nano_id;
                let journal = content.journal_nano(i32::from(nano_id));
                // Preserve the accepted gallery order by native identity.
                // Display names may be shared by distinct Nano variants.
                (definition.sort_number > 0 && user_equip_merged_nano_rank(nano_id).is_some()).then(
                    || {
                        // Production 0104 captures can store a Nano at an array
                        // index different from its ID. Clean checks the slot
                        // entity identity, so ownership must be identity-based.
                        let stored = bank
                            .iter()
                            .copied()
                            .find(|stored| stored.id == nano_id)
                            .unwrap_or(Nano0104 {
                                id: 0,
                                skill_id: 0,
                                stamina: 0,
                            });
                        let owned = stored.id == nano_id;
                        let selected_skill = owned.then_some(i32::from(stored.skill_id));
                        let selected_index = selected_skill.and_then(|skill_id| {
                            journal.and_then(|journal| {
                                journal
                                    .skills
                                    .iter()
                                    .position(|skill| skill.skill_id == skill_id)
                            })
                        });
                        let skill_order = match selected_index {
                            Some(1) => [1, 0, 2],
                            Some(2) => [2, 0, 1],
                            _ => [0, 1, 2],
                        };
                        let icon_path = if owned {
                            definition.icon_path.as_deref()
                        } else {
                            definition
                                .ready_icon_path
                                .as_deref()
                                .or(definition.icon_path.as_deref())
                        };
                        UserEquipNanoGalleryEntryProjection {
                            nano_id,
                            name: definition.name.clone(),
                            attribute: definition.attribute.clone(),
                            description: journal
                                .map(|journal| journal.description.clone())
                                .unwrap_or_default(),
                            skills: array::from_fn(|skill_index| {
                                let Some(skill) = journal
                                    .map(|journal| &journal.skills[skill_order[skill_index]])
                                else {
                                    return UserEquipNanoSkillProjection::default();
                                };
                                let icon = content
                                    .gameplay_user_equip_icon(12, 0, skill.skill_id, 2)
                                    .and_then(|icon| icon.icon_path.as_deref());
                                UserEquipNanoSkillProjection {
                                    tune_id: skill.tune_id,
                                    skill_id: skill.skill_id,
                                    name: skill.name.clone(),
                                    type_label: skill.type_label.clone(),
                                    description: skill.description.clone(),
                                    icon: semantic_presentation_icon(icon),
                                    required_item_id: skill.required_item_id,
                                    required_item_name: i16::try_from(skill.required_item_id)
                                        .ok()
                                        .and_then(|item_id| {
                                            content.gameplay_user_equip_item_detail(7, item_id)
                                        })
                                        .map(|item| item.name)
                                        .unwrap_or_default(),
                                    required_item_count: skill.required_item_count,
                                }
                            }),
                            sort_number: definition.sort_number,
                            column: 0,
                            row: 0,
                            owned,
                            equipped: equipped_ids.contains(&nano_id),
                            current_power: owned.then_some(stored.skill_id).filter(|id| *id > 0),
                            icon: semantic_presentation_icon(icon_path),
                            viewer_icon: semantic_presentation_icon(
                                definition.icon_path.as_deref(),
                            ),
                        }
                    },
                )
            })
            .collect::<Vec<_>>();
        gallery.sort_by_key(|entry| {
            user_equip_merged_nano_rank(entry.nano_id).unwrap_or(USER_EQUIP_NANO_GALLERY_COUNT)
        });
        for (visual_index, entry) in gallery.iter_mut().enumerate() {
            entry.sort_number = visual_index as i32;
            entry.column = visual_index % USER_EQUIP_NANO_COLUMNS;
            entry.row = visual_index / USER_EQUIP_NANO_COLUMNS;
        }
        Self { gallery, status }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

pub(super) fn project_user_equip_nano_status(
    slot_index: usize,
    authority: UserEquipNanoEquippedAuthority,
    content: &TutorialMissionContent,
) -> UserEquipNanoStatusProjection {
    let Some(nano_id) = authority.nano_id.filter(|id| *id > 0) else {
        return UserEquipNanoStatusProjection {
            slot_index,
            ..default()
        };
    };
    let Some(definition) = content.gameplay_nano(nano_id) else {
        return UserEquipNanoStatusProjection {
            slot_index,
            ..default()
        };
    };
    let current_power = (authority.skill_id > 0).then_some(authority.skill_id);
    let skill_icon = current_power
        .and_then(|skill_id| content.gameplay_user_equip_icon(12, 0, i32::from(skill_id), 2))
        .and_then(|icon| icon.icon_path.as_deref());
    UserEquipNanoStatusProjection {
        slot_index,
        nano_id: Some(nano_id),
        name: definition.name.clone(),
        attribute: definition.attribute.clone(),
        style: Some(definition.style),
        stamina: authority.stamina,
        max_stamina: definition.max_stamina,
        current_power,
        nano_icon: semantic_presentation_icon(definition.icon_path.as_deref()),
        skill_icon: semantic_presentation_icon(skill_icon),
        active: authority.active,
    }
}
