//! Region admission is limited to nine tiles, two I/O jobs and one install per frame.
use super::*;
use std::sync::{Mutex, mpsc};

pub(super) struct TileDocuments {
    pub sources: Vec<model::Source>,
    pub native_scene: Option<ffone_client::world::NativeWorldScene>,
    pub ground: Option<ground::Tile>,
}
pub(super) fn read_tile(
    root: &Path,
    id: &str,
    include_catalog: bool,
) -> Result<TileDocuments, String> {
    let mut sources = Vec::new();
    let mut paths = vec![
        format!("map/tiles/{id}/objects.json"),
        format!("map/tiles/{id}/scene.json"),
        format!("map/tiles/{id}/tile.json"),
    ];
    if include_catalog {
        paths.push("map/catalog.json".into());
    }
    for rel in paths {
        let path = root.join(rel);
        let base: Value = serde_json::from_slice(
            &fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?,
        )
        .map_err(|e| e.to_string())?;
        sources.push(model::Source {
            path,
            draft: base.clone(),
            base,
        });
    }
    let native_scene = serde_json::from_value(sources[1].draft.clone()).ok();
    Ok(TileDocuments {
        sources,
        native_scene,
        ground: ground::Tile::load(root, id).ok(),
    })
}
struct Job {
    id: String,
    receiver: Mutex<mpsc::Receiver<Result<TileDocuments, String>>>,
}
#[derive(Default, Resource)]
pub(super) struct TileStreaming {
    jobs: Vec<Job>,
    failed: BTreeSet<String>,
    region: Option<[i32; 2]>,
    was_3d: bool,
}
pub(super) fn update(
    mut e: ResMut<WorldEditor>,
    state: Res<EditorState>,
    mut stream: ResMut<TileStreaming>,
    window: Single<&Window>,
) {
    let entered_3d = state.world_open == Some(true) && !stream.was_3d;
    stream.was_3d = state.world_open == Some(true);
    if state.world_open.is_none() {
        return;
    }
    let camera_in_region = atlas::tile_at(e.center).is_some_and(|t| e.desired_tiles().contains(&t));
    if state.world_open == Some(true)
        && (e.region.is_none() || (entered_3d && !camera_in_region))
        && !e.map_picker
    {
        e.map_picker = true;
        atlas::fit(
            &mut e,
            view::rect(Vec2::new(window.width(), window.height())).1,
        );
    }
    if stream.region != e.region {
        stream.region = e.region;
        stream.failed.clear();
    }
    let desired: Vec<_> = e.desired_tiles().into_iter().map(atlas::tile_id).collect();
    // A drag holds placement indices. Install only after it ends, preserving one undo action.
    if e.drag.is_none() && e.brush.is_none() && e.grass_stroke.is_none() {
        for i in 0..stream.jobs.len() {
            let ready = stream.jobs[i]
                .receiver
                .lock()
                .ok()
                .and_then(|rx| match rx.try_recv() {
                    Ok(result) => Some(result),
                    Err(mpsc::TryRecvError::Disconnected) => Some(Err("Map loader stopped".into())),
                    Err(mpsc::TryRecvError::Empty) => None,
                });
            if let Some(result) = ready {
                let job = stream.jobs.remove(i);
                if desired.contains(&job.id) {
                    match result {
                        Ok(documents) => {
                            if documents.ground.is_none() {
                                stream.failed.insert(job.id.clone());
                            }
                            e.install_tile(job.id, documents);
                        }
                        Err(error) => {
                            stream.failed.insert(job.id);
                            e.error(error);
                        }
                    }
                }
                break;
            }
        }
    }
    if e.map_picker {
        return;
    }
    for id in desired {
        if stream.jobs.len() >= 2 {
            break;
        }
        if (e.maps.contains_key(&id) && e.ground.contains_key(&id))
            || stream.failed.contains(&id)
            || stream.jobs.iter().any(|j| j.id == id)
        {
            continue;
        }
        let root = e.root.clone();
        let include_catalog = !e
            .sources
            .iter()
            .any(|s| s.path == root.join("map/catalog.json"));
        let job_id = id.clone();
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(read_tile(&root, &job_id, include_catalog));
        });
        stream.jobs.push(Job {
            id,
            receiver: Mutex::new(rx),
        });
    }
}
