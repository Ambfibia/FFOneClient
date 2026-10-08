//! Authored object models can be placed or replaced with their complete collider closure.
use super::*;
use model::{Patch, Placement};

pub(super) fn snapshot(
    root: &Path,
    entry: &objects::Entry,
) -> Result<operations::Clipboard, String> {
    let documents = stream::read_tile(root, &entry.tile, false)?;
    let tile = map::Tile {
        id: entry.tile.clone(),
        objects: 0,
        scene: 1,
        native_scene: documents.native_scene,
    };
    let mut placements = vec![];
    tile.append_placements(&documents.sources, &mut placements);
    let p = placements
        .into_iter()
        .find(|p| {
            documents.sources[0]
                .draft
                .pointer(&p.pointer)
                .is_some_and(|r| r["sourceNode"] == entry.node)
        })
        .ok_or("Missing model source")?;
    let row = documents.sources[0]
        .draft
        .pointer(&p.pointer)
        .unwrap()
        .clone();
    let scene = &documents.sources[1].draft;
    let mut parts = BTreeMap::new();
    for collection in ["visuals", "colliders"] {
        parts.insert(
            collection.to_owned(),
            scene[collection]
                .as_array()
                .into_iter()
                .flatten()
                .filter(|r| {
                    r["name"]
                        .as_str()
                        .is_some_and(|n| map::matches_node(n, &entry.node))
                })
                .cloned()
                .collect::<Vec<_>>(),
        );
    }
    let models: BTreeSet<_> = parts
        .values()
        .flatten()
        .filter_map(|p| p["model"].as_str())
        .collect();
    let definitions = scene["models"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|m| m["id"].as_str().is_some_and(|i| models.contains(i)))
        .cloned()
        .collect();
    parts.insert("models".into(), definitions);
    Ok(operations::Clipboard {
        placement: p,
        row,
        parts,
    })
}
impl WorldEditor {
    pub(super) fn replace_object_model(
        &mut self,
        p: &Placement,
        template: operations::Clipboard,
    ) -> Result<(), String> {
        let tile = self
            .maps
            .values()
            .find(|t| t.objects == p.source)
            .ok_or("Open the selected object's region")?;
        let before = self.sources[p.source]
            .draft
            .pointer(&p.pointer)
            .ok_or("Missing object")?
            .clone();
        let node = before["sourceNode"]
            .as_str()
            .ok_or("Missing object identity")?;
        let source_node = template.row["sourceNode"]
            .as_str()
            .ok_or("Missing model identity")?;
        let origin = template
            .placement
            .key
            .split('/')
            .next()
            .ok_or("Missing model tile")?;
        if map::has_behaviour(&self.root, &tile.id, node)?
            || map::has_behaviour(&self.root, origin, source_node)?
        {
            return Err("Scripted objects must retain their behaviour closure".into());
        }
        let old = map::matrix(&before["worldMatrix"]).ok_or("Invalid object matrix")?;
        let source = map::matrix(&template.row["worldMatrix"]).ok_or("Invalid model matrix")?;
        let (_, q, _) = old.to_scale_rotation_translation();
        let (_, source_q, _) = source.to_scale_rotation_translation();
        let delta = Mat4::from_translation(p.position)
            * Mat4::from_quat(q * source_q.inverse())
            * Mat4::from_translation(-template.placement.position);
        let mut after = template.row.clone();
        after["sourceNode"] = Value::from(node);
        after["worldMatrix"] = matrix_value(delta * source);
        let mut patches = vec![Patch {
            source: p.source,
            pointer: p.pointer.clone(),
            before: Some(before.clone()),
            after: Some(after),
        }];
        for collection in ["visuals", "colliders"] {
            let before = self.sources[tile.scene].draft[collection]
                .as_array()
                .ok_or("Missing model collection")?
                .clone();
            let mut after: Vec<_> = before
                .iter()
                .filter(|r| {
                    !r["name"]
                        .as_str()
                        .is_some_and(|n| map::matches_node(n, node))
                })
                .cloned()
                .collect();
            for part in template.parts.get(collection).into_iter().flatten() {
                let mut part = part.clone();
                let name = part["name"].as_str().ok_or("Missing model part identity")?;
                part["name"] =
                    Value::from(name.replace(&format!("[{source_node}"), &format!("[{node}")));
                let t: ffone_client::world::AuthoredWorldTransform =
                    serde_json::from_value(part["transform"].clone()).map_err(|e| e.to_string())?;
                let mut t = t.try_to_bevy(node).map_err(|e| e.to_string())?;
                t.translation = delta.transform_point3(t.translation);
                t.rotation = q * source_q.inverse() * t.rotation;
                part["transform"] = serde_json::json!({"translation":t.translation.to_array(),"rotation":t.rotation.to_array(),"scale":t.scale.to_array()});
                after.push(part);
            }
            patches.push(Patch {
                source: tile.scene,
                pointer: format!("/{collection}"),
                before: Some(Value::Array(before)),
                after: Some(Value::Array(after)),
            });
        }
        let before = self.sources[tile.scene].draft["models"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let mut after = before.clone();
        for model in template.parts.get("models").into_iter().flatten() {
            if let Some(existing) = after.iter().find(|m| m["id"] == model["id"]) {
                if existing != model {
                    return Err("Conflicting model identity".into());
                }
            } else {
                after.push(model.clone());
            }
        }
        patches.push(Patch {
            source: tile.scene,
            pointer: "/models".into(),
            before: Some(Value::Array(before)),
            after: Some(Value::Array(after)),
        });
        self.commit(patches)
    }
}
fn matrix_value(m: Mat4) -> Value {
    let v = m.to_cols_array();
    Value::Array(
        (0..4)
            .map(|r| {
                Value::Array(
                    (0..4)
                        .map(|c| Value::from(v[c * 4 + r].to_string()))
                        .collect(),
                )
            })
            .collect(),
    )
}
