//! Combat target info, matchup, target icons and HUD NPC naming.

use super::assets::GameplayUiAssets;

use super::combat_frame::CombatDanger;

use super::hud::GameplayUiRect;

use super::model::GameplayUiModel;

use super::text::{spawn_localized_middle_left_text, spawn_middle_left_text};

use crate::{
    avatar_action::{LegacyAvatarActionContext, LegacyAvatarActionState},
    localization::LocalizedText,
};

use bevy::{prelude::*, ui::widget::NodeImageMode};

mod hud_npc;
mod target_icons;
pub(super) use hud_npc::GameplayHudNpcQuery;
pub use target_icons::{
    NANO_SKILL_TARGET_ICON_BYTES, NANO_SKILL_TARGET_ICON_PATH, NANO_SKILL_TARGET_ICON_SHA256,
    NANO_SKILL_TARGET_ICON_SIZE,
};
pub(super) use target_icons::{
    NPC_TALK_TARGET_ICON_PATH, PRIMARY_COMBAT_TARGET_ICON_PATH, SECONDARY_COMBAT_TARGET_ICON_PATH,
    bind_nano_skill_target_icons, bind_primary_combat_target_icon,
    bind_primary_combat_target_status, bind_secondary_combat_target_icons,
    friendly_target_ui_visible, spawn_nano_skill_target_icons, spawn_primary_combat_target_icon,
    spawn_primary_combat_target_status, spawn_secondary_combat_target_icons,
};
// Test-only paths: `gameplay_ui::tests` reaches these through `combat_target`.
#[cfg(test)]
pub(super) use hud_npc::{GameplayHudNpc, localized_tutorial_npc_name};
#[cfg(test)]
pub(super) use target_icons::{
    NPC_TALK_TARGET_ICON_SIZE, PrimaryTargetIconKind, combat_target_icon_path, primary_target_icon,
};

pub(super) const COMBAT_TARGET_HP_FILL_PATH: &str = "ui/en/gameplay/group/hp_bar.png";
pub(super) const COMBAT_TARGET_MOB_INFO_PATH: &str = "ui/en/gameplay/shared/mop_info.png";
pub(super) const COMBAT_TARGET_NPC_INFO_PATH: &str = "ui/en/gameplay/shared/npc_info.png";
pub(super) const COMBAT_TARGET_MOB_WIDTH: f32 = 290.0;
pub(super) const COMBAT_TARGET_NPC_WIDTH: f32 = 255.0;
pub(super) const COMBAT_TARGET_FRAME_HEIGHT: f32 = 66.0;
pub(super) const COMBAT_TARGET_MOB_GROUP_HEIGHT: f32 = 76.0;
pub const COMBAT_TARGET_AFFINITY_EFFECT_RECT: GameplayUiRect =
    GameplayUiRect::new(6.0, -13.0, 58.0, 72.0);
pub const COMBAT_TARGET_AFFINITY_ICON_RECT: GameplayUiRect =
    GameplayUiRect::new(25.0, 55.0, 21.0, 21.0);
pub const COMBAT_TARGET_SKILL_RECT: GameplayUiRect = GameplayUiRect::new(260.0, 7.0, 26.0, 26.0);

pub(super) const COMBAT_TARGET_MATCHUP_PATHS: [&str; 3] = [
    "ui/en/gameplay/shared/nano_win.png",
    "ui/en/gameplay/shared/nano_draw.png",
    "ui/en/gameplay/shared/nano_lose.png",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CombatTargetMatchup {
    Win,
    Tie,
    Lose,
}

impl CombatTargetMatchup {
    pub(super) const ALL: [Self; 3] = [Self::Win, Self::Tie, Self::Lose];

    pub(super) const fn index(self) -> usize {
        match self {
            Self::Win => 0,
            Self::Tie => 1,
            Self::Lose => 2,
        }
    }

    pub(super) const fn size(self) -> Vec2 {
        match self {
            Self::Win => Vec2::new(20.0, 16.0),
            Self::Tie => Vec2::new(17.0, 12.0),
            Self::Lose => Vec2::new(15.0, 16.0),
        }
    }
}

/// Exact `CnGuiMonster_info.DoWindowMob` comparison for the local active Nano.
/// Invalid/unavailable styles deliberately render no result texture.
pub(super) fn combat_target_matchup(
    mob_style: i32,
    active_nano_style: Option<u8>,
) -> Option<CombatTargetMatchup> {
    let nano_style = i32::from(active_nano_style?);
    if !(0..=2).contains(&mob_style) || !(0..=2).contains(&nano_style) {
        return None;
    }
    Some(if mob_style == nano_style {
        CombatTargetMatchup::Tie
    } else if mob_style - nano_style == 1 || mob_style - nano_style == -2 {
        CombatTargetMatchup::Win
    } else {
        CombatTargetMatchup::Lose
    })
}

#[derive(Component)]
pub(super) struct CombatTargetInfo;
#[derive(Component)]
pub(super) struct CombatTargetInfoBackground;
#[derive(Component)]
pub(super) struct CombatTargetInfoIcon;
#[derive(Component)]
pub(super) struct CombatTargetInfoName;
#[derive(Component)]
pub(super) struct CombatTargetInfoLevel;
#[derive(Component)]
pub(super) struct CombatTargetInfoHealthFill;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CombatTargetOverlayLayer {
    HostileDiskFront,
    AffinityEffectLower,
    AffinityEffectUpper,
    AffinityIcon,
    Matchup,
    FriendlyDiskFront,
}
#[derive(Component)]
pub(super) struct CombatTargetOverlayImage(pub(super) CombatTargetOverlayLayer);

/// Exact top-centre `CnGuiMonster_info` group. Hostile targets use the source
/// `mop_info` frame at 290x66; friendly NPCs use `npc_info` at 255x66 and do
/// not render the monster level/HP fields.
pub(super) fn spawn_combat_target_info(
    parent: &mut ChildSpawnerCommands,
    assets: &GameplayUiAssets,
) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                left: percent(50),
                top: px(5),
                width: px(COMBAT_TARGET_MOB_WIDTH),
                height: px(COMBAT_TARGET_MOB_GROUP_HEIGHT),
                margin: UiRect {
                    left: px(-COMBAT_TARGET_MOB_WIDTH * 0.5),
                    ..default()
                },
                overflow: Overflow::clip(),
                ..default()
            },
            ZIndex(880),
            CombatTargetInfo,
        ))
        .with_children(|info| {
            // `DoWindowMob`/`DoWindowNpc` draws the 66px frame at Window*.y=5
            // inside the group. The mob group alone is extended to 76px; the
            // frame itself is never stretched to that height.
            info.spawn((
                GameplayUiRect::new(
                    0.0,
                    5.0,
                    COMBAT_TARGET_MOB_WIDTH,
                    COMBAT_TARGET_FRAME_HEIGHT,
                )
                .node(),
                ImageNode {
                    image: assets.combat_target_mob_info.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                CombatTargetInfoBackground,
            ));
            info.spawn((
                Node {
                    display: Display::None,
                    ..GameplayUiRect::new(1.0, 49.0, 69.0, 23.0).node()
                },
                ImageNode {
                    image: assets.disk_front.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                CombatTargetOverlayImage(CombatTargetOverlayLayer::HostileDiskFront),
            ));
            spawn_localized_middle_left_text(
                info,
                GameplayUiRect::new(70.0, 0.0, 190.0, 20.0),
                &assets.chalet_font,
                CombatTargetInfoName,
            );
            spawn_middle_left_text(
                info,
                GameplayUiRect::new(90.0, 23.0, 60.0, 20.0),
                "",
                &assets.chalet_font,
                CombatTargetInfoLevel,
            );
            info.spawn((
                Node {
                    display: Display::None,
                    ..COMBAT_TARGET_AFFINITY_EFFECT_RECT.node()
                },
                ImageNode {
                    image: assets.nano_affinity_backs[0].clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                CombatTargetOverlayImage(CombatTargetOverlayLayer::AffinityEffectLower),
            ));
            info.spawn((
                Node {
                    display: Display::None,
                    ..GameplayUiRect::new(5.0, -3.0, 64.0, 64.0).node()
                },
                ImageNode {
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                CombatTargetInfoIcon,
            ));
            info.spawn((
                Node {
                    display: Display::None,
                    ..COMBAT_TARGET_AFFINITY_EFFECT_RECT.node()
                },
                ImageNode {
                    image: assets.nano_affinity_backs[0].clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                CombatTargetOverlayImage(CombatTargetOverlayLayer::AffinityEffectUpper),
            ));
            info.spawn((
                Node {
                    display: Display::None,
                    ..COMBAT_TARGET_AFFINITY_ICON_RECT.node()
                },
                ImageNode {
                    image: assets.nano_affinity_icons[0].clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                CombatTargetOverlayImage(CombatTargetOverlayLayer::AffinityIcon),
            ));
            crate::damage_bar::spawn_damage_bar(
                info,
                GameplayUiRect::new(58.0, 22.0, 199.0, 4.0).node(),
                ImageNode {
                    image: assets.combat_target_hp_fill.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                CombatTargetInfoHealthFill,
            );
            info.spawn((
                Node {
                    display: Display::None,
                    ..GameplayUiRect::new(0.0, 0.0, 20.0, 16.0).node()
                },
                ImageNode {
                    image: assets.combat_target_matchups[0].clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                CombatTargetOverlayImage(CombatTargetOverlayLayer::Matchup),
            ));
            // `DoWindowNpc` draws DiskFront after the NPC portrait, while the
            // hostile branch draws its copy immediately after the frame.
            info.spawn((
                Node {
                    display: Display::None,
                    ..GameplayUiRect::new(1.0, 49.0, 69.0, 23.0).node()
                },
                ImageNode {
                    image: assets.disk_front.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                CombatTargetOverlayImage(CombatTargetOverlayLayer::FriendlyDiskFront),
            ));
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_combat_target_info(
    model: Res<GameplayUiModel>,
    assets: Res<GameplayUiAssets>,
    asset_server: Res<AssetServer>,
    players: Query<(&LegacyAvatarActionState, &LegacyAvatarActionContext)>,
    targets: GameplayHudNpcQuery,
    mut info: Single<
        &mut Node,
        (
            With<CombatTargetInfo>,
            Without<CombatDanger>,
            Without<CombatTargetOverlayImage>,
        ),
    >,
    mut background: Single<
        (&mut Node, &mut ImageNode),
        (
            With<CombatTargetInfoBackground>,
            Without<CombatTargetInfo>,
            Without<CombatTargetInfoHealthFill>,
            Without<CombatTargetInfoIcon>,
            Without<CombatTargetOverlayImage>,
            Without<CombatDanger>,
        ),
    >,
    mut icon: Single<
        (&mut Node, &mut ImageNode),
        (
            With<CombatTargetInfoIcon>,
            Without<CombatTargetInfo>,
            Without<CombatTargetInfoBackground>,
            Without<CombatTargetInfoHealthFill>,
            Without<CombatTargetOverlayImage>,
            Without<CombatDanger>,
        ),
    >,
    mut name: Single<
        &mut LocalizedText,
        (With<CombatTargetInfoName>, Without<CombatTargetInfoLevel>),
    >,
    mut level: Single<
        &mut LocalizedText,
        (With<CombatTargetInfoLevel>, Without<CombatTargetInfoName>),
    >,
    mut hp_fill: Single<
        (&mut Node, &mut crate::damage_bar::DamageBarOwner),
        (
            With<CombatTargetInfoHealthFill>,
            Without<CombatTargetInfoBackground>,
            Without<CombatTargetInfoIcon>,
            Without<CombatTargetInfo>,
            Without<CombatTargetOverlayImage>,
            Without<CombatDanger>,
        ),
    >,
    mut overlays: Query<
        (&CombatTargetOverlayImage, &mut Node, &mut ImageNode),
        (
            Without<CombatTargetInfo>,
            Without<CombatTargetInfoBackground>,
            Without<CombatTargetInfoIcon>,
            Without<CombatTargetInfoHealthFill>,
            Without<CombatDanger>,
        ),
    >,
    mut danger: Single<
        &mut Node,
        (
            With<CombatDanger>,
            Without<CombatTargetInfo>,
            Without<CombatTargetInfoBackground>,
            Without<CombatTargetInfoIcon>,
            Without<CombatTargetInfoHealthFill>,
            Without<CombatTargetOverlayImage>,
        ),
    >,
) {
    info.display = Display::None;
    icon.0.display = Display::None;
    danger.display = Display::None;
    for (_, mut node, _) in &mut overlays {
        node.display = Display::None;
    }
    if !model.visible {
        return;
    }
    let combat_condition = players.iter().any(|(_, context)| context.combat_condition);
    danger.display = if combat_condition {
        Display::Flex
    } else {
        Display::None
    };
    let Some(target) = players
        .iter()
        .find_map(|(state, _)| state.target_selection.focused_npc)
    else {
        return;
    };
    let Some((_, target_npc)) = targets.get(target.entity) else {
        return;
    };
    let hostile =
        matches!(target.kind, crate::avatar_action::LegacyTargetKind::Npc { team } if team != 1);
    // `cnAvatarAttack.UpdateTargetUI` keeps the friendly NPC in its broad
    // 15-degree/25-unit view result, but calls `MobInfo(null)` until the
    // source-specific talk range is reached. Keeping this panel visible
    // at the full view distance made FFOne appear to target NPCs from afar.
    if !friendly_target_ui_visible(target) {
        return;
    }
    let width = if hostile {
        COMBAT_TARGET_MOB_WIDTH
    } else {
        COMBAT_TARGET_NPC_WIDTH
    };
    info.width = px(width);
    info.height = px(if hostile {
        COMBAT_TARGET_MOB_GROUP_HEIGHT
    } else {
        COMBAT_TARGET_FRAME_HEIGHT
    });
    info.margin.left = px(-width * 0.5);
    background.0.width = px(width);
    background.1.image = if hostile {
        assets.combat_target_mob_info.clone()
    } else {
        assets.combat_target_npc_info.clone()
    };
    if let Some(path) = target_npc.portrait_icon_path {
        icon.1.image = asset_server.load(path.to_owned());
        icon.0.display = Display::Flex;
    }
    **name = target_npc.name.localized();
    let level_value = if hostile {
        target_npc
            .level
            .map(|level| format!("{level:02}"))
            .unwrap_or_default()
    } else {
        String::new()
    };
    **level = LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", level_value);
    hp_fill.0.display = if hostile && target_npc.hp_fraction.is_some() {
        Display::Flex
    } else {
        Display::None
    };
    hp_fill.0.width = px(199.0 * target_npc.hp_fraction.unwrap_or(0.0));
    hp_fill.1.0 = target.entity.to_bits();

    let style_index = hostile
        .then_some(target_npc.affinity_style)
        .flatten()
        .and_then(|style| usize::try_from(style).ok())
        .filter(|style| *style < 3);
    let active_nano_style = model
        .nanos
        .iter()
        .find(|nano| nano.active && nano.nano_id.is_some())
        .and_then(|nano| nano.style);
    let matchup = hostile
        .then_some(target_npc.affinity_style)
        .flatten()
        .and_then(|style| combat_target_matchup(style, active_nano_style));
    for (marker, mut node, mut image) in &mut overlays {
        match marker.0 {
            CombatTargetOverlayLayer::HostileDiskFront => {
                node.display = if hostile {
                    Display::Flex
                } else {
                    Display::None
                };
            }
            CombatTargetOverlayLayer::FriendlyDiskFront => {
                node.display = if hostile {
                    Display::None
                } else {
                    Display::Flex
                };
            }
            CombatTargetOverlayLayer::AffinityEffectLower
            | CombatTargetOverlayLayer::AffinityEffectUpper => {
                if let Some(style) = style_index {
                    image.image = assets.nano_affinity_backs[style].clone();
                    node.display = Display::Flex;
                }
            }
            CombatTargetOverlayLayer::AffinityIcon => {
                if let Some(style) = style_index {
                    image.image = assets.nano_affinity_icons[style].clone();
                    node.display = Display::Flex;
                }
            }
            CombatTargetOverlayLayer::Matchup => {
                if let Some(result) = matchup {
                    let size = result.size();
                    node.width = px(size.x);
                    node.height = px(size.y);
                    image.image = assets.combat_target_matchups[result.index()].clone();
                    node.display = Display::Flex;
                }
            }
        }
    }

    info.display = Display::Flex;
    danger.display = Display::None;
}
