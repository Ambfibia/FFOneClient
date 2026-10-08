//! Collision visibility is editor-only; enabled state is authored and consumed at runtime.
use super::*;
impl WorldEditor {
    pub(super) fn object_collision(&self, p: &model::Placement) -> bool {
        let Some(tile) = self.maps.values().find(|t| t.objects == p.source) else { return false; };
        let Some(node) = self.sources[p.source].draft.pointer(&p.pointer).and_then(|r| r["sourceNode"].as_str()) else { return false; };
        self.sources[tile.scene].draft["colliders"].as_array().into_iter().flatten().any(|c|
            c["name"].as_str().is_some_and(|n| map::matches_node(n,node)) && c["enabled"].as_bool().unwrap_or(true))
    }
    pub(super) fn toggle_object_collision(&mut self) -> Result<(), String> {
        let p = self.selected().cloned().ok_or("Select an object")?;
        if p.kind != 4 { return Err("Select an object".into()); }
        let enabled = !self.object_collision(&p);
        let tile = self.maps.values().find(|t| t.objects == p.source).ok_or("Load the object's tile")?;
        let source = tile.scene;
        let node = self.sources[p.source].draft.pointer(&p.pointer).and_then(|r| r["sourceNode"].as_str()).ok_or("Missing object identity")?;
        let mut patches = Vec::new();
        for (i,c) in self.sources[source].draft["colliders"].as_array().into_iter().flatten().enumerate() {
            if c["name"].as_str().is_some_and(|n| map::matches_node(n,node)) {
                patches.push(model::Patch { source, pointer:format!("/colliders/{i}/enabled"), before:c.get("enabled").cloned(), after:Some(Value::from(enabled)) });
            }
        }
        if patches.is_empty() && enabled {
            let before = self.sources[source].draft["colliders"].clone();
            let mut after = before.clone();
            let scene = &self.sources[source].draft;
            for visual in scene["visuals"].as_array().into_iter().flatten().filter(|v|v["name"].as_str().is_some_and(|n|map::matches_node(n,node))) {
                let model = scene["models"].as_array().and_then(|rows|rows.iter().find(|m|m["id"]==visual["model"])).ok_or("Missing visual model")?;
                let path = model["path"].as_str().ok_or("Missing model path")?;
                let glb = gltf::Gltf::open(self.root.join(path)).map_err(|e|e.to_string())?;
                for mesh in glb.meshes() {
                    for primitive in mesh.primitives().filter(|p|p.mode()==gltf::mesh::Mode::Triangles) {
                        let vertices=primitive.get(&gltf::Semantic::Positions).ok_or("Missing collision vertices")?.count();
                        let indices=primitive.indices().map_or(vertices,|a|a.count());
                        after.as_array_mut().ok_or("Missing colliders")?.push(serde_json::json!({
                            "name":format!("FFOne object [{node} collision {} {}]",mesh.index(),primitive.index()),
                            "kind":"triangleMesh","model":visual["model"],"mesh":mesh.index(),"primitive":primitive.index(),
                            "isTrigger":false,"enabled":true,"expectedVertexCount":vertices,"expectedIndexCount":indices,"transform":visual["transform"]
                        }));
                    }
                }
            }
            if after == before { return Err("This object has no collision or visual mesh geometry".into()); }
            patches.push(model::Patch {source,pointer:"/colliders".into(),before:Some(before),after:Some(after)});
        }
        self.commit(patches)
    }
}
