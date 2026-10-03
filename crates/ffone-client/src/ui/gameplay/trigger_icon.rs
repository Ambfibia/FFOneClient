//! Local-player interaction cue, shared by usable world triggers.

use super::*;
use crate::{
    avatar_action::LegacyMoveMode,
    launcher_ui::LauncherUiModel,
    world_behaviour::{WorldRopeTraversal, WorldSlopeTraversal, WorldZiplineTraversal},
};

const TEXTURE_PATH: &str = "ui/en/gameplay/interaction/icons/use-trigger.png";
const SIZE: Vec2 = Vec2::new(37.0, 52.0);

#[derive(Component)]
pub(super) struct TriggerUseIcon;

pub(super) fn spawn(parent: &mut ChildSpawnerCommands, asset_server: &AssetServer) {
    parent.spawn((
        Node {
            position_type: PositionType::Absolute,
            display: Display::None,
            width: px(SIZE.x),
            height: px(SIZE.y),
            ..default()
        },
        ImageNode {
            image: asset_server
                .load_builder()
                .with_settings::<ImageLoaderSettings>(|settings| {
                    settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        mag_filter: ImageFilterMode::Linear,
                        min_filter: ImageFilterMode::Linear,
                        mipmap_filter: ImageFilterMode::Nearest,
                        anisotropy_clamp: 1,
                        ..default()
                    });
                })
                .load::<Image>(TEXTURE_PATH),
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        UiTransform::default(),
        ZIndex(900),
        TriggerUseIcon,
    ));
}

pub(super) fn bind(
    time: Res<Time>,
    model: Res<GameplayUiModel>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    cameras: Query<&LegacyOrbitCamera>,
    players: Query<(
        &LegacyAvatarActionState,
        &LegacyAvatarActionContext,
        Has<WorldZiplineTraversal>,
        Has<WorldRopeTraversal>,
        Has<WorldSlopeTraversal>,
    )>,
    entities: Query<Entity>,
    launcher: Option<Res<LauncherUiModel>>,
    mut icon: Single<(&mut Node, &mut ImageNode, &mut UiTransform), With<TriggerUseIcon>>,
) {
    let selected = cameras
        .iter()
        .find_map(|camera| players.get(camera.target).ok());
    let visible = model.visible
        && !launcher.is_some_and(|launcher| launcher.visible())
        && selected.is_some_and(|(state, context, zipline, rope, slope)| {
            context.ready_for_play
                && !context.dead
                // Skill preserves the previous target selection and icon;
                // other scripted movement explicitly clears the local cue.
                && context.move_mode != LegacyMoveMode::Other
                && !zipline
                && !rope
                && !slope
                && state
                    .target_selection
                    .trigger
                    .is_some_and(|entity| entities.contains(entity))
        });
    let window = windows.single().ok();
    let (node, image, transform) = &mut *icon;
    node.reborrow()
        .map_unchanged(|node| &mut node.display)
        .set_if_neq(if visible && window.is_some() {
            Display::Flex
        } else {
            Display::None
        });
    if !visible {
        return;
    }
    let Some(window) = window else {
        return;
    };
    // Preserve integer division in the source Rect, including the half-pixel
    // asymmetry of the odd-width bitmap. Scale about the image's center once.
    node.reborrow()
        .map_unchanged(|node| &mut node.left)
        .set_if_neq(px((window.width() * 0.5).floor() - (SIZE.x * 0.5).floor()));
    node.reborrow()
        .map_unchanged(|node| &mut node.top)
        .set_if_neq(px((window.height() * 0.5).floor() - (SIZE.y * 0.5).floor()));
    let scale = if model.ui_scale.is_finite() && model.ui_scale > 0.0 {
        model.ui_scale
    } else {
        1.0
    };
    transform
        .reborrow()
        .map_unchanged(|transform| &mut transform.scale)
        .set_if_neq(Vec2::splat(scale));
    let pulse = 0.5 + ((time.elapsed_secs() * 5.0).sin() + 1.0) * 0.25;
    image
        .reborrow()
        .map_unchanged(|image| &mut image.color)
        .set_if_neq(Color::srgba(pulse, pulse, pulse, pulse));
}

#[cfg(test)]
mod tests;
