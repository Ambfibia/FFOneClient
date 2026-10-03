//! Shared, owner-addressed text-entry surface. Submission never sends a packet.
use crate::{
    buddy_ui::*,
    localization::{LocalizationSet, LocalizedText},
};
use bevy::{
    input::{ButtonState, InputSystems, keyboard::KeyboardInput},
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    ui::widget::NodeImageMode,
    window::PrimaryWindow,
};
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub struct SharedInputRequest {
    pub owner: u64,
    pub title: LocalizedText,
    pub instruction: LocalizedText,
    pub submit: LocalizedText,
    pub max_utf16: usize,
}

pub const REDEEM_INPUT_OWNER: u64 = 0x5245_4445_454d_0001;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedeemSource {
    Inventory { pc: i32 },
    Bank { pc: i32, npc: i32 },
    Vendor { pc: i32, npc: i32 },
}

#[derive(Default, Resource)]
pub struct SharedRedeemCode {
    pub source: Option<RedeemSource>,
}

impl SharedRedeemCode {
    pub fn open(&mut self, source: RedeemSource, input: &mut SharedInputDialog) -> bool {
        if !input.open(SharedInputRequest {
            owner: REDEEM_INPUT_OWNER,
            title: LocalizedText::new("ui.shared_input.redeem_title", "REDEEM CODE"),
            instruction: LocalizedText::new(
                "ui.shared_input.redeem_instruction",
                "Have a code? Try entering it below to get an exclusive item!",
            ),
            submit: LocalizedText::new("ui.shared_input.redeem", "REDEEM"),
            max_utf16: 32,
        }) {
            return false;
        }
        self.source = Some(source);
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RedeemCodeError {
    TooShort,
    TooLong,
    ContainsSpace,
}

pub fn redeem_code_request(
    code: &str,
) -> Result<ffone_protocol::FreeChatRequest0104, RedeemCodeError> {
    let len = code.encode_utf16().count();
    if len < 3 {
        return Err(RedeemCodeError::TooShort);
    }
    if len > 32 {
        return Err(RedeemCodeError::TooLong);
    }
    if code.contains(' ') {
        return Err(RedeemCodeError::ContainsSpace);
    }
    Ok(ffone_protocol::FreeChatRequest0104 {
        message: ffone_protocol::FixedUtf16::from_str(&format!("/redeem {code} "))
            .expect("bounded code fits free chat"),
        emote_code: 0,
    })
}

#[derive(Default, Resource)]
pub struct SharedInputDialog {
    request: Option<SharedInputRequest>,
    value: String,
    actions: VecDeque<SharedInputAction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SharedInputAction {
    Submit { owner: u64, value: String },
    Cancel { owner: u64 },
}

impl SharedInputDialog {
    pub fn open(&mut self, request: SharedInputRequest) -> bool {
        if self.request.is_some() {
            return false;
        }
        self.value.clear();
        self.request = Some(request);
        true
    }
    pub fn owner(&self) -> Option<u64> {
        self.request.as_ref().map(|r| r.owner)
    }
    pub fn submit(&mut self) {
        if let Some(owner) = self.owner()
            && self.actions.is_empty()
        {
            self.actions.push_back(SharedInputAction::Submit {
                owner,
                value: self.value.clone(),
            });
        }
    }
    pub fn cancel(&mut self) {
        if let Some(owner) = self.owner()
            && self.actions.is_empty()
        {
            self.actions.push_back(SharedInputAction::Cancel { owner });
        }
    }
    pub fn close(&mut self, owner: u64) {
        if self.owner() == Some(owner) {
            self.request = None;
            self.value.clear();
        }
        self.actions.retain(|action| match action {
            SharedInputAction::Submit { owner: id, .. }
            | SharedInputAction::Cancel { owner: id } => *id != owner,
        });
    }
    pub fn pop_for(&mut self, owner: u64) -> Option<SharedInputAction> {
        let index = self.actions.iter().position(|action| match action {
            SharedInputAction::Submit { owner: id, .. }
            | SharedInputAction::Cancel { owner: id } => *id == owner,
        })?;
        self.actions.remove(index)
    }
    pub fn append(&mut self, text: &str) {
        let Some(request) = &self.request else {
            return;
        };
        let mut used = self.value.encode_utf16().count();
        for ch in text.chars().filter(|ch| !ch.is_control()) {
            if used + ch.len_utf16() > request.max_utf16 {
                break;
            }
            self.value.push(ch);
            used += ch.len_utf16();
        }
    }
}

pub struct SharedInputUiPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SharedInputSet {
    Input,
    Bind,
}

impl Plugin for SharedInputUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<SharedInputDialog>()
            .init_resource::<SharedRedeemCode>()
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_dialog)
            .add_systems(
                PreUpdate,
                capture_keyboard
                    .after(InputSystems)
                    .before(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(Update, handle_buttons.in_set(SharedInputSet::Input))
            .add_systems(Update, update_controls.after(SharedInputSet::Input))
            .add_systems(
                Update,
                bind_dialog
                    .in_set(SharedInputSet::Bind)
                    .after(SharedInputSet::Input)
                    .before(LocalizationSet::Apply),
            );
    }
}

#[derive(Component)]
struct DialogRoot;
#[derive(Component)]
struct DialogPanel;
#[derive(Component, Clone, Copy)]
enum Field {
    Title,
    Instruction,
    Value,
    Submit,
}
#[derive(Component, Clone, Copy)]
enum Control {
    Cancel,
    Submit,
}

fn rect(x: f32, y: f32, w: f32, h: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(x),
        top: px(y),
        width: px(w),
        height: px(h),
        ..default()
    }
}

fn spawn_dialog(mut commands: Commands, server: Res<AssetServer>) {
    commands
        .spawn((
            DialogRoot,
            Node {
                width: percent(100),
                height: percent(100),
                position_type: PositionType::Absolute,
                display: Display::None,
                ..default()
            },
            GlobalZIndex(BUDDY_MODAL_Z_INDEX),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
            Pickable {
                should_block_lower: true,
                is_hoverable: true,
            },
        ))
        .with_children(|root| {
            root.spawn((
                DialogPanel,
                rect(0.0, 0.0, 526.0, 164.0),
                ImageNode::new(server.load(BUDDY_ADD_DIALOG_PATH)),
            ))
            .with_children(|panel| {
                for (field, geometry, style) in [
                    (
                        Field::Title,
                        BuddyUiRect::new(176.0, 15.0, 173.0, 17.0),
                        BuddyTextStyle::Transparent3,
                    ),
                    (
                        Field::Instruction,
                        BuddyUiRect::new(118.0, 36.0, 288.0, 17.0),
                        BuddyTextStyle::DeleteText,
                    ),
                    (
                        Field::Value,
                        BuddyUiRect::new(75.0, 73.0, 378.0, 24.0),
                        BuddyTextStyle::DeleteText,
                    ),
                ] {
                    panel.spawn((
                        field,
                        buddy_text_node(geometry, style),
                        Text::new(""),
                        LocalizedText::new("ui.shared_input.value", "{value}")
                            .with_arg("value", ""),
                        text_font(&server, style),
                        buddy_text_color(style),
                        buddy_text_layout(style),
                        buddy_text_transform(style),
                        Pickable::IGNORE,
                        bevy::ui::FocusPolicy::Pass,
                    ));
                }
                for (control, x, path, label) in [
                    (
                        Control::Cancel,
                        40.0,
                        BUDDY_CANCEL_BUTTON_PATH,
                        LocalizedText::new("ui.shared_input.cancel", "CANCEL"),
                    ),
                    (
                        Control::Submit,
                        335.0,
                        BUDDY_RED_BUTTON_PATH,
                        LocalizedText::new("ui.shared_input.submit", "SUBMIT"),
                    ),
                ] {
                    let style = match control {
                        Control::Cancel => BuddyTextStyle::Cancel,
                        Control::Submit => BuddyTextStyle::QuitButton,
                    };
                    panel
                        .spawn((
                            control,
                            Button,
                            buddy_text_node(BuddyUiRect::new(x, 125.0, 152.0, 27.0), style),
                            ImageNode {
                                image: server.load(path),
                                visual_box: bevy::ui::VisualBox::BorderBox,
                                image_mode: NodeImageMode::Sliced(TextureSlicer {
                                    border: BorderRect::all(5.0),
                                    center_scale_mode: SliceScaleMode::Stretch,
                                    sides_scale_mode: SliceScaleMode::Stretch,
                                    ..default()
                                }),
                                ..default()
                            },
                        ))
                        .with_children(|button| {
                            let mut text = button.spawn((
                                Text::new(label.fallback.clone()),
                                label,
                                text_font(&server, style),
                                buddy_text_color(style),
                                buddy_text_layout(style),
                                buddy_text_transform(style),
                                Pickable::IGNORE,
                                bevy::ui::FocusPolicy::Pass,
                            ));
                            if matches!(control, Control::Submit) {
                                text.insert(Field::Submit);
                            }
                        });
                }
            });
        });
}

fn text_font(server: &AssetServer, style: BuddyTextStyle) -> (TextFont, bevy::text::LineHeight) {
    let spec = style.spec();
    (
        TextFont {
            font: (server.load(match spec.font_role {
                BuddyFontRole::Jeffe => BUDDY_FONT_PATH,
                BuddyFontRole::Chalet => BUDDY_CHALET_FONT_PATH,
            }))
            .into(),
            font_size: (spec.font_size).into(),
            ..default()
        },
        bevy::text::LineHeight::Px(spec.line_height),
    )
}

fn capture_keyboard(
    mut model: ResMut<SharedInputDialog>,
    system_message: Option<Res<crate::system_message_ui::SystemMessageUiModel>>,
    messages: Option<ResMut<Messages<KeyboardInput>>>,
    buttons: Option<ResMut<ButtonInput<KeyCode>>>,
) {
    if model.owner().is_none() || system_message.is_some_and(|message| message.is_popup()) {
        return;
    }
    if let Some(mut messages) = messages {
        for key in messages.drain() {
            if key.state != ButtonState::Pressed {
                continue;
            }
            if key.key_code == KeyCode::Backspace {
                model.value.pop();
            } else if let Some(text) = key.text {
                model.append(&text);
            }
        }
    }
    // The focused modal consumes game shortcuts as well as chat text.
    if let Some(mut buttons) = buttons {
        buttons.reset_all();
    }
}

fn handle_buttons(
    mut model: ResMut<SharedInputDialog>,
    system_message: Option<Res<crate::system_message_ui::SystemMessageUiModel>>,
    controls: Query<(&Control, &Interaction), Changed<Interaction>>,
) {
    if system_message.is_some_and(|message| message.is_popup()) {
        return;
    }
    let Some(_) = model.owner() else {
        return;
    };
    for (control, interaction) in &controls {
        if *interaction != Interaction::Pressed || !model.actions.is_empty() {
            continue;
        }
        match control {
            Control::Cancel => model.cancel(),
            Control::Submit => model.submit(),
        }
    }
}

fn update_controls(
    server: Res<AssetServer>,
    mut controls: Query<(&Control, &Interaction, &Children, &mut ImageNode), Changed<Interaction>>,
    mut colors: Query<&mut TextColor>,
) {
    for (control, interaction, children, mut image) in &mut controls {
        let (style, path) = match control {
            Control::Cancel => (BuddyTextStyle::Cancel, BUDDY_CANCEL_BUTTON_PATH),
            Control::Submit => (
                BuddyTextStyle::QuitButton,
                if *interaction == Interaction::Hovered {
                    BUDDY_RED_BUTTON_OVER_PATH
                } else {
                    BUDDY_RED_BUTTON_PATH
                },
            ),
        };
        let next = server.load(path);
        if image.image != next {
            image.image = next;
        }
        let spec = style.spec();
        let rgba = match interaction {
            Interaction::Pressed => spec.active_color,
            Interaction::Hovered => spec.hover_color,
            Interaction::None => spec.normal_color,
        };
        for child in children {
            if let Ok(mut color) = colors.get_mut(*child) {
                color.set_if_neq(TextColor(Color::srgba(rgba[0], rgba[1], rgba[2], rgba[3])));
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn bind_dialog(
    model: Res<SharedInputDialog>,
    server: Res<AssetServer>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut roots: Query<&mut Node, With<DialogRoot>>,
    mut panels: Query<&mut Node, (With<DialogPanel>, Without<DialogRoot>)>,
    mut fields: Query<(&Field, &TextFont, &mut LocalizedText)>,
) {
    for mut root in &mut roots {
        let display = if model.owner().is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if root.display != display {
            root.display = display;
        }
    }
    let Some(request) = &model.request else {
        return;
    };
    if let Ok(window) = windows.single() {
        for mut panel in &mut panels {
            let left = px((window.width() * 0.5).floor() - 263.0);
            let top = px((window.height() * 0.5).floor() - 82.0);
            if panel.left != left {
                panel.left = left;
            }
            if panel.top != top {
                panel.top = top;
            }
        }
    }
    for (field, font, mut text) in &mut fields {
        // Do not shape a prefilled draft against the temporary fallback font.
        // Once the native font arrives, publish the retained draft exactly once.
        if matches!(field, Field::Value)
            && let bevy::text::FontSource::Handle(handle) = &font.font
            && !server.is_loaded_with_dependencies(handle.id())
        {
            continue;
        }
        let next = match field {
            Field::Title => request.title.clone(),
            Field::Instruction => request.instruction.clone(),
            Field::Submit => request.submit.clone(),
            Field::Value => LocalizedText::new("ui.shared_input.value", "{value}")
                .with_arg("value", model.value.clone()),
        };
        if *text != next {
            *text = next;
        }
    }
}

#[cfg(test)]
mod tests;
