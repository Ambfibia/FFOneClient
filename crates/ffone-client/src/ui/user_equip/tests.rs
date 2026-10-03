use super::binding::user_equip_editable_layout_key;
use super::components::{
    UserEquipAvatarTurnControl, UserEquipBoundTextTarget, UserEquipCloseControl,
    UserEquipPopupButtonControl, UserEquipScrollThumbControl, UserEquipScrollTrackControl,
    UserEquipSlotControl,
};
use super::geometry::USER_EQUIP_MERGED_NANO_ORDER;
use super::interaction::{execute_user_equip_one_click, execute_user_equip_popup_command};
use super::item_popup::{
    user_equip_popup_command_rect, user_equip_popup_field_rect, user_equip_popup_variant_layout_key,
};
use super::popup_state::{
    UserEquipPopupLayoutVariant, user_equip_general_popup_variant, user_equip_popup_layout_variant,
};
use super::spawn::{
    spawn_bound_text_styled, user_equip_default_label_top_padding, user_equip_equipfont_node,
};
use super::view_model::equipment_slot_label_color;
use crate::inventory_runtime::EQUIPMENT_SLOT_COUNT_0104;
use crate::inventory_runtime::INVENTORY_SLOT_COUNT_0104;
use crate::inventory_runtime::InventoryRuntime0104;
use crate::tutorial_mission_content::TutorialMissionContent;
use ffone_protocol::ItemBase0104;
use ffone_protocol::Nano0104;
use ffone_ui_layout::UiLayoutDocument;

use crate::assets::AssetLocator;
use crate::user_equip_ui::*;
use bevy::{
    asset::AssetPlugin,
    window::{PrimaryWindow, WindowResolution},
};
use ffone_protocol::{ItemMoveSuccessPacket0104, PcLoadData0104};
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path};
use tempfile::tempdir;

mod operations;
mod constants;
mod output;
mod state;
mod assets_production_table_data_catalog_re;
mod layout;
mod interaction_scrollbar_thumb_drag_and_trough_;
mod frame;
mod textures;
mod audio;
mod projection;

use operations::{item, empty, assert_close, checker_pixel};
use constants::OWNER_PC_ID;
use output::write_item;
use state::runtime_with;
use assets_production_table_data_catalog_re::{TestCatalog, AllCatalog};
