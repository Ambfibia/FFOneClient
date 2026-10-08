//! Keyword search reads names/transforms in the background without loading meshes.
use super::*;
use std::{
    io::BufReader,
    sync::{Mutex, mpsc},
};

#[derive(Clone)]
pub(super) struct Entry {
    pub tile: String,
    pub node: String,
    pub name: String,
    pub position: Vec3,
    pub asset: String,
    pub model: Option<String>,
}
#[derive(Default, Resource)]
pub(super) struct Search {
    started: bool,
    receiver: Option<Mutex<mpsc::Receiver<Result<Vec<Entry>, String>>>>,
}
impl Search {
    pub(super) fn loading(&self) -> bool {
        self.receiver.is_some()
    }
}
#[derive(serde::Deserialize)]
struct Scene {
    visuals: Vec<Visual>,
    #[serde(default)]
    models: Vec<Model>,
}
#[derive(serde::Deserialize)]
struct Visual {
    name: String,
    model: Option<String>,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Model {
    id: String,
    root_name: Option<String>,
    path: Option<String>,
}
#[derive(serde::Deserialize)]
struct Objects {
    objects: Vec<Object>,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Object {
    #[serde(default)]
    object: String,
    source_node: String,
    world_matrix: Value,
}

pub(super) fn read(root: &Path, tiles: BTreeSet<[i32; 2]>) -> Result<Vec<Entry>, String> {
    let mut entries = vec![];
    for tile in tiles {
        let tile = atlas::tile_id(tile);
        let folder = root.join(format!("map/tiles/{tile}"));
        let scene: Scene = serde_json::from_reader(BufReader::new(
            fs::File::open(folder.join("scene.json")).map_err(|e| e.to_string())?,
        ))
        .map_err(|e| e.to_string())?;
        let models: BTreeMap<_, _> = scene
            .models
            .iter()
            .filter_map(|m| Some((m.id.as_str(), m.root_name.as_deref().or(m.path.as_deref())?)))
            .collect();
        let names: BTreeMap<_, _> = scene
            .visuals
            .iter()
            .filter_map(|v| {
                let (name, tail) = v.name.rsplit_once(" [")?;
                Some((
                    tail.split([' ', ']']).next()?,
                    map::object_label(
                        name,
                        v.model.as_deref().and_then(|id| models.get(id).copied()),
                    ),
                ))
            })
            .collect();
        let paths: BTreeMap<_, _>=scene.visuals.iter().filter_map(|v| {
            let (_,tail)=v.name.rsplit_once(" [")?;
            let model=scene.models.iter().find(|m|Some(m.id.as_str())==v.model.as_deref())?;
            Some((tail.split([' ',']']).next()?, model.path.clone()?))
        }).collect();
        let objects: Objects = serde_json::from_reader(BufReader::new(
            fs::File::open(folder.join("objects.json")).map_err(|e| e.to_string())?,
        ))
        .map_err(|e| e.to_string())?;
        for object in objects.objects {
            let Some(matrix) = map::matrix(&object.world_matrix) else {
                continue;
            };
            entries.push(Entry {
                tile: tile.clone(),
                name: names
                    .get(object.source_node.as_str())
                    .cloned()
                    .unwrap_or_else(|| object.source_node.clone()),
                model: paths.get(object.source_node.as_str()).cloned(),
                asset: object.object,
                node: object.source_node,
                position: matrix.w_axis.truncate(),
            });
        }
    }
    Ok(entries)
}
impl WorldEditor {
    pub(super) fn remote_objects(&self) -> Vec<usize> {
        if self.list_mode != 1 || self.search.trim().is_empty() || self.instance != 0 {
            return vec![];
        }
        let needle = self.search.trim().to_lowercase();
        self.object_index
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                !self.maps.contains_key(&p.tile)
                    && format!("{} {}", p.name, p.node)
                        .to_lowercase()
                        .contains(&needle)
            })
            .map(|(i, _)| i)
            .collect()
    }
    pub(super) fn select_remote_object(&mut self, index: usize) -> Result<(), String> {
        let p = self
            .object_index
            .get(index)
            .ok_or("Missing search result")?;
        let tile = p.tile.clone();
        let node = p.node.clone();
        let position = p.position;
        self.select_region(atlas::tile_at(position).ok_or("Invalid object position")?)?;
        self.center = position;
        self.map_picker = false;
        self.pending_object = Some((tile, node));
        self.edit_objects = true;
        self.revision += 1;
        Ok(())
    }
}
pub(super) fn update(
    mut e: ResMut<WorldEditor>,
    mut search: ResMut<Search>,
    state: Res<EditorState>,
) {
    if state.world_open.is_none() {
        return;
    }
    if !search.started && ((e.list_mode == 1 && !e.search.trim().is_empty()) || e.type_picker.as_ref().is_some_and(|p|p.kind==4)) {
        search.started = true;
        e.revision += 1;
        let root = e.root.clone();
        let tiles = e.available_tiles.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(read(&root, tiles));
        });
        search.receiver = Some(Mutex::new(rx));
    }
    let ready = search
        .receiver
        .as_ref()
        .and_then(|rx| rx.lock().ok()?.try_recv().ok());
    if let Some(result) = ready {
        search.receiver = None;
        match result {
            Ok(entries) => {
                e.object_index = entries;
                e.refresh_model_picker();
                e.revision += 1;
            }
            Err(error) => e.error(error),
        }
    }
    if let Some((tile, node)) = e.pending_object.clone()
        && let Some(map) = e.maps.get(&tile)
    {
        if let Some(index) = e.entities.iter().position(|p| {
            p.source == map.objects
                && e.sources[p.source]
                    .draft
                    .pointer(&p.pointer)
                    .is_some_and(|v| v["sourceNode"] == node)
        }) {
            e.select_entity(index);
            e.pending_object = None;
            e.revision += 1;
        }
    }
}
