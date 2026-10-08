//! Type selection is separate from placement selection; one click previews, two apply.
use super::*;

#[derive(Clone)]
pub(super) enum Choice {
    Actor {
        id: i64,
        catalog: usize,
        label: String,
    },
    Group {
        id: i64,
        row: Value,
        catalog: Option<usize>,
        label: String,
    },
    Shiny {
        id: i64,
        model: Option<String>,
        label: String,
    },
    Model(objects::Entry),
}
impl Choice {
    pub(super) fn label(&self) -> String {
        match self {
            Self::Actor { label, .. } | Self::Group { label, .. } | Self::Shiny { label, .. } => {
                label.clone()
            }
            Self::Model(e) => format!("{} · {}", e.asset, e.name),
        }
    }
    pub(super) fn model(&self, catalog: &EditorCatalog) -> Option<String> {
        match self {
            Self::Actor { catalog: i, .. } => Some(catalog.entries[*i].glb.clone()),
            Self::Group { catalog: i, .. } => i.map(|i| catalog.entries[i].glb.clone()),
            Self::Shiny { model, .. } => model.clone(),
            Self::Model(e) => e.model.clone(),
        }
    }
    pub(super) fn entry(&self) -> Option<usize> {
        match self {
            Self::Actor { catalog, .. } => Some(*catalog),
            Self::Group { catalog, .. } => *catalog,
            _ => None,
        }
    }
}
pub(super) struct Picker {
    pub kind: usize,
    pub target: Option<String>,
    pub choices: Vec<Choice>,
    pub selected: Option<usize>,
    pub filter: String,
    pub page: usize,
    last_click: Option<(usize, Instant)>,
    pub grab: Option<Vec2>,
    pub dragging: bool,
}
impl Picker {
    pub(super) fn filtered(&self) -> Vec<usize> {
        let needle = self.filter.trim().to_lowercase();
        self.choices
            .iter()
            .enumerate()
            .filter(|(_, c)| needle.is_empty() || c.label().to_lowercase().contains(&needle))
            .map(|(i, _)| i)
            .collect()
    }
}
impl WorldEditor {
    pub(super) fn open_type_picker(
        &mut self,
        selected: bool,
        catalog: &EditorCatalog,
    ) -> Result<(), String> {
        let target = if selected {
            Some(self.selected().ok_or("Select an entity")?.key.clone())
        } else {
            None
        };
        let kind = if self.terrain_tool==Some(4) && !selected {4} else if selected {
            self.selected().unwrap().kind
        } else {
            self.placement_kind
        };
        let mob_types: BTreeSet<_> = self
            .entities
            .iter()
            .filter(|p| p.kind == 1 || p.kind == 2)
            .map(|p| p.type_id)
            .collect();
        let mut choices = vec![];
        match kind {
            0 | 1 => {
                for (i, entry) in catalog
                    .entries
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.kind == CatalogKind::Npc)
                {
                    let Some(id) = entry.network_id else {
                        continue;
                    };
                    let mob = mob_types.contains(&id)
                        || entry
                            .npc_visual
                            .as_ref()
                            .is_some_and(|v| v.category == "mobs" || v.category == "fusions");
                    if kind == 1 && !mob {
                        continue;
                    }
                    if kind == 0 && mob {
                        continue;
                    }
                    choices.push(Choice::Actor {
                        id,
                        catalog: i,
                        label: format!("{id} · {}", entry.display_name),
                    });
                }
            }
            2 => {
                for p in self.entities.iter().filter(|p| p.kind == 2) {
                    let Some(row) = self.sources[p.source].draft.pointer(&p.pointer) else {
                        continue;
                    };
                    let index = catalog.entries.iter().position(|e| {
                        e.kind == CatalogKind::Npc && e.network_id == Some(p.type_id)
                    });
                    choices.push(Choice::Group {
                        id: p.type_id,
                        row: row.clone(),
                        catalog: index,
                        label: format!(
                            "{} · {} · {}",
                            p.key,
                            self.name(p, catalog),
                            row["aFollowers"].as_array().map_or(0, Vec::len)
                        ),
                    });
                }
            }
            3 => {
                if let Some(source) = self.sources.iter().find(|s| s.path.ends_with("eggs.json")) {
                    for row in source.draft["EggTypes"].as_array().into_iter().flatten() {
                        let Some(id) = row["Id"].as_i64() else {
                            continue;
                        };
                        choices.push(Choice::Shiny {
                            id,
                            model: catalog.shiny_models.get(&id).cloned(),
                            label: format!("{id} · Effect {}", row["EffectId"]),
                        });
                    }
                }
            }
            4 => {}
            _ => {}
        }
        self.type_picker = Some(Picker {
            kind,
            target,
            choices,
            selected: None,
            filter: String::new(),
            page: 0,
            last_click: None,
            grab: None,
            dragging: false,
        });
        if kind == 4 {
            self.refresh_model_picker();
        }
        self.focus = None;
        self.revision += 1;
        Ok(())
    }
    pub(super) fn refresh_model_picker(&mut self) {
        let Some(picker) = self.type_picker.as_mut().filter(|p| p.kind == 4) else {
            return;
        };
        let mut seen = BTreeSet::new();
        picker.choices = self
            .object_index
            .iter()
            .filter(|e| e.model.is_some() && seen.insert(e.asset.clone()))
            .cloned()
            .map(Choice::Model)
            .collect();
        if picker.choices.is_empty() {
            for p in self.entities.iter().filter(|p| p.kind == 4) {
                let row = &self.sources[p.source].draft.pointer(&p.pointer).unwrap();
                let asset = row["object"].as_str().unwrap_or_default().to_owned();
                if !seen.insert(asset.clone()) {
                    continue;
                }
                let node = row["sourceNode"].as_str().unwrap_or_default().to_owned();
                let Some(tile) = self.maps.values().find(|t| t.objects == p.source) else {
                    continue;
                };
                let scene = &self.sources[tile.scene].draft;
                let model = scene["visuals"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|v| {
                        v["name"]
                            .as_str()
                            .is_some_and(|n| map::matches_node(n, &node))
                    })
                    .and_then(|v| {
                        scene["models"]
                            .as_array()?
                            .iter()
                            .find(|m| m["id"] == v["model"])
                    })
                    .and_then(|m| m["path"].as_str())
                    .map(str::to_owned);
                picker.choices.push(Choice::Model(objects::Entry {
                    tile: tile.id.clone(),
                    node,
                    name: p.object_name.clone().unwrap_or_default(),
                    position: p.position,
                    asset,
                    model,
                }));
            }
        }
        picker.selected = None;
        picker.last_click = None;
    }
    pub(super) fn confirm_type(&mut self) -> Result<(), String> {
        let picker = self.type_picker.as_ref().ok_or("Open type picker")?;
        let choice = picker
            .selected
            .and_then(|i| picker.choices.get(i))
            .ok_or("Select a type")?
            .clone();
        let target = picker.target.clone();
        let kind = picker.kind;
        if let Choice::Model(entry) = choice {
            let snapshot = if let Some(p) = self.entities.iter().find(|p| {
                p.kind == 4
                    && p.key.starts_with(&format!("{}/", entry.tile))
                    && self.sources[p.source]
                        .draft
                        .pointer(&p.pointer)
                        .is_some_and(|r| r["sourceNode"] == entry.node)
            }) {
                self.snapshot(p)?
            } else {
                model_templates::snapshot(&self.root, &entry)?
            };
            if let Some(key) = target {
                let p = self
                    .entities
                    .iter()
                    .find(|p| p.key == key)
                    .ok_or("Selected object changed")?
                    .clone();
                self.replace_object_model(&p, snapshot)?;
            } else {
                if self.terrain_tool==Some(4) {
                    let mut snapshot=snapshot;snapshot.parts.insert("colliders".into(),Vec::new());self.grass_template=Some(snapshot);
                } else {
                    self.object_template = Some(snapshot);
                    self.placement_kind = 4;
                    self.edit_objects = true;
                }
            }
        } else {
            let (id, group) = match choice {
                Choice::Actor { id, .. } | Choice::Shiny { id, .. } => (id, None),
                Choice::Group { id, row, .. } => (id, Some(row)),
                _ => unreachable!(),
            };
            if let Some(key) = target {
                let p = self
                    .entities
                    .iter()
                    .find(|p| p.key == key)
                    .ok_or("Selected entity changed")?;
                let before = self.sources[p.source]
                    .draft
                    .pointer(&p.pointer)
                    .ok_or("Missing entity")?
                    .clone();
                let mut after = before.clone();
                after[if kind == 3 { "iType" } else { "iNPCType" }] = Value::from(id);
                if let Some(group) = group {
                    after["aFollowers"] = group["aFollowers"].clone();
                }
                self.commit(vec![model::Patch {
                    source: p.source,
                    pointer: p.pointer.clone(),
                    before: Some(before),
                    after: Some(after),
                }])?;
            } else {
                self.type_id = id;
                self.placement_kind = kind;
                self.group_template = group;
            }
        }
        self.type_picker = None;
        self.focus = None;
        self.revision += 1;
        Ok(())
    }
}
pub(super) fn action(
    e: &mut WorldEditor,
    action: Action,
    catalog: &EditorCatalog,
) -> Result<(), String> {
    match action {
        Action::ChooseType(selected) => e.open_type_picker(selected, catalog),
        Action::PickerConfirm => e.confirm_type(),
        Action::PickerCancel => {
            e.type_picker = None;
            e.focus = None;
            e.revision += 1;
            Ok(())
        }
        Action::PickerPage(next) => {
            let p = e.type_picker.as_mut().ok_or("Open type picker")?;
            let pages = p.filtered().len().div_ceil(8).max(1);
            p.page = if next {
                (p.page + 1).min(pages - 1)
            } else {
                p.page.saturating_sub(1)
            };
            e.revision += 1;
            Ok(())
        }
        Action::PickerSelect(index) => {
            let p = e.type_picker.as_mut().ok_or("Open type picker")?;
            let double = p
                .last_click
                .is_some_and(|(old, at)| old == index && at.elapsed().as_millis() < 400);
            p.selected = Some(index);
            p.last_click = Some((index, Instant::now()));
            e.focus = None;
            e.revision += 1;
            if double { e.confirm_type() } else { Ok(()) }
        }
        _ => routes::action(e, action),
    }
}
pub(super) fn draw(p: &mut ChildSpawnerCommands, f: &EditorFonts, e: &WorldEditor) {
    use view::{MUTED, WHITE, button, field, label, row, value};
    let Some(picker) = &e.type_picker else {
        return;
    };
    if picker.dragging {return;}
    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: percent(8),
            right: percent(8),
            top: px(90),
            bottom: px(25),
            padding: UiRect::all(px(16)),
            flex_direction: FlexDirection::Column,
            row_gap: px(6),
            ..default()
        },
        GlobalZIndex(30),
        BackgroundColor(Color::srgb(0.025, 0.055, 0.10)),
    ))
    .with_children(|p| {
        label(
            p,
            f,
            LocalizedText::new("ui.editor.world.choose_type", "Choose type"),
            22.,
            WHITE,
        );
        field(
            p,
            f,
            e,
            Field::PickerSearch,
            "search",
            picker.filter.clone(),
        );
        p.spawn(Node {
            flex_grow: 1.,
            min_height: px(0),
            column_gap: px(12),
            ..default()
        })
        .with_children(|p| {
            p.spawn(Node {
                width: percent(62),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            })
            .with_children(|p| {
                for i in picker.filtered().into_iter().skip(picker.page * 8).take(8) {
                    button(
                        p,
                        f,
                        Action::PickerSelect(i),
                        "value",
                        &picker.choices[i].label(),
                        0.,
                        picker.selected == Some(i),
                    );
                }
                p.spawn(row()).with_children(|p| {
                    button(p, f, Action::PickerPage(false), "previous", "←", 55., false);
                    button(p, f, Action::PickerPage(true), "next", "→", 55., false);
                    value(
                        p,
                        f,
                        format!(
                            "{}/{}",
                            picker.page + 1,
                            picker.filtered().len().div_ceil(8).max(1)
                        ),
                        14.,
                    );
                });
            });
            p.spawn(Node {
                flex_grow: 1.,
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                ..default()
            })
            .with_children(|p| {
                if let Some(image) = &e.type_image {
                    p.spawn((
                        Node {
                            width: percent(100),
                            aspect_ratio: Some(1.),
                            ..default()
                        },
                        ImageNode::new(image.clone()),
                    ));
                }
                if let Some(choice) = picker.selected.and_then(|i| picker.choices.get(i)) {
                    value(p, f, choice.label(), 14.);
                }
                label(
                    p,
                    f,
                    LocalizedText::new(
                        "ui.editor.world.type_click_help",
                        "One click previews the model. Double-click or Choose applies its type.",
                    ),
                    13.,
                    MUTED,
                );
            });
        });
        p.spawn(row()).with_children(|p| {
            button(p, f, Action::PickerConfirm, "choose", "Choose", 145., false);
            button(p, f, Action::PickerCancel, "cancel", "Cancel", 145., false);
        });
    });
}
