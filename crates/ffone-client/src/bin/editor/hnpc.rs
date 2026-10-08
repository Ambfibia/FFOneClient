//! Native HNPC appearance drafts, published part variants and live runtime preview.
use super::*;
use bevy::color::LinearRgba;
use ffone_client::{
    hnpc_runtime::HnpcRuntimeAppearance,
    player_preview::{NativePlayerLook, NativePlayerPartKind, NativePlayerPartLook},
};
use std::io::Write;
#[cfg(test)]
#[path = "hnpc_tests.rs"]
mod tests;
#[path = "hnpc_view.rs"]
mod view;
#[path="hnpc_wardrobe.rs"]
mod wardrobe;

pub(super) struct HnpcEditorPlugin;
impl Plugin for HnpcEditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ffone_client::text_edit::TextEditPlugin>() {
            app.add_plugins(ffone_client::text_edit::TextEditPlugin);
        }
        app.init_resource::<HnpcEditor>().add_systems(
            Update,
            (buttons, preview, view::draw)
                .chain()
                .after(xdt::XdtInput)
                .before(spawn_selected_model)
                .before(LocalizationSet::Apply),
        );
    }
}
#[derive(Clone)]
struct Variant {
    value: Value,
    part: NativePlayerPartLook,
    profile: Option<ffone_client::tutorial_player_presentation::PlayerWeaponAnimationProfile>,
    gender: String,
    caption: String,
}
#[derive(Resource, Default)]
pub(super) struct HnpcEditor {
    open: bool,
    root: PathBuf,
    base: Value,
    draft: Value,
    palette: Value,
    original: Option<HnpcRuntimeCatalog>,
    variants: Vec<Variant>,
    source: usize,
    revision: u64,
    preview_value: Value,
    status: String,
    undo: Vec<Value>,
    redo: Vec<Value>,
    picker: Option<String>,
    page: usize,
    backup: Option<(usize, CatalogKind, EditorCatalogEntry)>,
    users: usize,
    owner_height: i32,
    owner_id: i64,
    filter: String,
    filter_edit: ffone_client::text_edit::TextEdit,
    filter_focused: bool,
}
impl HnpcEditor {
    pub(super) fn active(&self) -> bool {
        self.open
    }
    pub(super) fn open(
        &mut self,
        table_path: &std::path::Path,
        owner: &Value,
        catalog: &EditorCatalog,
    ) -> Result<(), String> {
        self.root = table_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .ok_or("Missing asset root")?
            .to_path_buf();
        self.owner_height = owner["m_iHeight"].as_i64().unwrap_or(200) as i32;
        self.owner_id = owner["m_iNpcNumber"].as_i64().unwrap_or(1);
        self.base = serde_json::from_slice(
            &fs::read(self.root.join("data/hnpc/catalog.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        self.palette = serde_json::from_slice(
            &fs::read(self.root.join("data/hnpc/palette.json")).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        let original = catalog
            .hnpc
            .as_ref()
            .ok_or("Missing HNPC runtime catalog")?;
        self.source = if owner["m_iHNpc"].as_i64().unwrap_or(0) != 0 {
            owner["m_iHNpcNum"].as_u64().unwrap_or(1) as usize
        } else {
            1
        };
        self.draft = self.base["appearances"]
            .get(self.source)
            .ok_or("Missing HNPC appearance")?
            .clone();
        self.users = catalog
            .entries
            .iter()
            .filter(|e| {
                e.hnpc_visual
                    .as_ref()
                    .is_some_and(|v| v.appearance_index == self.source)
            })
            .count();
        self.variants.clear();
        let mut unique = BTreeSet::new();
        for appearance in self.base["appearances"].as_array().into_iter().flatten() {
            let Some(index) = appearance["index"].as_u64() else {
                continue;
            };
            let Some(look) = original
                .authored_look(index as usize)
            else {
                continue;
            };
            for part in appearance["parts"].as_array().into_iter().flatten() {
                let Some(kind) = native_kind(part["kind"].as_str().unwrap_or("")) else {
                    continue;
                };
                let signature = format!("{}:{}", appearance["gender"], part);
                if !unique.insert(signature) {
                    continue;
                }
                let Some(native) = look.parts.iter().find(|p| p.kind == kind) else {
                    continue;
                };
                let name = catalog
                    .entries
                    .iter()
                    .find(|e| e.glb == native.glb)
                    .map(|e| e.display_name.clone())
                    .unwrap_or_else(|| part["trueName"].as_str().unwrap_or("?").replace('_', " "));
                let textures = part["textures"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(" / ");
                self.variants.push(Variant {
                    value: part.clone(),
                    part: native.clone(),
                    profile: look.weapon_animation_profile,
                    gender: appearance["gender"].as_str().unwrap_or("male").into(),
                    caption: format!("{name} · {textures}"),
                });
            }
        }
        self.add_player_wardrobe()?;
        self.original = Some(original.clone());
        self.undo.clear();
        self.redo.clear();
        self.picker = None;
        self.filter.clear();
        self.filter_focused = false;
        self.filter_edit = default();
        self.page = 0;
        self.status.clear();
        self.preview_value = Value::Null;
        self.revision = self.revision.wrapping_add(1).max(1);
        self.open = true;
        Ok(())
    }
    fn mutate(&mut self, next: Value) {
        if next == self.draft {
            return;
        }
        self.undo.push(self.draft.clone());
        self.redo.clear();
        self.draft = next;
        self.status.clear();
        self.revision += 1;
    }
    fn choices(&self, kind: &str) -> Vec<usize> {
        let needle = self.filter.to_lowercase();
        self.variants
            .iter()
            .enumerate()
            .filter(|(_, v)| {
                v.gender == self.draft["gender"].as_str().unwrap_or("male")
                    && v.value["kind"].as_str() == Some(kind)
            })
            .filter(|(_, v)| needle.is_empty() || v.caption.to_lowercase().contains(&needle))
            .map(|(i, _)| i)
            .collect()
    }
    fn choose(&mut self, index: usize) -> Result<(), String> {
        let v = self.variants.get(index).ok_or("Missing part variant")?;
        if v.gender != self.draft["gender"].as_str().unwrap_or("male") {
            return Err("Part gender does not match appearance".into());
        }
        let mut next = self.draft.clone();
        let parts = next["parts"]
            .as_array_mut()
            .ok_or("Missing appearance parts")?;
        if let Some(slot) = parts.iter().position(|p| p["kind"] == v.value["kind"]) {
            parts[slot] = v.value.clone();
        } else {
            parts.push(v.value.clone());
        }
        self.mutate(next);
        Ok(())
    }
    fn look(&self) -> Result<NativePlayerLook, String> {
        let original = self.original.as_ref().ok_or("Missing preview catalog")?;
        let gender = self.draft["gender"].as_str().ok_or("Missing gender")?;
        let mut look = original
            .appearance(self.source)
            .and_then(|a| a.look.as_ref())
            .cloned()
            .or_else(|| {
                self.base["appearances"]
                    .as_array()?
                    .iter()
                    .filter(|a| a["gender"].as_str() == Some(gender))
                    .find_map(|a| {
                        original
                            .appearance(a["index"].as_u64()? as usize)?
                            .look
                            .clone()
                    })
            })
            .ok_or("Empty appearance")?;
        if (look.gender == ffone_runtime_contracts::PlayerRigGender::Male) != (gender == "male") {
            look = self.base["appearances"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|a| a["gender"].as_str() == Some(gender))
                .find_map(|a| {
                    original
                        .appearance(a["index"].as_u64()? as usize)?
                        .look
                        .clone()
                })
                .ok_or("Missing gender rig")?;
        }
        look.identity = format!("Editor HNPC {} draft", self.source);
        look.parts.clear();
        look.weapon_animation_profile = None;
        for part in self.draft["parts"].as_array().ok_or("Missing parts")? {
            let v = self
                .variants
                .iter()
                .find(|v| v.gender == gender && v.value == *part)
                .ok_or("Unsupported part variant")?;
            look.parts.push(v.part.clone());
            if v.part.kind == NativePlayerPartKind::Weapon {
                look.weapon_animation_profile = v.profile;
            }
        }
        look.height_selector = self.draft["height"].as_i64().ok_or("Missing height")? as i8;
        let hat_type = self.draft["parts"].as_array().into_iter().flatten().find(|p| p["kind"] == "hat").and_then(|p| p["equipType"].as_u64()).map(|n| n as u8);
        original.apply_equipment_visibility(&mut look, hat_type)?;
        look.body_selector = self.draft["shape"].as_i64().ok_or("Missing shape")? as i8;
        for (field, palette, color) in [
            ("skinColor", "skin", &mut look.skin_color),
            ("hairColor", "hair", &mut look.hair_color),
        ] {
            let index = self.draft[field].as_i64().ok_or("Missing palette color")?;
            let Some(rgba) = usize::try_from(index)
                .ok()
                .and_then(|index| self.palette[palette].get(index))
                .and_then(Value::as_array)
            else {
                *color = LinearRgba::WHITE;
                continue;
            };
            *color = LinearRgba::new(
                rgba[0].as_f64().unwrap_or(0.) as f32,
                rgba[1].as_f64().unwrap_or(0.) as f32,
                rgba[2].as_f64().unwrap_or(0.) as f32,
                1.,
            );
        }
        look.validate()?;
        Ok(look)
    }
    fn save(&self, shared: bool) -> Result<(usize, HnpcRuntimeCatalog), String> {
        let path = self.root.join("data/hnpc/catalog.json");
        let disk: Value = serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        self.look()?;
        let (index, next) = publish_document(&self.base, &self.draft, &disk, self.source, shared)?;
        let locator = AssetLocator::open(&self.root).map_err(|e| e.to_string())?;
        let runtime = HnpcRuntimeCatalog::from_json(
            &locator,
            self.original
                .as_ref()
                .ok_or("Missing rig catalog")?
                .rig_catalog(),
            next.clone(),
        )?;
        // Publish the validated native catalog atomically, retaining external edits to other appearances.
        let bytes = serde_json::to_vec_pretty(&next).map_err(|e| e.to_string())?;
        let mut file =
            tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(|e| e.to_string())?;
        file.write_all(&bytes)
            .and_then(|_| file.write_all(b"\n"))
            .and_then(|_| file.flush())
            .map_err(|e| e.to_string())?;
        file.as_file().sync_all().map_err(|e| e.to_string())?;
        let latest: Value = serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        if latest != disk {
            return Err("HNPC catalog changed during saving".into());
        }
        file.persist(&path).map_err(|e| e.to_string())?;
        Ok((index, runtime))
    }
}
fn publish_document(
    base: &Value,
    draft: &Value,
    disk: &Value,
    source: usize,
    shared: bool,
) -> Result<(usize, Value), String> {
    let mut next = disk.clone();
    let rows = next["appearances"]
        .as_array_mut()
        .ok_or("Missing appearances")?;
    let index = if shared {
        if disk["appearances"].get(source) != base["appearances"].get(source) {
            return Err("Appearance changed on disk; reopen it before saving".into());
        }
        if source >= rows.len() {
            return Err("Missing source appearance".into());
        }
        source
    } else {
        rows.len()
    };
    let mut appearance = draft.clone();
    appearance["index"] = Value::from(index);
    if shared {
        rows[index] = appearance;
    } else {
        rows.push(appearance);
    }
    Ok((index, next))
}
fn native_kind(kind: &str) -> Option<NativePlayerPartKind> {
    Some(match kind {
        "face" => NativePlayerPartKind::Face,
        "hair" => NativePlayerPartKind::Hair,
        "shirt" => NativePlayerPartKind::Shirt,
        "pants" => NativePlayerPartKind::Pants,
        "shoes" => NativePlayerPartKind::Shoes,
        "hat" => NativePlayerPartKind::Hat,
        "glasses" => NativePlayerPartKind::Glasses,
        "back" => NativePlayerPartKind::Back,
        "rightWeapon" => NativePlayerPartKind::Weapon,
        _ => return None,
    })
}
#[derive(Component)]
struct Root;
#[derive(Component)]
struct Scroll;
#[derive(Component)]
struct SearchText;
#[derive(Component, Clone)]
enum Action {
    Search,
    Back,
    Save(bool),
    Select(String, i64),
    Gender(String),
    Picker(String),
    Part(usize),
    Remove(String),
    Page(bool),
    Undo,
    Redo,
}
fn restore(
    e: &mut HnpcEditor,
    catalog: &mut EditorCatalog,
    state: &mut EditorState,
    preview: &mut ModelPreview,
) {
    if let Some((index, kind, entry)) = e.backup.take() {
        catalog.entries[index] = entry;
        state.selected = index;
        state.kind = kind;
    }
    if let Some(original) = &e.original {
        catalog.hnpc = Some(original.clone());
    }
    preview.current_index = None;
    preview.preserve_camera = false;
    e.open = false;
    e.revision += 1;
}
fn buttons(
    mut e: ResMut<HnpcEditor>,
    mut xdt: ResMut<xdt::XdtEditor>,
    mut state: ResMut<EditorState>,
    mut catalog: ResMut<EditorCatalog>,
    mut model: ResMut<ModelPreview>,
    interactions: Query<(&Interaction, &Action), Changed<Interaction>>,
    tabs: Query<(&Interaction, &EditorAction), Changed<Interaction>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut events: MessageReader<KeyboardInput>,
    search_texts: Query<
        (
            &bevy::text::ComputedTextBlock,
            &ComputedNode,
            &UiGlobalTransform,
            &ffone_client::text_edit::EditVisual,
        ),
        With<SearchText>,
    >,
    window: Single<&Window>,
    mut wheels: MessageReader<MouseWheel>,
    mut scrolls: Query<(&RelativeCursorPosition, &ComputedNode, &mut ScrollPosition), With<Scroll>>,
) {
    if !e.open {
        wheels.clear();
        events.clear();
        return;
    }
    if state.xdt_open
        || state.strings_open
        || tabs
            .iter()
            .any(|(i, a)| *i == Interaction::Pressed && matches!(a, EditorAction::Tab(_)))
    {
        let destination = (state.selected, state.kind);
        restore(&mut e, &mut catalog, &mut state, &mut model);
        if !state.xdt_open && !state.strings_open {
            state.selected = destination.0;
            state.kind = destination.1;
        }
        return;
    }
    for (cursor, node, mut pos) in &mut scrolls {
        if cursor.cursor_over() {
            for wheel in wheels.read() {
                let amount = match wheel.unit {
                    MouseScrollUnit::Line => wheel.y * 32.,
                    MouseScrollUnit::Pixel => wheel.y,
                };
                let max =
                    (node.content_size().y - node.size().y).max(0.) * node.inverse_scale_factor();
                pos.y = (pos.y - amount).clamp(0., max);
            }
        }
    }
    let mut actions: Vec<_> = interactions
        .iter()
        .filter(|(i, _)| **i == Interaction::Pressed)
        .map(|(_, a)| a.clone())
        .collect();
    if keys.just_pressed(KeyCode::Escape) && !e.filter_focused {
        actions.push(Action::Back);
    }
    if !e.filter_focused && keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]) {
        if keys.just_pressed(KeyCode::KeyZ) {
            actions.push(
                if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
                    Action::Redo
                } else {
                    Action::Undo
                },
            );
        } else if keys.just_pressed(KeyCode::KeyY) {
            actions.push(Action::Redo);
        }
    }
    if e.filter_focused {
        let (control, shift) = ffone_client::text_edit::modifiers(Some(&keys));
        for event in events
            .read()
            .filter(|event| event.state == ButtonState::Pressed)
        {
            if event.logical_key == Key::Escape || event.logical_key == Key::Enter {
                e.filter_focused = false;
                e.revision += 1;
                continue;
            }
            let HnpcEditor {
                filter,
                filter_edit,
                ..
            } = &mut *e;
            if control && matches!(event.key_code, KeyCode::KeyC | KeyCode::KeyX) {
                let selected: String = filter
                    .chars()
                    .skip(filter_edit.range().start)
                    .take(filter_edit.range().len())
                    .collect();
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    let _ = clipboard.set_text(selected);
                }
                if event.key_code == KeyCode::KeyX && !filter_edit.range().is_empty() {
                    filter_edit.key(filter, KeyCode::Delete, false, false);
                }
            } else if control && event.key_code == KeyCode::KeyV {
                if let Ok(text) = arboard::Clipboard::new().and_then(|mut c| c.get_text()) {
                    filter_edit.insert(filter, &text, 100, false);
                }
            } else if !filter_edit.key(filter, event.key_code, control, shift) && !control {
                if let Some(text) = &event.text {
                    filter_edit.insert(filter, text, 100, false);
                }
            }
            e.page = 0;
            e.revision += 1;
        }
    } else {
        events.clear();
    }
    for action in actions {
        if !matches!(action, Action::Search) {
            e.filter_focused = false;
        }
        let result: Result<(), String> = (|| {
            match action {
                Action::Search => {
                    e.filter_focused = true;
                    let text = e.filter.clone();
                    e.filter_edit.end(&text);
                    if let Some(cursor) = window.cursor_position() {
                        for (block, node, transform, visual) in &search_texts {
                            if let Some(position) = ffone_client::text_edit::hit_position(
                                block,
                                node,
                                transform,
                                &e.filter,
                                visual,
                                cursor * window.resolution.scale_factor(),
                                visual.inset,
                            ) {
                                e.filter_edit.place(
                                    position,
                                    keys.pressed(KeyCode::ShiftLeft)
                                        || keys.pressed(KeyCode::ShiftRight),
                                );
                            }
                        }
                    }
                    e.revision += 1;
                }
                Action::Back => {
                    restore(&mut e, &mut catalog, &mut state, &mut model);
                    state.xdt_open = true;
                }
                Action::Save(shared) => {
                    let (index, runtime) = e.save(shared)?;
                    xdt.apply_hnpc(index)?;
                    restore(&mut e, &mut catalog, &mut state, &mut model);
                    catalog.hnpc = Some(runtime);
                    state.xdt_open = true;
                }
                Action::Select(field, id) => {
                    let mut next = e.draft.clone();
                    next[&field] = Value::from(id);
                    e.mutate(next);
                }
                Action::Gender(gender) => {
                    if let Some(template) = e.base["appearances"]
                        .as_array()
                        .and_then(|rows| {
                            rows.iter().find(|a| {
                                a["gender"].as_str() == Some(&gender)
                                    && a["parts"].as_array().is_some_and(|p| !p.is_empty())
                            })
                        })
                        .cloned()
                    {
                        let mut next = e.draft.clone();
                        next["gender"] = template["gender"].clone();
                        let old = next["legacyType"].as_i64().unwrap_or(1);
                        next["legacyType"] = Value::from(if gender == "female" {
                            old & !1
                        } else {
                            old | 1
                        });
                        next["parts"] = template["parts"].clone();
                        e.mutate(next);
                        e.picker = None;
                    }
                }
                Action::Picker(kind) => {
                    e.picker = Some(kind);
                    e.filter.clear();
                    e.filter_focused = false;
                    e.filter_edit = default();
                    e.page = 0;
                    e.revision += 1;
                }
                Action::Part(index) => e.choose(index)?,
                Action::Remove(kind) => {
                    let mut next = e.draft.clone();
                    next["parts"]
                        .as_array_mut()
                        .unwrap()
                        .retain(|p| p["kind"].as_str() != Some(&kind));
                    e.mutate(next);
                }
                Action::Page(down) => {
                    e.page = if down {
                        e.page + 1
                    } else {
                        e.page.saturating_sub(1)
                    };
                    e.revision += 1;
                }
                Action::Undo | Action::Redo => {
                    let redo = matches!(action, Action::Redo);
                    let value = if redo { e.redo.pop() } else { e.undo.pop() };
                    if let Some(value) = value {
                        let previous = e.draft.clone();
                        if redo {
                            e.undo.push(previous)
                        } else {
                            e.redo.push(previous)
                        }
                        e.draft = value;
                        e.revision += 1;
                    }
                }
            }
            Ok(())
        })();
        if let Err(error) = result {
            e.status = error;
            e.revision += 1;
        }
    }
}
fn preview(
    mut e: ResMut<HnpcEditor>,
    mut state: ResMut<EditorState>,
    mut catalog: ResMut<EditorCatalog>,
    mut model: ResMut<ModelPreview>,
) {
    if !e.open || (e.backup.is_some() && e.preview_value == e.draft) {
        return;
    }
    e.preview_value = e.draft.clone();
    let first = e.backup.is_none();
    if e.backup.is_none() {
        e.backup = Some((
            state.selected,
            state.kind,
            catalog.entries[state.selected].clone(),
        ));
    }
    let index = e.backup.as_ref().unwrap().0;
    match e.look().and_then(|look| {
        e.original.as_ref().unwrap().with_appearance_override(
            e.source,
            HnpcRuntimeAppearance {
                legacy_type: e.draft["legacyType"].as_i64().unwrap_or(1) as i32,
                look: Some(look),
            },
        )
    }) {
        Ok(runtime) => {
            catalog.hnpc = Some(runtime);
            state.kind = CatalogKind::Npc;
            state.selected = index;
            let entry = &mut catalog.entries[index];
            entry.kind = CatalogKind::Npc;
            entry.npc_visual = None;
            entry.glb.clear();
            entry.network_id = Some(e.owner_id);
            entry.semantic_id = format!("hnpc/{}", e.source);
            entry.logical_name = format!("HNPC appearance {}", e.source);
            entry.height_server_units = Some(e.owner_height as i64);
            entry.hnpc_visual = Some(NetworkHnpcVisualDefinition0104 {
                npc_type: entry.network_id.unwrap_or(1) as i32,
                appearance_index: e.source,
                height_server_units: e.owner_height,
                walk_animation_speed: 1.,
                run_animation_speed: 1.,
                idle_clips: None,
            });
            entry.display_name = "HNPC".into();
            model.preserve_camera = !first;
            model.current_index = None;
            e.preview_value = e.draft.clone();
        }
        Err(error) => e.status = error,
    }
}
