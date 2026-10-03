//! Sort the visible row indexes, never the stored arrays used by XDT references.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum SortKey {
    Index,
    Field(String),
}

impl From<&str> for SortKey {
    fn from(field: &str) -> Self {
        Self::Field(field.into())
    }
}

pub(super) fn identity(table: &str, fields: &[String]) -> SortKey {
    let field = schema::identity(table)
        .or_else(|| match table.rsplit('/').next()? {
            "m_pSkillData" | "m_pSkillManagerData" => Some("m_iSkillNumber"),
            "m_pSkillBuffData" | "m_pSkillBuffElement" => Some("m_iBuffNumber"),
            "m_pShinyData" => Some("m_iShinyID"),
            "m_pWarpData" => Some("m_iWarpNumber"),
            "m_pXComData" => Some("m_iXcomNumber"),
            "m_pEmoteAnimationData" => Some("m_iEmoteNumber"),
            "m_pTransportationWarpLocation" | "m_pBroomstickLocation" => Some("m_iLocationID"),
            _ => None,
        })
        .or_else(|| {
            ["id", "m_iID"]
                .into_iter()
                .find(|key| fields.iter().any(|f| f.as_str() == *key))
        });
    field
        .filter(|key| fields.iter().any(|f| f.as_str() == *key))
        .map(SortKey::from)
        .unwrap_or(SortKey::Index)
}

pub(super) fn heading(e: &XdtEditor, key: &SortKey, title: String) -> String {
    match &e.sort {
        Some((active, descending)) if active == key => {
            format!("{title} {}", if *descending { "↓" } else { "↑" })
        }
        _ => title,
    }
}

impl XdtEditor {
    pub(super) fn toggle_sort(&mut self, key: SortKey) {
        let descending = self
            .sort
            .as_ref()
            .is_some_and(|(active, down)| active == &key && !down);
        self.sort = Some((key, descending));
        self.offset = 0;
        self.refresh();
    }
}
