use super::*;
use std::io::Write;

#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct Source {
    pub path: PathBuf,
    pub base: Value,
    pub draft: Value,
}
#[derive(Clone)]
pub(super) struct Placement {
    pub source: usize,
    pub pointer: String,
    pub key: String,
    pub kind: usize,
    pub type_id: i64,
    pub position: Vec3,
    pub angle: f32,
    pub instance: u32,
    pub overrides: BTreeMap<String, (usize, String)>,
    pub object_name: Option<String>,
}
#[derive(Clone)]
pub(super) struct Edit {
    pub patches: Vec<Patch>,
    pub selection: Option<String>,
}
#[derive(Clone)]
pub(super) struct Patch {
    pub source: usize,
    pub pointer: String,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

pub(super) fn native(v: &Value) -> Option<Vec3> {
    let num = |key| {
        v.get(key)?
            .as_f64()
            .filter(|n| n.is_finite())
            .map(|n| n as f32)
    };
    Some(Vec3::new(
        -num("iX")? * 0.01,
        num("iZ")? * 0.01,
        num("iY")? * 0.01,
    ))
}
fn set_pointer(document: &mut Value, pointer: &str, value: Option<Value>) -> Result<(), String> {
    if pointer.is_empty() { *document = value.unwrap_or(Value::Null); return Ok(()); }
    let (parent, key) = pointer
        .rsplit_once('/')
        .ok_or("Invalid placement pointer")?;
    let parent = document
        .pointer_mut(parent)
        .ok_or("Missing placement container")?;
    if let Some(map) = parent.as_object_mut() {
        if let Some(value) = value {
            map.insert(key.into(), value);
        } else {
            map.remove(key);
        }
    } else if let Some(rows) = parent.as_array_mut() {
        let index: usize = key.parse().map_err(|_| "Invalid placement index")?;
        match value {
            Some(value) if index < rows.len() => rows[index] = value,
            Some(value) if index == rows.len() => rows.push(value),
            None if index < rows.len() => {
                rows.remove(index);
            }
            _ => return Err("Placement array changed".into()),
        }
    } else {
        return Err("Invalid placement container".into());
    }
    Ok(())
}

pub(super) fn write_atomic(path: &Path, value: &Value) -> Result<(), String> {
    backups::retain(path)?;
    if value.is_null() { if path.exists() { fs::remove_file(path).map_err(|e|e.to_string())?; } return Ok(()); }
    let parent = path.parent().ok_or("Missing output directory")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut file = tempfile::NamedTempFile::new_in(parent).map_err(|e| e.to_string())?;
    if value["schema"] == "ffone.editor-pixels.v1" {
        file.write_all(&terrain::pixel_bytes(value)?)
            .map_err(|e| e.to_string())?;
    } else {
        let mut writer = std::io::BufWriter::with_capacity(64 * 1024, &mut file);
        serde_json::to_writer_pretty(&mut writer, value).map_err(|e| e.to_string())?;
        writer.write_all(b"\n").map_err(|e| e.to_string())?;
        writer.flush().map_err(|e| e.to_string())?;
    }
    file.as_file().sync_all().map_err(|e| e.to_string())?;
    file.persist(path).map_err(|e| e.to_string())?;
    Ok(())
}

impl WorldEditor {
    pub(super) fn open(root: PathBuf) -> Self {
        let mut editor = Self {
            root,
            open_mode: None,
            session_request: None,
            folder: None,
            sources: vec![],
            entities: vec![],
            selected: None,
            selected_point: None,
            coordinate_clipboard: None,
            last_entity_click: None,
            show_collisions: false,
            instance: 0,
            search: String::new(),
            page: 0,
            center: Vec3::ZERO,
            zoom: 2.,
            yaw: 0.,
            pitch: 0.75,
            distance: 100.,
            snap: 100.,
            placing: false,
            placement_kind: 0,
            type_id: 1,
            type_picker: None,
            type_image: None,
            object_template: None,
            group_template: None,
            routes: None,
            focus: None,
            edit: String::new(),
            select_text: false,
            drag: None,
            ignore_pointer_press: false,
            last_cursor: None,
            undo: vec![],
            redo: vec![],
            changed_at: None,
            revision: 1,
            geometry_revision: 1,
            status: None,
            maps: BTreeMap::new(),
            region: None,
            available_tiles: BTreeSet::new(),
            npc_map_icons: BTreeMap::new(),
            object_npc_types: BTreeSet::new(),
            actor_enabled: [true; 4],
            list_mode: 0,
            list_search: Default::default(),
            edit_objects: false,
            clipboard: None,
            instances: instances::names(),
            instance_menu: None,
            ground: BTreeMap::new(),
            terrain_tool: None,
            creating_terrain: false,
            square_menu: false,
            square_choices: None,
            music_choices: Vec::new(),
            square_names: BTreeMap::new(),
            grass_template: None,
            grass_stroke: None,
            grass_last: None,
            brush_radius: 12.,
            brush_strength: 5.,
            brush_layer: 0,
            brush: None,
            brush_cursor: None,
            object_index: vec![],
            pending_object: None,
            map_picker: false,
            atlas_center: Vec3::new(-4096., 0., 4096.),
            atlas_zoom: 0.05,
        };
        editor.scan_tiles();
        editor
    }
    pub(super) fn restore_work(&mut self) -> Result<(), String> {
        let mut editor = Self::open(self.root.clone());
        let work: Value = serde_json::from_slice(
            &fs::read(editor.work_path()).map_err(|e| e.to_string())?,
        ).map_err(|e| e.to_string())?;
        if work["schema"] != "ffone.world-workspace.v1" {
            return Err("Unsupported world backup".into());
        }
        let folder = work["folder"].as_str().map(PathBuf::from);
        if self.folder.is_some() && folder != self.folder {
            return Err("World backup belongs to another server folder".into());
        }
        editor.sources = serde_json::from_value::<Vec<Source>>(work["sources"].clone())
            .map_err(|e| e.to_string())?;
        for source in &mut editor.sources {
            if source.base == source.draft {
                if let Ok(bytes) = fs::read(&source.path) {
                    if let Ok(document) = serde_json::from_slice::<Value>(&bytes) {
                        source.base = document.clone();
                        source.draft = document;
                    }
                }
            }
        }
        editor.folder = work["folder"].as_str().map(PathBuf::from);
        let tile_ids: Vec<_> = work["tiles"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .chain(work["tile"].as_str())
            .collect();
        editor.region = serde_json::from_value(work["region"].clone())
            .ok()
            .or_else(|| {
                work["tile"]
                    .as_str()
                    .and_then(|id| id.strip_prefix("map_")?.split_once('_'))
                    .and_then(|(x, y)| Some([x.parse().ok()?, y.parse().ok()?]))
            });
        editor.scan_tiles();
        let desired: BTreeSet<_> = editor
            .desired_tiles()
            .into_iter()
            .map(atlas::tile_id)
            .collect();
        for id in tile_ids.into_iter().filter(|id| desired.contains(*id)) {
            let objects = editor.sources.iter().position(|s| {
                s.path == editor.root.join(format!("map/tiles/{id}/objects.json"))
            });
            let scene = editor.sources.iter().position(|s| {
                s.path == editor.root.join(format!("map/tiles/{id}/scene.json"))
            });
            if let (Some(objects), Some(scene)) = (objects, scene) {
                editor.maps.insert(
                    id.into(),
                    map::Tile {
                        id: id.into(),
                        objects,
                        scene,
                        native_scene: serde_json::from_value(
                            editor.sources[scene].draft.clone(),
                        )
                        .ok(),
                    },
                );
            }
        }
        editor.rebuild();
        editor.sync_terrain()?;
        editor.selected = work["selected"]
            .as_u64()
            .map(|i| i as usize)
            .filter(|i| *i < editor.entities.len());
        editor.instance = work["instance"].as_u64().unwrap_or(0) as u32;
        if let Ok(instances) =
            serde_json::from_value::<BTreeMap<u32, String>>(work["instances"].clone())
        {
            editor.instances.extend(instances);
        }
        if let Ok(enabled) =
            serde_json::from_value::<[bool; 4]>(work["actor_enabled"].clone())
        {
            editor.actor_enabled = enabled;
        }
        editor.center_selected();
        if let Some(center) = work["center"].as_array().filter(|a| a.len() == 3) {
            let center = Vec3::new(
                center[0].as_f64().unwrap_or(0.) as f32,
                center[1].as_f64().unwrap_or(0.) as f32,
                center[2].as_f64().unwrap_or(0.) as f32,
            );
            if center.is_finite() {
                editor.center = center;
            }
        }
        editor.zoom = work["zoom"].as_f64().unwrap_or(2.) as f32;
        if !editor.zoom.is_finite() || editor.zoom <= 0. {
            editor.zoom = 2.;
        }
        editor.revision = self.revision + 1;
        editor.geometry_revision = self.geometry_revision + 1;
        *self = editor;
        Ok(())
    }
    pub(super) fn work_path(&self) -> PathBuf {
        self.root.join("../editor/world-workspace.json")
    }
    pub(super) fn load_server(&mut self, folder: &Path) -> Result<(), String> {
        if self.folder.as_deref() == Some(folder) {
            return Ok(());
        }
        if self.sources.iter().any(|s| s.base != s.draft) {
            return Err("Publish the current world draft before selecting another server".into());
        }
        let mut sources = Vec::new();
        for name in ["NPCs.json", "mobs.json", "eggs.json", "gruntwork.json"] {
            let path = folder.join(name);
            if !path.exists() && name != "NPCs.json" {
                continue;
            }
            let base: Value = serde_json::from_slice(
                &fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
            )
            .map_err(|e| e.to_string())?;
            sources.push(Source {
                path,
                draft: base.clone(),
                base,
            });
        }
        self.sources = sources;
        self.maps.clear();
        self.region = None;
        self.folder = Some(folder.to_path_buf());
        self.undo.clear();
        self.redo.clear();
        self.rebuild();
        self.selected = (!self.entities.is_empty()).then_some(0);
        self.selected_point = None;
        self.center_selected();
        self.geometry_revision += 1;
        Ok(())
    }
    pub(super) fn rebuild(&mut self) {
        self.scan_tiles();
        let sources=&self.sources;
        self.maps.retain(|_,m|!sources[m.scene].draft.is_null());
        let selected_key = self.selected().map(|p| p.key.clone());
        self.entities.clear();
        for (source, data) in self.sources.iter().enumerate() {
            let filename = data
                .path
                .file_name()
                .and_then(|p| p.to_str())
                .unwrap_or_default();
            let containers: &[(&str, usize)] = match filename {
                "NPCs.json" => &[("NPCs", 0)],
                "mobs.json" => &[("mobs", 1), ("groups", 2)],
                "eggs.json" => &[("Eggs", 3)],
                "gruntwork.json" => &[("mobs", 1), ("groups", 2)],
                _ => &[],
            };
            for &(key, kind) in containers {
                let rows: Vec<(String, &Value)> = if let Some(rows) = data.draft[key].as_object() {
                    let mut rows: Vec<_> = rows.iter().map(|(k, v)| (k.clone(), v)).collect();
                    rows.sort_by_key(|(k, _)| k.parse::<i64>().unwrap_or(i64::MAX));
                    rows
                } else {
                    data.draft[key]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .enumerate()
                        .map(|(i, v)| (i.to_string(), v))
                        .collect()
                };
                for (id, value) in rows {
                    if let Some(position) = native(value) {
                        self.entities.push(Placement {
                            source,
                            pointer: format!("/{key}/{id}"),
                            key: format!("{filename}/{key}/{id}"),
                            kind,
                            type_id: value[if kind == 3 { "iType" } else { "iNPCType" }]
                                .as_i64()
                                .unwrap_or(0),
                            position,
                            angle: value["iAngle"].as_f64().unwrap_or(0.) as f32,
                            instance: value["iMapNum"].as_u64().unwrap_or(0) as u32,
                            overrides: BTreeMap::new(),
                            object_name: None,
                        });
                    }
                }
            }
        }
        // The server applies these after NPCs.json; edit the actual authoritative field.
        for (source, data) in self
            .sources
            .iter()
            .enumerate()
            .filter(|(_, s)| s.path.ends_with("gruntwork.json"))
        {
            for (container, field) in [("rotations", "iAngle"), ("instances", "iMapNum")] {
                for (i, row) in data.draft[container]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .enumerate()
                {
                    if let Some(p) = self.entities.iter_mut().find(|p| {
                        p.kind == 0
                            && p.pointer
                                .rsplit('/')
                                .next()
                                .and_then(|k| k.parse::<i64>().ok())
                                .map(|n| n + 1)
                                == row["iNPCID"].as_i64()
                    }) {
                        p.overrides
                            .insert(field.into(), (source, format!("/{container}/{i}/{field}")));
                        if field == "iAngle" {
                            p.angle = row[field].as_f64().unwrap_or(0.) as f32;
                        } else {
                            p.instance = row[field].as_u64().unwrap_or(0) as u32;
                        }
                    }
                }
            }
        }
        for tile in self.maps.values() {
            tile.append_placements(&self.sources, &mut self.entities);
        }
        self.selected =
            selected_key.and_then(|key| self.entities.iter().position(|p| p.key == key));
    }
    pub(super) fn commit(&mut self, patches: Vec<Patch>) -> Result<(), String> {
        if patches.iter().all(|p| p.before == p.after) {
            return Ok(());
        }
        for p in &patches {
            set_pointer(
                &mut self.sources[p.source].draft,
                &p.pointer,
                p.after.clone(),
            )?;
        }
        self.undo.push(Edit {
            patches,
            selection: self.selected().map(|p| p.key.clone()),
        });
        self.redo.clear();
        self.sync_terrain()?;
        self.rebuild();
        self.touch();
        Ok(())
    }
    pub(super) fn transform(
        &mut self,
        index: usize,
        position: Vec3,
        angle: f32,
        instance: u32,
    ) -> Result<(), String> {
        let p = self.entities.get(index).ok_or("Select an entity")?.clone();
        if !position.is_finite() || !angle.is_finite() {
            return Err("Coordinates must be finite".into());
        }
        if p.kind == 4 {
            return self.transform_object(&p, position, angle, instance);
        }
        let raw = [-position.x * 100., position.z * 100., position.y * 100.];
        if raw
            .iter()
            .any(|n| *n < i32::MIN as f32 || *n >= i32::MAX as f32)
        {
            return Err("Coordinates outside server range".into());
        }
        let mut row = self.sources[p.source]
            .draft
            .pointer(&p.pointer)
            .ok_or("Missing entity")?
            .clone();
        // Preserve fractional coordinates on untouched axes and every unknown field.
        for (axis, field) in ["iX", "iY", "iZ"].iter().enumerate() {
            if (position - p.position)[[0, 2, 1][axis]].abs() > 0.00001 {
                row[*field] = Value::from(raw[axis].round() as i64);
            }
        }
        let mut patches = vec![];
        for (field, value, changed) in [
            (
                "iAngle",
                Value::from(angle.round() as i64),
                angle != p.angle,
            ),
            ("iMapNum", Value::from(instance), instance != p.instance),
        ] {
            if !changed {
                continue;
            }
            if let Some((source, pointer)) = p.overrides.get(field) {
                patches.push(Patch {
                    source: *source,
                    pointer: pointer.clone(),
                    before: self.sources[*source].draft.pointer(pointer).cloned(),
                    after: Some(value),
                });
            } else {
                row[field] = value;
            }
        }
        patches.push(Patch {
            source: p.source,
            pointer: p.pointer.clone(),
            before: self.sources[p.source].draft.pointer(&p.pointer).cloned(),
            after: Some(row),
        });
        patches.extend(self.waypoint_patches(&p, position)?);
        self.commit(patches)
    }
    pub(super) fn place(&mut self, position: Vec3, duplicate: bool) -> Result<(), String> {
        if !position.is_finite() || position.abs().max_element() * 100. >= i32::MAX as f32 {
            return Err("Coordinates outside server range".into());
        }
        let original = duplicate.then(|| self.selected().cloned()).flatten();
        if !duplicate && self.placement_kind == 4 {
            let template=self.object_template.clone().ok_or("Choose a world model")?;
            self.paste_object(template,position)?;
            self.placing=false;
            return Ok(());
        }
        if original.as_ref().is_some_and(|p| p.kind == 4) {
            return self.duplicate_object(original.unwrap(), position);
        }
        let snapshot = original.and_then(|p| {
            self.sources[p.source]
                .draft
                .pointer(&p.pointer)
                .cloned()
                .map(|row| (p, row))
        });
        self.place_row(position, snapshot)
    }
    pub(super) fn place_row(
        &mut self,
        position: Vec3,
        snapshot: Option<(Placement, Value)>,
    ) -> Result<(), String> {
        let original = snapshot.as_ref().map(|(p, _)| p);
        let kind = original.as_ref().map_or(self.placement_kind, |p| p.kind);
        let (file, key) = match kind {
            0 => ("NPCs.json", "NPCs"),
            1 => ("mobs.json", "mobs"),
            2 => ("mobs.json", "groups"),
            _ => ("eggs.json", "Eggs"),
        };
        let source = self
            .sources
            .iter()
            .position(|s| s.path.ends_with(file))
            .ok_or("Selected server has no placement file")?;
        let rows = self.sources[source].draft[key]
            .as_object()
            .ok_or("Missing placement table")?;
        let id = rows
            .keys()
            .filter_map(|k| k.parse::<i32>().ok())
            .max()
            .unwrap_or(-1)
            .checked_add(1)
            .ok_or("No free placement ID")?;
        let mut row = snapshot
            .as_ref()
            .map(|(_, row)| row.clone())
            .unwrap_or_else(|| serde_json::json!({}));
        if original.is_none() && kind==2 {
            if let Some(template)=&self.group_template {row=template.clone();}
        }
        row[if kind == 3 { "iType" } else { "iNPCType" }] =
            Value::from(original.as_ref().map_or(self.type_id, |p| p.type_id));
        row["iX"] = Value::from((-position.x * 100.).round() as i64);
        row["iY"] = Value::from((position.z * 100.).round() as i64);
        row["iZ"] = Value::from((position.y * 100.).round() as i64);
        row["iMapNum"] = Value::from(self.instance);
        if kind != 3 {
            row["iAngle"] = Value::from(original.as_ref().map_or(0., |p| p.angle).round() as i64);
        }
        if kind == 2 && row.get("aFollowers").is_none() {
            row["aFollowers"] = serde_json::json!([]);
        }
        let pointer = format!("/{key}/{id}");
        self.commit(vec![Patch {
            source,
            pointer: pointer.clone(),
            before: None,
            after: Some(row),
        }])?;
        self.selected = self
            .entities
            .iter()
            .position(|p| p.source == source && p.pointer == pointer);
        self.selected_point = None;
        self.placing = false;
        self.revision += 1;
        Ok(())
    }
    pub(super) fn undo(&mut self, redo: bool) -> Result<(), String> {
        let stack = if redo { &mut self.redo } else { &mut self.undo };
        let Some(edit) = stack.pop() else {
            return Ok(());
        };
        if redo {
            for p in &edit.patches {
                set_pointer(
                    &mut self.sources[p.source].draft,
                    &p.pointer,
                    p.after.clone(),
                )?;
            }
        } else {
            for p in edit.patches.iter().rev() {
                set_pointer(
                    &mut self.sources[p.source].draft,
                    &p.pointer,
                    p.before.clone(),
                )?;
            }
        }
        let selected_key = edit.selection.clone();
        if redo {
            self.undo.push(edit);
        } else {
            self.redo.push(edit);
        }
        self.rebuild();
        self.sync_terrain()?;
        self.selected =
            selected_key.and_then(|key| self.entities.iter().position(|p| p.key == key));
        self.touch();
        self.selected_point = None;
        Ok(())
    }
    pub(super) fn save_work(&mut self) -> Result<(), String> {
        if self.sources.is_empty() {
            return Ok(());
        }
        write_atomic(
            &self.work_path(),
            &serde_json::json!({"schema":"ffone.world-workspace.v1", "folder":self.folder,
            "sources": self.sources, "selected":self.selected, "instance":self.instance,
            "tiles":self.maps.keys().collect::<Vec<_>>(),"region":self.region,"center":self.center.to_array(),"zoom":self.zoom,
            "instances":self.instances,"actor_enabled":self.actor_enabled}),
        )?;
        self.changed_at = None;
        Ok(())
    }
    pub(super) fn publish(&mut self) -> Result<(), String> {
        self.prepare_terrain_publish()?;
        self.refresh_guards()?;
        let changed: Vec<_> = self
            .sources
            .iter()
            .enumerate()
            .filter(|(_, s)| s.base != s.draft)
            .map(|(i, _)| i)
            .collect();
        for &i in &changed {
            let s = &self.sources[i];
            let disk = if s.path.exists() { terrain::read_source(&s.path)? } else { Value::Null };
            if disk != s.base {
                return Err(format!("File changed externally: {}", s.path.display()));
            }
        }
        // Check the whole transaction before replacing any file; restore on a write failure.
        let mut written: Vec<usize> = vec![];
        for &i in &changed {
            let s = &self.sources[i];
            if let Err(e) = write_atomic(&s.path, &s.draft) {
                for old in written.iter().rev() {
                    write_atomic(&self.sources[*old].path, &self.sources[*old].base)?;
                }
                return Err(e);
            }
            written.push(i);
        }
        for i in changed {
            self.sources[i].base = self.sources[i].draft.clone();
        }
        self.save_work()?;
        self.status = Some(LocalizedText::new(
            "ui.editor.world.published",
            "World saved. Restart the server to apply entity placements.",
        ));
        self.revision += 1;
        Ok(())
    }
}
