//! Semantic asset routes, fonts and the loaded image set of the mission UI.

use crate::{
    character_selection_ui::{
        CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH, CHARACTER_SELECTION_BLUE_BUTTON_PATH,
        CHARACTER_SELECTION_CANCEL_NORMAL_PATH, CHARACTER_SELECTION_RED_BUTTON_OVER_PATH,
        CHARACTER_SELECTION_RED_BUTTON_PATH,
    },
    gameplay_ui::NpcServiceKind,
    option_ui::OPTION_JEFFE_FONT_PATH,
    tutorial_mission_content::TutorialMissionContent,
};
use bevy::prelude::*;

pub(super) const MISSION_TOP: &str = "ui/en/gameplay/mission/npc/mission_top.png";
pub(super) const MISSION_BODY: &str = "ui/en/gameplay/mission/npc/mission_body.png";
pub(super) const MISSION_BOTTOM: &str = "ui/en/gameplay/mission/npc/mission_bottom_func.png";
pub(super) const MISSION_BOTTOM_MULTI: &str = "ui/en/gameplay/mission/npc/mission_bottom_func2.png";
pub(super) const MISSION_BUTTON: &str = "ui/en/gameplay/mission/npc/mission_button.png";
pub(super) const MISSION_BUTTON_OVER: &str = "ui/en/gameplay/mission/npc/mission_button_over.png";
pub(super) const NPC_SEL_BAR: &str = "ui/en/gameplay/journal/sel_bar.png";
pub(super) const NPC_WORLD_MISSION_ICON: &str = "ui/en/gameplay/journal/world_icon.png";
pub(super) const NPC_BLUE_BUTTON: &str = "ui/en/gameplay/chat/blue_button_normal.png";
pub(super) const NPC_BLUE_BUTTON_OVER: &str = "ui/en/gameplay/chat/blue_button_over.png";
pub(super) const MISSION_BACK: &str = "ui/en/gameplay/mission/npc/mission_back.png";
pub(super) const CURRENT_MISSION: &str = "ui/en/gameplay/mission/npc/current_mission.png";
pub(super) const NPC_WINDOW: &str = "ui/en/gameplay/mission/npc/npc_window.png";
pub(super) const NPC_MULTI_WINDOW: &str = "ui/en/gameplay/mission/npc/npc_multi_window.png";
pub(super) const NPCICON_WARP: &str = "ui/en/gameplay/mission/npc/npcicon_warp.png";
pub(super) const NPCICON_EXIT: &str = "ui/en/gameplay/mission/npc/npcicon_exit.png";
pub(super) const NPCICON_VENDOR: &str = "ui/en/gameplay/journal/npcicon_vendor.png";
pub(super) const NPCICON_NANOTUNE: &str = "ui/en/gameplay/journal/npcicon_nanotune.png";
pub(super) const NPCICON_BANK: &str = "ui/en/gameplay/journal/npcicon_bank.png";
pub(super) const NPCICON_GUIDE: &str = "ui/en/gameplay/journal/npcicon_guide.png";
pub(super) const NPCICON_EP: &str = "ui/en/gameplay/journal/npcicon_ep.png";
// The clean service-button array reuses the race flag for the rank row.
pub(super) const NPCICON_RANK: &str = "ui/en/gameplay/journal/npcicon_08.png";
// Clean primary authority: `Icons.resourceFile` (5,800,411 bytes), serialized
// asset `CustomAssetBundle-784fa24bcf2da4f5eabe9547958616eb`, container
// routes `icons/npcicon_10.png` and `icons/npcicon_11.png`. These are the
// actual category-16 monkey/wyvern and category-15 S.C.A.M.P. service icons;
// a visually similar journal icon is not an accepted substitute.
pub(super) const NPCICON_MONKEY: &str = "ui/en/gameplay/journal/npcicon_10.png";
pub(super) const NPCICON_SCAMP: &str = "ui/en/gameplay/journal/npcicon_11.png";
// Combine has its own service glyph. Enchant and Rule intentionally share the
// clean question-bubble glyph; none of these routes are NPC portraits.
pub(super) const NPCICON_COMBINE: &str = "ui/en/gameplay/journal/npcicon_20.png";
pub(super) const NPCICON_ENCHANT: &str = "ui/en/gameplay/journal/npcicon_22.png";
pub(super) const NPCICON_RULE: &str = "ui/en/gameplay/journal/npcicon_23.png";
pub(super) const ACTIVE_DIALOG: &str = "ui/en/gameplay/mission/journal/activedlg.png";
pub(super) const END_DIALOG: &str = "ui/en/gameplay/mission/journal/enddlg.png";
pub(super) const OFFER_DIALOG: &str = "ui/en/gameplay/mission/journal/offdlg.png";
pub(super) const ACCEPT_BUTTON: &str = "ui/en/gameplay/mission/journal/acceptbut.png";
pub(super) const JOURNAL_CLOSE: &str = "ui/en/gameplay/mission/journal/close.png";
pub(super) const JOURNAL_CLOSE_OVER: &str = "ui/en/gameplay/mission/journal/close_over.png";
pub(super) const JOURNAL_REWARD_BOX: &str = "ui/en/gameplay/mission/journal/reward_box.png";
pub(super) const JOURNAL_ITEM_SLOT: &str = "ui/en/gameplay/mission/journal/item_slot.png";
pub(super) const JOURNAL_FM_ICON: &str = "ui/en/gameplay/mission/journal/fmicon.png";
pub(super) const JOURNAL_TAROS_ICON: &str = "ui/en/gameplay/journal/tarosicon.png";
pub(super) const JOURNAL_BACKDROP: &str = "ui/en/shared/panelback.png";
pub(super) const JOURNAL_WINDOW: &str = "ui/en/gameplay/mission/journal/window.png";
pub(super) const JOURNAL_NPC_ICON_BACK: &str = "ui/en/gameplay/mission/journal/npc_icon_back.png";
pub(super) const JOURNAL_ALLOW_RIGHT: &str = "ui/en/gameplay/mission/journal/allow_right.png";
pub(super) const JOURNAL_ACTIVE_PANEL: &str = "ui/en/gameplay/journal/active_tab.png";
pub(super) const JOURNAL_COMPLETED_PANEL: &str = "ui/en/gameplay/journal/compl_tab.png";
pub(super) const JOURNAL_ACTIVE_TAB: &str = "ui/en/gameplay/journal/inventab.png";
pub(super) const JOURNAL_ACTIVE_TAB_OVER: &str = "ui/en/gameplay/journal/inventabover.png";
pub(super) const JOURNAL_COMPLETED_TAB: &str = "ui/en/shared/nanotab.png";
pub(super) const JOURNAL_COMPLETED_TAB_OVER: &str = "ui/en/gameplay/journal/nanotabover.png";
// The clean `GUI.Box(..., GUI.skin.GetStyle("window"))` composite is the black
// ramp serialized by the `onNormal` Shadow state. The normal Shadow object's
// white RGB below the alpha ramp produces bright bars with Bevy's blend.
pub(super) const JOURNAL_RIGHT_FRAME: &str = "ui/en/gameplay/journal/shadow.png";
pub(super) const JOURNAL_SELECTED_MISSION_BACK: &str = "ui/en/gameplay/mission/npc/sel_mission_back.png";
pub(super) const JOURNAL_TRACKED_MISSION_BACK: &str = "ui/en/gameplay/journal/cu_mission_back.png";
pub(super) const JOURNAL_CURRENT_POINT: &str = "ui/en/gameplay/journal/point.png";
pub(super) const JOURNAL_CURRENT_CHECK: &str = "ui/en/character/creation/body/checkbox/CCCheckboxChecked.png";
pub(super) const JOURNAL_CURRENT_CHECK_OVER: &str =
    "ui/en/character/creation/body/checkbox/CCCheckboxCkeckedOver.png";
pub(super) const JOURNAL_UNCHECKED: &str = "ui/en/character/creation/body/checkbox/CCCheckboxNormal.png";
pub(super) const JOURNAL_UNCHECKED_OVER: &str = "ui/en/character/creation/body/checkbox/CCCheckboxOver.png";
pub(super) const JOURNAL_CATEGORY_EXPAND: &str = "ui/en/gameplay/journal/ex_normal.png";
pub(super) const JOURNAL_CATEGORY_EXPAND_OVER: &str = "ui/en/gameplay/journal/ex_over.png";
pub(super) const JOURNAL_CATEGORY_COLLAPSE: &str = "ui/en/gameplay/journal/co_normal.png";
pub(super) const JOURNAL_CATEGORY_COLLAPSE_OVER: &str = "ui/en/gameplay/journal/co_over.png";
pub(super) const JOURNAL_EMPTY_BOX: &str = "ui/en/gameplay/journal/leftbox.png";
pub(super) const JOURNAL_HELP: &str = "ui/en/world-map/controls/NanoMachineHelpButton.png";
pub(super) const JOURNAL_HELP_OVER: &str = "ui/en/world-map/controls/NanoMachineHelpButtonOver.png";
pub(super) const JOURNAL_NANO_CATEGORY_ICON: &str = "ui/en/gameplay/journal/nrnanoicon.png";
pub(super) const JOURNAL_GUIDE_CATEGORY_ICON: &str = "ui/en/gameplay/journal/nrcomicon.png";
pub(super) const JOURNAL_WORLD_CATEGORY_ICON: &str = "ui/en/gameplay/journal/nrworldicon.png";
pub(super) const JOURNAL_WORLD_BANNER: &str = "ui/en/gameplay/journal/world_banner.png";
pub(super) const JOURNAL_NANO_BANNER: &str = "ui/en/gameplay/journal/nano_banner.png";
pub(super) const JOURNAL_NANO_POWER: &str = "ui/en/gameplay/journal/nano_power.png";
pub(super) const JOURNAL_NANO_SKILL_BACK: &str = "ui/en/gameplay/nano/skill/buff_icon_back.png";
pub(super) const JOURNAL_NUMBUH_TWO_PORTRAIT: &str = "icons/entities/npc/npcicon_87.png";
pub(super) const JOURNAL_BUTTERCUP_PORTRAIT: &str = "icons/entities/npc/npcicon_88.png";
pub(super) const JOURNAL_DEXTER_PORTRAIT: &str = "icons/entities/npc/npcicon_00.png";
pub(super) const NANOCOM_MENU_BOX: &str = "ui/en/gameplay/mission/nanocom/menu_box.png";
pub(super) const NANOCOM_CLOSE: &str = "ui/en/shared/close_normal.png";
pub(super) const NANOCOM_CLOSE_OVER: &str = "ui/en/shared/close_over.png";
pub(super) const CHAT_QUICK_MENU_BOX: &str = "ui/en/gameplay/journal/menuchat_box2.png";
pub(super) const SYSTEM_DIALOG_BOX: &str = "ui/en/gameplay/system/systemDialogBox.png";
pub(super) const SYSTEM_DIALOG_ICON_BOX: &str = "ui/en/gameplay/journal/equipbox.png";
pub(super) const SYSTEM_DIALOG_WARNING: &str = "ui/en/gameplay/journal/messicon_warning.png";
pub(super) const CHALET_FONT: &str = "fonts/chaletbook-regular.ttf";
pub(super) const JEFFE_14_SOURCE_FONT_PATH_ID: i64 = 903;
pub(super) const JEFFE_14_REPLACEMENT_FONT_SIZE: f32 = 12.0;
pub(super) const JEFFE_14_LINE_HEIGHT: f32 = 13.710_000_04;
pub(super) const CENTERED_MENU_JEFFE_FONT_SIZE: f32 = 14.0;
pub(super) const CENTERED_MENU_JEFFE_VERTICAL_SCALE: f32 = 0.7;

#[derive(Clone, Resource)]
pub(super) struct MissionUiAssets {
    pub(super) mission_top: Handle<Image>,
    pub(super) mission_body: Handle<Image>,
    pub(super) mission_bottom: Handle<Image>,
    pub(super) mission_bottom_multi: Handle<Image>,
    pub(super) mission_button: Handle<Image>,
    pub(super) mission_button_over: Handle<Image>,
    pub(super) npc_sel_bar: Handle<Image>,
    pub(super) npc_world_mission_icon: Handle<Image>,
    pub(super) npc_blue_button: Handle<Image>,
    pub(super) npc_blue_button_over: Handle<Image>,
    pub(super) mission_back: Handle<Image>,
    pub(super) current_mission: Handle<Image>,
    pub(super) npc_window: Handle<Image>,
    pub(super) npc_multi_window: Handle<Image>,
    pub(super) npcicon_warp: Handle<Image>,
    pub(super) npcicon_exit: Handle<Image>,
    pub(super) npcicon_vendor: Handle<Image>,
    pub(super) npcicon_nanotune: Handle<Image>,
    pub(super) npcicon_bank: Handle<Image>,
    pub(super) npcicon_guide: Handle<Image>,
    pub(super) npcicon_ep: Handle<Image>,
    pub(super) npcicon_rank: Handle<Image>,
    pub(super) npcicon_monkey: Handle<Image>,
    pub(super) npcicon_scamp: Handle<Image>,
    pub(super) npcicon_combine: Handle<Image>,
    pub(super) npcicon_enchant: Handle<Image>,
    pub(super) npcicon_rule: Handle<Image>,
    pub(super) npcicon_barber: Handle<Image>,
    pub(super) active_dialog: Handle<Image>,
    pub(super) end_dialog: Handle<Image>,
    pub(super) offer_dialog: Handle<Image>,
    pub(super) accept_button: Handle<Image>,
    pub(super) journal_close: Handle<Image>,
    pub(super) journal_close_over: Handle<Image>,
    pub(super) journal_reward_box: Handle<Image>,
    pub(super) journal_item_slot: Handle<Image>,
    pub(super) journal_fm_icon: Handle<Image>,
    pub(super) journal_taros_icon: Handle<Image>,
    pub(super) journal_backdrop: Handle<Image>,
    pub(super) journal_window: Handle<Image>,
    pub(super) journal_npc_icon_back: Handle<Image>,
    pub(super) journal_allow_right: Handle<Image>,
    pub(super) journal_active_panel: Handle<Image>,
    pub(super) journal_completed_panel: Handle<Image>,
    pub(super) journal_active_tab: Handle<Image>,
    pub(super) journal_active_tab_over: Handle<Image>,
    pub(super) journal_completed_tab: Handle<Image>,
    pub(super) journal_completed_tab_over: Handle<Image>,
    pub(super) journal_right_frame: Handle<Image>,
    pub(super) journal_selected_mission_back: Handle<Image>,
    pub(super) journal_tracked_mission_back: Handle<Image>,
    pub(super) journal_current_point: Handle<Image>,
    pub(super) journal_current_check: Handle<Image>,
    pub(super) journal_current_check_over: Handle<Image>,
    pub(super) journal_unchecked: Handle<Image>,
    pub(super) journal_unchecked_over: Handle<Image>,
    pub(super) journal_category_expand: Handle<Image>,
    pub(super) journal_category_expand_over: Handle<Image>,
    pub(super) journal_category_collapse: Handle<Image>,
    pub(super) journal_category_collapse_over: Handle<Image>,
    pub(super) journal_empty_box: Handle<Image>,
    pub(super) journal_help: Handle<Image>,
    pub(super) journal_help_over: Handle<Image>,
    pub(super) journal_category_icons: [Handle<Image>; 3],
    pub(super) journal_world_banner: Handle<Image>,
    pub(super) journal_nano_banner: Handle<Image>,
    pub(super) journal_nano_power: Handle<Image>,
    pub(super) journal_nano_skill_back: Handle<Image>,
    pub(super) journal_npc_portraits: [Handle<Image>; 3],
    pub(super) nanocom_menu: Handle<Image>,
    pub(super) nanocom_close: Handle<Image>,
    pub(super) nanocom_close_over: Handle<Image>,
    pub(super) chat_quick_menu_box: Handle<Image>,
    pub(super) system_dialog_box: Handle<Image>,
    pub(super) system_dialog_icon_box: Handle<Image>,
    pub(super) system_dialog_warning: Handle<Image>,
    pub(super) blue_button: Handle<Image>,
    pub(super) blue_button_over: Handle<Image>,
    pub(super) red_button: Handle<Image>,
    pub(super) red_button_over: Handle<Image>,
    pub(super) cancel_normal: Handle<Image>,
    pub(super) font: Handle<Font>,
    pub(super) replacement_jeffe_font: Handle<Font>,
}

impl MissionUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            mission_top: asset_server.load(MISSION_TOP),
            mission_body: asset_server.load(MISSION_BODY),
            mission_bottom: asset_server.load(MISSION_BOTTOM),
            mission_bottom_multi: asset_server.load(MISSION_BOTTOM_MULTI),
            mission_button: asset_server.load(MISSION_BUTTON),
            mission_button_over: asset_server.load(MISSION_BUTTON_OVER),
            npc_sel_bar: asset_server.load(NPC_SEL_BAR),
            npc_world_mission_icon: asset_server.load(NPC_WORLD_MISSION_ICON),
            npc_blue_button: asset_server.load(NPC_BLUE_BUTTON),
            npc_blue_button_over: asset_server.load(NPC_BLUE_BUTTON_OVER),
            mission_back: asset_server.load(MISSION_BACK),
            current_mission: asset_server.load(CURRENT_MISSION),
            npc_window: asset_server.load(NPC_WINDOW),
            npc_multi_window: asset_server.load(NPC_MULTI_WINDOW),
            npcicon_warp: asset_server.load(NPCICON_WARP),
            npcicon_exit: asset_server.load(NPCICON_EXIT),
            npcicon_vendor: asset_server.load(NPCICON_VENDOR),
            npcicon_nanotune: asset_server.load(NPCICON_NANOTUNE),
            npcicon_bank: asset_server.load(NPCICON_BANK),
            npcicon_guide: asset_server.load(NPCICON_GUIDE),
            npcicon_ep: asset_server.load(NPCICON_EP),
            npcicon_rank: asset_server.load(NPCICON_RANK),
            npcicon_monkey: asset_server.load(NPCICON_MONKEY),
            npcicon_scamp: asset_server.load(NPCICON_SCAMP),
            npcicon_combine: asset_server.load(NPCICON_COMBINE),
            npcicon_enchant: asset_server.load(NPCICON_ENCHANT),
            npcicon_rule: asset_server.load(NPCICON_RULE),
            npcicon_barber: asset_server.load("ui/en/gameplay/interaction/icons/auction.png"),
            active_dialog: asset_server.load(ACTIVE_DIALOG),
            end_dialog: asset_server.load(END_DIALOG),
            offer_dialog: asset_server.load(OFFER_DIALOG),
            accept_button: asset_server.load(ACCEPT_BUTTON),
            journal_close: asset_server.load(JOURNAL_CLOSE),
            journal_close_over: asset_server.load(JOURNAL_CLOSE_OVER),
            journal_reward_box: asset_server.load(JOURNAL_REWARD_BOX),
            journal_item_slot: asset_server.load(JOURNAL_ITEM_SLOT),
            journal_fm_icon: asset_server.load(JOURNAL_FM_ICON),
            journal_taros_icon: asset_server.load(JOURNAL_TAROS_ICON),
            journal_backdrop: asset_server.load(JOURNAL_BACKDROP),
            journal_window: asset_server.load(JOURNAL_WINDOW),
            journal_npc_icon_back: asset_server.load(JOURNAL_NPC_ICON_BACK),
            journal_allow_right: asset_server.load(JOURNAL_ALLOW_RIGHT),
            journal_active_panel: asset_server.load(JOURNAL_ACTIVE_PANEL),
            journal_completed_panel: asset_server.load(JOURNAL_COMPLETED_PANEL),
            journal_active_tab: asset_server.load(JOURNAL_ACTIVE_TAB),
            journal_active_tab_over: asset_server.load(JOURNAL_ACTIVE_TAB_OVER),
            journal_completed_tab: asset_server.load(JOURNAL_COMPLETED_TAB),
            journal_completed_tab_over: asset_server.load(JOURNAL_COMPLETED_TAB_OVER),
            journal_right_frame: asset_server.load(JOURNAL_RIGHT_FRAME),
            journal_selected_mission_back: asset_server.load(JOURNAL_SELECTED_MISSION_BACK),
            journal_tracked_mission_back: asset_server.load(JOURNAL_TRACKED_MISSION_BACK),
            journal_current_point: asset_server.load(JOURNAL_CURRENT_POINT),
            journal_current_check: asset_server.load(JOURNAL_CURRENT_CHECK),
            journal_current_check_over: asset_server.load(JOURNAL_CURRENT_CHECK_OVER),
            journal_unchecked: asset_server.load(JOURNAL_UNCHECKED),
            journal_unchecked_over: asset_server.load(JOURNAL_UNCHECKED_OVER),
            journal_category_expand: asset_server.load(JOURNAL_CATEGORY_EXPAND),
            journal_category_expand_over: asset_server.load(JOURNAL_CATEGORY_EXPAND_OVER),
            journal_category_collapse: asset_server.load(JOURNAL_CATEGORY_COLLAPSE),
            journal_category_collapse_over: asset_server.load(JOURNAL_CATEGORY_COLLAPSE_OVER),
            journal_empty_box: asset_server.load(JOURNAL_EMPTY_BOX),
            journal_help: asset_server.load(JOURNAL_HELP),
            journal_help_over: asset_server.load(JOURNAL_HELP_OVER),
            journal_category_icons: [
                asset_server.load(JOURNAL_NANO_CATEGORY_ICON),
                asset_server.load(JOURNAL_GUIDE_CATEGORY_ICON),
                asset_server.load(JOURNAL_WORLD_CATEGORY_ICON),
            ],
            journal_world_banner: asset_server.load(JOURNAL_WORLD_BANNER),
            journal_nano_banner: asset_server.load(JOURNAL_NANO_BANNER),
            journal_nano_power: asset_server.load(JOURNAL_NANO_POWER),
            journal_nano_skill_back: asset_server.load(JOURNAL_NANO_SKILL_BACK),
            journal_npc_portraits: [
                asset_server.load(JOURNAL_NUMBUH_TWO_PORTRAIT),
                asset_server.load(JOURNAL_BUTTERCUP_PORTRAIT),
                asset_server.load(JOURNAL_DEXTER_PORTRAIT),
            ],
            nanocom_menu: asset_server.load(NANOCOM_MENU_BOX),
            nanocom_close: asset_server.load(NANOCOM_CLOSE),
            nanocom_close_over: asset_server.load(NANOCOM_CLOSE_OVER),
            chat_quick_menu_box: asset_server.load(CHAT_QUICK_MENU_BOX),
            system_dialog_box: asset_server.load(SYSTEM_DIALOG_BOX),
            system_dialog_icon_box: asset_server.load(SYSTEM_DIALOG_ICON_BOX),
            system_dialog_warning: asset_server.load(SYSTEM_DIALOG_WARNING),
            blue_button: asset_server.load(CHARACTER_SELECTION_BLUE_BUTTON_PATH),
            blue_button_over: asset_server.load(CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH),
            red_button: asset_server.load(CHARACTER_SELECTION_RED_BUTTON_PATH),
            red_button_over: asset_server.load(CHARACTER_SELECTION_RED_BUTTON_OVER_PATH),
            cancel_normal: asset_server.load(CHARACTER_SELECTION_CANCEL_NORMAL_PATH),
            font: asset_server.load(CHALET_FONT),
            replacement_jeffe_font: asset_server.load(OPTION_JEFFE_FONT_PATH),
        }
    }

    pub(super) fn npc_service_icon(&self, service: NpcServiceKind) -> &Handle<Image> {
        match service {
            NpcServiceKind::Vendor => &self.npcicon_vendor,
            NpcServiceKind::NanoStation => &self.npcicon_nanotune,
            NpcServiceKind::Bank | NpcServiceKind::LocalBank => &self.npcicon_bank,
            NpcServiceKind::GuideChanger => &self.npcicon_guide,
            NpcServiceKind::PastWarp => &self.npcicon_warp,
            NpcServiceKind::TransportationWarp => &self.npcicon_scamp,
            NpcServiceKind::TransportationWyvern => &self.npcicon_monkey,
            NpcServiceKind::Race => &self.npcicon_ep,
            NpcServiceKind::RaceRank => &self.npcicon_rank,
            NpcServiceKind::Combine => &self.npcicon_combine,
            NpcServiceKind::Enchant => &self.npcicon_enchant,
            NpcServiceKind::Rule => &self.npcicon_rule,
            NpcServiceKind::Barber => &self.npcicon_barber,
        }
    }

    pub(super) fn fallback_journal_npc_portrait(&self, npc_type: i32) -> &Handle<Image> {
        match npc_type {
            2672 => &self.journal_npc_portraits[1],
            2673 => &self.journal_npc_portraits[2],
            _ => &self.journal_npc_portraits[0],
        }
    }

    pub(super) fn journal_npc_portrait(
        &self,
        content: &TutorialMissionContent,
        asset_server: &AssetServer,
        npc_type: i32,
    ) -> Handle<Image> {
        content
            .gameplay_npc_portrait_icon_path(npc_type)
            .map(|path| asset_server.load(path.to_owned()))
            .unwrap_or_else(|| self.fallback_journal_npc_portrait(npc_type).clone())
    }

    pub(super) fn journal_banner(&self, mission_type: i32) -> &Handle<Image> {
        if mission_type == 2 {
            &self.journal_nano_banner
        } else {
            &self.journal_world_banner
        }
    }
}
