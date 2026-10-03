use super::*;
use crate::localization::{Language, Localization};
use crate::tutorial_mission_content::TutorialMissionContent;
use bevy::input::{ButtonState, keyboard::KeyboardInput};

#[derive(Resource)]
pub(crate) struct BankSearch {
    query: String,
    focused: bool,
    owner: Option<(i32, i32)>,
    visible: Vec<usize>,
}

#[cfg(test)]
mod tests;

impl Default for BankSearch {
    fn default() -> Self {
        Self {
            query: String::new(),
            focused: false,
            owner: None,
            visible: (0..BANK_SLOT_COUNT_0104).collect(),
        }
    }
}

impl BankSearch {
    pub(crate) fn content_height(&self) -> f32 {
        self.visible.len().div_ceil(6) as f32 * 67.0
    }
    pub(crate) fn maximum(&self) -> f32 {
        (self.content_height() - BANK_VIEWPORT_HEIGHT).max(0.0)
    }
    pub(super) fn visible_index(&self, slot: usize) -> Option<usize> {
        self.visible.iter().position(|s| *s == slot)
    }
}

#[derive(Component)]
pub(super) struct SearchInput;
#[derive(Component)]
pub(super) struct SearchText;

pub(super) fn spawn(parent: &mut ChildSpawnerCommands, assets: &BankUiAssets) {
    parent.spawn((
        BankUiRect::new(231.0, 0.0, 250.0, 32.0).node(),
        stretched_image(assets.image(BankStaticAssetRole::SearchField)),
        Pickable::IGNORE,
        bevy::ui::FocusPolicy::Pass,
    ));
    parent
        .spawn((
            SearchInput,
            Button,
            sliced_image(
                assets.image(BankStaticAssetRole::SearchInputBackground),
                BorderRect::all(3.0),
            ),
            Node {
                overflow: Overflow::clip(),
                padding: UiRect::new(px(3), px(3), px(1), px(3)),
                ..BankUiRect::new(295.0, 7.0, 177.0, 18.0).node()
            },
        ))
        .with_children(|field| {
            field.spawn((
                SearchText,
                Node {
                    flex_shrink: 0.0,
                    ..default()
                },
                Text::new("Search by item name..."),
                LocalizedText::new("ui.bank.search.placeholder", "Search by item name..."),
                BankUiTextStyle::SearchUpperLeft.font(assets.search_font.clone()),
                TextLayout::new(Justify::Left, LineBreak::NoWrap),
                TextColor(Color::srgb(0.9959677, 1.0, 1.0)),
                BankUiTextStyle::SearchUpperLeft,
                UiTransform::from_translation(Val2::px(0.0, BANK_TEXT_REPLACEMENT_Y_OFFSET)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
        });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn input(
    mut search: ResMut<BankSearch>,
    mut state: ResMut<BankUiState>,
    modal: Res<BankModalState>,
    projection: Res<BankModeProjection0104>,
    content: Option<Res<TutorialMissionContent>>,
    localization: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    messages: Option<ResMut<Messages<KeyboardInput>>>,
    keys: Option<ResMut<ButtonInput<KeyCode>>>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    windows: Query<&Window>,
    fields: Query<&Interaction, With<SearchInput>>,
    mut labels: Query<&mut LocalizedText, With<SearchText>>,
) {
    let owner = (state.phase != BankLifecyclePhase::Hidden)
        .then_some((projection.owner_pc_id, projection.npc_id));
    let reset = search.owner != owner;
    if reset {
        *search = BankSearch { owner, ..default() };
    }
    let enabled = state.input_capabilities(*modal).item_move;
    if !enabled || windows.iter().any(|w| !w.focused) {
        search.focused = false;
    } else if mouse
        .as_ref()
        .is_some_and(|m| m.just_pressed(MouseButton::Left))
    {
        search.focused = fields.iter().any(|i| *i == Interaction::Pressed);
    }
    let old_query = search.query.clone();
    if search.focused {
        if let Some(mut messages) = messages {
            for event in messages.drain() {
                if event.state != ButtonState::Pressed {
                    continue;
                }
                match event.key_code {
                    KeyCode::Escape | KeyCode::Enter | KeyCode::NumpadEnter => {
                        search.focused = false
                    }
                    KeyCode::Backspace => {
                        search.query.pop();
                    }
                    _ => {
                        if let Some(text) = event.text {
                            search
                                .query
                                .extend(text.chars().filter(|c| !c.is_control()));
                        }
                    }
                }
            }
        }
        if let Some(mut keys) = keys {
            keys.reset_all();
        }
    }
    let changed = reset
        || old_query != search.query
        || projection.is_changed()
        || content.as_ref().is_some_and(|c| c.is_changed())
        || localization.as_ref().is_some_and(|l| l.is_changed())
        || language.as_ref().is_some_and(|l| l.is_changed());
    if changed {
        let query = search.query.to_lowercase();
        search.visible = projection
            .bank
            .iter()
            .filter(|slot| {
                if query.is_empty() {
                    return true;
                }
                if slot.empty {
                    return false;
                }
                let Some(content) = content.as_ref() else {
                    return false;
                };
                let item = slot.item;
                let id = if (0..=3).contains(&item.item_type) && (item.option >> 16) as i16 > 0 {
                    (item.option >> 16) as i16
                } else {
                    item.item_id
                };
                let Some((text, _)) = content.gameplay_user_equip_item_text(item.item_type, id)
                else {
                    return false;
                };
                let name = match (localization.as_ref(), language.as_ref()) {
                    (Some(l), Some(language)) => l.text(language, &text),
                    _ => text.fallback,
                };
                name.to_lowercase().contains(&query)
            })
            .map(|slot| slot.slot_index)
            .collect();
    }
    let offset = state.bank_scroll_y.clamp(0.0, search.maximum());
    if state.bank_scroll_y != offset {
        state.bank_scroll_y = offset;
    }
    let label = if search.query.is_empty() && !search.focused {
        LocalizedText::new("ui.bank.search.placeholder", "Search by item name...")
    } else {
        LocalizedText::new("ui.bank.search.value", "{query}").with_arg("query", &search.query)
    };
    for mut text in &mut labels {
        if *text != label {
            *text = label.clone();
        }
    }
}
