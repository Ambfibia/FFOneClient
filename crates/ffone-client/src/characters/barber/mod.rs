//! Server-priced appearance drafts. No local appearance/currency writes before confirmation.
use bevy::prelude::*;
use ffone_protocol::{
    DecodedFrame,
    wire_0104::{PcBarberOpenSuccess0104, PcStyle0104},
};
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum BarberField {
    Gender,
    Height,
    Body,
    Skin,
    Hair,
    HairColor,
    Face,
    Eye,
}

impl BarberField {
    pub const ALL: [Self; 8] = [
        Self::Gender,
        Self::Height,
        Self::Body,
        Self::Skin,
        Self::Hair,
        Self::HairColor,
        Self::Face,
        Self::Eye,
    ];
    pub fn value(self, style: &PcStyle0104) -> i8 {
        match self {
            Self::Gender => style.gender,
            Self::Height => style.height,
            Self::Body => style.body,
            Self::Skin => style.skin_color,
            Self::Hair => style.hair_style,
            Self::HairColor => style.hair_color,
            Self::Face => style.face_style,
            Self::Eye => style.eye_color,
        }
    }
    fn set(self, style: &mut PcStyle0104, value: i8) {
        match self {
            Self::Gender => style.gender = value,
            Self::Height => style.height = value,
            Self::Body => style.body = value,
            Self::Skin => style.skin_color = value,
            Self::Hair => style.hair_style = value,
            Self::HairColor => style.hair_color = value,
            Self::Face => style.face_style = value,
            Self::Eye => style.eye_color = value,
        }
    }
    pub fn label(self) -> crate::localization::LocalizedText {
        use crate::localization::LocalizedText;
        let (key, text) = match self {
            Self::Gender => ("gender", "GENDER"),
            Self::Height => ("height", "HEIGHT"),
            Self::Body => ("body", "BODY SHAPE"),
            Self::Skin => ("skin", "SKIN COLOR"),
            Self::Hair => ("hair", "HAIR STYLE"),
            Self::HairColor => ("hair_color", "HAIR COLOR"),
            Self::Face => ("face", "FACE STYLE"),
            Self::Eye => ("eye", "EYE COLOR"),
        };
        LocalizedText::new(format!("ui.barber.{key}"), text)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BarberPhase {
    Hidden,
    Opening,
    Editing,
    Confirming,
}

#[derive(Resource)]
pub struct BarberModel {
    pub phase: BarberPhase,
    pub npc_id: Option<i32>,
    pub original: Option<PcStyle0104>,
    pub draft: Option<PcStyle0104>,
    pub prices: Option<PcBarberOpenSuccess0104>,
    pub hair: [Vec<(i8, String)>; 2],
    pub face: [Vec<(i8, String)>; 2],
    pub palettes: [Vec<Color>; 3],
    pub appearance_keys: std::collections::BTreeMap<(i8, u8, i8), String>,
    pub pages: [usize; 3],
    pub taros: i32,
    pub yaw: f32,
    pub distance: f32,
    pub elapsed: f32,
    pub error: Option<crate::localization::LocalizedText>,
    pub confirm_requested: bool,
    pub close_requested: bool,
}

impl Default for BarberModel {
    fn default() -> Self {
        Self {
            phase: BarberPhase::Hidden,
            npc_id: None,
            original: None,
            draft: None,
            prices: None,
            hair: default(),
            face: default(),
            palettes: default(),
            appearance_keys: default(),
            pages: [0; 3],
            taros: 0,
            yaw: 0.0,
            distance: 2.0,
            elapsed: 0.0,
            error: None,
            confirm_requested: false,
            close_requested: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct BarberInbox(pub VecDeque<DecodedFrame>);

impl BarberModel {
    pub fn active(&self) -> bool {
        self.phase != BarberPhase::Hidden
    }
    pub fn request_open(&mut self, npc_id: i32) {
        if !self.active() {
            *self = Self {
                npc_id: Some(npc_id),
                phase: BarberPhase::Opening,
                ..default()
            };
        }
    }
    pub fn body_allowed(&self) -> bool {
        self.prices.as_ref().is_some_and(|p| p.change_gender > 0)
    }
    pub fn reset(&mut self) {
        if self.phase == BarberPhase::Editing {
            self.draft = self.original.clone();
            self.pages = [0; 3];
        }
    }
    pub fn set_color(&mut self, field: BarberField, index: usize) {
        let palette = match field {
            BarberField::Skin => 0,
            BarberField::HairColor => 1,
            BarberField::Eye => 2,
            _ => return,
        };
        if self.phase != BarberPhase::Editing
            || (field == BarberField::Skin && !self.body_allowed())
        {
            return;
        }
        if index < self.palettes[palette].len()
            && let Ok(code) = i8::try_from(index + 1)
            && let Some(draft) = &mut self.draft
        {
            field.set(draft, code);
        }
    }
    pub fn step(&mut self, field: BarberField, delta: i32) {
        if self.phase != BarberPhase::Editing
            || (matches!(
                field,
                BarberField::Gender | BarberField::Height | BarberField::Body | BarberField::Skin
            ) && !self.body_allowed())
        {
            return;
        }
        let Some(draft) = &mut self.draft else {
            return;
        };
        let gender = usize::from(draft.gender == 2);
        let original_gender = self
            .original
            .as_ref()
            .map_or(gender, |s| usize::from(s.gender == 2));
        let original_hair = self
            .original
            .as_ref()
            .and_then(|s| {
                self.hair[original_gender]
                    .iter()
                    .position(|r| r.0 == s.hair_style)
            })
            .unwrap_or(0);
        let original_face = self
            .original
            .as_ref()
            .and_then(|s| {
                self.face[original_gender]
                    .iter()
                    .position(|r| r.0 == s.face_style)
            })
            .unwrap_or(0);
        let values: Vec<i8> = match field {
            BarberField::Gender => vec![1, 2],
            BarberField::Height => (0..5).collect(),
            BarberField::Body => (0..3).collect(),
            BarberField::Hair => self.hair[gender].iter().map(|r| r.0).collect(),
            BarberField::Face => self.face[gender].iter().map(|r| r.0).collect(),
            _ => return,
        };
        if values.is_empty() {
            return;
        }
        let index = values
            .iter()
            .position(|v| *v == field.value(draft))
            .unwrap_or(0) as i32;
        field.set(
            draft,
            values[(index + delta).rem_euclid(values.len() as i32) as usize],
        );
        if field == BarberField::Gender {
            let gender = usize::from(draft.gender == 2);
            for (field, choices, index) in [
                (BarberField::Hair, &self.hair[gender], original_hair),
                (BarberField::Face, &self.face[gender], original_face),
            ] {
                if let Some(choice) = choices.get(index.min(choices.len().saturating_sub(1))) {
                    field.set(draft, choice.0);
                }
            }
        }
    }
    pub fn charges(&self) -> Vec<(BarberField, i32)> {
        let (Some(a), Some(b), Some(p)) = (&self.original, &self.draft, &self.prices) else {
            return vec![];
        };
        let gender = a.gender != b.gender;
        [
            (BarberField::Gender, gender, p.gender_cost),
            (
                BarberField::Body,
                a.body != b.body || a.height != b.height,
                p.body_cost,
            ),
            (
                BarberField::Skin,
                a.skin_color != b.skin_color,
                p.skin_color_cost,
            ),
            (
                BarberField::Hair,
                gender || a.hair_style != b.hair_style,
                p.hair_style_cost,
            ),
            (
                BarberField::HairColor,
                a.hair_color != b.hair_color,
                p.hair_color_cost,
            ),
            (
                BarberField::Face,
                gender || a.face_style != b.face_style,
                p.face_style_cost,
            ),
            (
                BarberField::Eye,
                a.eye_color != b.eye_color,
                p.eye_color_cost,
            ),
        ]
        .into_iter()
        .filter_map(|(field, changed, cost)| changed.then_some((field, cost)))
        .collect()
    }
    pub fn cost(&self) -> Option<i32> {
        self.charges().iter().try_fold(0i32, |sum, (_, price)| {
            if *price < 0 {
                None
            } else {
                sum.checked_add(*price)
            }
        })
    }
    pub fn can_confirm(&self) -> bool {
        self.phase == BarberPhase::Editing
            && !self.charges().is_empty()
            && self.cost().is_some_and(|cost| cost <= self.taros)
    }
}

/// Commit only appearance fields; identity, flags and progression remain authoritative.
pub fn apply_appearance(target: &mut ffone_protocol::CharacterStyle0104, style: &PcStyle0104) {
    target.gender = style.gender;
    target.face_style = style.face_style;
    target.hair_style = style.hair_style;
    target.hair_color = style.hair_color;
    target.skin_color = style.skin_color;
    target.eye_color = style.eye_color;
    target.height = style.height;
    target.body = style.body;
}

#[cfg(test)]
mod tests;
