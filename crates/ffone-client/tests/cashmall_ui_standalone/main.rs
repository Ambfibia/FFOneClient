//! Focused clean-Retrobution Cash Mall parity tests. The module is included
//! directly so this tranche stays independent from shared crate wiring.

#![allow(dead_code)]
#[allow(unused_imports)]
use ffone_client::{scene_hierarchy, ui_support};

pub use ffone_client::{semantic_audio, ui_startup};

pub mod inventory_runtime {
    pub const INVENTORY_SLOT_COUNT_0104: usize = 50;
}

pub use ffone_client::localization;

pub mod user_equip_ui {
    use bevy::{prelude::*, sprite::BorderRect};
    use ffone_protocol::ItemBase0104;
    use std::array;

    use crate::inventory_runtime::INVENTORY_SLOT_COUNT_0104;

    pub const USER_EQUIP_BACKDROP_PATH: &str = "ui/en/user-equip/backdrop.png";
    pub const USER_EQUIP_RIGHT_PANEL_PATH: &str = "ui/en/user-equip/right-panel.png";
    pub const USER_EQUIP_INVENTORY_PANEL_PATH: &str = "ui/en/user-equip/inventory-panel.png";
    pub const USER_EQUIP_SLOT_OCCUPIED_PATH: &str = "ui/en/user-equip/slot-occupied.png";
    pub const USER_EQUIP_SLOT_EMPTY_PATH: &str = "ui/en/user-equip/slot-empty.png";
    pub const USER_EQUIP_EQUIP_TITLE_PATH: &str = "ui/en/user-equip/equip-title.png";
    pub const USER_EQUIP_NANO_TAB_PATH: &str = "ui/en/user-equip/nano-tab.png";
    pub const USER_EQUIP_CLOSE_PATH: &str = "ui/en/user-equip/close.png";
    pub const USER_EQUIP_TRASH_PATH: &str = "ui/en/user-equip/trash.png";
    pub const USER_EQUIP_HELP_PATH: &str = "ui/en/user-equip/help.png";
    pub const USER_EQUIP_COMBINED_PATH: &str = "ui/en/user-equip/combined.png";
    pub const USER_EQUIP_FONT_PATH: &str = "fonts/jeffe.otf";
    pub const USER_EQUIP_BACKDROP_WIDTH: f32 = 1_920.0;
    pub const USER_EQUIP_BACKDROP_HEIGHT: f32 = 1_440.0;
    pub const USER_EQUIP_INVENTORY_COLUMNS: usize = 5;
    pub const USER_EQUIP_INVENTORY_SLOT_SIZE: f32 = 67.0;
    pub const USER_EQUIP_INVENTORY_SLOT_STRIDE: f32 = 69.0;
    pub const USER_EQUIP_INVENTORY_CONTENT_WIDTH: f32 = 345.0;
    pub const USER_EQUIP_INVENTORY_CONTENT_HEIGHT: f32 = 690.0;
    pub const USER_EQUIP_EQUIPMENT_STRIP_COUNT: usize = 9;
    pub const USER_EQUIP_EQUIPMENT_SLOT_SIZE: f32 = 64.0;
    pub const USER_EQUIP_EQUIPMENT_SLOT_STRIDE: f32 = 62.0;
    pub const USER_EQUIP_COMBINED_BADGE_LEFT: f32 = 36.0;
    pub const USER_EQUIP_COMBINED_BADGE_TOP: f32 = 36.0;
    pub const USER_EQUIP_COMBINED_BADGE_SIZE: f32 = 26.0;
    pub const USER_EQUIP_COUNT_LABEL_LEFT: f32 = 5.0;
    pub const USER_EQUIP_COUNT_LABEL_TOP: f32 = 5.0;
    pub const USER_EQUIP_COUNT_FONT_SIZE: f32 = 12.0;
    pub const USER_EQUIP_TAB_FONT_SIZE: f32 = 12.0;
    pub const USER_EQUIP_REGULAR_FONT_LINE_HEIGHT: f32 = 13.560_000_42;
    pub const USER_EQUIP_SMALL_FONT_SIZE: f32 = 7.0;
    pub const USER_EQUIP_SMALL_FONT_LINE_HEIGHT: f32 = 6.780_000_21;
    pub const USER_EQUIP_INVENTORY_PANEL_BORDER: BorderRect = BorderRect {
        min_inset: bevy::math::Vec2::new(136.0, 33.0),
        max_inset: bevy::math::Vec2::new(10.0, 77.0),
    };
    pub const USER_EQUIP_RIGHT_PANEL_BORDER: BorderRect = BorderRect {
        min_inset: bevy::math::Vec2::new(5.0, 0.0),
        max_inset: bevy::math::Vec2::new(5.0, 0.0),
    };

    #[derive(Clone, Copy, Debug, Default, PartialEq)]
    pub struct UserEquipUiRect {
        pub left: f32,
        pub top: f32,
        pub width: f32,
        pub height: f32,
    }

    impl UserEquipUiRect {
        pub const fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
            Self {
                left,
                top,
                width,
                height,
            }
        }

        pub const fn translated(self, x: f32, y: f32) -> Self {
            Self::new(self.left + x, self.top + y, self.width, self.height)
        }
    }

    pub const USER_EQUIP_PC_STUFF_RECT: UserEquipUiRect =
        UserEquipUiRect::new(0.0, 0.0, 380.0, 632.0);
    pub const USER_EQUIP_EQUIP_STRIP_RECT: UserEquipUiRect =
        UserEquipUiRect::new(0.0, 0.0, 66.0, 639.0);
    pub const USER_EQUIP_INVENTORY_VIEWPORT_RECT: UserEquipUiRect =
        UserEquipUiRect::new(6.0, 36.0, 366.0, 500.0);
    pub const USER_EQUIP_EQUIPMENT_CONTENT_RECT: UserEquipUiRect =
        UserEquipUiRect::new(0.0, 14.0, 66.0, 625.0);
    pub const USER_EQUIP_EQUIPMENT_TITLE_RECT: UserEquipUiRect =
        UserEquipUiRect::new(0.0, 3.0, 64.0, 13.0);
    pub const USER_EQUIP_NANO_TAB_TEXTURE_RECT: UserEquipUiRect =
        UserEquipUiRect::new(96.0, 0.0, 129.0, 29.0);
    pub const USER_EQUIP_ITEM_TAB_HIT_RECT: UserEquipUiRect =
        UserEquipUiRect::new(10.0, 5.0, 105.0, 15.0);
    pub const USER_EQUIP_NANO_TAB_HIT_RECT: UserEquipUiRect =
        UserEquipUiRect::new(135.0, 5.0, 60.0, 15.0);
    pub const USER_EQUIP_CLOSE_RECT: UserEquipUiRect = UserEquipUiRect::new(400.0, 5.0, 30.0, 30.0);
    pub const USER_EQUIP_TRASH_RECT: UserEquipUiRect =
        UserEquipUiRect::new(390.0, 555.0, 32.0, 32.0);
    pub const USER_EQUIP_HELP_RECT: UserEquipUiRect =
        UserEquipUiRect::new(390.0, 590.0, 32.0, 32.0);

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub enum UserEquipEquipmentSlotKind {
        Head,
        Face,
        Back,
        UpperBody,
        LowerBody,
        Foot,
        PrimaryWeapon,
        SecondaryWeapon,
        Vehicle,
    }

    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub struct UserEquipEquipmentSlotSpec {
        pub kind: UserEquipEquipmentSlotKind,
        pub label_key: &'static str,
        pub label_ordinal: Option<u8>,
    }

    pub const USER_EQUIP_EQUIPMENT_STRIP_ORDER: [UserEquipEquipmentSlotSpec; 9] = [
        UserEquipEquipmentSlotSpec {
            kind: UserEquipEquipmentSlotKind::Head,
            label_key: "HEAD",
            label_ordinal: None,
        },
        UserEquipEquipmentSlotSpec {
            kind: UserEquipEquipmentSlotKind::Face,
            label_key: "FACE",
            label_ordinal: None,
        },
        UserEquipEquipmentSlotSpec {
            kind: UserEquipEquipmentSlotKind::Back,
            label_key: "BACK",
            label_ordinal: None,
        },
        UserEquipEquipmentSlotSpec {
            kind: UserEquipEquipmentSlotKind::UpperBody,
            label_key: "CHEST",
            label_ordinal: None,
        },
        UserEquipEquipmentSlotSpec {
            kind: UserEquipEquipmentSlotKind::LowerBody,
            label_key: "LEGS",
            label_ordinal: None,
        },
        UserEquipEquipmentSlotSpec {
            kind: UserEquipEquipmentSlotKind::Foot,
            label_key: "FEET",
            label_ordinal: None,
        },
        UserEquipEquipmentSlotSpec {
            kind: UserEquipEquipmentSlotKind::PrimaryWeapon,
            label_key: "WEAPON",
            label_ordinal: Some(1),
        },
        UserEquipEquipmentSlotSpec {
            kind: UserEquipEquipmentSlotKind::SecondaryWeapon,
            label_key: "WEAPON",
            label_ordinal: Some(2),
        },
        UserEquipEquipmentSlotSpec {
            kind: UserEquipEquipmentSlotKind::Vehicle,
            label_key: "VEHICLE",
            label_ordinal: None,
        },
    ];

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct UserEquipIconRef(String);

    impl UserEquipIconRef {
        pub fn new(runtime_path: impl Into<String>) -> Result<Self, ()> {
            let runtime_path = runtime_path.into();
            if runtime_path.starts_with("icons/") && runtime_path.ends_with(".png") {
                Ok(Self(runtime_path))
            } else {
                Err(())
            }
        }

        pub fn runtime_path(&self) -> &str {
            &self.0
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub enum UserEquipProjectedIcon {
        Empty,
        Resolved(UserEquipIconRef),
        MissingChecker(()),
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct UserEquipItemProjection {
        pub item: ItemBase0104,
        pub empty: bool,
        pub show_combined_badge: bool,
        pub icon: UserEquipProjectedIcon,
    }

    impl Default for UserEquipItemProjection {
        fn default() -> Self {
            Self {
                item: ItemBase0104 {
                    item_type: 0,
                    item_id: 0,
                    option: 0,
                    time_limit: 0,
                },
                empty: true,
                show_combined_badge: false,
                icon: UserEquipProjectedIcon::Empty,
            }
        }
    }

    impl UserEquipItemProjection {
        fn from_preview(item: ItemBase0104, icon: Option<&str>) -> Self {
            let empty = item.item_id == 0;
            Self {
                item,
                empty,
                show_combined_badge: !empty
                    && (0..4).contains(&item.item_type)
                    && ((item.option as u32) >> 16) > 0,
                icon: if empty {
                    UserEquipProjectedIcon::Empty
                } else if let Some(icon) = icon.and_then(|path| UserEquipIconRef::new(path).ok()) {
                    UserEquipProjectedIcon::Resolved(icon)
                } else {
                    UserEquipProjectedIcon::MissingChecker(())
                },
            }
        }
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct UserEquipInventorySlotProjection {
        pub item: UserEquipItemProjection,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    pub struct UserEquipEquipmentSlotProjection {
        pub item: UserEquipItemProjection,
    }

    #[derive(Clone, Debug, Eq, PartialEq, Resource)]
    pub struct UserEquipItemModeProjection {
        pub inventory: [UserEquipInventorySlotProjection; INVENTORY_SLOT_COUNT_0104],
        pub equipment: [UserEquipEquipmentSlotProjection; USER_EQUIP_EQUIPMENT_STRIP_COUNT],
    }

    impl Default for UserEquipItemModeProjection {
        fn default() -> Self {
            Self {
                inventory: array::from_fn(|_| UserEquipInventorySlotProjection {
                    item: Default::default(),
                }),
                equipment: array::from_fn(|_| UserEquipEquipmentSlotProjection {
                    item: Default::default(),
                }),
            }
        }
    }

    impl UserEquipItemModeProjection {
        pub fn set_preview_inventory(
            &mut self,
            slot: usize,
            item: ItemBase0104,
            icon: Option<&str>,
        ) {
            self.inventory[slot].item = UserEquipItemProjection::from_preview(item, icon);
        }

        pub fn set_preview_equipment(
            &mut self,
            slot: usize,
            item: ItemBase0104,
            icon: Option<&str>,
        ) {
            self.equipment[slot].item = UserEquipItemProjection::from_preview(item, icon);
        }
    }

    #[derive(Clone, Debug, Default, Eq, PartialEq)]
    pub enum UserEquipPresentationIcon {
        #[default]
        Empty,
        Resolved(String),
        MissingChecker,
    }

    impl UserEquipPresentationIcon {
        pub fn from_projection(icon: &UserEquipProjectedIcon) -> Self {
            match icon {
                UserEquipProjectedIcon::Empty => Self::Empty,
                UserEquipProjectedIcon::Resolved(icon) => {
                    Self::Resolved(icon.runtime_path().to_owned())
                }
                UserEquipProjectedIcon::MissingChecker(_) => Self::MissingChecker,
            }
        }
    }

    #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
    pub enum UserEquipSlotFrameVisual {
        #[default]
        Empty,
        Occupied,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct UserEquipItemModeLayout {
        pub pc_stuff_panel: UserEquipUiRect,
        pub equipment_panel: UserEquipUiRect,
        pub inventory_viewport: UserEquipUiRect,
    }

    pub fn clamp_user_equip_scroll(value: f32) -> f32 {
        if value.is_nan() || value <= 0.0 {
            0.0
        } else {
            value.min(190.0)
        }
    }

    pub fn user_equip_item_mode_layout(
        viewport_width: u32,
        viewport_height: u32,
        opening_elapsed_seconds: f32,
        _scroll_y: f32,
    ) -> UserEquipItemModeLayout {
        let width = viewport_width.min(i32::MAX as u32) as i32;
        let height = viewport_height.min(i32::MAX as u32) as i32;
        let center_x = if width > 1_020 {
            (width - 1_020) / 2
        } else {
            0
        };
        let top = ((height - 638) / 2).max(0);
        let eased = if opening_elapsed_seconds <= 0.0 {
            0.0
        } else if opening_elapsed_seconds >= 1.0 {
            1.0
        } else {
            (opening_elapsed_seconds * std::f32::consts::FRAC_PI_2).sin()
        };
        let remaining = 1.0 - eased;
        let pc_final = 585 + center_x;
        let equip_final = 504 + center_x;
        let pc_x = pc_final + ((1_020 - pc_final) as f32 * remaining) as i32;
        let equip_x = equip_final + ((1_020 - equip_final) as f32 * remaining) as i32;
        UserEquipItemModeLayout {
            pc_stuff_panel: USER_EQUIP_PC_STUFF_RECT.translated(pc_x as f32, top as f32),
            equipment_panel: USER_EQUIP_EQUIP_STRIP_RECT.translated(equip_x as f32, top as f32),
            inventory_viewport: USER_EQUIP_INVENTORY_VIEWPORT_RECT
                .translated(pc_x as f32, top as f32),
        }
    }

    pub fn user_equip_missing_checker_rgba() -> Vec<u8> {
        let mut rgba = vec![0; 64 * 64 * 4];
        for y in 0..64 {
            for x in 0..64 {
                let offset = (y * 64 + x) * 4;
                let color = if (x / 32 + y / 32) % 2 == 0 {
                    [255, 0, 255, 255]
                } else {
                    [0, 0, 0, 255]
                };
                rgba[offset..offset + 4].copy_from_slice(&color);
            }
        }
        rgba
    }
}

pub mod vendor_ui {
    use bevy::sprite::BorderRect;

    pub const VENDOR_PANEL_PATH: &str = "ui/en/vendor/vendor-panel.png";
    pub const VENDOR_INFO_PATH: &str = "ui/en/vendor/info.png";
    pub const VENDOR_LIST_BACK_PATH: &str = "ui/en/vendor/list-back.png";
    pub const VENDOR_BUTTON_NORMAL_PATH: &str = "ui/en/vendor/button-normal.png";
    pub const VENDOR_BUTTON_HOVER_PATH: &str = "ui/en/vendor/button-hover.png";
    pub const VENDOR_RESTRICTED_ITEM_FRAME_PATH: &str = "ui/en/vendor/restricted-item-frame.png";
    pub const VENDOR_SCROLL_SHADOW_PATH: &str = "ui/en/vendor/scroll-shadow.png";
    pub const VENDOR_LIST_BACK_BORDER: BorderRect = BorderRect {
        min_inset: bevy::math::Vec2::new(6.0, 14.0),
        max_inset: bevy::math::Vec2::new(6.0, 55.0),
    };
    pub const VENDOR_SCROLL_SHADOW_BORDER: BorderRect = BorderRect {
        min_inset: bevy::math::Vec2::new(0.0, 32.0),
        max_inset: bevy::math::Vec2::new(0.0, 32.0),
    };
    pub const VENDOR_BUTTON_BORDER: BorderRect = BorderRect {
        min_inset: bevy::math::Vec2::new(6.0, 6.0),
        max_inset: bevy::math::Vec2::new(6.0, 4.0),
    };
}
#[path = "../../src/ui/cashmall/mod.rs"]
pub mod cashmall_ui;

#[cfg(test)]
use bevy::{
    asset::AssetPlugin, prelude::*, sprite::BorderRect, text::LineHeight, ui::widget::NodeImageMode,
};
#[cfg(test)]
use cashmall_ui::*;
#[cfg(test)]
use ffone_protocol::ItemBase0104;
#[cfg(test)]
use localization::LocalizedText;
#[cfg(test)]
use sha2::{Digest, Sha256};
#[cfg(test)]
use std::{fs, path::Path};
#[cfg(test)]
use tempfile::{TempDir, tempdir};

#[path = "operations.rs"]
mod operations;
#[path = "state.rs"]
mod state;
#[path = "containers.rs"]
mod containers;
#[path = "layout.rs"]
mod layout;
#[path = "interaction.rs"]
mod interaction;
#[path = "codec.rs"]
mod codec;
#[path = "audio.rs"]
mod audio;
#[path = "assets.rs"]
mod assets;
#[path = "textures.rs"]
mod textures;

#[cfg(test)]
use operations::{item, one_row_projection, spawned_cashmall_app};
#[cfg(test)]
use state::{inventory_item, visible_state};
