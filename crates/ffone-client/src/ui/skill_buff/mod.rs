//! Exact clean-Retrobution skill-buff icon rows.
//!
//! `CnGuiSkillBuffIcon` is deliberately small and passive. It owns no
//! tooltips, hover input, stack counts or normal-buff countdowns. Membership
//! comes only from server condition bit flags. Cash buffs alone retain and
//! display a coarse client-side countdown, while target icons mirror the
//! selected character's `Status.iSkillconditionBitFlag`.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    assets::AssetLocator,
    gameplay_ui::{GameplayUiSet, NanoWheelTransientUi},
    localization::{LocalizationSet, LocalizedText},
    mission_ui::{MissionUiModel, gameplay_chrome_visible},
    tutorial_mission_content::TutorialMissionContent,
};
use bevy::{
    prelude::*,
    text::{LineBreak, LineHeight},
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};
use ffone_protocol::{
    CharTimeBuffTimeout0104, PcBuffUpdate0104, PcCashBuffUpdate0104, PcLoadData0104,
    SkillBuffPacket0104, TimeBuffDotDamageTick0104,
};

#[cfg(test)]
mod tests;

mod constants;
mod assets_skill_buff_ui_catalog;
mod layout;
mod types;
mod models;
mod operations;
mod projection;
mod localization_skill_buff_cash_time_localized;
mod view;
mod systems;

pub use constants::{
    SKILL_BUFF_INFECTION_FLAG, SKILL_BUFF_SOURCE_BUILD, SKILL_BUFF_LEGACY_CLASS,
    SKILL_BUFF_ICON_SIZE, SKILL_BUFF_LOCAL_CENTER_X, SKILL_BUFF_LOCAL_Y, SKILL_BUFF_CASH_X,
    SKILL_BUFF_CASH_Y, SKILL_BUFF_TARGET_CENTER_OFFSET_X, SKILL_BUFF_TARGET_Y,
    SKILL_BUFF_MAX_ICONS, SKILL_BUFF_MAX_ID, SKILL_BUFF_FONT_SIZE, SKILL_BUFF_FONT_Y_OFFSET,
    SKILL_BUFF_LABEL_COLOR, SKILL_BUFF_BUFFS, SKILL_BUFF_DEBUFFS
};
use constants::STIM_FLAGS;
pub use assets_skill_buff_ui_catalog::{
    SKILL_BUFF_GAME_HUD_PATH_ID, SKILL_BUFF_COMPONENT_PATH_ID, SKILL_BUFF_SCRIPT_PATH_ID,
    SKILL_BUFF_SKIN_PATH_ID, SKILL_BUFF_BACK_PATH_ID, SKILL_BUFF_FONT_PATH_ID,
    SKILL_BUFF_BACK_PATH, SKILL_BUFF_FONT_PATH, SKILL_BUFF_UI_Z_INDEX, SkillBuffUiCatalog
};
pub use layout::{SKILL_BUFF_FONT_LINE_HEIGHT, SkillBuffUiRect};
use layout::{icon_rect, bind_rect};
pub use types::{
    SkillBuffTextAnchor, SkillBuffTextStyle, SkillBuffTextStyleSpec, SkillBuffUiDefinition,
    SkillBuffTargetUi, SkillBuffIconView, SkillBuffUiView, SkillBuffUiRoot, SkillBuffUiSet,
    SkillBuffUiPlugin
};
use types::{IconProjection, SkillBuffRow, SkillBuffUiElement, SkillBuffUiAssets};
pub use models::SkillBuffUiModel;
pub use operations::{skill_buff_ui_view, format_cash_time};
use operations::{cash_buff_ids, bind_skill_buff_ui};
#[cfg(test)]
use operations::{local_buff_ids, target_buff_ids};
use projection::project_icons;
pub use localization_skill_buff_cash_time_localized::skill_buff_cash_time_localized;
use view::spawn_skill_buff_ui;
use systems::{advance_skill_buff_cash_timer, sync_skill_buff_nano_gumballs};
