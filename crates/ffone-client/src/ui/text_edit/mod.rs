//! Native single-line editing. Positions count Unicode scalars, never UTF-8 bytes.
use bevy::{prelude::*, text::ComputedTextBlock};

#[derive(Clone, Debug, PartialEq)]
pub struct TextEdit {
    pub cursor: usize,
    pub anchor: usize,
}

impl Default for TextEdit {
    fn default() -> Self {
        Self {
            cursor: usize::MAX,
            anchor: usize::MAX,
        }
    }
}

impl TextEdit {
    pub fn clamp(&mut self, text: &str) {
        let len = text.chars().count();
        self.cursor = self.cursor.min(len);
        self.anchor = self.anchor.min(len);
    }

    pub fn end(&mut self, text: &str) {
        self.place(text.chars().count(), false);
    }

    pub fn place(&mut self, position: usize, extend: bool) {
        self.cursor = position;
        if !extend {
            self.anchor = position;
        }
    }

    pub fn range(&self) -> std::ops::Range<usize> {
        self.cursor.min(self.anchor)..self.cursor.max(self.anchor)
    }

    pub fn key(&mut self, text: &mut String, key: KeyCode, control: bool, shift: bool) -> bool {
        self.clamp(text);
        let len = text.chars().count();
        match key {
            KeyCode::KeyA if control => {
                self.anchor = 0;
                self.cursor = len;
            }
            KeyCode::ArrowLeft | KeyCode::ArrowRight => {
                let left = key == KeyCode::ArrowLeft;
                let selected = self.range();
                let position = if !shift && !selected.is_empty() {
                    if left { selected.start } else { selected.end }
                } else if left {
                    self.cursor.saturating_sub(1)
                } else {
                    (self.cursor + 1).min(len)
                };
                self.place(position, shift);
            }
            KeyCode::Home => self.place(0, shift),
            KeyCode::End => self.place(len, shift),
            KeyCode::Backspace | KeyCode::Delete => {
                if self.range().is_empty() {
                    if key == KeyCode::Backspace {
                        self.anchor = self.cursor.saturating_sub(1);
                    } else {
                        self.anchor = (self.cursor + 1).min(len);
                    }
                }
                self.delete_selection(text);
            }
            _ => return false,
        }
        true
    }

    fn delete_selection(&mut self, text: &mut String) {
        self.clamp(text);
        let range = self.range();
        let start = byte_at(text, range.start);
        let end = byte_at(text, range.end);
        text.replace_range(start..end, "");
        self.place(range.start, false);
    }

    pub fn insert(&mut self, text: &mut String, produced: &str, limit: usize, utf16: bool) {
        let filtered: String = produced.chars().filter(|c| !c.is_control()).collect();
        if filtered.is_empty() {
            return;
        }
        self.delete_selection(text);
        let mut used = if utf16 {
            text.encode_utf16().count()
        } else {
            text.chars().count()
        };
        let mut inserted = String::new();
        for ch in filtered.chars() {
            let cost = if utf16 { ch.len_utf16() } else { 1 };
            if used + cost > limit {
                break;
            }
            used += cost;
            inserted.push(ch);
        }
        text.insert_str(byte_at(text, self.cursor), &inserted);
        self.place(self.cursor + inserted.chars().count(), false);
    }
}

fn byte_at(text: &str, position: usize) -> usize {
    text.char_indices()
        .nth(position)
        .map_or(text.len(), |(byte, _)| byte)
}

pub fn modifiers(keys: Option<&ButtonInput<KeyCode>>) -> (bool, bool) {
    let Some(keys) = keys else {
        return (false, false);
    };
    (
        keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]),
        keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]),
    )
}

/// Scalar boundaries from the actual shaped advances (including spaces/ligatures).
pub fn advances(block: &ComputedTextBlock, text: &str, scale: f32) -> Vec<f32> {
    layout_advances(block.buffer(), text, scale)
}

fn layout_advances<B: parley::Brush>(
    layout: &parley::Layout<B>,
    text: &str,
    scale: f32,
) -> Vec<f32> {
    let mut positions = vec![0.0; text.chars().count() + 1];
    for line in layout.lines() {
        let mut x = line.metrics().offset;
        for run in line.runs() {
            for cluster in run.visual_clusters() {
                let range = cluster.text_range();
                let (Some(before), Some(through)) =
                    (text.get(..range.start), text.get(..range.end))
                else {
                    continue;
                };
                let start = before.chars().count();
                let end = through.chars().count();
                for index in start..=end {
                    let fraction = (index - start) as f32 / (end - start).max(1) as f32;
                    let fraction = if cluster.is_rtl() {
                        1.0 - fraction
                    } else {
                        fraction
                    };
                    positions[index] = (x + cluster.advance() * fraction) / scale.max(0.001);
                }
                x += cluster.advance();
            }
        }
    }
    positions
}

pub fn nearest(positions: &[f32], x: f32) -> usize {
    positions
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| (*a - x).abs().total_cmp(&(*b - x).abs()))
        .map_or(0, |(index, _)| index)
}

pub fn hit_position(
    block: &ComputedTextBlock,
    computed: &ComputedNode,
    transform: &UiGlobalTransform,
    text: &str,
    visual: &EditVisual,
    physical_cursor: Vec2,
    inset: f32,
) -> Option<usize> {
    if visual.shaped_text != text || block.needs_rerender(false, false) {
        return None;
    }
    let inverse = transform.try_inverse()?;
    let x = (inverse.transform_point2(physical_cursor).x + computed.size().x / 2.0)
        * computed.inverse_scale_factor()
        - inset;
    Some(nearest(
        &advances(block, text, computed.inverse_scale_factor().recip()),
        x,
    ))
}

#[derive(Component, Default)]
pub struct EditVisual {
    pub edit: TextEdit,
    pub active: bool,
    pub inset: f32,
    pub scroll: f32,
    /// Exact text associated with the last completed Parley layout.
    pub shaped_text: String,
}

#[derive(Component)]
pub struct EditDecoration(pub bool);

pub fn spawn_decorations(parent: &mut ChildSpawnerCommands) {
    for caret in [false, true] {
        parent.spawn((
            EditDecoration(caret),
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                ..default()
            },
            BackgroundColor(if caret {
                Color::WHITE
            } else {
                Color::srgba(0.2, 0.5, 1.0, 0.45)
            }),
            Pickable::IGNORE,
        ));
    }
}

pub struct TextEditPlugin;
impl Plugin for TextEditPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            update_visuals.after(crate::localization::LocalizationSet::Apply),
        )
        .add_systems(
            PostUpdate,
            capture_shaped_text.after(bevy::ui::widget::text_system),
        );
    }
}

fn capture_shaped_text(
    mut texts: Query<(&Text, &ComputedTextBlock, &mut EditVisual), Changed<ComputedTextBlock>>,
) {
    for (text, block, mut visual) in &mut texts {
        if !block.needs_rerender(false, false) && visual.shaped_text != text.0 {
            visual.shaped_text.clone_from(&text.0);
        }
    }
}

fn scroll_to_cursor(previous: f32, cursor: f32, content: f32, available: f32) -> f32 {
    let maximum = (content - available).max(0.0);
    previous
        .min(maximum)
        .max(cursor - available)
        .min(cursor)
        .max(0.0)
}

fn update_visuals(
    mut texts: Query<
        (
            &Text,
            &ComputedTextBlock,
            &ComputedNode,
            &ChildOf,
            &Children,
            &mut EditVisual,
            &mut Node,
        ),
        Without<EditDecoration>,
    >,
    parents: Query<&ComputedNode>,
    mut decorations: Query<(&EditDecoration, &mut Node), Without<EditVisual>>,
) {
    for (text, block, computed, parent, children, mut visual, mut node) in &mut texts {
        if !visual.active {
            if visual.scroll != 0.0 {
                visual.scroll = 0.0;
            }
            if node.left != px(0) {
                node.left = px(0);
            }
            for child in children {
                if let Ok((_, mut decoration)) = decorations.get_mut(*child)
                    && decoration.display != Display::None
                {
                    decoration.display = Display::None;
                }
            }
            continue;
        }
        // Localization has updated Text, but layout may still describe the previous frame.
        if visual.shaped_text != text.0 || block.needs_rerender(false, false) {
            continue;
        }
        let scale = computed.inverse_scale_factor().recip();
        let positions = advances(block, &text.0, scale);
        let cursor = visual.edit.cursor.min(positions.len() - 1);
        let anchor = visual.edit.anchor.min(positions.len() - 1);
        let width = parents.get(parent.parent()).map_or(0.0, |n| {
            (n.size().x - n.content_inset().min_inset.x - n.content_inset().max_inset.x)
                * n.inverse_scale_factor()
        });
        let available = (width - visual.inset * 2.0 - 2.0).max(1.0);
        let x = positions[cursor];
        let scroll = if visual.active {
            scroll_to_cursor(
                visual.scroll,
                x,
                positions.last().copied().unwrap_or(0.0),
                available,
            )
        } else {
            0.0
        };
        if visual.scroll != scroll {
            visual.scroll = scroll;
        }
        if node.left != px(-scroll) {
            node.left = px(-scroll);
        }
        for child in children {
            let Ok((decoration, mut decoration_node)) = decorations.get_mut(*child) else {
                continue;
            };
            let visible = visual.active && (decoration.0 || cursor != anchor);
            let mut next = decoration_node.clone();
            next.display = if visible {
                Display::Flex
            } else {
                Display::None
            };
            next.left = px(visual.inset
                + if decoration.0 {
                    x
                } else {
                    x.min(positions[anchor])
                });
            next.top = px(visual.inset);
            next.width = px(if decoration.0 {
                1.0
            } else {
                (x - positions[anchor]).abs()
            });
            next.height = px((computed.size().y * computed.inverse_scale_factor()
                - visual.inset * 2.0)
                .max(12.0));
            if *decoration_node != next {
                *decoration_node = next;
            }
        }
    }
}

#[cfg(test)]
mod tests;
