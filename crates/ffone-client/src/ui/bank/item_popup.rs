//! Bank item inspection and transfer; inventory snapshots remain server-owned.
use super::*;
use crate::user_equip_ui::{
    USER_EQUIP_BUTTON_HOVER_PATH, USER_EQUIP_BUTTON_NORMAL_PATH, USER_EQUIP_EQUIP_INFO_PATH,
    USER_EQUIP_EQUIP_POPUP_PATH, USER_EQUIP_USE_DIALOG_PATH,
};

#[derive(Clone, Copy, Debug)]
struct Selection {
    pc: i32,
    npc: i32,
    slot: BankSlotRef0104,
    item: ItemBase0104,
}

#[derive(Default, Resource)]
pub struct BankItemPopupState {
    selection: Option<Selection>,
}
impl BankItemPopupState {
    pub(crate) fn card_snapshot(&self) -> Option<crate::item_card::CardSnapshot> {
        Some(crate::item_card::CardSnapshot {
            item: self.selection?.item,
            catalog: false,
            price: None,
        })
    }

    pub fn is_open(&self) -> bool {
        self.selection.is_some()
    }
    pub fn close(&mut self) {
        self.selection = None;
    }
    pub(super) fn open(&mut self, projection: &BankModeProjection0104, slot: BankSlotRef0104) {
        self.selection = (!projection.slot_locked(slot) && projection.item_at(slot).item_id > 0)
            .then(|| Selection {
                pc: projection.owner_pc_id,
                npc: projection.npc_id,
                slot,
                item: projection.item_at(slot),
            });
    }
    fn valid(&self, p: &BankModeProjection0104) -> bool {
        self.selection.is_some_and(|s| {
            s.pc == p.owner_pc_id
                && s.npc == p.npc_id
                && !p.slot_locked(s.slot)
                && s.item == p.item_at(s.slot)
        })
    }
}

pub(super) fn reconcile(
    mut popup: ResMut<BankItemPopupState>,
    projection: Res<BankModeProjection0104>,
    state: Res<BankUiState>,
    mut modal: ResMut<BankModalState>,
    input: Option<Res<crate::shared_input_ui::SharedInputDialog>>,
) {
    if popup.is_open()
        && (!popup.valid(&projection)
            || state.phase != BankLifecyclePhase::Visible
            || state.send_pending)
    {
        popup.close();
    }
    // The shared text-entry owner may also hold this gate.
    let open = popup.is_open()
        || input
            .as_ref()
            .is_some_and(|input| input.owner() == Some(crate::shared_input_ui::REDEEM_INPUT_OWNER));
    if modal.inventory_popup != open {
        modal.inventory_popup = open;
    }
}

#[derive(Component, Clone, Copy)]
pub(super) enum Part {
    Root,
    Back,
    Icon,
    CombinedBadge,
    Name,
    Level,
    Description,
    Info,
    Detail(u8),
    Accept,
    Delete,
    Close,
}

pub(super) fn input(
    mut popup: ResMut<BankItemPopupState>,
    projection: Res<BankModeProjection0104>,
    mut state: ResMut<BankUiState>,
    mut modal: ResMut<BankModalState>,
    mut outbox: ResMut<BankUiOutbox0104>,
    mut requests: ResMut<BankLocalRequests>,
    buttons: Query<(&Part, &Interaction), (With<Button>, Changed<Interaction>)>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut audio: Option<ResMut<crate::gameplay_audio::GameplayAudioRuntime>>,
    deletion: Option<ResMut<BankItemDeleteState>>,
    messages: Option<ResMut<crate::system_message_ui::SystemMessageUiModel>>,
) {
    if !popup.is_open()
        || modal.help
        || modal.generic_popup
        || modal.system_popup
        || windows.iter().any(|w| !w.focused)
    {
        return;
    }
    let action = buttons
        .iter()
        .find_map(|(part, interaction)| (*interaction == Interaction::Pressed).then_some(*part));
    let escape = keys
        .as_ref()
        .is_some_and(|keys| keys.just_pressed(KeyCode::Escape));
    if !escape && matches!(action, Some(Part::Delete)) {
        if popup.valid(&projection)
            && let Some(selected) = popup.selection
            && selected.slot.location() == BankSlotLocation0104::Inventory
            && let (Some(mut deletion), Some(mut messages)) = (deletion, messages)
            && deletion.open(&projection, selected.slot.index(), &mut messages)
        {
            popup.close();
            modal.system_popup = true;
            if let Some(audio) = audio.as_mut() {
                audio.queue_legacy_button_sound();
            }
        }
        return;
    }
    if !escape && !matches!(action, Some(Part::Accept | Part::Close)) {
        return;
    }
    let selected = popup.selection;
    let valid = popup.valid(&projection);
    popup.close();
    modal.inventory_popup = false;
    if let Some(audio) = audio.as_mut() {
        audio.queue_legacy_button_sound();
    }
    if escape || !matches!(action, Some(Part::Accept)) || !valid {
        modal.inventory_popup = true; // Retain the input gate through this frame.
        return;
    }
    if let Some(selected) = selected {
        match state.request_one_click(*modal, &projection, selected.slot, &mut outbox) {
            Err(BankActionBlocked::Transfer(BankTransferError0104::BankFull)) => {
                requests.full = Some(BankSlotLocation0104::Bank)
            }
            Err(BankActionBlocked::Transfer(BankTransferError0104::InventoryFull)) => {
                requests.full = Some(BankSlotLocation0104::Inventory)
            }
            _ => {}
        }
    }
    modal.inventory_popup = true;
}

fn label(
    parent: &mut ChildSpawnerCommands,
    part: Part,
    rect: BankUiRect,
    copy: LocalizedText,
    font: &Handle<Font>,
    size: f32,
) {
    let button = matches!(part, Part::Accept);
    let mut spawned = parent.spawn((
        part,
        rect.node(),
        Text::new(""),
        copy,
        (
            TextFont {
                font: (font.clone()).into(),
                font_size: (if button { 14. } else { size }).into(),
                ..default()
            },
            LineHeight::Px(if button {
                11.3
            } else {
                USER_EQUIP_REGULAR_FONT_LINE_HEIGHT
            }),
        ),
        TextColor(Color::srgb(0.8, 1., 1.)),
        TextLayout::default().with_justify(if button {
            Justify::Center
        } else {
            Justify::Left
        }),
        UiTransform::from_translation(Val2::px(0., if button { 8.35 } else { 0. })),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    if matches!(part, Part::Name | Part::Description) {
        let text_font = (
            TextFont {
                font: (font.clone()).into(),
                font_size: (size).into(),
                ..default()
            },
            LineHeight::Px(USER_EQUIP_REGULAR_FONT_LINE_HEIGHT),
        );
        spawned.insert(crate::localization::UiTextAutoFit::new(
            if matches!(part, Part::Name) {
                170.
            } else {
                280.
            },
            if matches!(part, Part::Name) { 31. } else { 40. },
            &text_font,
        ));
    }
}

fn transfer_label(inventory: bool) -> LocalizedText {
    if inventory {
        LocalizedText::new("ui.bank.popup.to_vault", "MOVE TO VAULT")
    } else {
        LocalizedText::new("ui.bank.popup.to_inventory", "MOVE TO MY EQUIPMENT")
    }
}

pub(super) fn spawn(
    mut commands: Commands,
    server: Res<AssetServer>,
    assets: Res<BankUiRuntimeAssets>,
) {
    let assets = &assets.0;
    commands
        .spawn((
            Part::Root,
            Node {
                display: Display::None,
                ..BankUiRect::new(0., 0., 310., 449.).node()
            },
            GlobalZIndex(BANK_UI_Z_INDEX + 10),
        ))
        .with_children(|root| {
            crate::item_card::spawn(root, crate::item_card::CardOwner::Bank, &assets.font);
            for (part, rect, path) in [
                (
                    Part::Back,
                    BankUiRect::new(0., 14., 310., 435.),
                    USER_EQUIP_EQUIP_POPUP_PATH,
                ),
                (
                    Part::Icon,
                    BankUiRect::new(16., 16., 64., 64.),
                    USER_EQUIP_EQUIP_POPUP_PATH,
                ),
            ] {
                root.spawn((
                    part,
                    rect.node(),
                    ImageNode::new(server.load(path)),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ));
            }
            root.spawn((
                Part::CombinedBadge,
                BankUiRect::new(52., 52., 26., 26.).node(),
                ImageNode::new(server.load(crate::user_equip_ui::USER_EQUIP_COMBINED_PATH)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            root.spawn((
                Part::Info,
                BankUiRect::new(2., 171., 305., 84.).node(),
                sliced_image(
                    server.load(USER_EQUIP_EQUIP_INFO_PATH),
                    BorderRect::axes(0., 5.),
                ),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            label(
                root,
                Part::Name,
                BankUiRect::new(82., 16., 170., 40.),
                LocalizedText::new("ui.inventory.popup.item_name", "{name}").with_arg("name", ""),
                &assets.font,
                12.,
            );
            label(
                root,
                Part::Level,
                BankUiRect::new(82., 50., 170., 20.),
                LocalizedText::new("ui.inventory.popup.level", "Level {level}")
                    .with_arg("level", ""),
                &assets.font,
                12.,
            );
            label(
                root,
                Part::Description,
                BankUiRect::new(12., 95., 280., 40.),
                LocalizedText::new("ui.inventory.popup.description", "{description}")
                    .with_arg("description", ""),
                &assets.font,
                8.,
            );
            for (index, (x, y, w, key, fallback)) in [
                (5., 140., 200., "ui.inventory.popup.status", "STATUS"),
                (28., 219., 80., "ui.inventory.popup.point_value", "{value}"),
                (115., 219., 80., "ui.inventory.popup.group_value", "{value}"),
                (
                    207.,
                    219.,
                    80.,
                    "ui.inventory.popup.defense_value",
                    "{value}",
                ),
                (10., 238., 200., "ui.inventory.popup.info", "INFO"),
                (8., 256., 130., "ui.inventory.popup.type", "Type"),
                (8., 276., 130., "ui.inventory.popup.range", "Range"),
                (8., 296., 130., "ui.inventory.popup.rarity", "Rarity"),
                (
                    8.,
                    316.,
                    130.,
                    "ui.inventory.popup.trade",
                    "Trade Availability",
                ),
                (150., 260., 130., "ui.inventory.popup.not_available", "N/A"),
                (150., 280., 130., "ui.inventory.popup.not_available", "N/A"),
                (150., 300., 130., "ui.inventory.popup.not_available", "N/A"),
                (150., 320., 130., "ui.inventory.popup.not_available", "N/A"),
            ]
            .into_iter()
            .enumerate()
            {
                label(
                    root,
                    Part::Detail(index as u8),
                    BankUiRect::new(x, y, w, 20.),
                    LocalizedText::new(key, fallback).with_arg("value", ""),
                    &assets.font,
                    12.,
                );
            }
            root.spawn((
                Part::Delete,
                BankItemDeleteButton,
                Button,
                BankUiRect::new(10., 375., 32., 32.).node(),
                ImageNode::new(server.load(USER_EQUIP_TRASH_PATH)),
            ));
            root.spawn((
                Part::Close,
                Button,
                BankUiRect::new(277., 0., 32., 32.).node(),
                ImageNode::new(server.load(USER_EQUIP_CLOSE_PATH)),
            ));
            root.spawn((
                Part::Accept,
                Button,
                BankUiRect::new(140., 380., 150., 28.).node(),
                sliced_image(
                    server.load(USER_EQUIP_BUTTON_NORMAL_PATH),
                    BorderRect {
                        min_inset: Vec2::new(6., 6.),
                        max_inset: Vec2::new(6., 4.),
                    },
                ),
            ))
            .with_children(|button| {
                label(
                    button,
                    Part::Accept,
                    BankUiRect::new(0., 0., 150., 28.),
                    transfer_label(true),
                    &assets.font,
                    12.,
                )
            });
        });
}

pub(super) fn bind(
    popup: Res<BankItemPopupState>,
    projection: Res<BankModeProjection0104>,
    content: Option<Res<crate::tutorial_mission_content::TutorialMissionContent>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    server: Res<AssetServer>,
    assets: Res<BankUiRuntimeAssets>,
    mut parts: Query<(
        &Part,
        &mut Node,
        Option<&mut ImageNode>,
        Option<&mut LocalizedText>,
        Option<&Interaction>,
        Option<&mut TextColor>,
    )>,
) {
    let assets = &assets.0;
    macro_rules! assign {
        ($field:expr, $value:expr) => {{
            let value = $value;
            if $field != value {
                $field = value;
            }
        }};
    }
    let Some(window) = windows.iter().next() else {
        return;
    };
    let selected = popup.selection;
    let detail = selected.and_then(|s| {
        content
            .as_ref()?
            .gameplay_user_equip_item_detail(s.item.item_type, s.item.item_id)
    });
    let combined = selected
        .is_some_and(|s| (0..=3).contains(&s.item.item_type) && (s.item.option >> 16) as i16 > 0);
    let display_icon = selected.and_then(|s| content.as_ref()?.gameplay_item_display_icon(s.item));
    let item_text = selected.and_then(|s| {
        content.as_ref()?.gameplay_user_equip_item_text(
            s.item.item_type,
            crate::user_equip_ui::user_equip_display_text_id(s.item),
        )
    });
    let chest = selected.is_some_and(|s| s.item.item_type == 9);
    // Clean EquipPopup.ValueColor compares with the player's same-slot item.
    let rating_colors = selected.zip(content.as_deref()).map(|(s, content)| {
        crate::user_equip_ui::user_equip_item_rating_colors(
            content,
            Color::srgb(0.8, 1., 1.),
            s.item,
            usize::try_from(s.item.item_type)
                .ok()
                .and_then(|slot| projection.item_mode.equipped_item(slot)),
            false,
        )
    });
    for (part, mut node, image, mut copy, interaction, color) in &mut parts {
        assign!(
            node.display,
            if matches!(part, Part::Root) && selected.is_none()
                || chest && matches!(part, Part::Info | Part::Detail(_))
                || selected.is_some_and(|s| s.item.item_type == 10)
                    && matches!(part, Part::Info | Part::Detail(0..=3))
                || matches!(part, Part::CombinedBadge)
                    && !selected.is_some_and(|s| (0..=3).contains(&s.item.item_type)
                        && ((s.item.option >> 16) as i16) > 0)
            {
                Display::None
            } else {
                Display::Flex
            }
        );
        let Some(s) = selected else {
            continue;
        };
        if let Some(mut color) = color {
            let value = match part {
                Part::Level => Color::srgb(1., 1., 0.),
                Part::Detail(10) if s.item.item_type != 0 && s.item.item_type != 10 => {
                    Color::srgb(0., 0., 1.)
                }
                Part::Detail(12) => {
                    if !combined && detail.as_ref().is_some_and(|d| d.tradeable) {
                        Color::srgb(0., 1., 0.)
                    } else {
                        Color::srgb(1., 0., 0.)
                    }
                }
                _ => Color::srgb(0.8, 1., 1.),
            };
            if matches!(part, Part::Level | Part::Detail(10 | 12)) {
                assign!(color.0, value);
            }
            if let (Part::Detail(index @ 1..=3), Some(colors)) = (part, rating_colors) {
                assign!(color.0, colors[usize::from(*index) - 1]);
            }
        }
        let inventory = s.slot.location() == BankSlotLocation0104::Inventory;
        let mut value = None;
        match part {
            Part::Root => {
                assign!(
                    node.left,
                    px(if inventory { 610. } else { 200. }
                        + ((window.width() as i32 - 1020).max(0) / 2) as f32)
                );
                assign!(
                    node.top,
                    px(if chest { 90. } else { 60. }
                        + ((window.height() as i32 - 638).max(0) / 2) as f32)
                );
                assign!(node.height, px(if chest { 235. } else { 449. }));
            }
            Part::Back => {
                assign!(node.top, px(if chest { 0. } else { 14. }));
                assign!(node.height, px(if chest { 235. } else { 435. }));
                if let Some(mut image) = image {
                    assign!(
                        image.image,
                        server.load(if chest {
                            USER_EQUIP_USE_DIALOG_PATH
                        } else {
                            USER_EQUIP_EQUIP_POPUP_PATH
                        })
                    );
                }
            }
            Part::Icon => {
                let icon = if inventory {
                    &projection.item_mode.inventory[s.slot.index()].item.icon
                } else {
                    &projection.bank[s.slot.index()].icon
                };
                if let Some(mut image) = image {
                    assign!(
                        image.image,
                        if let Some(path) = display_icon {
                            server.load(path.to_owned())
                        } else if combined && content.is_some() {
                            assets.missing_checker.clone()
                        } else {
                            match icon {
                                UserEquipProjectedIcon::Resolved(icon) => {
                                    server.load(icon.runtime_path().to_owned())
                                }
                                UserEquipProjectedIcon::MissingChecker(_) => {
                                    assets.missing_checker.clone()
                                }
                                UserEquipProjectedIcon::Empty => Handle::default(),
                            }
                        }
                    );
                }
            }
            Part::Name => {
                value = Some(
                    item_text
                        .as_ref()
                        .map(|text| text.0.clone())
                        .unwrap_or_else(|| {
                            LocalizedText::new("ui.inventory.popup.item_name", "{name}")
                                .with_arg("name", "")
                        }),
                )
            }
            Part::Level => {
                value = Some(
                    LocalizedText::new("ui.inventory.popup.level", "Level {level}").with_arg(
                        "level",
                        detail
                            .as_ref()
                            .map_or(String::new(), |d| d.level.to_string()),
                    ),
                )
            }
            Part::Description => {
                value = Some(
                    item_text
                        .as_ref()
                        .map(|text| text.1.clone())
                        .unwrap_or_else(|| {
                            LocalizedText::new("ui.inventory.popup.description", "{description}")
                                .with_arg("description", "")
                        }),
                )
            }
            Part::Detail(index) => {
                use crate::user_equip_ui::{
                    user_equip_item_type_localized, user_equip_range_localized,
                    user_equip_rarity_localized, user_equip_weapon_type_localized,
                };
                value = match index {
                    1..=3 => Some(
                        LocalizedText::new(
                            match index {
                                1 => "ui.inventory.popup.point_value",
                                2 => "ui.inventory.popup.group_value",
                                _ => "ui.inventory.popup.defense_value",
                            },
                            "{value}",
                        )
                        .with_arg(
                            "value",
                            detail.as_ref().map_or(String::new(), |d| {
                                match index {
                                    1 => d.point_rating,
                                    2 => d.group_rating,
                                    _ => d.defense_rating,
                                }
                                .to_string()
                            }),
                        ),
                    ),
                    6 => Some(LocalizedText::new(
                        if s.item.item_type == 10 {
                            "ui.inventory.popup.speed"
                        } else {
                            "ui.inventory.popup.range"
                        },
                        if s.item.item_type == 10 {
                            "Speed"
                        } else {
                            "Range"
                        },
                    )),
                    10 if s.item.item_type == 10 => Some(
                        LocalizedText::new("ui.inventory.vehicle_class", "{speed} Class").with_arg(
                            "speed",
                            detail
                                .as_ref()
                                .and_then(|d| d.vehicle_speed_class)
                                .unwrap_or_default()
                                .to_string(),
                        ),
                    ),
                    9 if s.item.item_type == 0 => Some(user_equip_weapon_type_localized(
                        detail.as_ref().and_then(|d| d.target_mode),
                    )),
                    9 => Some(user_equip_item_type_localized(s.item.item_type)),
                    10 => Some(user_equip_range_localized(
                        detail
                            .as_ref()
                            .filter(|_| s.item.item_type == 0)
                            .and_then(|d| d.equip_type),
                    )),
                    11 if combined => {
                        Some(LocalizedText::new("ui.inventory.rarity.special", "Special"))
                    }
                    11 => Some(user_equip_rarity_localized(
                        detail.as_ref().and_then(|d| d.rarity),
                    )),
                    12 => Some(
                        if !combined && detail.as_ref().is_some_and(|d| d.tradeable) {
                            LocalizedText::new("ui.inventory.popup.trade_value", "Tradable")
                        } else {
                            LocalizedText::new("ui.inventory.popup.not_tradable", "Not tradable")
                        },
                    ),
                    _ => None,
                };
            }
            Part::Delete => {
                assign!(
                    node.display,
                    if inventory {
                        Display::Flex
                    } else {
                        Display::None
                    }
                );
                assign!(node.top, px(if chest { 180. } else { 375. }));
            }
            Part::Accept => {
                assign!(node.width, px(if inventory { 150. } else { 220. }));
                if let Some(mut image) = image {
                    assign!(node.left, px(if inventory { 140. } else { 70. }));
                    assign!(node.top, px(if chest { 180. } else { 380. }));
                    assign!(
                        image.image,
                        server.load(
                            if matches!(
                                interaction,
                                Some(Interaction::Hovered | Interaction::Pressed)
                            ) {
                                USER_EQUIP_BUTTON_HOVER_PATH
                            } else {
                                USER_EQUIP_BUTTON_NORMAL_PATH
                            }
                        )
                    );
                }
                value = Some(transfer_label(inventory));
            }
            _ => {}
        }
        if let (Some(copy), Some(value)) = (copy.as_mut(), value) {
            if **copy != value {
                **copy = value;
            }
        }
    }
}

#[cfg(test)]
mod tests;
