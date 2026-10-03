//! Clean-client rectangles of the mission, Nanocom, chat quick-menu and system-dialog windows.

use super::layout::MissionUiRect;

pub const JOURNAL_WINDOW_RECT: MissionUiRect = MissionUiRect::new(122.0, 33.0, 1036.0, 654.0);
pub const JOURNAL_LEFT_BOX_RECT: MissionUiRect = MissionUiRect::new(16.0, 30.0, 565.0, 612.0);
pub const JOURNAL_RIGHT_BOX_RECT: MissionUiRect = MissionUiRect::new(597.0, 11.0, 382.0, 633.0);
/// `DoWindowRight`'s `GUI.BeginScrollView(Rect(11, 40, 368, 558))` inside
/// `RightBox`; the category headers and mission rows scroll within it.
pub const JOURNAL_SCROLL_VIEW_RECT: MissionUiRect = MissionUiRect::new(
    JOURNAL_RIGHT_BOX_RECT.x + 11.0,
    JOURNAL_RIGHT_BOX_RECT.y + 40.0,
    368.0,
    558.0,
);
/// Unity IMGUI scroll views move by `Event.delta * 20`, and a Windows wheel
/// notch reports a three-line delta.
pub(super) const JOURNAL_SCROLL_WHEEL_LINE: f32 = 60.0;
pub const JOURNAL_ALLOW_GROUP_RECT: MissionUiRect = MissionUiRect::new(8.0, 11.0, 566.0, 636.0);
pub const JOURNAL_ACTIVE_GROUP_RECT: MissionUiRect = MissionUiRect::new(9.0, 28.0, 565.0, 612.0);
pub const JOURNAL_COMPLETED_GROUP_RECT: MissionUiRect = JOURNAL_ACTIVE_GROUP_RECT;
pub const JOURNAL_ACTIVE_TAB_RECT: MissionUiRect = MissionUiRect::new(597.0, 11.0, 100.0, 30.0);
pub const JOURNAL_COMPLETED_TAB_RECT: MissionUiRect = MissionUiRect::new(680.0, 11.0, 100.0, 30.0);
pub const JOURNAL_REWARD_GROUP_RECT: MissionUiRect = JOURNAL_ALLOW_GROUP_RECT;
pub const JOURNAL_CLOSE_RECT: MissionUiRect = MissionUiRect::new(996.0, 11.0, 32.0, 32.0);
/// `rectofferdlg + Rect(339, 551, 218, 85)`.
pub const JOURNAL_ACCEPT_RECT: MissionUiRect = MissionUiRect::new(347.0, 562.0, 218.0, 85.0);
/// `rectofferdlg + Rect(319, 551, 233, 70)` in the reward branch.
pub const JOURNAL_COMPLETE_RECT: MissionUiRect = MissionUiRect::new(327.0, 562.0, 233.0, 70.0);
pub const JOURNAL_MISSION_TITLE_RECT: MissionUiRect = MissionUiRect::new(44.0, 33.0, 317.0, 47.0);
pub const JOURNAL_MISSION_DIFFICULTY_RECT: MissionUiRect =
    MissionUiRect::new(44.0, 79.0, 317.0, 20.0);
pub const JOURNAL_NPC_FRAME_RECT: MissionUiRect = MissionUiRect::new(22.0, 113.0, 68.0, 68.0);
pub const JOURNAL_NPC_NAME_RECT: MissionUiRect = MissionUiRect::new(98.0, 111.0, 300.0, 20.0);
pub const JOURNAL_NPC_POSITION_RECT: MissionUiRect = MissionUiRect::new(98.0, 129.0, 300.0, 20.0);
pub const JOURNAL_OBJECTIVE_HEADER_RECT: MissionUiRect =
    MissionUiRect::new(98.0, 166.0, 300.0, 20.0);
pub const JOURNAL_DESCRIPTION_RECT: MissionUiRect = MissionUiRect::new(58.0, 185.0, 475.0, 100.0);
pub const JOURNAL_REWARD_HEADER_RECT: MissionUiRect = MissionUiRect::new(21.0, 309.0, 300.0, 20.0);
pub const JOURNAL_FM_CARD_RECT: MissionUiRect = MissionUiRect::new(30.0, 336.0, 250.0, 76.0);
pub const JOURNAL_FM_ICON_RECT: MissionUiRect = MissionUiRect::new(40.0, 342.0, 64.0, 64.0);
pub const JOURNAL_FM_AMOUNT_RECT: MissionUiRect = MissionUiRect::new(110.0, 342.0, 170.0, 40.0);
pub const JOURNAL_FM_CAPTION_RECT: MissionUiRect = MissionUiRect::new(110.0, 352.0, 170.0, 40.0);
pub const JOURNAL_TAROS_CARD_RECT: MissionUiRect = MissionUiRect::new(290.0, 336.0, 250.0, 76.0);
pub const JOURNAL_TAROS_ICON_RECT: MissionUiRect = MissionUiRect::new(300.0, 342.0, 64.0, 64.0);
pub const JOURNAL_TAROS_AMOUNT_RECT: MissionUiRect = MissionUiRect::new(370.0, 342.0, 170.0, 40.0);
pub const JOURNAL_TAROS_CAPTION_RECT: MissionUiRect = MissionUiRect::new(370.0, 352.0, 170.0, 40.0);
pub const JOURNAL_COMPLETED_BANNER_RECT: MissionUiRect =
    MissionUiRect::new(21.0, 31.0, 549.0, 74.0);
pub const JOURNAL_COMPLETED_MISSION_TITLE_RECT: MissionUiRect =
    MissionUiRect::new(45.0, 40.0, 317.0, 47.0);
pub const JOURNAL_COMPLETED_MISSION_DIFFICULTY_RECT: MissionUiRect =
    MissionUiRect::new(45.0, 86.0, 317.0, 20.0);
pub const JOURNAL_COMPLETED_NPC_FRAME_RECT: MissionUiRect =
    MissionUiRect::new(23.0, 120.0, 68.0, 68.0);
pub const JOURNAL_COMPLETED_NPC_NAME_RECT: MissionUiRect =
    MissionUiRect::new(99.0, 118.0, 300.0, 20.0);
pub const JOURNAL_COMPLETED_NPC_POSITION_RECT: MissionUiRect =
    MissionUiRect::new(99.0, 136.0, 300.0, 20.0);
pub const JOURNAL_COMPLETED_NOTES_HEADER_RECT: MissionUiRect =
    MissionUiRect::new(35.0, 198.0, 500.0, 18.0);
pub const JOURNAL_COMPLETED_NOTES_RECT: MissionUiRect =
    MissionUiRect::new(35.0, 218.0, 500.0, 100.0);
pub const JOURNAL_COMPLETED_REWARD_HEADER_RECT: MissionUiRect =
    MissionUiRect::new(22.0, 343.0, 300.0, 20.0);
pub const JOURNAL_COMPLETED_REWARD_VIEW_RECT: MissionUiRect =
    MissionUiRect::new(21.0, 368.0, 577.0, 245.0);
pub const JOURNAL_COMPLETED_NANO_LABEL_RECT: MissionUiRect =
    MissionUiRect::new(24.0, 345.0, 200.0, 30.0);
pub const JOURNAL_COMPLETED_NANO_GROUP_RECT: MissionUiRect =
    MissionUiRect::new(14.0, 362.0, 555.0, 245.0);

pub const NPC_SINGLE_MISSION_RECT: MissionUiRect = MissionUiRect::new(794.0, 268.5, 332.0, 183.0);
pub const NPC_WARP_AND_CLOSE_RECT: MissionUiRect = MissionUiRect::new(839.0, 274.5, 242.0, 171.0);
pub const NPC_CLOSE_RECT: MissionUiRect = MissionUiRect::new(839.0, 303.0, 242.0, 114.0);

pub const NANOCOM_MENU_RECT: MissionUiRect = MissionUiRect::new(1102.0, 112.0, 178.0, 294.0);
pub const NANOCOM_JOURNAL_RECT: MissionUiRect = MissionUiRect::new(11.0, 100.0, 156.0, 26.0);
/// Clean `cnGUINanocom.RenderMenu` draws this after leaving the inner button
/// group. `bkRect` is still the mutated inner rect at that point: it has moved
/// by `(10, 50)` since the menu background draw. The native menu node already
/// owns the source's right-edge group adaptation, so its verified local X
/// remains `153`; carrying the mutated Y through yields `(153, 42, 23, 23)`.
pub const NANOCOM_CLOSE_RECT: MissionUiRect = MissionUiRect::new(153.0, 42.0, 23.0, 23.0);
/// The fully-open large-chat `MenuChatScript.EmoteMenuPos` captured by the
/// clean Retrobution HUD. Its child rectangles come from `InitLargeChatStyle`.
pub const CHAT_QUICK_MENU_RECT: MissionUiRect = MissionUiRect::new(0.0, 1.0, 170.0, 127.0);
pub const CHAT_WINDOW_HEIGHT: f32 = 150.0;
pub const CHAT_EMOTE_CONTAINER_HEIGHT: f32 = 235.0;
pub const CHAT_QUICK_GROUP_RECT: MissionUiRect = MissionUiRect::new(10.0, 22.0, 155.0, 25.0);
pub const CHAT_QUICK_WARP_RECT: MissionUiRect = MissionUiRect::new(10.0, 52.0, 155.0, 25.0);
pub const CHAT_QUICK_VEHICLE_RECT: MissionUiRect = MissionUiRect::new(10.0, 82.0, 155.0, 25.0);

pub const SYSTEM_DIALOG_OVERLAY_ALPHA: f32 = 0.75;
pub const SYSTEM_DIALOG_PANEL_RECT: MissionUiRect = MissionUiRect::new(0.0, 0.0, 600.0, 164.0);
pub const SYSTEM_DIALOG_CONTENT_RECT: MissionUiRect = MissionUiRect::new(140.0, 20.0, 420.0, 104.0);
pub const SYSTEM_DIALOG_OKAY_RECT: MissionUiRect = MissionUiRect::new(454.0, 124.0, 110.0, 25.0);
pub const SYSTEM_DIALOG_CANCEL_RECT: MissionUiRect = MissionUiRect::new(44.0, 124.0, 150.0, 25.0);
pub const SYSTEM_DIALOG_ICON_RECT: MissionUiRect = MissionUiRect::new(60.0, 27.0, 62.0, 62.0);
pub const JOURNAL_HELP_RECT: MissionUiRect = MissionUiRect::new(986.0, 608.0, 30.0, 30.0);
pub const JOURNAL_ALLOW_SECONDARY_RECT: MissionUiRect =
    MissionUiRect::new(24.0, 589.0, 190.0, 25.0);
pub const JOURNAL_ACTIVE_DELETE_RECT: MissionUiRect = MissionUiRect::new(24.0, 602.0, 190.0, 25.0);
pub const JOURNAL_ACTIVE_MAKE_CURRENT_RECT: MissionUiRect =
    MissionUiRect::new(321.0, 602.0, 243.0, 25.0);
