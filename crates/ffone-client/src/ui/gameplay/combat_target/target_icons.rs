//! Primary/secondary combat target, Nano skill target and NPC talk target icons and status.

use super::super::GameplayHud;
use super::super::assets::GameplayUiAssets;
use super::super::model::GameplayUiModel;
use super::super::text::hud_font;
use super::hud_npc::GameplayHudNpcQuery;
use crate::{
    avatar_action::{
        LegacyAvatarActionContext, LegacyAvatarActionState, LegacyFocusedTarget,
        LegacyTargetSelection,
    },
    localization::LocalizedText,
    movement::LegacyOrbitCamera,
};
use bevy::{prelude::*, ui::widget::NodeImageMode};

pub(in super::super) const PRIMARY_COMBAT_TARGET_ICON_PATH: &str =
    "ui/en/gameplay/shared/target_icon1a.png";
pub(in super::super) const PRIMARY_COMBAT_TARGET_ICON_SIZE: Vec2 = Vec2::new(100.0, 86.0);
pub(in super::super) const SECONDARY_COMBAT_TARGET_ICON_PATH: &str =
    "ui/en/gameplay/shared/target_icon1b.png";
pub(in super::super) const SECONDARY_COMBAT_TARGET_ICON_SIZE: Vec2 = Vec2::new(80.0, 70.0);
pub(in super::super) const SECONDARY_COMBAT_TARGET_ICON_CAPACITY: usize = 2;
pub const NANO_SKILL_TARGET_ICON_PATH: &str = "ui/en/gameplay/shared/nano_skill_target.png";
pub const NANO_SKILL_TARGET_ICON_BYTES: u64 = 3_856;
pub const NANO_SKILL_TARGET_ICON_SHA256: &str =
    "f4e6e6c5550b51b41689aaa0e3a37f9e6df8ab34d736a2866fa30769b212da2e";
pub const NANO_SKILL_TARGET_ICON_SIZE: Vec2 = Vec2::new(74.0, 80.0);
pub(in super::super) const NANO_SKILL_TARGET_ICON_CAPACITY: usize = 4;
pub(in super::super) const NPC_TALK_TARGET_ICON_PATH: &str =
    "ui/en/gameplay/shared/target_icon2.png";
pub(in super::super) const NPC_TALK_TARGET_ICON_SIZE: Vec2 = Vec2::new(42.0, 41.0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in super::super) enum PrimaryTargetIconKind {
    Combat,
    Talk,
}

pub(in super::super) fn friendly_target_ui_visible(target: LegacyFocusedTarget) -> bool {
    !matches!(
        target.kind,
        crate::avatar_action::LegacyTargetKind::Npc { team: 1 }
    ) || target.talk_enabled
}

pub(in super::super) fn primary_target_icon(
    selection: &LegacyTargetSelection,
) -> Option<(Entity, PrimaryTargetIconKind)> {
    selection
        .attack_targets
        .first()
        .map(|target| (target.entity, PrimaryTargetIconKind::Combat))
        .or_else(|| {
            selection
                .focused_npc
                .filter(|target| {
                    !selection.check_attack_target
                        && target.talk_enabled
                        && matches!(
                            target.kind,
                            crate::avatar_action::LegacyTargetKind::Npc { team: 1 }
                        )
                })
                .map(|target| (target.entity, PrimaryTargetIconKind::Talk))
        })
        .or_else(|| {
            selection
                .focused_player
                .filter(|target| {
                    selection.focused_npc.is_none()
                        && !selection.check_attack_target
                        && target.talk_enabled
                })
                .map(|target| (target.entity, PrimaryTargetIconKind::Talk))
        })
}

/// Exact `NpcTableElement.m_iIcon1 -> m_pNpcIconData -> AvatarUtil.GetIconName`
/// routes for every actor type instantiated by tutorial choreography. Entries whose source
/// table resolves to the generic, unpublished `wpnicon_00` intentionally stay
/// empty, exactly like a null `NpcMoveController.pMobIcon`.
pub(in super::super) const fn combat_target_icon_path(npc_type: i32) -> Option<&'static str> {
    match npc_type {
        2663 => Some("icons/entities/npc/npcicon_35.png"),
        2664 => Some("icons/entities/npc/npcicon_48.png"),
        2665 | 2672 => Some("icons/entities/npc/npcicon_88.png"),
        2666 => Some("icons/entities/npc/npcicon_02.png"),
        2667 => Some("icons/entities/npc/npcicon_06.png"),
        2668 | 2673 | 2902 => Some("icons/entities/npc/npcicon_00.png"),
        2669 => Some("icons/entities/npc/npcicon_10.png"),
        2670 => Some("icons/entities/npc/npcicon_68.png"),
        2671 | 2903 => Some("icons/entities/npc/npcicon_87.png"),
        2674 | 2897 => Some("icons/entities/mobs/mobicon_00.png"),
        2675 => Some("icons/entities/mobs/mobicon_24.png"),
        2676 => Some("icons/entities/mobs/mobicon_11.png"),
        2677 => Some("icons/entities/mobs/mobicon_21.png"),
        2678 => Some("icons/entities/mobs/mobicon_235.png"),
        2694 | 2696 => Some("icons/entities/npc/npcicon_58.png"),
        _ => None,
    }
}

#[derive(Component)]
pub(in super::super) struct PrimaryCombatTargetIcon;
#[derive(Component)]
pub(in super::super) struct SecondaryCombatTargetIcon(pub(in super::super) usize);
#[derive(Component)]
pub(in super::super) struct NanoSkillTargetIcon(pub(in super::super) usize);
#[derive(Component)]
pub(in super::super) struct PrimaryCombatTargetStatus;
#[derive(Component)]
pub(in super::super) struct PrimaryCombatTargetName;
#[derive(Component)]
pub(in super::super) struct PrimaryPlayerHealth;
#[derive(Component)]
pub(in super::super) struct PrimaryPlayerHealthFill;
#[derive(Component)]
pub(in super::super) struct PrimaryPlayerLevel;

pub(in super::super) fn spawn_primary_combat_target_icon(
    parent: &mut ChildSpawnerCommands,
    assets: &GameplayUiAssets,
) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            display: Display::None,
            width: px(PRIMARY_COMBAT_TARGET_ICON_SIZE.x),
            height: px(PRIMARY_COMBAT_TARGET_ICON_SIZE.y),
            ..default()
        },
        ImageNode {
            image: assets.primary_combat_target_icon.clone(),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        ZIndex(900),
        PrimaryCombatTargetIcon,
    ));
}

pub(in super::super) fn spawn_secondary_combat_target_icons(
    parent: &mut ChildSpawnerCommands,
    assets: &GameplayUiAssets,
) {
    for index in 0..SECONDARY_COMBAT_TARGET_ICON_CAPACITY {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                width: px(SECONDARY_COMBAT_TARGET_ICON_SIZE.x),
                height: px(SECONDARY_COMBAT_TARGET_ICON_SIZE.y),
                ..default()
            },
            ImageNode {
                image: assets.secondary_combat_target_icon.clone(),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
            ZIndex(900),
            SecondaryCombatTargetIcon(index),
        ));
    }
}

pub(in super::super) fn spawn_nano_skill_target_icons(
    parent: &mut ChildSpawnerCommands,
    assets: &GameplayUiAssets,
) {
    for index in 0..NANO_SKILL_TARGET_ICON_CAPACITY {
        parent.spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                width: px(NANO_SKILL_TARGET_ICON_SIZE.x),
                height: px(NANO_SKILL_TARGET_ICON_SIZE.y),
                ..default()
            },
            ImageNode {
                image: assets.nano_skill_target_icon.clone(),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
            // `PrintName` leaves GUI.depth at the combat-icon depth before it
            // draws SkillIcon, so both target layers share the same depth.
            ZIndex(900),
            NanoSkillTargetIcon(index),
        ));
    }
}

/// Selected player name, authoritative level and health.
pub(in super::super) fn spawn_primary_combat_target_status(
    parent: &mut ChildSpawnerCommands,
    assets: &GameplayUiAssets,
) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                width: px(200),
                height: px(30),
                ..default()
            },
            ZIndex(899),
            PrimaryCombatTargetStatus,
        ))
        .with_children(|status| {
            status
                .spawn((
                    PrimaryPlayerHealth,
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(12),
                        left: px(50),
                        width: px(100),
                        height: px(8),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        PrimaryPlayerHealthFill,
                        Node {
                            width: percent(100),
                            height: percent(100),
                            ..default()
                        },
                        ImageNode::new(assets.combat_target_hp_fill.clone()),
                    ));
                    bar.spawn((
                        PrimaryPlayerLevel,
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(-26),
                            top: px(-4),
                            ..default()
                        },
                        Text::new(""),
                        hud_font(&assets.chalet_font, 12.0),
                        TextColor(Color::WHITE),
                    ));
                });
            status
                .spawn((Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(-15),
                    width: px(200),
                    height: px(30),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },))
                .with_child((
                    Text::new(""),
                    hud_font(&assets.chalet_font, 12.0),
                    TextColor(Color::srgb(0.89, 0.17, 0.17)),
                    TextLayout::default().with_justify(Justify::Center),
                    LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", ""),
                    PrimaryCombatTargetName,
                ));
        });
}

pub(in super::super) fn bind_primary_combat_target_status(
    model: Res<GameplayUiModel>,
    players: Query<(Entity, &LegacyAvatarActionState)>,
    cameras: Query<(&Camera, &GlobalTransform, &LegacyOrbitCamera), With<Camera3d>>,
    targets: GameplayHudNpcQuery,
    mut status: Single<
        &mut Node,
        (
            With<PrimaryCombatTargetStatus>,
            Without<PrimaryPlayerHealth>,
            Without<PrimaryPlayerHealthFill>,
        ),
    >,
    mut health: Single<
        &mut Node,
        (
            With<PrimaryPlayerHealth>,
            Without<PrimaryCombatTargetStatus>,
            Without<PrimaryPlayerHealthFill>,
        ),
    >,
    mut fill: Single<
        &mut Node,
        (
            With<PrimaryPlayerHealthFill>,
            Without<PrimaryCombatTargetStatus>,
            Without<PrimaryPlayerHealth>,
        ),
    >,
    mut level: Single<&mut Text, With<PrimaryPlayerLevel>>,
    mut name: Single<(&mut LocalizedText, &mut TextColor), With<PrimaryCombatTargetName>>,
) {
    status.display = Display::None;
    if !model.visible {
        return;
    }
    let Some((camera, camera_transform, state)) = players.iter().find_map(|(entity, state)| {
        cameras
            .iter()
            .find(|(_, _, orbit)| orbit.target == entity)
            .map(|(camera, transform, _)| (camera, transform, state))
    }) else {
        return;
    };
    let selected_player = state.target_selection.focused_player.filter(|target| {
        let combat_target = state
            .target_selection
            .attack_targets
            .iter()
            .any(|attack_target| attack_target.entity == target.entity);
        let talk_target = state.target_selection.focused_npc.is_none()
            && !state.target_selection.check_attack_target
            && target.talk_enabled;
        combat_target || talk_target
    });
    // A focused mob is already named by the combat target frame; repeating the
    // name over the pulsing target reticle only duplicated it.
    let Some(target) = selected_player else {
        return;
    };
    let Some(world_position) = targets.projected_world_position(target.entity, 0.8) else {
        return;
    };
    let Some(localized_name) = targets.localized_player_name(target.entity) else {
        return;
    };
    let combat_color = state
        .target_selection
        .attack_targets
        .iter()
        .any(|attack_target| attack_target.entity == target.entity);
    let Ok(viewport_position) = camera.world_to_viewport(camera_transform, world_position) else {
        return;
    };

    health.display = Display::None;
    if let Some((value, fraction)) = targets.player_health(target.entity) {
        health.display = Display::Flex;
        fill.width = percent(fraction * 100.0);
        let value = value.to_string();
        if level.0 != value {
            level.0 = value;
        }
    }
    status.left = px(viewport_position.x - 100.0);
    status.top = px(viewport_position.y);
    status.display = Display::Flex;
    let (name, color) = &mut *name;
    **name = localized_name;
    color.0 = if combat_color {
        Color::srgb(0.89, 0.17, 0.17)
    } else {
        Color::WHITE
    };
}

pub(in super::super) fn bind_primary_combat_target_icon(
    time: Res<Time>,
    model: Res<GameplayUiModel>,
    assets: Res<GameplayUiAssets>,
    players: Query<(Entity, &LegacyAvatarActionState, &LegacyAvatarActionContext)>,
    cameras: Query<(&Camera, &GlobalTransform, &LegacyOrbitCamera), With<Camera3d>>,
    targets: GameplayHudNpcQuery,
    mut icon: Single<
        (&mut Node, &mut ImageNode),
        (With<PrimaryCombatTargetIcon>, Without<GameplayHud>),
    >,
) {
    icon.0.display = Display::None;
    if !model.visible {
        return;
    }
    let Some((camera, camera_transform, state, context)) =
        players.iter().find_map(|(entity, state, context)| {
            cameras
                .iter()
                .find(|(_, _, orbit)| orbit.target == entity)
                .map(|(camera, transform, _)| (camera, transform, state, context))
        })
    else {
        return;
    };
    if context.weapon_target_mode != crate::avatar_action::LegacyWeaponTargetMode::Normal {
        return;
    }
    let Some((target_entity, icon_kind)) = primary_target_icon(&state.target_selection) else {
        return;
    };
    let (image, size, height_factor) = match icon_kind {
        PrimaryTargetIconKind::Combat => {
            // PrintName uses `move.fHeight * 0.5` for target icons 0/1.
            (
                assets.primary_combat_target_icon.clone(),
                PRIMARY_COMBAT_TARGET_ICON_SIZE,
                0.5,
            )
        }
        PrimaryTargetIconKind::Talk => {
            // `SetCombatIcon(3)` resolves to the shared target_icon2 Texture2D
            // (pathId 342) and PrintName projects icons >1 at fHeight * 0.8.
            (
                assets.npc_talk_target_icon.clone(),
                NPC_TALK_TARGET_ICON_SIZE,
                0.8,
            )
        }
    };
    let Some(world_position) = targets.projected_world_position(target_entity, height_factor)
    else {
        return;
    };
    let Ok(viewport_position) = camera.world_to_viewport(camera_transform, world_position) else {
        return;
    };
    icon.0.left = px(viewport_position.x - size.x * 0.5);
    icon.0.top = px(viewport_position.y - size.y * 0.5);
    icon.0.width = px(size.x);
    icon.0.height = px(size.y);
    icon.0.display = Display::Flex;
    icon.1.image = image;
    // Exact PrintName pulse: white * (0.5 + (sin(Time.time*5)+1)*0.25).
    let pulse = 0.5 + ((time.elapsed_secs() * 5.0).sin() + 1.0) * 0.25;
    icon.1.color = Color::srgba(pulse, pulse, pulse, pulse);
}

pub(in super::super) fn bind_secondary_combat_target_icons(
    time: Res<Time>,
    model: Res<GameplayUiModel>,
    players: Query<(Entity, &LegacyAvatarActionState, &LegacyAvatarActionContext)>,
    cameras: Query<(&Camera, &GlobalTransform, &LegacyOrbitCamera), With<Camera3d>>,
    targets: GameplayHudNpcQuery,
    mut icons: Query<(&SecondaryCombatTargetIcon, &mut Node, &mut ImageNode)>,
) {
    for (_, mut node, _) in &mut icons {
        node.display = Display::None;
    }
    if !model.visible {
        return;
    }
    let Some((camera, camera_transform, state, context)) =
        players.iter().find_map(|(entity, state, context)| {
            cameras
                .iter()
                .find(|(_, _, orbit)| orbit.target == entity)
                .map(|(camera, transform, _)| (camera, transform, state, context))
        })
    else {
        return;
    };
    if context.weapon_target_mode != crate::avatar_action::LegacyWeaponTargetMode::Normal {
        return;
    }
    let pulse = 0.5 + ((time.elapsed_secs() * 5.0).sin() + 1.0) * 0.25;
    for (marker, mut node, mut image) in &mut icons {
        let Some(target) = state.target_selection.attack_targets.get(marker.0 + 1) else {
            continue;
        };
        let Some(world_position) = targets.projected_world_position(target.entity, 0.5) else {
            continue;
        };
        let Ok(viewport_position) = camera.world_to_viewport(camera_transform, world_position)
        else {
            continue;
        };
        node.left = px(viewport_position.x - SECONDARY_COMBAT_TARGET_ICON_SIZE.x * 0.5);
        node.top = px(viewport_position.y - SECONDARY_COMBAT_TARGET_ICON_SIZE.y * 0.5);
        node.display = Display::Flex;
        image.color = Color::srgba(pulse, pulse, pulse, pulse);
    }
}

pub(in super::super) fn bind_nano_skill_target_icons(
    time: Res<Time>,
    model: Res<GameplayUiModel>,
    players: Query<(Entity, &LegacyAvatarActionState)>,
    cameras: Query<(&Camera, &GlobalTransform, &LegacyOrbitCamera), With<Camera3d>>,
    targets: GameplayHudNpcQuery,
    mut icons: Query<(&NanoSkillTargetIcon, &mut Node, &mut ImageNode)>,
) {
    for (_, mut node, _) in &mut icons {
        node.display = Display::None;
    }
    if !model.visible {
        return;
    }
    let Some((camera, camera_transform, state)) = players.iter().find_map(|(entity, state)| {
        cameras
            .iter()
            .find(|(_, _, orbit)| orbit.target == entity)
            .map(|(camera, transform, _)| (camera, transform, state))
    }) else {
        return;
    };
    let pulse = 0.5 + ((time.elapsed_secs() * 5.0).sin() + 1.0) * 0.25;
    for (marker, mut node, mut image) in &mut icons {
        let Some(target) = state.target_selection.nano_targets.get(marker.0) else {
            continue;
        };
        // Exact `PrintName.SkillIcon`: project mob origin + `fHeight * 0.5`, then center
        // NPCPrefab PrintName path ID 1459's serialized 74x80 `target_n` Texture2D
        // (path ID 513) on that point.
        let Some(world_position) = targets.projected_world_position(target.entity, 0.5) else {
            continue;
        };
        let Ok(viewport_position) = camera.world_to_viewport(camera_transform, world_position)
        else {
            continue;
        };
        node.left = px(viewport_position.x - NANO_SKILL_TARGET_ICON_SIZE.x * 0.5);
        node.top = px(viewport_position.y - NANO_SKILL_TARGET_ICON_SIZE.y * 0.5);
        node.display = Display::Flex;
        image.color = Color::srgba(pulse, pulse, pulse, pulse);
    }
}
