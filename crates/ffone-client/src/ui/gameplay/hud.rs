//! HUD root spawning, scale/layout, camera activity, menu transition and HUD buttons.

use super::actions::{
    GameplayUiAction, GameplayUiAudioCue, GameplayUiAudioOutbox, GameplayUiOutbox,
};
use super::active_nano::{ACTIVE_NANO_INFO_RECT, ActiveNanoInfoRoot, spawn_active_nano_info};
use super::assets::GameplayUiAssets;
use super::chat_model::{ChatInputHistory, clamped_chat_size};
use super::chat_spawn::{ChatRoot, ChatTabButton, ChatTextFieldButton, SendChatButton, spawn_chat};
use super::combat_frame::{CombatModeNoticeText, spawn_combat_danger, spawn_combat_frame};
use super::combat_target::{
    spawn_combat_target_info, spawn_nano_skill_target_icons, spawn_primary_combat_target_icon,
    spawn_primary_combat_target_status, spawn_secondary_combat_target_icons,
};
use super::minimap::{MinimapRoot, spawn_minimap};
use super::minimap_model::MINIMAP_GROUP_RECT;
use super::model::{GameplayMenuTransition, GameplayUiModel};
use super::nano_wheel::{NanoWheelRoot, spawn_nano_wheel};
use super::player_status::{PlayerStatusRoot, spawn_player_status};
use super::quick_chat::{
    EmoteButton, MenuChatButton, QuickChatMenuMode, QuickChatRowButton, spawn_quick_chat_columns,
};
use super::speech_bubbles::{NPC_BARKER_GLOBAL_Z_INDEX, NpcBarkerBubbleLayer};
use super::{CircularMinimapTileMaterial, FusionMatterMeterMaterial};
use super::{
    GAMEPLAY_UI_CAMERA_ORDER, GAMEPLAY_UI_SCALE_NUDGE, GAMEPLAY_UI_SCALE_REFERENCE_HEIGHT,
    GameplayHud, GameplayUiCamera, rewards, trigger_icon,
};
use crate::localization::LocalizedText;
use crate::text_edit::TextEdit;
use bevy::{prelude::*, text::LineHeight};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GameplayUiRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl GameplayUiRect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub(super) fn node(self) -> Node {
        Node {
            position_type: PositionType::Absolute,
            left: px(self.x),
            top: px(self.y),
            width: px(self.width),
            height: px(self.height),
            ..default()
        }
    }
}

/// Exact default `FFGUIUtility.GetUiScale` result.
///
/// Retrobution enables `cnDisplayOption.bScaleUI` by default. Its
/// `cnGraphicOption.GetUiScale` returns
/// `max(1, Screen.height / 768 * 1.05)`.
#[must_use]
pub fn gameplay_ui_scale(viewport_height: f32) -> f32 {
    if !viewport_height.is_finite() || viewport_height <= 0.0 {
        return 1.0;
    }
    (viewport_height / GAMEPLAY_UI_SCALE_REFERENCE_HEIGHT * GAMEPLAY_UI_SCALE_NUDGE).max(1.0)
}

pub(super) fn spawn_gameplay_hud(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    existing_assets: Option<Res<GameplayUiAssets>>,
    mut minimap_materials: ResMut<Assets<CircularMinimapTileMaterial>>,
    mut fusion_meter_materials: ResMut<Assets<FusionMatterMeterMaterial>>,
) {
    // The native UI phase can return to Gameplay after character selection.
    // Keep the resident HUD and its camera instead of spawning a second
    // order-100 camera on every re-entry.
    if existing_assets.is_some() {
        return;
    }
    let assets = GameplayUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands.spawn((
        Camera2d,
        Camera {
            order: GAMEPLAY_UI_CAMERA_ORDER,
            clear_color: ClearColorConfig::None,
            ..default()
        },
        IsDefaultUiCamera,
        GameplayUiCamera,
    ));
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: percent(100),
                height: percent(100),
                ..default()
            },
            GameplayHud,
        ))
        .with_children(|hud| {
            rewards::spawn(hud, &assets, &asset_server);
            spawn_combat_frame(hud, &assets);
            spawn_player_status(hud, &assets);
            spawn_active_nano_info(hud, &assets);
            spawn_minimap(
                hud,
                &assets,
                &mut minimap_materials,
                &mut fusion_meter_materials,
            );
            spawn_chat(hud, &assets);
            spawn_quick_chat_columns(hud, &assets);
            spawn_nano_wheel(hud, &assets);
            spawn_primary_combat_target_icon(hud, &assets);
            trigger_icon::spawn(hud, &asset_server);
            spawn_secondary_combat_target_icons(hud, &assets);
            spawn_nano_skill_target_icons(hud, &assets);
            spawn_primary_combat_target_status(hud, &assets);
            spawn_combat_target_info(hud, &assets);
            spawn_combat_danger(hud, &assets);
            for shadow in [true, false] {
                hud.spawn((
                    Node {
                        display: Display::None,
                        position_type: PositionType::Absolute,
                        height: px(40),
                        padding: UiRect::new(px(10), px(6), px(4), px(6)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    Text::new(""),
                    LocalizedText::new("ui.gameplay.combat.enabled", "Combat mode enabled"),
                    (
                        TextFont {
                            font: (assets.jeffe_font.clone()).into(),
                            font_size: (14.0).into(),
                            ..default()
                        },
                        LineHeight::Px(16.451_999_66),
                    ),
                    TextLayout::default().with_justify(Justify::Center),
                    TextColor(Color::WHITE),
                    CombatModeNoticeText { shadow },
                    Pickable::IGNORE,
                    ZIndex(950),
                ));
            }
        });
    // Clean `PrintName.DrawBarker` is a resident world-space IMGUI pass, not
    // a child of `CnGuiNanocom`. Keep the bubble layer independent from the
    // HUD root so opening an NPC mode cannot hide the greeting with the HUD.
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: px(0),
            top: px(0),
            width: percent(100),
            height: percent(100),
            ..default()
        },
        GlobalZIndex(NPC_BARKER_GLOBAL_Z_INDEX),
        Pickable::IGNORE,
        NpcBarkerBubbleLayer,
    ));
}

pub(super) fn sync_gameplay_camera_activity(
    phase: Res<State<crate::ui_startup::NativeUiStartupPhase>>,
    mut cameras: Query<&mut Camera, With<GameplayUiCamera>>,
) {
    let active = phase.get() == &crate::ui_startup::NativeUiStartupPhase::Gameplay;
    for mut camera in &mut cameras {
        camera.is_active = active;
    }
}

pub(super) fn advance_gameplay_menu_transition(
    time: Res<Time>,
    mut transition: ResMut<GameplayMenuTransition>,
) {
    if transition.remaining > 0.0 {
        transition.advance(time.delta_secs());
    }
}

pub(super) fn update_gameplay_hud_layout(
    model: Res<GameplayUiModel>,
    mut groups: ParamSet<(
        Query<(&mut Node, &mut UiTransform), With<PlayerStatusRoot>>,
        Query<(&mut Node, &mut UiTransform), With<ActiveNanoInfoRoot>>,
        Query<(&mut Node, &mut UiTransform), With<MinimapRoot>>,
        Query<(&mut Node, &mut UiTransform), With<ChatRoot>>,
        Query<(&mut Node, &mut UiTransform), With<NanoWheelRoot>>,
    )>,
) {
    // A repeated coordinate must not invalidate Taffy's cached subtree. Project
    // through Mut without dirtying it, then publish only an actual field change.
    let scale = if model.ui_scale.is_finite() && model.ui_scale > 0.0 {
        model.ui_scale
    } else {
        1.0
    };

    if let Ok((mut node, mut transform)) = groups.p0().single_mut() {
        node.reborrow()
            .map_unchanged(|value| &mut value.left)
            .set_if_neq(px(289.0 * (scale - 1.0) * 0.5));
        node.reborrow()
            .map_unchanged(|value| &mut value.top)
            .set_if_neq(px(72.0 * (scale - 1.0) * 0.5));
        transform
            .reborrow()
            .map_unchanged(|value| &mut value.scale)
            .set_if_neq(Vec2::splat(scale));
    }
    if let Ok((mut node, mut transform)) = groups.p1().single_mut() {
        node.reborrow()
            .map_unchanged(|value| &mut value.left)
            .set_if_neq(px(
                ACTIVE_NANO_INFO_RECT.x * scale + ACTIVE_NANO_INFO_RECT.width * (scale - 1.0) * 0.5
            ));
        node.reborrow()
            .map_unchanged(|value| &mut value.top)
            .set_if_neq(px(ACTIVE_NANO_INFO_RECT.y * scale
                + ACTIVE_NANO_INFO_RECT.height * (scale - 1.0) * 0.5));
        transform
            .reborrow()
            .map_unchanged(|value| &mut value.scale)
            .set_if_neq(Vec2::splat(scale));
    }
    if let Ok((mut node, mut transform)) = groups.p2().single_mut() {
        node.reborrow()
            .map_unchanged(|value| &mut value.right)
            .set_if_neq(px(MINIMAP_GROUP_RECT.width * (scale - 1.0) * 0.5));
        node.reborrow()
            .map_unchanged(|value| &mut value.top)
            .set_if_neq(px(
                MINIMAP_GROUP_RECT.y * scale + MINIMAP_GROUP_RECT.height * (scale - 1.0) * 0.5
            ));
        transform
            .reborrow()
            .map_unchanged(|value| &mut value.scale)
            .set_if_neq(Vec2::splat(scale));
    }
    if let Ok((mut node, mut transform)) = groups.p3().single_mut() {
        let size = clamped_chat_size(model.chat.window_size);
        node.reborrow()
            .map_unchanged(|value| &mut value.left)
            .set_if_neq(px(size.x * (scale - 1.0) * 0.5));
        node.reborrow()
            .map_unchanged(|value| &mut value.bottom)
            .set_if_neq(px(size.y * (scale - 1.0) * 0.5));
        transform
            .reborrow()
            .map_unchanged(|value| &mut value.scale)
            .set_if_neq(Vec2::splat(scale));
    }
    if let Ok((mut node, mut transform)) = groups.p4().single_mut() {
        node.reborrow()
            .map_unchanged(|value| &mut value.right)
            .set_if_neq(px(20.0 * scale + 223.0 * (scale - 1.0) * 0.5));
        node.reborrow()
            .map_unchanged(|value| &mut value.bottom)
            .set_if_neq(px(10.0 * scale + 101.0 * (scale - 1.0) * 0.5));
        transform
            .reborrow()
            .map_unchanged(|value| &mut value.scale)
            .set_if_neq(Vec2::splat(scale));
    }
}

pub(super) fn handle_gameplay_ui_buttons(
    mut model: ResMut<GameplayUiModel>,
    mut history: ResMut<ChatInputHistory>,
    tabs: Query<(&Interaction, &ChatTabButton), Changed<Interaction>>,
    text_field: Query<&Interaction, (Changed<Interaction>, With<ChatTextFieldButton>)>,
    menu: Query<&Interaction, (Changed<Interaction>, With<MenuChatButton>)>,
    emote: Query<&Interaction, (Changed<Interaction>, With<EmoteButton>)>,
    quick_rows: Query<(&Interaction, &QuickChatRowButton), Changed<Interaction>>,
    send: Query<&Interaction, (Changed<Interaction>, With<SendChatButton>)>,
    mut outbox: ResMut<GameplayUiOutbox>,
    mut audio: ResMut<GameplayUiAudioOutbox>,
) {
    if text_field
        .iter()
        .any(|interaction| *interaction == Interaction::Pressed)
        && model.visible
        && model.chat.visible
        && model.chat.input_enabled
        && !model.chat.active
    {
        model.chat.active = true;
        history.reset_cursor();
        outbox.push(GameplayUiAction::OpenNanocomMenu);
        audio.push(GameplayUiAudioCue::OpenScreen);
    }
    for (interaction, tab) in &tabs {
        // `CnGuiChat` replaces unselected toggles with non-interactive labels
        // while either quick-chat surface is open.
        if *interaction == Interaction::Pressed
            && model.chat.quick_menu.mode == QuickChatMenuMode::Closed
        {
            if tab.0 != model.chat.selected {
                audio.push(GameplayUiAudioCue::TabClick01);
            }
            outbox
                .actions
                .push_back(GameplayUiAction::SelectChatChannel(tab.0));
        }
    }
    if menu.iter().any(|value| *value == Interaction::Pressed)
        && model.chat.quick_menu.mode != QuickChatMenuMode::MenuChat
    {
        audio.push(GameplayUiAudioCue::ButtonSound);
        audio.push(GameplayUiAudioCue::ClickWindowSlideOut);
        outbox.actions.push_back(GameplayUiAction::ToggleMenuChat);
    }
    if emote.iter().any(|value| *value == Interaction::Pressed)
        && model.chat.active
        && model.chat.quick_menu.mode != QuickChatMenuMode::Emotes
    {
        audio.push(GameplayUiAudioCue::ButtonSound);
        audio.push(GameplayUiAudioCue::ClickWindowSlideOut);
        outbox.actions.push_back(GameplayUiAction::ToggleEmotes);
    }
    for (interaction, row) in &quick_rows {
        if *interaction != Interaction::Pressed
            || !model.visible
            || !model.chat.visible
            || !model.chat.input_enabled
        {
            continue;
        }
        let Some(item) = row.item else {
            continue;
        };
        audio.push(GameplayUiAudioCue::ButtonSound);
        if item.container {
            audio.push(GameplayUiAudioCue::ClickWindowSlideOut);
        }
        outbox.push(GameplayUiAction::SelectQuickChatItem(item));
    }
    let send_pressed = send.iter().any(|value| *value == Interaction::Pressed)
        && model.chat.input_enabled
        && model.chat.active;
    if send_pressed {
        audio.push(GameplayUiAudioCue::ButtonSound);
    }
    if send_pressed && !model.chat.input.is_empty() {
        // `CnGuiChat.Chat()` clears and records text but, unlike Enter, the
        // SEND button leaves chat/Nanocom open.
        model.chat.edit = TextEdit::default();
        let message = std::mem::take(&mut model.chat.input);
        history.record(message.clone());
        outbox
            .actions
            .push_back(GameplayUiAction::SendChat(message));
    }
}
