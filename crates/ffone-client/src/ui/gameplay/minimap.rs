//! Minimap spawning, zoom, marker/waypoint binding and the player/minimap HUD binder.

use super::actions::{GameplayUiAudioCue, GameplayUiAudioOutbox};
use super::assets::GameplayUiAssets;
use super::current_objective::spawn_current_objective;
use super::hud::GameplayUiRect;
use super::minimap_model::{
    MINIMAP_FRAME_RECT, MINIMAP_FUSION_METER_RECT, MINIMAP_GROUP_RECT, MINIMAP_MAP_RECT,
    MINIMAP_MARKER_CAPACITY, MINIMAP_NAME_RECT, MinimapMarkerIcon,
};
use super::model::{GameplayMenuTransition, GameplayUiModel};
use super::player_status::{
    PLAYER_HEALTH_RECT, PlayerCombatToggleIcon, PlayerFreeChatIcon, PlayerHealthFill,
    PlayerLevelText, PlayerNameText,
};
use super::text::spawn_shadowed_center_text;
use super::{CircularMinimapTileMaterial, FusionMatterMeterMaterial};
use super::{GameplayHud, new_mail_icon};
use crate::localization::LocalizedText;
use bevy::{prelude::*, ui::widget::NodeImageMode};

#[derive(Component)]
pub(super) struct MinimapRoot;

#[derive(Component)]
pub(super) struct MinimapTileLayer;
#[derive(Component)]
pub(super) struct MinimapTileImage(pub(super) usize);
#[derive(Component)]
pub(super) struct MinimapCameraOverlay;
#[derive(Component)]
pub(super) struct MinimapPlayerIcon;
#[derive(Component)]
pub(super) struct MinimapWaypointMarker;
#[derive(Component)]
pub(super) struct MinimapMarker(pub(super) usize);
#[derive(Component)]
pub(super) struct MinimapFusionMatterMeter;
#[derive(Component)]
pub(super) struct MinimapNameText;
#[derive(Component)]
pub(super) struct MinimapNameShadow;

pub(super) fn spawn_minimap(
    parent: &mut ChildSpawnerCommands,
    assets: &GameplayUiAssets,
    minimap_materials: &mut Assets<CircularMinimapTileMaterial>,
    fusion_meter_materials: &mut Assets<FusionMatterMeterMaterial>,
) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                right: px(0),
                top: px(MINIMAP_GROUP_RECT.y),
                width: px(MINIMAP_GROUP_RECT.width),
                height: px(MINIMAP_GROUP_RECT.height),
                ..default()
            },
            UiTransform::default(),
            MinimapRoot,
            ZIndex(8),
        ))
        .with_children(|minimap| {
            // Exact Retrobution draw order:
            // RenderMenu(FM, frame), RenderMinimap(map, markers, camera,
            // player, LineEffect), then the `centerbox2` location label.
            minimap.spawn((
                MINIMAP_FUSION_METER_RECT.node(),
                MaterialNode(fusion_meter_materials.add(FusionMatterMeterMaterial {
                    // `InitCycleFMBar` uses pivot (99,91); the 176x176 draw
                    // rectangle starts at (12,3), so its local pivot is
                    // exactly (87,88).
                    parameters: Vec4::new(0.0, 87.0 / 176.0, 88.0 / 176.0, 0.0),
                    color_texture: assets.minimap_fusion_meter.clone(),
                    right_color_texture: assets.minimap_fusion_meter_right.clone(),
                    left_mask_texture: assets.minimap_fusion_meter_left_mask.clone(),
                    rotating_mask_texture: assets.minimap_fusion_meter_rotating_mask.clone(),
                })),
                MinimapFusionMatterMeter,
                ZIndex(0),
            ));
            minimap.spawn((
                MINIMAP_FRAME_RECT.node(),
                ImageNode {
                    image: assets.minimap_frame.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                ZIndex(1),
            ));
            new_mail_icon::spawn(minimap, assets.minimap_new_mail.clone());
            minimap
                .spawn((MINIMAP_MAP_RECT.node(), ZIndex(2)))
                .with_children(|map| {
                    map.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(0),
                            top: px(0),
                            width: px(148),
                            height: px(148),
                            ..default()
                        },
                        UiTransform::IDENTITY,
                        MinimapTileLayer,
                        ZIndex(0),
                    ))
                    .with_children(|layer| {
                        for index in 0..4 {
                            layer.spawn((
                                Node {
                                    position_type: PositionType::Absolute,
                                    display: Display::None,
                                    ..default()
                                },
                                MaterialNode(minimap_materials.add(CircularMinimapTileMaterial {
                                    source_uv: Vec4::new(0.0, 0.0, 1.0, 1.0),
                                    destination_uv: Vec4::new(0.0, 0.0, 1.0, 1.0),
                                    color_texture: assets.minimap_tiles[0].clone(),
                                    alpha_texture: assets.minimap_alpha.clone(),
                                })),
                                MinimapTileImage(index),
                                ZIndex(0),
                            ));
                        }
                    });
                });
            // These are siblings of the masked map, exactly like the
            // post-RenderMaps Graphics.DrawTexture calls. No map alpha or
            // clipping state may leak into any marker.
            minimap.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    display: Display::None,
                    left: px(MINIMAP_MAP_RECT.x + 65.5),
                    top: px(MINIMAP_MAP_RECT.y + 65.5),
                    width: px(17),
                    height: px(17),
                    ..default()
                },
                ImageNode::new(assets.minimap_waypoint_icons[0].clone()),
                UiTransform::IDENTITY,
                MinimapWaypointMarker,
                ZIndex(4),
            ));
            for index in 0..MINIMAP_MARKER_CAPACITY {
                minimap.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        display: Display::None,
                        width: px(16),
                        height: px(16),
                        ..default()
                    },
                    ImageNode::new(assets.minimap_marker_icons[0].clone()),
                    MinimapMarker(index),
                    ZIndex(3),
                ));
            }
            minimap.spawn((
                MINIMAP_MAP_RECT.node(),
                ImageNode::new(assets.minimap_camera.clone()),
                UiTransform::IDENTITY,
                MinimapCameraOverlay,
                ZIndex(5),
            ));
            minimap.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(MINIMAP_MAP_RECT.x + 65.0),
                    top: px(MINIMAP_MAP_RECT.y + 65.0),
                    width: px(18),
                    height: px(18),
                    ..default()
                },
                ImageNode::new(assets.minimap_player.clone()),
                UiTransform::IDENTITY,
                MinimapPlayerIcon,
                ZIndex(6),
            ));
            minimap.spawn((
                MINIMAP_MAP_RECT.node(),
                ImageNode::new(assets.minimap_line.clone()),
                ZIndex(7),
            ));
            spawn_shadowed_center_text(
                minimap,
                MINIMAP_NAME_RECT,
                "",
                &assets.chalet_font,
                12.0,
                // Exact `UnityEngine.Color.yellow`.
                Color::srgb(1.0, 0.921_568_6, 0.015_686_28),
                MinimapNameText,
            );
            spawn_current_objective(minimap, assets);
            for index in 0..6 {
                minimap.spawn((
                    Node {
                        display: Display::None,
                        ..GameplayUiRect::new(0.0, 0.0, 17.0, 17.0).node()
                    },
                    ImageNode::default(),
                    UiTransform::default(),
                    CustomMinimapWaypoint(index),
                    Pickable::IGNORE,
                    ZIndex(5),
                ));
            }

            for (zoom_in, y) in [(true, 68.0), (false, 92.0)] {
                let image = if zoom_in {
                    assets.minimap_zoom_in.clone()
                } else {
                    assets.minimap_zoom_out.clone()
                };
                minimap
                    .spawn((
                        Button,
                        Node {
                            display: Display::None,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..GameplayUiRect::new(130.0, y, 18.0, 18.0).node()
                        },
                        MinimapZoomButton(zoom_in),
                        ZIndex(9),
                    ))
                    .with_children(|button| {
                        button.spawn((
                            Node {
                                width: px(17.0),
                                height: px(16.0),
                                ..default()
                            },
                            ImageNode::new(image),
                            Pickable::IGNORE,
                        ));
                    });
            }
        });
}

#[derive(Component)]
pub(super) struct MinimapZoomButton(pub(super) bool);

pub(super) fn update_minimap_zoom(
    menu: Res<GameplayMenuTransition>,
    mut model: ResMut<GameplayUiModel>,
    mut audio: ResMut<GameplayUiAudioOutbox>,
    mut buttons: Query<(&MinimapZoomButton, Ref<Interaction>, &mut Node)>,
) {
    let visible = model.visible && menu.open;
    for (button, interaction, mut node) in &mut buttons {
        let display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        if visible
            && model.chat.input_enabled
            && interaction.is_changed()
            && *interaction == Interaction::Pressed
        {
            model.minimap.ratio =
                (model.minimap.ratio + if button.0 { -1.0 } else { 1.0 }).clamp(4.0, 16.0);
            audio.push(GameplayUiAudioCue::ButtonSound);
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn bind_hud_player_minimap(
    model: Res<GameplayUiModel>,
    assets: Res<GameplayUiAssets>,
    mut minimap_materials: ResMut<Assets<CircularMinimapTileMaterial>>,
    mut fusion_meter_materials: ResMut<Assets<FusionMatterMeterMaterial>>,
    fusion_meter: Single<&MaterialNode<FusionMatterMeterMaterial>, With<MinimapFusionMatterMeter>>,
    mut hud: Single<
        &mut Node,
        (
            With<GameplayHud>,
            Without<PlayerHealthFill>,
            Without<PlayerFreeChatIcon>,
            Without<MinimapTileImage>,
        ),
    >,
    mut player_name: Single<
        &mut LocalizedText,
        (
            With<PlayerNameText>,
            Without<MinimapNameShadow>,
            Without<PlayerLevelText>,
            Without<MinimapNameText>,
        ),
    >,
    mut player_level: Single<
        &mut LocalizedText,
        (
            With<PlayerLevelText>,
            Without<MinimapNameShadow>,
            Without<PlayerNameText>,
            Without<MinimapNameText>,
        ),
    >,
    mut health: Single<
        (&mut Node, &mut crate::damage_bar::DamageBarOwner),
        (
            With<PlayerHealthFill>,
            Without<GameplayHud>,
            Without<PlayerFreeChatIcon>,
            Without<PlayerCombatToggleIcon>,
            Without<MinimapTileImage>,
            Without<MinimapMarker>,
        ),
    >,
    mut free_chat: Single<
        &mut Node,
        (
            With<PlayerFreeChatIcon>,
            Without<GameplayHud>,
            Without<PlayerHealthFill>,
            Without<PlayerCombatToggleIcon>,
            Without<MinimapTileImage>,
            Without<MinimapMarker>,
        ),
    >,
    mut minimap_name: Single<
        &mut LocalizedText,
        (
            With<MinimapNameText>,
            Without<PlayerNameText>,
            Without<MinimapNameShadow>,
            Without<PlayerLevelText>,
        ),
    >,
    mut minimap_layer: Single<
        &mut UiTransform,
        (
            With<MinimapTileLayer>,
            Without<MinimapCameraOverlay>,
            Without<MinimapPlayerIcon>,
        ),
    >,
    mut minimap_camera: Single<
        &mut UiTransform,
        (
            With<MinimapCameraOverlay>,
            Without<MinimapTileLayer>,
            Without<MinimapPlayerIcon>,
        ),
    >,
    mut minimap_player: Single<
        (&mut UiTransform, &mut ImageNode),
        (
            With<MinimapPlayerIcon>,
            Without<MinimapTileLayer>,
            Without<MinimapCameraOverlay>,
            Without<MinimapWaypointMarker>,
            Without<MinimapMarker>,
        ),
    >,
    mut minimap_waypoint: Single<
        (&mut Node, &mut UiTransform, &mut ImageNode),
        (
            With<MinimapWaypointMarker>,
            Without<GameplayHud>,
            Without<PlayerHealthFill>,
            Without<PlayerFreeChatIcon>,
            Without<MinimapTileImage>,
            Without<MinimapTileLayer>,
            Without<MinimapCameraOverlay>,
            Without<MinimapPlayerIcon>,
            Without<MinimapMarker>,
        ),
    >,
    mut minimap_tiles: Query<
        (
            &MinimapTileImage,
            &mut Node,
            &MaterialNode<CircularMinimapTileMaterial>,
        ),
        (
            Without<GameplayHud>,
            Without<PlayerHealthFill>,
            Without<PlayerFreeChatIcon>,
            Without<PlayerCombatToggleIcon>,
            Without<MinimapPlayerIcon>,
            Without<MinimapWaypointMarker>,
            Without<MinimapMarker>,
        ),
    >,
) {
    if !model.is_changed() {
        return;
    }
    hud.reborrow()
        .map_unchanged(|value| &mut value.display)
        .set_if_neq(if model.visible {
            Display::Flex
        } else {
            Display::None
        });
    player_name.set_if_neq(
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", &model.player.name),
    );
    let player_level_value = format!("{:02}", model.player.level);
    player_level.set_if_neq(
        LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", player_level_value),
    );
    health
        .0
        .reborrow()
        .map_unchanged(|value| &mut value.width)
        .set_if_neq(px(PLAYER_HEALTH_RECT.width * model.player.health_fraction()));
    if health.1.0 != model.player.owner {
        health.1.0 = model.player.owner;
    }
    free_chat
        .reborrow()
        .map_unchanged(|value| &mut value.display)
        .set_if_neq(if model.player.free_chat {
            Display::Flex
        } else {
            Display::None
        });
    let fraction = model.minimap.fusion_matter_fraction();
    if fusion_meter_materials
        .get(&fusion_meter.0)
        .is_some_and(|material| material.parameters.x != fraction)
    {
        if let Some(mut material) = fusion_meter_materials.get_mut(&fusion_meter.0) {
            material.parameters.x = fraction;
        }
    }
    minimap_name.set_if_neq(crate::localization::localized_world_location_text(
        &model.minimap.map_name,
    ));
    // The map remains north-up. The full-size camera cone and the 18x18 avatar
    // arrow are independent legacy layers with positive Unity GUI rotation.
    minimap_layer
        .reborrow()
        .map_unchanged(|value| &mut value.rotation)
        .set_if_neq(Rot2::IDENTITY);
    minimap_camera
        .reborrow()
        .map_unchanged(|value| &mut value.rotation)
        .set_if_neq(Rot2::radians(
            model.minimap.camera_heading_degrees.to_radians(),
        ));
    minimap_player
        .0
        .reborrow()
        .map_unchanged(|value| &mut value.rotation)
        .set_if_neq(Rot2::radians(
            model.minimap.avatar_heading_degrees.to_radians(),
        ));
    minimap_player
        .1
        .reborrow()
        .map_unchanged(|value| &mut value.color)
        .set_if_neq(Color::srgba(
            1.0,
            1.0,
            1.0,
            model.minimap.player_marker_alpha.clamp(0.0, 1.0),
        ));
    if let Some(sample) = model.minimap.waypoint {
        minimap_waypoint
            .0
            .reborrow()
            .map_unchanged(|value| &mut value.display)
            .set_if_neq(Display::Flex);
        minimap_waypoint
            .0
            .reborrow()
            .map_unchanged(|value| &mut value.left)
            .set_if_neq(px(MINIMAP_MAP_RECT.x + sample.left));
        minimap_waypoint
            .0
            .reborrow()
            .map_unchanged(|value| &mut value.top)
            .set_if_neq(px(MINIMAP_MAP_RECT.y + sample.top));
        minimap_waypoint
            .1
            .reborrow()
            .map_unchanged(|value| &mut value.rotation)
            .set_if_neq(Rot2::radians(sample.rotation_degrees.to_radians()));
        minimap_waypoint
            .2
            .reborrow()
            .map_unchanged(|value| &mut value.image)
            .set_if_neq(assets.minimap_waypoint_icons[sample.icon.index()].clone());
        minimap_waypoint
            .2
            .reborrow()
            .map_unchanged(|value| &mut value.color)
            .set_if_neq(Color::srgba(1.0, 1.0, 1.0, sample.alpha.clamp(0.0, 1.0)));
    } else {
        minimap_waypoint
            .0
            .reborrow()
            .map_unchanged(|value| &mut value.display)
            .set_if_neq(Display::None);
    }
    for (marker, mut node, material_handle) in &mut minimap_tiles {
        let Some(sample) = model.minimap.tiles.get(marker.0) else {
            node.reborrow()
                .map_unchanged(|value| &mut value.display)
                .set_if_neq(Display::None);
            continue;
        };
        node.reborrow()
            .map_unchanged(|value| &mut value.display)
            .set_if_neq(Display::Flex);
        node.reborrow()
            .map_unchanged(|value| &mut value.left)
            .set_if_neq(px(sample.destination.x));
        node.reborrow()
            .map_unchanged(|value| &mut value.top)
            .set_if_neq(px(sample.destination.y));
        node.reborrow()
            .map_unchanged(|value| &mut value.width)
            .set_if_neq(px(sample.destination.width));
        node.reborrow()
            .map_unchanged(|value| &mut value.height)
            .set_if_neq(px(sample.destination.height));
        let Some(tile) = sample
            .tile_number
            .checked_sub(1)
            .map(usize::from)
            .filter(|index| *index < assets.minimap_tiles.len())
        else {
            node.reborrow()
                .map_unchanged(|value| &mut value.display)
                .set_if_neq(Display::None);
            continue;
        };
        let source_uv = Vec4::new(
            sample.source.x / 512.0,
            sample.source.y / 512.0,
            sample.source.width / 512.0,
            sample.source.height / 512.0,
        );
        let destination_uv = Vec4::new(
            sample.destination.x / MINIMAP_MAP_RECT.width,
            sample.destination.y / MINIMAP_MAP_RECT.height,
            sample.destination.width / MINIMAP_MAP_RECT.width,
            sample.destination.height / MINIMAP_MAP_RECT.height,
        );
        let Some(material) = minimap_materials.get(&material_handle.0) else {
            node.reborrow()
                .map_unchanged(|value| &mut value.display)
                .set_if_neq(Display::None);
            continue;
        };
        // A camera heading change does not change the north-up map texture.
        // Read first: Assets::get_mut emits Modified even for identical values.
        if material.source_uv != source_uv
            || material.destination_uv != destination_uv
            || material.color_texture != assets.minimap_tiles[tile]
        {
            let mut material = minimap_materials
                .get_mut(&material_handle.0)
                .expect("checked above");
            material.source_uv = source_uv;
            material.destination_uv = destination_uv;
            material.color_texture = assets.minimap_tiles[tile].clone();
        }
    }
}

pub(super) fn bind_minimap_name_shadow(
    model: Res<GameplayUiModel>,
    mut shadow: Single<&mut LocalizedText, With<MinimapNameShadow>>,
) {
    if !model.is_changed() {
        return;
    }
    **shadow = crate::localization::localized_world_location_text(&model.minimap.map_name);
}

pub(super) fn bind_minimap_markers(
    model: Res<GameplayUiModel>,
    assets: Res<GameplayUiAssets>,
    mut markers: Query<(&MinimapMarker, &mut Node, &mut ImageNode)>,
) {
    if !model.is_changed() {
        return;
    }
    for (marker, mut node, mut image) in &mut markers {
        let Some(sample) = model.minimap.markers.get(marker.0) else {
            node.reborrow()
                .map_unchanged(|value| &mut value.display)
                .set_if_neq(Display::None);
            continue;
        };
        node.reborrow()
            .map_unchanged(|value| &mut value.display)
            .set_if_neq(Display::Flex);
        node.reborrow()
            .map_unchanged(|value| &mut value.left)
            .set_if_neq(px(MINIMAP_MAP_RECT.x + sample.left));
        node.reborrow()
            .map_unchanged(|value| &mut value.top)
            .set_if_neq(px(MINIMAP_MAP_RECT.y + sample.top));
        node.reborrow()
            .map_unchanged(|value| &mut value.width)
            .set_if_neq(px(sample.width));
        node.reborrow()
            .map_unchanged(|value| &mut value.height)
            .set_if_neq(px(sample.height));
        image
            .reborrow()
            .map_unchanged(|value| &mut value.image)
            .set_if_neq(match sample.icon {
                MinimapMarkerIcon::TableData(icon) => {
                    assets.minimap_table_marker_icons[usize::from(icon.index())].clone()
                }
                fixed => assets.minimap_marker_icons[fixed
                    .fixed_index()
                    .expect("fixed minimap marker must own a fixed asset index")]
                .clone(),
            });
    }
}

#[derive(Component)]
pub(super) struct CustomMinimapWaypoint(pub(super) usize);

pub(super) fn bind_custom_minimap_waypoints(
    model: Res<GameplayUiModel>,
    assets: Option<Res<GameplayUiAssets>>,
    mut images: ResMut<Assets<Image>>,
    mut variants: ResMut<crate::ui_icon_variants::UiIconVariants>,
    mut markers: Query<(
        &CustomMinimapWaypoint,
        &mut Node,
        &mut ImageNode,
        &mut UiTransform,
    )>,
) {
    let Some(assets) = assets else { return };
    for (marker, mut node, mut image, mut transform) in &mut markers {
        let Some((color, sample)) = model.minimap.custom_waypoints.get(marker.0) else {
            if node.display != Display::None {
                node.display = Display::None;
            }
            continue;
        };
        let source = &assets.minimap_waypoint_icons[sample.icon.index()];
        if let Some(handle) = variants.image(
            source,
            crate::ui_icon_variants::IconVariant::Hue(
                crate::map_preferences::WAYPOINT_HUES[usize::from(*color)],
            ),
            &mut images,
        ) {
            if image.image != handle {
                image.image = handle;
            }
        }
        let alpha = sample.alpha.clamp(0.0, 1.0);
        if image.color.alpha() != alpha {
            image.color.set_alpha(alpha);
        }
        if node.display != Display::Flex {
            node.display = Display::Flex;
        }
        let left = px(MINIMAP_MAP_RECT.x + sample.left);
        if node.left != left {
            node.left = left;
        }
        let top = px(MINIMAP_MAP_RECT.y + sample.top);
        if node.top != top {
            node.top = top;
        }
        let rotation = Rot2::degrees(sample.rotation_degrees);
        if transform.rotation != rotation {
            transform.rotation = rotation;
        }
    }
}
