use super::*;

/// Station requests are drafts. Only the server may change the equipped slots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserEquipNanoStationAction {
    Equip { nano_id: i16, slot: usize },
    Unequip { nano_id: i16, slot: usize },
}

impl UserEquipUiState {
    pub fn open_nano_station(&mut self, npc_id: i32) {
        self.open_item_mode();
        self.nano_station_npc = Some(npc_id);
        self.select_nano_tab();
    }

    pub fn nano_station_npc(&self) -> Option<i32> {
        self.nano_station_npc
    }

    pub fn nano_station_send_pending(&self) -> bool {
        self.nano_station_pending.is_some()
    }

    pub fn begin_nano_station_request(&mut self, action: UserEquipNanoStationAction) {
        self.nano_station_pending = Some((action, 0.0));
    }

    pub fn acknowledge_nano_station_request(&mut self, action: UserEquipNanoStationAction) -> bool {
        if self
            .nano_station_pending
            .is_some_and(|(pending, _)| pending == action)
        {
            self.nano_station_pending = None;
            true
        } else {
            false
        }
    }
}

pub fn nano_station_action_allowed(
    state: &UserEquipUiState,
    modal: UserEquipModalState,
    projection: &UserEquipNanoModeProjection,
    action: UserEquipNanoStationAction,
) -> bool {
    if state.nano_station_npc().is_none()
        || state.mode() != UserEquipMode::Nano
        || !state.input_capabilities(modal).panel_controls
        || state.nano_station_send_pending()
    {
        return false;
    }
    let (nano_id, slot) = match action {
        UserEquipNanoStationAction::Equip { nano_id, slot }
        | UserEquipNanoStationAction::Unequip { nano_id, slot } => (nano_id, slot),
    };
    let Some(entry) = projection
        .gallery
        .iter()
        .find(|entry| entry.nano_id == nano_id)
    else {
        return false;
    };
    let Some(target) = projection.status.get(slot) else {
        return false;
    };
    if !entry.owned || !entry.current_power.is_some_and(|power| power > 0) {
        return false;
    }
    match action {
        UserEquipNanoStationAction::Equip { .. } => !entry.equipped && !target.active,
        UserEquipNanoStationAction::Unequip { .. } => target.nano_id == Some(nano_id),
    }
}

#[derive(Component)]
pub(super) struct StationControl(usize);

#[derive(Component)]
pub(super) struct StationLabel;

#[derive(Component)]
pub(super) struct StationEquipTitle;

pub(super) fn spawn_controls(parent: &mut ChildSpawnerCommands, assets: &UserEquipUiAssets) {
    parent.spawn((
        StationEquipTitle,
        ZIndex(2),
        UserEquipUiRect::new(155.0, 556.0, 55.0, 14.0).node(),
        Text::new("EQUIP"),
        (
            TextFont {
                font: (assets.font.clone()).into(),
                font_size: (14.0).into(),
                ..default()
            },
            LineHeight::Px(11.3),
        ),
        TextColor(Color::srgb(0.8, 1.0, 1.0)),
        LocalizedText::new("ui.nano_station.equip", "EQUIP"),
        Pickable::IGNORE,
    ));
    for index in 0..4 {
        let label = if index == 3 {
            LocalizedText::new("ui.nano_station.unequip", "UNEQUIP")
        } else {
            LocalizedText::new("ui.nano_station.slot", "NANO {slot}")
                .with_arg("slot", (index + 1).to_string())
        };
        parent
            .spawn((
                Button,
                StationControl(index),
                ZIndex(2),
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                sliced_image(
                    assets.image(UserEquipStaticAssetRole::ButtonNormal),
                    BorderRect {
                        min_inset: Vec2::new(6.0, 6.0),
                        max_inset: Vec2::new(6.0, 4.0),
                    },
                ),
            ))
            .with_children(|button| {
                button.spawn((
                    StationLabel,
                    Text::default(),
                    (
                        TextFont {
                            font: (assets.font.clone()).into(),
                            font_size: (14.0).into(),
                            ..default()
                        },
                        LineHeight::Px(11.3),
                    ),
                    TextColor(Color::srgb(0.8, 1.0, 1.0)),
                    TextLayout::new(Justify::Center, LineBreak::NoWrap),
                    label,
                    Pickable::IGNORE,
                ));
            });
    }
}

fn action_for(
    index: usize,
    entry: &UserEquipNanoGalleryEntryProjection,
    projection: &UserEquipNanoModeProjection,
) -> Option<UserEquipNanoStationAction> {
    if index < 3 && !entry.equipped {
        Some(UserEquipNanoStationAction::Equip {
            nano_id: entry.nano_id,
            slot: index,
        })
    } else if index == 3 && entry.equipped {
        projection
            .status
            .iter()
            .position(|slot| slot.nano_id == Some(entry.nano_id))
            .map(|slot| UserEquipNanoStationAction::Unequip {
                nano_id: entry.nano_id,
                slot,
            })
    } else {
        None
    }
}

pub(super) fn collect_actions(
    state: Res<UserEquipUiState>,
    modal: Res<UserEquipModalState>,
    viewer: Res<UserEquipNanoViewerState>,
    projection: Res<UserEquipNanoModeProjection>,
    controls: Query<(&Interaction, &StationControl), Changed<Interaction>>,
    mut outbox: ResMut<UserEquipUiOutbox>,
    mut audio: ResMut<UserEquipUiAudioOutbox>,
) {
    let Some(entry) = viewer
        .selected_visual_index()
        .and_then(|index| projection.gallery.get(index))
    else {
        return;
    };
    for (interaction, control) in &controls {
        if *interaction == Interaction::Pressed
            && let Some(action) = action_for(control.0, entry, &projection)
            && nano_station_action_allowed(&state, *modal, &projection, action)
        {
            outbox.push(UserEquipUiAction::NanoStation(action));
            audio.push(UserEquipUiAudioCue::ButtonSound);
            break;
        }
    }
}

pub(super) fn bind_controls(
    assets: Res<UserEquipUiRuntimeAssets>,
    state: Res<UserEquipUiState>,
    modal: Res<UserEquipModalState>,
    viewer: Res<UserEquipNanoViewerState>,
    projection: Res<UserEquipNanoModeProjection>,
    mut controls: Query<
        (
            &StationControl,
            &Interaction,
            &Children,
            &mut Node,
            &mut ImageNode,
        ),
        Without<StationEquipTitle>,
    >,
    mut labels: Query<&mut TextColor, With<StationLabel>>,
    mut title: Query<&mut Node, (With<StationEquipTitle>, Without<StationControl>)>,
) {
    let selected = viewer
        .selected_visual_index()
        .and_then(|index| projection.gallery.get(index))
        .filter(|entry| entry.owned && state.nano_station_npc().is_some() && state.is_active());
    for mut node in &mut title {
        node.display = if selected.is_some_and(|entry| !entry.equipped) {
            Display::Flex
        } else {
            Display::None
        };
    }
    for (control, interaction, children, mut node, mut image) in &mut controls {
        let action = selected.and_then(|entry| action_for(control.0, entry, &projection));
        node.display = if action.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if action.is_none() {
            continue;
        }
        let Some(entry) = selected else {
            continue;
        };
        let rect = if control.0 == 3 {
            UserEquipUiRect::new(220.0, 574.0, 120.0, 26.0)
        } else {
            UserEquipUiRect::new(25.0 + control.0 as f32 * 105.0, 574.0, 101.0, 26.0)
        };
        bind_rect(
            &mut node,
            user_equip_nano_popup_content_rect(rect, entry.equipped),
        );
        let enabled = action
            .is_some_and(|action| nano_station_action_allowed(&state, *modal, &projection, action));
        let hovered = enabled && *interaction == Interaction::Hovered;
        image.image = assets.0.image(if hovered {
            UserEquipStaticAssetRole::ButtonHover
        } else {
            UserEquipStaticAssetRole::ButtonNormal
        });
        image.color = Color::srgba(1.0, 1.0, 1.0, if enabled { 1.0 } else { 0.3 });
        for child in children.iter() {
            if let Ok(mut color) = labels.get_mut(child) {
                color.0 = if hovered {
                    Color::srgb(0.0, 0.2784314, 0.4784314)
                } else {
                    Color::srgba(0.8, 1.0, 1.0, if enabled { 1.0 } else { 0.3 })
                };
            }
        }
    }
}

#[cfg(test)]
mod tests;
