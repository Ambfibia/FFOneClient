//! Combat frame/danger overlay and the combat-mode notice.

use super::assets::GameplayUiAssets;
use super::hud::GameplayUiRect;
use super::model::GameplayUiModel;
use crate::{
    avatar_action::LegacyAvatarActionContext,
    legacy_npc_nano_animation::LegacyNanoStandRandomStream, localization::LocalizedText,
};
use bevy::{prelude::*, ui::widget::NodeImageMode, window::PrimaryWindow};

pub const COMBAT_DANGER_RECT: GameplayUiRect = GameplayUiRect::new(0.0, 0.0, 302.0, 90.0);

pub const COMBAT_FRAME_PATH: &str = "ui/en/gameplay/shared/combat-frame.png";
pub const COMBAT_FRAME_BYTES: u64 = 1_947;
pub const COMBAT_FRAME_SHA256: &str =
    "ecbeed2fbc2104a1ae4d4ce56c310faaf545f0bfdf9495972e6ef9ea5a316a36";
pub(super) const COMBAT_DANGER_PATH: &str = "ui/en/gameplay/shared/danger.png";

#[derive(Component)]
pub(super) struct CombatFrame;
#[derive(Component)]
pub(super) struct CombatDanger;

// Location and combat-mode messages share the source's single display slot.
#[derive(Resource, Default)]
pub struct CombatModeNotice {
    pub(super) enabled: bool,
    pub(super) elapsed: Option<f32>,
    pub(super) location: Option<String>,
    pub(super) movement: usize,
}

impl CombatModeNotice {
    pub fn show(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.location = None;
        self.movement = usize::from(!enabled);
        self.elapsed = Some(0.0);
    }

    pub fn show_location(&mut self, name: &str, random: &mut LegacyNanoStandRandomStream) {
        self.location = Some(name.to_owned());
        self.movement = random.next_index(8);
        self.elapsed = Some(0.0);
    }
}

#[derive(Component)]
pub(super) struct CombatModeNoticeText {
    pub(super) shadow: bool,
}

pub(super) fn update_combat_mode_notice(
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    model: Res<GameplayUiModel>,
    mut notice: ResMut<CombatModeNotice>,
    localization: Option<Res<crate::localization::Localization>>,
    language: Option<Res<crate::localization::Language>>,
    mut labels: Query<(
        &CombatModeNoticeText,
        &mut Node,
        &mut LocalizedText,
        &mut TextColor,
        &mut UiTransform,
    )>,
) {
    if !model.visible {
        notice.elapsed = None;
    }
    if let Some(elapsed) = notice.elapsed.as_mut() {
        *elapsed += time.delta_secs();
        if *elapsed > 2.0 {
            notice.elapsed = None;
        }
    }
    let Ok(window) = windows.single() else { return };
    let text = if let Some(location) = &notice.location {
        crate::localization::localized_world_location_text(location)
    } else if notice.enabled {
        LocalizedText::new("ui.gameplay.combat.enabled", "Combat mode enabled")
    } else {
        LocalizedText::new("ui.gameplay.combat.disabled", "Combat mode disabled")
    };
    let resolved = match (localization.as_deref(), language.as_deref()) {
        (Some(catalog), Some(language)) => catalog.text(language, &text),
        _ => text.fallback.clone(),
    };
    for (label, mut node, mut localized, mut color, mut transform) in &mut labels {
        let Some(elapsed) = notice.elapsed else {
            if node.display != Display::None {
                node.display = Display::None;
            }
            continue;
        };
        let mut t = elapsed / 2.0;
        t += (t * 2.0 * 3.14).sin() * 0.1;
        let travel = t + (t * 2.0 * 3.14).sin() * 0.15;
        let wave = (t * 6.0 * 3.14).sin() * (t * 3.14).cos().abs() * 0.1;
        let (x, y) = match notice.movement {
            0 => (travel, 0.25),
            1 => (1.0 - travel, 0.25),
            2 => (0.5, travel),
            3 => (0.5, 1.0 - travel),
            4 => (travel, 0.25 + wave),
            5 => (1.0 - travel, 0.25 + wave),
            6 => (0.5 + wave, travel),
            _ => (0.5 + wave, 1.0 - travel),
        };
        let half_text = resolved.encode_utf16().count() as f32 * 8.0;
        let shadow = if label.shadow { 1.0 } else { 0.0 };
        node.display = Display::Flex;
        node.left = px(x * window.width() - half_text + shadow);
        node.top = px(window.height() * y + shadow);
        node.width = px(half_text * 2.0 + 20.0);
        transform.scale = Vec2::splat(model.ui_scale);
        localized.set_if_neq(text.clone());
        color.0 = if label.shadow {
            Color::srgba(0.0, 0.0, 0.0, ((1.0 - t) * 2.0).clamp(0.0, 1.0))
        } else {
            Color::srgba(1.0, 1.0, 1.0, ((1.0 - t) * 2.0).clamp(0.0, 1.0))
        };
    }
}

/// Exact `GameFrame.OnGUI` full-screen combat overlay. The source stretches
/// Texture2D path ID 384 to the current Screen rectangle and multiplies its
/// alpha by a one-hertz realtime sine pulse.
pub(super) fn spawn_combat_frame(parent: &mut ChildSpawnerCommands, assets: &GameplayUiAssets) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            display: Display::None,
            left: Val::ZERO,
            top: Val::ZERO,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        ImageNode {
            image: assets.combat_frame.clone(),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        // Unity GUI.depth 2 is behind the ordinary default-depth HUD. Bevy's
        // Z axis is opposite (larger is nearer), so retain that ordering with
        // a negative sibling index.
        ZIndex(-2),
        Pickable::IGNORE,
        CombatFrame,
    ));
}

#[must_use]
pub(super) fn combat_frame_alpha(elapsed_seconds: f32) -> f32 {
    if !elapsed_seconds.is_finite() {
        return 0.0;
    }
    ((elapsed_seconds * std::f32::consts::TAU).sin() + 1.0) * 0.5
}

pub(super) fn bind_combat_frame(
    model: Res<GameplayUiModel>,
    time: Res<Time<Real>>,
    players: Query<&LegacyAvatarActionContext>,
    mut frame: Single<(&mut Node, &mut ImageNode), With<CombatFrame>>,
) {
    let visible = model.visible
        && players
            .iter()
            .any(|context| context.combat_condition && !context.tutorial_event);
    frame.0.display = if visible {
        Display::Flex
    } else {
        Display::None
    };
    let alpha = combat_frame_alpha(time.elapsed_secs());
    frame.1.color = Color::srgba(1.0, 1.0, 1.0, alpha);
}

pub(super) fn spawn_combat_danger(parent: &mut ChildSpawnerCommands, assets: &GameplayUiAssets) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            display: Display::None,
            left: percent(50),
            top: px(COMBAT_DANGER_RECT.y),
            width: px(COMBAT_DANGER_RECT.width),
            height: px(COMBAT_DANGER_RECT.height),
            margin: UiRect {
                left: px(-COMBAT_DANGER_RECT.width * 0.5),
                ..default()
            },
            ..default()
        },
        ImageNode {
            image: assets.combat_danger.clone(),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        ZIndex(880),
        CombatDanger,
    ));
}
