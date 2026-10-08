//! Placement operations keep authored identities and visual/collider ownership together.
use super::*;
use model::{Patch, Source};

#[derive(Clone)]
pub(super) struct Clipboard {
    pub placement: model::Placement,
    pub row: Value,
    pub parts: BTreeMap<String, Vec<Value>>,
}

impl WorldEditor {
    pub(super) fn switch_list(&mut self, mode: u8) {
        self.list_search[self.list_mode as usize] = self.search.clone();
        self.list_mode = mode;
        self.search = self.list_search[mode as usize].clone();
        if mode != 2 {
            self.edit_objects = mode == 1;
            self.terrain_tool = None;
        }
        self.page = 0;
        self.focus = None;
        self.revision += 1;
    }
    pub(super) fn copy(&mut self) -> Result<(), String> {
        self.clipboard = Some(self.snapshot(self.selected().ok_or("Select an entity")?)?);
        Ok(())
    }
    pub(super) fn copy_coordinates(&mut self) -> Result<(), String> {
        let point=self.selected().map(|p|p.position).or(self.selected_point).ok_or("Select a point or entity first")?;
        self.coordinate_clipboard=Some(point);
        self.status=Some(LocalizedText::new("ui.editor.world.coordinates_copied","Coordinates copied: X {x}, Y {y}, Z {z}")
            .with_arg("x",format!("{:.2}",-point.x*100.))
            .with_arg("y",format!("{:.2}",point.z*100.))
            .with_arg("z",format!("{:.2}",point.y*100.)));
        self.revision+=1;
        Ok(())
    }
    pub(super) fn paste_coordinates(&mut self) -> Result<(), String> {
        let point=self.coordinate_clipboard.ok_or("Copy coordinates first")?;
        let index=self.selected.ok_or("Select an entity first")?;
        let placement=self.selected().cloned().ok_or("Select an entity first")?;
        self.transform(index,point,placement.angle,placement.instance)?;
        self.status=None;
        self.revision+=1;
        Ok(())
    }
    pub(super) fn snapshot(&self, p: &model::Placement) -> Result<Clipboard, String> {
        let row = self.sources[p.source]
            .draft
            .pointer(&p.pointer)
            .ok_or("Missing entity")?
            .clone();
        let mut parts: BTreeMap<String, Vec<Value>> = BTreeMap::new();
        if p.kind == 4 {
            let tile = self
                .maps
                .values()
                .find(|t| t.objects == p.source)
                .ok_or("Open the object's tile")?;
            let node = row["sourceNode"]
                .as_str()
                .ok_or("Missing object identity")?;
            for collection in ["visuals", "colliders"] {
                let rows = self.sources[tile.scene].draft[collection]
                    .as_array()
                    .ok_or("Missing object parts")?;
                parts.insert(
                    collection.into(),
                    rows.iter()
                        .filter(|r| {
                            r["name"]
                                .as_str()
                                .is_some_and(|n| map::matches_node(n, node))
                        })
                        .cloned()
                        .collect(),
                );
            }
            let model_ids: BTreeSet<_> = parts
                .values()
                .flatten()
                .filter_map(|v| v["model"].as_str())
                .collect();
            let models = self.sources[tile.scene].draft["models"]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|m| m["id"].as_str().is_some_and(|id| model_ids.contains(id)))
                .cloned()
                .collect();
            parts.insert("models".into(), models);
        }
        Ok(Clipboard {
            placement: p.clone(),
            row,
            parts,
        })
    }
    pub(super) fn paste(&mut self) -> Result<(), String> {
        let original = self.clipboard.clone().ok_or("Copy an entity first")?;
        let position = self.snap_to_ground(self.center)?;
        if original.placement.kind == 4 {
            return self.paste_object(original, position);
        }
        self.place_row(position, Some((original.placement, original.row)))
    }
    pub(super) fn delete_selected(&mut self) -> Result<(), String> {
        let p = self.selected().ok_or("Select an entity")?.clone();
        if p.kind == 4 {
            return self.delete_object(&p);
        }
        let mut patches = vec![Patch {
            source: p.source,
            pointer: p.pointer.clone(),
            before: self.sources[p.source].draft.pointer(&p.pointer).cloned(),
            after: None,
        }];
        for (source, pointer) in p.overrides.values() {
            let parent = pointer.rsplit_once('/').ok_or("Invalid override")?.0;
            if patches
                .iter()
                .any(|patch| patch.source == *source && patch.pointer == parent)
            {
                continue;
            }
            patches.push(Patch {
                source: *source,
                pointer: parent.into(),
                before: self.sources[*source].draft.pointer(parent).cloned(),
                after: None,
            });
        }
        // Removing an array slot changes later indices; record the full array for undo.
        for patch in &mut patches {
            let (parent, key) = patch
                .pointer
                .rsplit_once('/')
                .ok_or("Invalid entity pointer")?;
            if let Some(rows) = self.sources[patch.source]
                .draft
                .pointer(parent)
                .and_then(Value::as_array)
            {
                let mut after = rows.clone();
                after.remove(key.parse::<usize>().map_err(|_| "Invalid entity index")?);
                patch.before = Some(Value::Array(rows.clone()));
                patch.after = Some(Value::Array(after));
                patch.pointer = parent.into();
            }
        }
        self.commit(patches)?;
        self.selected = None;
        self.selected_point = None;
        Ok(())
    }
    pub(super) fn waypoint_patches(
        &mut self,
        p: &model::Placement,
        position: Vec3,
    ) -> Result<Vec<Patch>, String> {
        if p.kind != 0 || position == p.position {
            return Ok(vec![]);
        }
        let path = self.root.join("data/missions/client-npc-waypoints.json");
        let source = if let Some(i) = self.sources.iter().position(|s| s.path == path) {
            i
        } else {
            if !path.exists() {
                return Ok(vec![]);
            }
            let base: Value = serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            self.sources.push(Source {
                path,
                draft: base.clone(),
                base,
            });
            self.sources.len() - 1
        };
        let rows = self.sources[source].draft["rows"]
            .as_array()
            .ok_or("Missing waypoint rows")?;
        let mut patches = vec![];
        for (i, row) in rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r["npcType"].as_i64() == Some(p.type_id))
        {
            patches.push(Patch {
                source,
                pointer: format!("/rows/{i}/clientPosition"),
                before: Some(row["clientPosition"].clone()),
                after: Some(serde_json::json!([-position.x, position.y, position.z])),
            });
        }
        if patches.is_empty() {
            let mut after = rows.clone();
            after.push(serde_json::json!({"npcType":p.type_id,"clientPosition":[-position.x,position.y,position.z]}));
            patches.push(Patch {
                source,
                pointer: "/rows".into(),
                before: Some(Value::Array(rows.clone())),
                after: Some(Value::Array(after)),
            });
        }
        Ok(patches)
    }
    pub(super) fn object_rotation(&self, p: &model::Placement) -> Vec3 {
        let Some(matrix) = self.sources[p.source]
            .draft
            .pointer(&p.pointer)
            .and_then(|v| map::matrix(&v["worldMatrix"]))
        else {
            return Vec3::ZERO;
        };
        let (_, q, _) = matrix.to_scale_rotation_translation();
        let (x, y, z) = q.to_euler(EulerRot::XYZ);
        Vec3::new(x.to_degrees(), y.to_degrees(), z.to_degrees())
    }
}
