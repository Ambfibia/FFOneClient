use super::*;

pub(super) fn current_contract(
    projection: &VendorModeProjection0104,
    source: VendorItemPopupSource0104,
) -> Option<VendorItemActionPopup0104> {
    let activation = match source {
        VendorItemPopupSource0104::CatalogRow { row_index } => {
            projection.primary_row_activation(VendorTab0104::Buy, row_index)
        }
        VendorItemPopupSource0104::BuybackRow { row_index } => {
            projection.primary_row_activation(VendorTab0104::Buyback, row_index)
        }
        VendorItemPopupSource0104::InventorySlot { inventory_slot } => {
            projection.primary_inventory_activation(inventory_slot)
        }
    };
    match activation {
        VendorActivationOutcome0104::Popup(popup) => Some(popup),
        _ => None,
    }
}

pub(super) fn reconcile(
    mut popup: ResMut<VendorItemPopupState>,
    projection: Res<VendorModeProjection0104>,
    state: Res<VendorUiState>,
    mut modal: ResMut<VendorModalState>,
    mut close_gate: ResMut<VendorCloseGate0104>,
    help: Option<Res<crate::game_guide_ui::GameGuideUiModel>>,
) {
    if popup.is_open()
        && (!state
            .input_capabilities(VendorModalState::default())
            .row_actions
            || !popup.valid(&projection))
    {
        popup.close();
    }
    let open = popup.is_open();
    if modal.inventory_popup != open {
        modal.inventory_popup = open;
    }
    let help_open = help.as_ref().is_some_and(|help| help.modal_active());
    if modal.help != help_open {
        modal.help = help_open;
    }
    if close_gate.target_action_idle == open {
        close_gate.target_action_idle = !open;
    }
}

pub(super) fn rotate_try_on(
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    modal: Res<VendorModalState>,
    mut popup: ResMut<VendorItemPopupState>,
    buttons: Query<(&Part, &Interaction), With<Button>>,
) {
    if windows.iter().any(|w| !w.focused)
        || popup.try_on_item().is_none()
        || modal.system_popup
        || modal.help
        || modal.generic_popup
    {
        return;
    }
    for (part, interaction) in &buttons {
        if *interaction == Interaction::Pressed {
            let direction = match part {
                Part::TryLeft => 1.,
                Part::TryRight => -1.,
                _ => continue,
            };
            popup.try_on_yaw =
                (popup.try_on_yaw + direction * time.delta_secs() * 100.).rem_euclid(360.);
        }
    }
}

pub(super) fn text(
    parent: &mut ChildSpawnerCommands,
    part: Part,
    rect: VendorUiRect,
    value: LocalizedText,
    font: &Handle<Font>,
    size: f32,
) {
    let button_label = matches!(part, Part::Accept | Part::TryOn);
    let mut spawned = parent.spawn((
        part,
        rect.node(),
        Text::default(),
        value,
        (
            TextFont {
                font: (font.clone()).into(),
                font_size: (if button_label { 14. } else { size }).into(),
                ..default()
            },
            LineHeight::Px(if button_label {
                11.3
            } else {
                USER_EQUIP_REGULAR_FONT_LINE_HEIGHT
            }),
        ),
        TextColor(if matches!(part, Part::Amount) {
            Color::srgb(1., 1., 0.)
        } else {
            Color::srgb(0.8, 1.0, 1.0)
        }),
        TextLayout::default().with_justify(match part {
            Part::Accept | Part::TryOn | Part::Digit(_) | Part::AmountLabel => Justify::Center,
            Part::Amount => Justify::Right,
            _ => Justify::Left,
        }),
        UiTransform::from_translation(Val2::px(0., if button_label { 8.35 } else { 0. })),
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

pub(super) fn spawn(mut commands: Commands, server: Res<AssetServer>, assets: Res<VendorUiAssets>) {
    commands
        .spawn((
            Part::Root,
            Node {
                display: Display::None,
                ..VendorUiRect::new(0., 0., 310., 449.).node()
            },
            GlobalZIndex(VENDOR_UI_Z_INDEX + 10),
        ))
        .with_children(|root| {
            crate::item_card::spawn(root, crate::item_card::CardOwner::Vendor, &assets.font);
            root.spawn((
                Part::Back,
                VendorUiRect::new(0., 14., 310., 435.).node(),
                ImageNode::new(server.load(USER_EQUIP_EQUIP_POPUP_PATH)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            root.spawn((
                Part::Icon,
                VendorUiRect::new(16., 16., 64., 64.).node(),
                ImageNode::default(),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            root.spawn((
                Part::CombinedBadge,
                VendorUiRect::new(52., 52., 26., 26.).node(),
                ImageNode::new(server.load(crate::user_equip_ui::USER_EQUIP_COMBINED_PATH)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            root.spawn((
                Part::Info,
                VendorUiRect::new(2., 171., 305., 84.).node(),
                sliced_image(
                    server.load(crate::user_equip_ui::USER_EQUIP_EQUIP_INFO_PATH),
                    BorderRect::axes(0., 5.),
                ),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            text(
                root,
                Part::Description,
                VendorUiRect::new(12., 95., 280., 40.),
                LocalizedText::new("ui.inventory.popup.description", "{description}")
                    .with_arg("description", ""),
                &assets.font,
                8.,
            );
            for (index, (left, top, width, key, fallback)) in [
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
                text(
                    root,
                    Part::Detail(index as u8),
                    VendorUiRect::new(left, top, width, 20.),
                    LocalizedText::new(key, fallback).with_arg("value", ""),
                    &assets.font,
                    12.,
                );
            }
            text(
                root,
                Part::Name,
                VendorUiRect::new(82., 16., 170., 40.),
                vendor_item_name_localized(""),
                &assets.font,
                12.,
            );
            text(
                root,
                Part::Level,
                VendorUiRect::new(82., 50., 170., 20.),
                LocalizedText::new("ui.inventory.popup.level", "Level {level}")
                    .with_arg("level", ""),
                &assets.font,
                12.,
            );
            root.spawn((
                Part::Pad,
                VendorUiRect::new(87., 162., 136., 109.).node(),
                ImageNode::new(server.load(USER_EQUIP_CALCULATOR_BACK_PATH)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            text(
                root,
                Part::Amount,
                VendorUiRect::new(88., 163., 132., 20.),
                LocalizedText::new("ui.inventory.popup.amount_value", "{amount}")
                    .with_arg("amount", "0"),
                &assets.font,
                16.,
            );
            text(
                root,
                Part::AmountLabel,
                VendorUiRect::new(87., 140., 137., 20.),
                LocalizedText::new("ui.inventory.popup.amount", "Amount"),
                &assets.font,
                12.,
            );
            for index in 0..11 {
                let part = if index == 9 {
                    Part::Clear
                } else {
                    Part::Digit(if index == 10 { 0 } else { index + 1 })
                };
                let copy = match part {
                    Part::Digit(digit) => {
                        LocalizedText::new("ui.inventory.popup.keypad_digit", "{digit}")
                            .with_arg("digit", digit.to_string())
                    }
                    _ => LocalizedText::new("ui.inventory.popup.clear_short", "C"),
                };
                root.spawn((
                    Button,
                    part,
                    VendorUiRect::new(
                        88. + f32::from(index % 3) * 45.,
                        191. + f32::from(index / 3) * 20.,
                        44.,
                        19.,
                    )
                    .node(),
                    BackgroundColor(Color::NONE),
                ))
                .with_children(|button| {
                    text(
                        button,
                        Part::Digit(255),
                        VendorUiRect::new(0., 0., 44., 19.),
                        copy,
                        &assets.font,
                        16.,
                    )
                });
            }
            root.spawn((
                Button,
                Part::TryOn,
                VendorUiRect::new(94., 390., 100., 28.).node(),
                sliced_image(
                    server.load(USER_EQUIP_BUTTON_NORMAL_PATH),
                    BorderRect {
                        min_inset: Vec2::new(6., 6.),
                        max_inset: Vec2::new(6., 4.),
                    },
                ),
            ))
            .with_children(|button| {
                text(
                    button,
                    Part::TryOn,
                    VendorUiRect::new(0., 0., 100., 28.),
                    LocalizedText::new("ui.vendor.popup.try_on", "TRY ON"),
                    &assets.font,
                    12.,
                )
            });
            root.spawn((
                Part::TryPanel,
                VendorUiRect::new(310., 40., 215., 375.).node(),
                ImageNode::new(server.load("ui/en/user-equip/try-on-background.png")),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Part::TryAvatar,
                    VendorUiRect::new(0., 4., 210., 375.).node(),
                    ImageNode::default(),
                    Pickable::IGNORE,
                    bevy::ui::FocusPolicy::Pass,
                ));
                for (part, rect, path) in [
                    (
                        Part::TryClose,
                        VendorUiRect::new(182., 0., 32., 32.),
                        USER_EQUIP_CLOSE_PATH,
                    ),
                    (
                        Part::TryLeft,
                        VendorUiRect::new(19., 270., 43., 78.),
                        crate::user_equip_ui::USER_EQUIP_TURN_RIGHT_PATH,
                    ),
                    (
                        Part::TryRight,
                        VendorUiRect::new(160., 270., 43., 78.),
                        crate::user_equip_ui::USER_EQUIP_TURN_LEFT_PATH,
                    ),
                ] {
                    panel.spawn((Button, part, rect.node(), ImageNode::new(server.load(path))));
                }
            });
            root.spawn((
                Button,
                Part::Close,
                VendorUiRect::new(277., 0., 32., 32.).node(),
                ImageNode::new(server.load(USER_EQUIP_CLOSE_PATH)),
            ));
            root.spawn((
                Button,
                Part::Delete,
                VendorUiRect::new(10., 375., 32., 32.).node(),
                ImageNode::new(server.load(USER_EQUIP_TRASH_PATH)),
            ));
            root.spawn((
                Button,
                Part::Accept,
                VendorUiRect::new(210., 390., 85., 28.).node(),
                sliced_image(
                    server.load(USER_EQUIP_BUTTON_NORMAL_PATH),
                    BorderRect {
                        min_inset: Vec2::new(6., 6.),
                        max_inset: Vec2::new(6., 4.),
                    },
                ),
            ))
            .with_children(|button| {
                text(
                    button,
                    Part::Accept,
                    VendorUiRect::new(0., 0., 85., 28.),
                    LocalizedText::new("ui.vendor.popup.buy", "BUY"),
                    &assets.font,
                    12.,
                )
            });
        });
}
