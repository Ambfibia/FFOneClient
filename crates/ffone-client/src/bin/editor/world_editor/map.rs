//! Native object edits keep visual/collision parts together and refresh the guard closure.
use super::*;
use model::{Patch, Placement, Source};

pub(super) struct Tile {
    pub id: String,
    pub objects: usize,
    pub scene: usize,
    pub native_scene: Option<ffone_client::world::NativeWorldScene>,
}
pub(super) fn matches_node(name: &str, node: &str) -> bool {
    name.split_once(&format!("[{node}"))
        .is_some_and(|(_, tail)| tail.starts_with(' ') || tail.starts_with(']'))
}
pub(super) fn object_label(label: &str, model: Option<&str>) -> String {
    match model.filter(|m| !m.is_empty() && !label.to_lowercase().contains(&m.to_lowercase())) {
        Some(model) => format!("{model} · {label}"),
        None => label.to_owned(),
    }
}
pub(super) fn has_behaviour(root: &Path, tile: &str, node: &str) -> Result<bool, String> {
    let document = fs::read_to_string(root.join(format!("map/tiles/{tile}/behaviour.json")))
        .map_err(|e| e.to_string())?;
    // Compare the complete serialized identity. A node ending in #1 must not
    // inherit the script ownership of #10, #100, etc.
    Ok(document.contains(&serde_json::to_string(node).map_err(|e| e.to_string())?))
}
pub(super) fn matrix(value: &Value) -> Option<Mat4> {
    let rows = value.as_array()?;
    let mut values = [0.; 16];
    for row in 0..4 {
        for col in 0..4 {
            let v = &rows.get(row)?.as_array()?.get(col)?;
            values[col * 4 + row] = v.as_f64().or_else(|| v.as_str()?.parse().ok())? as f32;
        }
    }
    Some(Mat4::from_cols_array(&values))
}
fn matrix_json(m: Mat4) -> Value {
    let values = m.to_cols_array();
    Value::Array(
        (0..4)
            .map(|row| {
                Value::Array(
                    (0..4)
                        .map(|col| Value::String(values[col * 4 + row].to_string()))
                        .collect(),
                )
            })
            .collect(),
    )
}
impl Tile {
    pub(super) fn append_placements(&self, sources: &[Source], into: &mut Vec<Placement>) {
        let models: BTreeMap<_, _> = sources[self.scene].draft["models"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|m| {
                Some((
                    m["id"].as_str()?,
                    m["rootName"].as_str().or_else(|| m["path"].as_str())?,
                ))
            })
            .collect();
        let names: BTreeMap<_, _> = sources[self.scene].draft["visuals"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| {
                let name = v["name"].as_str()?;
                let (label, tail) = name.rsplit_once(" [")?;
                let node = tail.split([' ', ']']).next()?;
                Some((
                    node,
                    object_label(
                        label,
                        v["model"].as_str().and_then(|id| models.get(id).copied()),
                    ),
                ))
            })
            .collect();
        for (i, object) in sources[self.objects].draft["objects"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
        {
            let Some(m) = matrix(&object["worldMatrix"]) else {
                continue;
            };
            let name = object["sourceNode"].as_str().unwrap_or_default();
            let visual_name = names.get(name).cloned().unwrap_or_else(|| name.to_owned());
            into.push(Placement {
                source: self.objects,
                pointer: format!("/objects/{i}"),
                key: format!("{}/{i}", self.id),
                kind: 4,
                type_id: 0,
                position: m.w_axis.truncate(),
                angle: -m.z_axis.x.atan2(m.z_axis.z).to_degrees(),
                instance: 0,
                overrides: BTreeMap::new(),
                object_name: Some(visual_name),
            });
        }
    }
}
impl WorldEditor {
    #[cfg(test)]
    pub(super) fn load_tile(&mut self) -> Result<(), String> {
        let tile = atlas::tile_at(self.center).ok_or("Invalid map position")?;
        self.select_region(tile)?;
        let id = atlas::tile_id(tile);
        if self.maps.contains_key(&id) {
            return Ok(());
        }
        let documents = stream::read_tile(
            &self.root,
            &id,
            !self
                .sources
                .iter()
                .any(|s| s.path == self.root.join("map/catalog.json")),
        )?;
        self.install_tile(id, documents);
        Ok(())
    }
    pub(super) fn scan_tiles(&mut self) {
        self.available_tiles = fs::read_dir(self.root.join("map/tiles"))
            .into_iter()
            .flatten()
            .filter_map(|entry| {
                let entry=entry.ok()?;if !entry.path().join("scene.json").is_file(){return None;}
                let name = entry.file_name().to_string_lossy().into_owned();
                let (x, y) = name.strip_prefix("map_")?.split_once('_')?;
                Some([x.parse().ok()?, y.parse().ok()?])
            })
            .collect();
        for source in &self.sources {
            if source.path.starts_with(self.root.join("map/tiles")) && source.path.ends_with("scene.json") && !source.draft.is_null() {
                if let Ok(tile)=serde_json::from_value::<[i32;2]>(source.draft["tile"].clone()){self.available_tiles.insert(tile);}
            }
        }
    }
    pub(super) fn desired_tiles(&self) -> Vec<[i32; 2]> {
        self.region
            .map(atlas::neighbours)
            .unwrap_or_default()
            .into_iter()
            .filter(|t| self.available_tiles.contains(t))
            .collect()
    }
    pub(super) fn select_region(&mut self, tile: [i32; 2]) -> Result<(), String> {
        self.scan_tiles();
        if !self.available_tiles.contains(&tile) {
            return Err(format!("{} is not published", atlas::tile_id(tile)));
        }
        // Source indices remain stable for draft history; only these nine tiles own live render roots.
        self.region = Some(tile);
        let desired: BTreeSet<_> = self
            .desired_tiles()
            .into_iter()
            .map(atlas::tile_id)
            .collect();
        self.maps.retain(|id, _| desired.contains(id));
        self.ground.retain(|id, _| desired.contains(id));
        self.rebuild();
        self.geometry_revision += 1;
        self.revision += 1;
        self.changed_at = Some(Instant::now());
        Ok(())
    }
    pub(super) fn install_tile(&mut self, id: String, documents: stream::TileDocuments) {
        if let Some(ground) = documents.ground {
            self.ground.entry(id.clone()).or_insert(ground);
        }
        let mut indices = Vec::new();
        let mut fresh_scene = false;
        for source in documents.sources {
            let index = if let Some(i) = self.sources.iter().position(|s| s.path == source.path) {
                i
            } else {
                if source.path.ends_with("scene.json") {
                    fresh_scene = true;
                }
                self.sources.push(source);
                self.sources.len() - 1
            };
            indices.push(index);
        }
        let native_scene = if fresh_scene {
            documents.native_scene
        } else {
            serde_json::from_value(self.sources[indices[1]].draft.clone()).ok()
        };
        self.maps.insert(
            id.clone(),
            Tile {
                id,
                objects: indices[0],
                scene: indices[1],
                native_scene,
            },
        );
        if let Err(error) = self.sync_terrain() {
            self.error(error);
        }
        self.rebuild();
        self.revision += 1;
        self.geometry_revision += 1;
    }
    pub(super) fn transform_object(
        &mut self,
        p: &Placement,
        position: Vec3,
        angle: f32,
        instance: u32,
    ) -> Result<(), String> {
        self.transform_object_delta(
            p,
            position,
            Quat::from_rotation_y(-(angle - p.angle).to_radians()),
            instance,
        )
    }
    pub(super) fn rotate_object(
        &mut self,
        index: usize,
        axis: usize,
        degrees: f32,
        absolute: bool,
    ) -> Result<(), String> {
        let p = self.entities.get(index).ok_or("Select an object")?.clone();
        if p.kind != 4 {
            return Err("Select a static object".into());
        }
        let old = self.object_rotation(&p);
        let mut next = old;
        next[axis] = if absolute {
            degrees
        } else {
            old[axis] + degrees
        };
        let quat = |v: Vec3| {
            Quat::from_euler(
                EulerRot::XYZ,
                v.x.to_radians(),
                v.y.to_radians(),
                v.z.to_radians(),
            )
        };
        self.transform_object_delta(&p, p.position, quat(next) * quat(old).inverse(), p.instance)
    }
    fn transform_object_delta(
        &mut self,
        p: &Placement,
        position: Vec3,
        rotation: Quat,
        instance: u32,
    ) -> Result<(), String> {
        if instance != 0 {
            return Err(
                "Map geometry is shared; per-instance placement applies to server entities".into(),
            );
        }
        let tile = self
            .maps
            .values()
            .find(|t| t.objects == p.source)
            .ok_or("Open the object's map tile")?;
        let object = self.sources[p.source]
            .draft
            .pointer(&p.pointer)
            .ok_or("Missing object")?
            .clone();
        let node = object["sourceNode"]
            .as_str()
            .ok_or("Missing object identity")?;
        if has_behaviour(&self.root, &tile.id, node)? {
            return Err("This object owns scripted behaviour; its behaviour transform must be edited together".into());
        }
        let delta = Mat4::from_translation(position)
            * Mat4::from_quat(rotation)
            * Mat4::from_translation(-p.position);
        let mut after = object.clone();
        after["worldMatrix"] =
            matrix_json(delta * matrix(&object["worldMatrix"]).ok_or("Invalid object transform")?);
        let mut patches = vec![Patch {
            source: p.source,
            pointer: p.pointer.clone(),
            before: Some(object.clone()),
            after: Some(after),
        }];
        for collection in ["visuals", "colliders"] {
            for (i, row) in self.sources[tile.scene].draft[collection]
                .as_array()
                .into_iter()
                .flatten()
                .enumerate()
            {
                if !row["name"].as_str().is_some_and(|n| matches_node(n, node)) {
                    continue;
                }
                let transform: ffone_client::world::AuthoredWorldTransform =
                    serde_json::from_value(row["transform"].clone()).map_err(|e| e.to_string())?;
                let mut transform = transform.try_to_bevy(node).map_err(|e| e.to_string())?;
                transform.translation = delta.transform_point3(transform.translation);
                transform.rotation = rotation * transform.rotation;
                let pointer = format!("/{collection}/{i}/transform");
                patches.push(Patch {source:tile.scene,pointer,before:Some(row["transform"].clone()),after:Some(serde_json::json!({
                    "translation":transform.translation.to_array(),"rotation":transform.rotation.to_array(),"scale":transform.scale.to_array()}))});
            }
        }
        if patches.len() == 1 {
            return Err("Object has no matching visual or collision parts".into());
        }
        self.commit(patches)
    }
    pub(super) fn delete_object(&mut self, p: &Placement) -> Result<(), String> {
        let tile = self
            .maps
            .values()
            .find(|t| t.objects == p.source)
            .ok_or("Open the object's map tile")?;
        let object = self.sources[p.source]
            .draft
            .pointer(&p.pointer)
            .ok_or("Missing object")?;
        let node = object["sourceNode"]
            .as_str()
            .ok_or("Missing object identity")?;
        if has_behaviour(&self.root, &tile.id, node)? {
            return Err("Scripted objects must retain their behaviour identity".into());
        }
        let mut patches = vec![];
        for (source, collection) in [
            (tile.objects, "objects"),
            (tile.scene, "visuals"),
            (tile.scene, "colliders"),
        ] {
            let rows = self.sources[source].draft[collection]
                .as_array()
                .ok_or("Missing object parts")?;
            let next: Vec<_> = rows
                .iter()
                .filter(|row| {
                    if collection == "objects" {
                        row["sourceNode"].as_str() != Some(node)
                    } else {
                        !row["name"]
                            .as_str()
                            .is_some_and(|name| matches_node(name, node))
                    }
                })
                .cloned()
                .collect();
            patches.push(Patch {
                source,
                pointer: format!("/{collection}"),
                before: Some(Value::Array(rows.clone())),
                after: Some(Value::Array(next)),
            });
        }
        self.commit(patches)?;
        self.selected = None;
        Ok(())
    }
    pub(super) fn duplicate_object(&mut self, p: Placement, position: Vec3) -> Result<(), String> {
        let snapshot = self.snapshot(&p)?;
        self.paste_object(snapshot, position)
    }
    pub(super) fn paste_object(
        &mut self,
        snapshot: operations::Clipboard,
        position: Vec3,
    ) -> Result<(), String> {
        let p = &snapshot.placement;
        let origin_id = p.key.split('/').next().ok_or("Missing object tile")?;
        let coordinate = atlas::tile_at(position).ok_or("Invalid object destination")?;
        let destination_id = atlas::tile_id(coordinate);
        if !self.maps.contains_key(&destination_id) {
            if !self.available_tiles.contains(&coordinate) {
                return Err("Select a published destination tile".into());
            }
            self.select_region(coordinate)?;
            let documents = stream::read_tile(
                &self.root,
                &destination_id,
                !self
                    .sources
                    .iter()
                    .any(|s| s.path == self.root.join("map/catalog.json")),
            )?;
            self.install_tile(destination_id.clone(), documents);
        }
        let tile = &self.maps[&destination_id];
        let objects = tile.objects;
        let scene = tile.scene;
        let original = snapshot.row;
        let source_node = original["sourceNode"]
            .as_str()
            .ok_or("Missing object identity")?;
        if has_behaviour(&self.root, &origin_id, source_node)? {
            return Err("Scripted objects need a new behaviour identity before duplication".into());
        }
        let count = self.sources[objects].draft["objects"]
            .as_array()
            .ok_or("Missing objects")?
            .len();
        let mut identity = count;
        let node = loop {
            let candidate = format!("FFOneEditor-{}#{identity}", tile.id);
            if !self.sources[objects].draft["objects"]
                .as_array()
                .unwrap()
                .iter()
                .any(|r| r["sourceNode"] == candidate)
            {
                break candidate;
            }
            identity += 1;
        };
        let delta = Mat4::from_translation(position - p.position);
        let mut new = original.clone();
        new["sourceNode"] = Value::from(node.clone());
        new["worldMatrix"] =
            matrix_json(delta * matrix(&original["worldMatrix"]).ok_or("Invalid transform")?);
        let mut patches = vec![Patch {
            source: objects,
            pointer: format!("/objects/{count}"),
            before: None,
            after: Some(new),
        }];
        if let Some(models) = snapshot
            .parts
            .get("models")
            .filter(|models| !models.is_empty())
        {
            let before = self.sources[scene].draft["models"].clone();
            let mut after = before.clone();
            let rows = after.as_array_mut().ok_or("Missing destination models")?;
            for model in models {
                if let Some(existing) = rows.iter().find(|r| r["id"] == model["id"]) {
                    if existing != model {
                        return Err("Destination model identity has different content".into());
                    }
                } else {
                    rows.push(model.clone());
                }
            }
            if before != after {
                patches.push(Patch {
                    source: scene,
                    pointer: "/models".into(),
                    before: Some(before),
                    after: Some(after),
                });
            }
        }
        for collection in ["visuals", "colliders"] {
            let rows = self.sources[scene].draft[collection]
                .as_array()
                .ok_or("Missing scene parts")?;
            let mut index = rows.len();
            for row in snapshot.parts.get(collection).into_iter().flatten() {
                let mut new = row.clone();
                new["name"] =
                    Value::from(row["name"].as_str().unwrap().replace(source_node, &node));
                let t = &row["transform"]["translation"];
                let translation = Vec3::new(
                    t[0].as_f64().ok_or("Invalid translation")? as f32,
                    t[1].as_f64().unwrap_or(0.) as f32,
                    t[2].as_f64().unwrap_or(0.) as f32,
                );
                new["transform"]["translation"] =
                    serde_json::json!((translation + position - p.position).to_array());
                patches.push(Patch {
                    source: scene,
                    pointer: format!("/{collection}/{index}"),
                    before: None,
                    after: Some(new),
                });
                index += 1;
            }
        }
        self.commit(patches)?;
        self.selected = self
            .entities
            .iter()
            .position(|p| p.source == objects && p.pointer == format!("/objects/{count}"));
        self.selected_point = None;
        Ok(())
    }
    pub(super) fn refresh_guards(&mut self) -> Result<(), String> {
        let mut guards = BTreeMap::new();
        for s in &self.sources {
            if s.path.starts_with(&self.root)
                && s.base != s.draft
                && !s.draft.is_null()
                && s.path
                    .file_name()
                    .is_some_and(|p| p != "catalog.json" && p != "tile.json")
            {
                let bytes = terrain::source_bytes(&s.draft)?;
                let path = s
                    .path
                    .strip_prefix(&self.root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                guards.insert(
                    path,
                    (bytes.len(), blake3::hash(&bytes).to_hex().to_string()),
                );
            }
        }
        fn refresh(value: &mut Value, guards: &BTreeMap<String, (usize, String)>) {
            match value {
                Value::Object(map) => {
                    if let Some((size, hash)) = map
                        .get("path")
                        .and_then(Value::as_str)
                        .and_then(|p| guards.get(p))
                    {
                        map.insert("bytes".into(), Value::from(*size));
                        map.insert("blake3".into(), Value::from(hash.clone()));
                    }
                    for v in map.values_mut() {
                        refresh(v, guards);
                    }
                }
                Value::Array(rows) => {
                    for v in rows {
                        refresh(v, guards);
                    }
                }
                _ => {}
            }
        }
        for s in self
            .sources
            .iter_mut()
            .filter(|s| s.path.ends_with("tile.json"))
        {
            if s.draft.is_null(){continue;}
            if let Some(files)=s.draft["files"].as_array_mut() {
                let folder=s.path.parent().unwrap().strip_prefix(&self.root).unwrap().to_string_lossy().replace('\\',"/");
                for (path,(size,hash)) in guards.iter().filter(|(p,_)|p.starts_with(&format!("{folder}/"))) {
                    if !files.iter().any(|f|f["path"]==*path){files.push(serde_json::json!({"path":path,"bytes":size,"blake3":hash}));}
                }
            }
            refresh(&mut s.draft, &guards);
            if s.base != s.draft {
                let mut bytes = serde_json::to_vec_pretty(&s.draft).map_err(|e| e.to_string())?;
                bytes.push(b'\n');
                guards.insert(
                    s.path
                        .strip_prefix(&self.root)
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/"),
                    (bytes.len(), blake3::hash(&bytes).to_hex().to_string()),
                );
            }
        }
        for s in self
            .sources
            .iter_mut()
            .filter(|s| s.path.ends_with("catalog.json"))
        {
            refresh(&mut s.draft, &guards);
        }
        Ok(())
    }
}
