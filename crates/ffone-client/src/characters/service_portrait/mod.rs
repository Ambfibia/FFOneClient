//! Presentation endpoints for live NPC service portraits.
use bevy::prelude::*;

/// Each surface owns an independent render target, including waiting panels.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServicePortraitSlot {
    CombiPrimary,
    CombiWaiting,
    EnchantPrimary,
    EnchantWaiting,
    Vendor,
}

impl ServicePortraitSlot {
    pub const ALL: [Self; 5] = [
        Self::CombiPrimary,
        Self::CombiWaiting,
        Self::EnchantPrimary,
        Self::EnchantWaiting,
        Self::Vendor,
    ];

    pub const fn index(self) -> usize {
        self as usize
    }

    pub fn size(self) -> UVec2 {
        use crate::{combi_ui::*, enchant_ui::*};
        match self {
            Self::Vendor => UVec2::new(200, 150),
            Self::CombiPrimary => UVec2::new(
                COMBI_PRIMARY_NPC_PREVIEW_RECT.width as u32,
                COMBI_PRIMARY_NPC_PREVIEW_RECT.height as u32,
            ),
            Self::CombiWaiting => UVec2::new(
                COMBI_WAITING_NPC_PREVIEW_RECT.width as u32,
                COMBI_WAITING_NPC_PREVIEW_RECT.height as u32,
            ),
            Self::EnchantPrimary => UVec2::new(
                ENCHANT_PRIMARY_NPC_RECT.width as u32,
                ENCHANT_PRIMARY_NPC_RECT.height as u32,
            ),
            Self::EnchantWaiting => UVec2::new(
                ENCHANT_WAITING_NPC_RECT.width as u32,
                ENCHANT_WAITING_NPC_RECT.height as u32,
            ),
        }
    }
}
