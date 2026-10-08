use super::*;

pub(super) fn update(
    commands: &mut Commands,
    e: &mut WorldEditor,
    preview: &mut WorldPreview,
    root: Entity,
) {
    let desired: Vec<_> = e.desired_tiles().into_iter().map(atlas::tile_id)
        .filter(|id|e.maps.get(id).is_none_or(|map|!e.sources[map.scene].draft["nativeTerrain"].is_null())).collect();
    for id in &desired {
        if preview.terrains.contains_key(id){continue;}
        if let Some(tile)=e.ground.get(id).filter(|t|t.authored.is_some()) {
            let terrain=Arc::new(tile.authored.as_ref().unwrap().clone());
            let entity=spawn_pending_native_heightmap(commands,root,format!("editor {id}"),Transform::from_matrix(tile.transform),terrain.clone());
            preview.terrain_data.insert(id.clone(),terrain);preview.terrain_applied.insert(id.clone(),0);preview.terrains.insert(id.clone(),entity);
        }
    }
    if preview.terrain_region != e.region {
        preview.terrain_region = e.region;
        preview.terrain_failed.clear();
    }
    let stale: Vec<_> = preview
        .terrains
        .keys()
        .filter(|id| !desired.contains(id))
        .cloned()
        .collect();
    for id in stale {
        preview.terrain_data.remove(&id);
        preview.terrain_applied.remove(&id);
        if let Some(entity) = preview.terrains.remove(&id) {
            commands.entity(entity).insert(Visibility::Hidden);
            preview.retiring.push(entity);
        }
    }
    // Completed stale jobs still occupy their slot until reaped; changing region cannot
    // start an unbounded number of terrain decodes.
    for i in 0..preview.terrain_jobs.len() {
        let ready =
            preview.terrain_jobs[i]
                .receiver
                .lock()
                .ok()
                .and_then(|rx| match rx.try_recv() {
                    Ok(result) => Some(result),
                    Err(mpsc::TryRecvError::Disconnected) => {
                        Some(Err("Terrain loader stopped".into()))
                    }
                    Err(mpsc::TryRecvError::Empty) => None,
                });
        if let Some(result) = ready {
            let job = preview.terrain_jobs.remove(i);
            if desired.contains(&job.tile) {
                match result {
                    Ok(terrain) => {
                        let terrain = Arc::new(terrain);
                        preview
                            .terrain_data
                            .insert(job.tile.clone(), terrain.clone());
                        preview.terrain_applied.insert(job.tile.clone(),0);
                        let entity = spawn_pending_native_heightmap(
                            commands,
                            root,
                            format!("editor {}", job.tile),
                            job.transform,
                            terrain,
                        );
                        preview.terrains.insert(job.tile, entity);
                        e.revision += 1;
                    }
                    Err(error) => {
                        preview.terrain_failed.insert(job.tile);
                        e.error(error);
                    }
                }
            }
            break;
        }
    }
    for id in desired {
        if preview.terrain_jobs.len() >= 2 {
            break;
        }
        if preview.terrains.contains_key(&id)
            || preview.terrain_failed.contains(&id)
            || preview.terrain_jobs.iter().any(|j| j.tile == id)
        {
            continue;
        }
        let Some(tile) = e.maps.get(&id) else {
            continue;
        };
        let document = &e.sources[tile.scene].draft;
        let result = (|| {
            let root: ffone_client::world::AuthoredWorldTransform =
                serde_json::from_value(document["root"].clone()).ok()?;
            let mut transform = root.try_to_bevy("editor terrain root").ok()?;
            let instance: ffone_client::world::NativeWorldTerrainInstance =
                serde_json::from_value(document["nativeTerrain"].clone()).ok()?;
            if let Some(chain) = &instance.root_chain {
                for node in chain.nodes.iter().rev() {
                    transform = transform.mul_transform(
                        node.native_local_transform
                            .try_to_bevy("editor terrain hierarchy")
                            .ok()?,
                    );
                }
            }
            transform =
                transform.mul_transform(instance.transform.try_to_bevy("editor terrain").ok()?);
            Some((instance, transform))
        })();
        let Some((instance, transform)) = result else {
            preview.terrain_failed.insert(id);
            continue;
        };
        let asset_root = e.root.clone();
        let environment = document["nativeTerrain"]["environment"]["blake3"]
            .as_str()
            .map(str::to_owned);
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let result = NativeTerrain::open_with_authoritative_environment(
                asset_root,
                &instance.path,
                &instance.blake3,
                environment.as_deref(),
            )
            .map_err(|e| e.to_string());
            let _ = tx.send(result);
        });
        preview.terrain_jobs.push(TerrainJob {
            tile: id,
            transform,
            receiver: Mutex::new(rx),
        });
    }
}
