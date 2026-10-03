//! `cnGUINanocom.SetNewMail` minimap mail alarm.

use super::*;
use crate::mission_ui::MissionUiModel;

pub(super) const TEXTURE_PATH: &str = "ui/en/gameplay/minimap/new-mail.png";
/// `SetNewMail` arms `fEMailAlarmTime = 2f`; `Update` clears `bNewMail` once
/// the timer drops below zero.
pub const MINIMAP_NEW_MAIL_ALARM_SECONDS: f32 = 2.0;
/// `RenderMenu` draws `GUI.Label(mailRect, mailTexture, "imagewindow2")` in
/// the NanoRect group. `mailRect` is (140,150,30,30) and `FusionFallHUDSkin`
/// `imagewindow2` is MiddleCenter/ImageAbove with zero padding and no
/// background, so the 19x14 `mail_but` bitmap is centered at native size.
pub const MINIMAP_NEW_MAIL_RECT: GameplayUiRect = GameplayUiRect::new(145.5, 158.0, 19.0, 14.0);

/// Clean `cnGUINanocom.bNewMail` plus `fEMailAlarmTime`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Resource)]
pub struct MinimapNewMailAlarm {
    remaining_seconds: Option<f32>,
}

impl MinimapNewMailAlarm {
    pub fn arm(&mut self) {
        self.remaining_seconds = Some(MINIMAP_NEW_MAIL_ALARM_SECONDS);
    }

    #[must_use]
    pub const fn active(&self) -> bool {
        self.remaining_seconds.is_some()
    }

    fn advance(&mut self, delta_seconds: f32) {
        if let Some(remaining) = self.remaining_seconds.as_mut() {
            *remaining -= delta_seconds;
            if *remaining < 0.0 {
                self.remaining_seconds = None;
            }
        }
    }
}

/// `Mathf.Abs(Mathf.Cos(Time.time * PI * 2))`.
#[must_use]
pub fn minimap_new_mail_alpha(elapsed_seconds: f32) -> f32 {
    (elapsed_seconds * std::f32::consts::TAU).cos().abs()
}

#[derive(Component)]
pub(super) struct MinimapNewMailIcon;

pub(super) fn spawn(parent: &mut ChildSpawnerCommands, image: Handle<Image>) {
    parent.spawn((
        Node {
            display: Display::None,
            ..MINIMAP_NEW_MAIL_RECT.node()
        },
        ImageNode {
            image,
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        MinimapNewMailIcon,
        Pickable::IGNORE,
        // Painted by `RenderMenu` after the FM frame, before `RenderMinimap`.
        ZIndex(1),
    ));
}

pub(super) fn bind(
    time: Res<Time>,
    mut alarm: ResMut<MinimapNewMailAlarm>,
    mission: Option<Res<MissionUiModel>>,
    mut icon: Single<(&mut Node, &mut ImageNode), With<MinimapNewMailIcon>>,
) {
    // `Update` counts the alarm down before `OnGUI` paints it.
    if alarm.active() {
        alarm.advance(time.delta_secs());
    }
    let visible =
        alarm.active() && !mission.is_some_and(|mission| mission.nanocom_main_menu_visible);
    let (node, image) = &mut *icon;
    node.reborrow()
        .map_unchanged(|node| &mut node.display)
        .set_if_neq(if visible {
            Display::Flex
        } else {
            Display::None
        });
    if visible {
        let alpha = minimap_new_mail_alpha(time.elapsed_secs());
        image
            .reborrow()
            .map_unchanged(|image| &mut image.color)
            .set_if_neq(Color::srgba(1.0, 1.0, 1.0, alpha));
    }
}

#[cfg(test)]
mod tests;
