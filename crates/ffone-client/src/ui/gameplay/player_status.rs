//! Player frame, name/level/health, portrait and auxiliary status.

use super::assets::GameplayUiAssets;
use super::hud::GameplayUiRect;
use super::model::{GameplayPortraitImage, GameplayUiModel};
use super::text::spawn_middle_left_text;
use bevy::{prelude::*, ui::widget::NodeImageMode};

pub const PLAYER_FRAME_RECT: GameplayUiRect = GameplayUiRect::new(0.0, 6.0, 255.0, 66.0);
pub const PLAYER_NAME_RECT: GameplayUiRect = GameplayUiRect::new(60.0, 1.0, 190.0, 20.0);
pub const PLAYER_LEVEL_RECT: GameplayUiRect = GameplayUiRect::new(90.0, 24.0, 60.0, 20.0);
pub const PLAYER_HEALTH_RECT: GameplayUiRect = GameplayUiRect::new(58.0, 23.0, 186.0, 4.0);
pub const PLAYER_PORTRAIT_RECT: GameplayUiRect = GameplayUiRect::new(7.0, 0.0, 50.0, 60.0);
pub const PLAYER_DISK_FRONT_RECT: GameplayUiRect = GameplayUiRect::new(1.0, 49.0, 69.0, 23.0);
pub const PLAYER_FREE_CHAT_RECT: GameplayUiRect = GameplayUiRect::new(3.0, 10.0, 19.0, 14.0);
pub const PLAYER_COMBAT_TOGGLE_RECT: GameplayUiRect = GameplayUiRect::new(260.0, 6.0, 29.0, 29.0);

#[derive(Clone, Debug, PartialEq)]
pub struct PlayerStatusUi {
    pub owner: u64,
    pub name: String,
    pub level: u16,
    pub hp: i32,
    pub max_hp: i32,
    pub free_chat: bool,
    /// Retrobution F1 player-targeting toggle. The crossed-swords icon is
    /// drawn only while this is false.
    pub allow_player_interaction: bool,
}

impl Default for PlayerStatusUi {
    fn default() -> Self {
        Self {
            owner: 0,
            name: String::new(),
            level: 1,
            hp: 1,
            max_hp: 1,
            free_chat: false,
            allow_player_interaction: true,
        }
    }
}

impl PlayerStatusUi {
    pub fn health_fraction(&self) -> f32 {
        if self.max_hp <= 0 {
            0.0
        } else {
            (self.hp.max(0) as f32 / self.max_hp as f32).clamp(0.0, 1.0)
        }
    }
}

#[derive(Component)]
pub(super) struct PlayerStatusRoot;

#[derive(Component)]
pub(super) struct PlayerNameText;
#[derive(Component)]
pub(super) struct PlayerLevelText;
#[derive(Component)]
pub(super) struct PlayerHealthFill;
#[derive(Component)]
pub(super) struct PlayerFreeChatIcon;
#[derive(Component)]
pub(super) struct PlayerCombatToggleIcon;
#[derive(Component)]
pub(super) struct PlayerPortrait;

pub(super) fn spawn_player_status(parent: &mut ChildSpawnerCommands, assets: &GameplayUiAssets) {
    parent
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: px(289),
                height: px(72),
                ..default()
            },
            UiTransform::default(),
            PlayerStatusRoot,
            ZIndex(10),
        ))
        .with_children(|panel| {
            panel.spawn((
                PLAYER_FRAME_RECT.node(),
                ImageNode {
                    image: assets.player_frame.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
            ));
            panel.spawn((
                Node {
                    display: Display::None,
                    ..PLAYER_PORTRAIT_RECT.node()
                },
                ImageNode::default(),
                PlayerPortrait,
            ));
            crate::damage_bar::spawn_damage_bar(
                panel,
                PLAYER_HEALTH_RECT.node(),
                ImageNode {
                    image: assets.health_bar.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
                PlayerHealthFill,
            );
            spawn_middle_left_text(
                panel,
                PLAYER_NAME_RECT,
                "",
                &assets.chalet_font,
                PlayerNameText,
            );
            spawn_middle_left_text(
                panel,
                PLAYER_LEVEL_RECT,
                "01",
                &assets.chalet_font,
                PlayerLevelText,
            );
            panel.spawn((
                PLAYER_DISK_FRONT_RECT.node(),
                ImageNode {
                    image: assets.disk_front.clone(),
                    image_mode: NodeImageMode::Stretch,
                    ..default()
                },
            ));
            panel.spawn((
                Node {
                    display: Display::None,
                    ..PLAYER_FREE_CHAT_RECT.node()
                },
                ImageNode::new(assets.free_chat.clone()),
                PlayerFreeChatIcon,
            ));
            panel.spawn((
                Node {
                    display: Display::None,
                    ..PLAYER_COMBAT_TOGGLE_RECT.node()
                },
                ImageNode::new(assets.combat_toggle.clone()),
                PlayerCombatToggleIcon,
            ));
        });
}

pub(super) fn bind_player_auxiliary_status(
    model: Res<GameplayUiModel>,
    mut combat_toggle: Single<&mut Node, With<PlayerCombatToggleIcon>>,
) {
    if !model.is_changed() {
        return;
    }
    combat_toggle.display = if model.player.allow_player_interaction {
        Display::None
    } else {
        Display::Flex
    };
}

pub(super) fn bind_portrait_image(
    portrait: Res<GameplayPortraitImage>,
    mut portrait_image: Single<(&mut ImageNode, &mut Node), With<PlayerPortrait>>,
) {
    if portrait.is_changed() {
        let (image, node) = &mut *portrait_image;
        if let Some(handle) = &portrait.0 {
            image.image = handle.clone();
            node.display = Display::Flex;
        } else {
            image.image = default();
            node.display = Display::None;
        }
    }
}
