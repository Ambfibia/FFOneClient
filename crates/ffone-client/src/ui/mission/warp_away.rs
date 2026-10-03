//! Warp-away countdown timing and presentation.

use super::assets::MissionUiAssets;
use super::components::MissionUiView;
use super::model::MissionUiModel;
use super::widgets::{mission_text_color, mission_text_font};
use crate::{
    gameplay_ui::GameplayUiOutbox, gui_skin::gui_style as mission_gui_style,
    localization::LocalizedText,
};
use bevy::{prelude::*, text::LineBreak};

pub const WARP_AWAY_DELAY_SECONDS: f32 = 20.0;
pub const WARP_AWAY_COOLDOWN_SECONDS: f32 = 60.0;

pub(super) fn spawn_warp_away_countdown(root: &mut ChildSpawnerCommands, assets: &MissionUiAssets) {
    let style = mission_gui_style("FusionFallChatSkin", "BigFont70")
        .expect("clean Retrobution Warp Away countdown style must remain converted");
    debug_assert_eq!(style.alignment, 4);
    root.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            display: Display::None,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            overflow: Overflow::clip(),
            ..default()
        },
        MissionUiView::WarpAwayCountdownRoot,
        Pickable::IGNORE,
    ))
    .with_child((
        Text::new("WARP\n20"),
        LocalizedText::new("ui.mission.chat.warp_countdown", "WARP\n{seconds}")
            .with_arg("seconds", "20"),
        mission_text_font(assets, "FusionFallChatSkin", "BigFont70"),
        TextColor(mission_text_color(style)),
        TextLayout::new(Justify::Center, LineBreak::NoWrap),
        MissionUiView::WarpAwayCountdownText,
        Pickable::IGNORE,
    ));
}

pub(super) fn advance_warp_away(
    time: Res<Time>,
    mut model: ResMut<MissionUiModel>,
    mut outbox: ResMut<GameplayUiOutbox>,
) {
    if model.warp_away_countdown_seconds.is_none() && model.warp_away_cooldown_seconds <= 0.0 {
        return;
    }
    model.advance_warp_away(time.delta_secs(), &mut outbox);
}
