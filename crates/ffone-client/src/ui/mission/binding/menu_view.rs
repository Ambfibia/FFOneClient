//! Binding of Nanocom, chat quick-menu, warp-away countdown and system dialog views.

use super::super::components::MissionUiView;
use super::super::labels::mission_ui_localized_text;
use super::MissionUiBindContext;
use crate::localization::LocalizedText;
use bevy::prelude::*;

/// Nanocom, chat quick-menu, warp-away countdown and system dialog views.
pub(super) fn bind_menu_view(
    context: &MissionUiBindContext<'_>,
    view: &MissionUiView,
    node: Option<Mut<Node>>,
    localized_text: Option<Mut<LocalizedText>>,
    _image: Option<Mut<ImageNode>>,
) {
    let MissionUiBindContext { model, transition, system_dialog, .. } = *context;
    let mut node = node;
    let mut localized_text = localized_text;
    macro_rules! set_text {
        ($localized:expr) => {
            if let Some(component) = &mut localized_text {
                **component = $localized;
            }
        };
    }
    match *view {
        MissionUiView::NanocomRoot => {
            if let Some(node) = &mut node {
                node.display = if model.enabled && transition.visible_or_transitioning() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::ChatQuickRoot => {
            if let Some(node) = &mut node {
                node.display = if model.enabled && transition.visible_or_transitioning() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::WarpAwayCountdownRoot => {
            if let Some(node) = &mut node {
                node.display = if model.warp_away_countdown_seconds().is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::WarpAwayCountdownText => {
            set_text!(
                LocalizedText::new("ui.mission.chat.warp_countdown", "WARP\n{seconds}",)
                    .with_arg(
                        "seconds",
                        model.warp_away_display_seconds().unwrap_or(0).to_string(),
                    )
            );
        }
        MissionUiView::SystemDialogRoot => {
            if let Some(node) = &mut node {
                node.display = if system_dialog.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
            }
        }
        MissionUiView::SystemDialogPanel => {}
        MissionUiView::SystemDialogText => {
            set_text!(system_dialog.map_or_else(
                || mission_ui_localized_text(""),
                |dialog| mission_ui_localized_text(dialog.text()),
            ));
        }
        _ => {}
    }
}
