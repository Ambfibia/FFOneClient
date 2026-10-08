//! Publish the shared XDT and mission destinations together with client content.
use super::*;
use sha2::{Digest, Sha256};

pub(super) struct Publication { pub path: PathBuf, pub(super) digest: Vec<u8> }

impl Publication {
    pub(super) fn open(path: PathBuf) -> Result<Self, String> {
        let bytes = fs::read(&path).map_err(|_| "Selected folder has no readable server xdt.json".to_owned())?;
        let document = ffone_client::xdt::from_slice(&bytes)?;
        if !document["tables"].as_array().into_iter().flatten().any(|t|
            t["value"]["m_pMissionTable"]["m_pMissionData"].is_array()) {
            return Err("Selected xdt.json has no mission table".into());
        }
        Ok(Self { path, digest: Sha256::digest(bytes).to_vec() })
    }
    pub fn discover(root: &Path) -> Option<Self> {
        if let Ok(bytes) = fs::read(mission_server::settings_path(root)) {
            if let Ok(settings) = serde_json::from_slice::<Value>(&bytes) {
                if let Some(path) = settings["xdt_path"].as_str() {
                    // A saved destination is authoritative. An unavailable path
                    // must not redirect writes to an automatically found server.
                    return Self::open(PathBuf::from(path)).ok();
                }
            }
        }
        let client = root.parent()?.parent()?;
        let parent = client.parent()?;
        for path in [parent.join("Server/tabledata/xdt.json"), parent.join("RustyFusion/tabledata/xdt.json")] {
            if let Ok(bytes) = fs::read(&path) {
                return Some(Self { path, digest: Sha256::digest(bytes).to_vec() });
            }
        }
        None
    }
    fn check(&self) -> Result<(), String> {
        let bytes = fs::read(&self.path).map_err(|e| e.to_string())?;
        if Sha256::digest(bytes).as_slice() != self.digest {
            return Err("Server TableData changed externally; reopen the editor before saving".into());
        }
        Ok(())
    }
    fn remember(&mut self) -> Result<(), String> {
        self.digest = Sha256::digest(fs::read(&self.path).map_err(|e| e.to_string())?).to_vec();
        Ok(())
    }
}

pub(super) fn changed_tasks<'a>(before: &'a Value, after: &'a Value) -> Vec<&'a Value> {
    let mut result = Vec::new();
    for table in after["tables"].as_array().into_iter().flatten() {
        let old = before["tables"].as_array().into_iter().flatten().find(|t| t["name"] == table["name"]);
        let old_rows = old.map(|t| &t["value"]["m_pMissionTable"]["m_pMissionData"]);
        for task in table["value"]["m_pMissionTable"]["m_pMissionData"].as_array().into_iter().flatten() {
            if task["m_iHTaskID"].as_i64().is_none_or(|id| id <= 0) { continue; }
            let previous = old_rows.and_then(Value::as_array)
                .and_then(|rows| rows.iter().find(|r| r["m_iHTaskID"] == task["m_iHTaskID"]));
            if previous != Some(task) { result.push(task); }
        }
    }
    result
}

pub(super) fn validate_tasks(before: &Value, after: &Value) -> Result<(), String> {
    for task in changed_tasks(before, after) {
        for (text, speaker) in mission_events::PAIRS {
            if task[*text].as_i64().is_some_and(|id| id > 0)
                && task[*speaker].as_i64().unwrap_or(0) <= 0 {
                return Err(schema::task_error(task, format!("{speaker}: mission_speaker")));
            }
        }
        for slot in 0..3 {
            if task["m_iSTItemDropRate"][slot].as_i64().is_some_and(|rate| rate > 0) {
                if task["m_iSTItemID"][slot].as_i64().unwrap_or(0) <= 0 {
                    return Err(schema::task_error(task, "m_iSTItemID: mission_drop_item"));
                }
                if !task["m_iCSUEnemyID"].as_array().into_iter().flatten().any(|id| id.as_i64().is_some_and(|id| id > 0)) {
                    return Err(schema::task_error(task, "m_iCSUEnemyID: mission_drop_enemy"));
                }
            }
        }
    }
    Ok(())
}

impl XdtEditor {
    pub(super) fn prepare_waypoints(&self, document: &Value) -> Result<Option<(PathBuf, Value)>, String> {
        let root = self.path.parent().and_then(Path::parent).and_then(Path::parent).ok_or("Missing asset root")?;
        let path = root.join(ffone_client::world_mission_indicators::CLIENT_NPC_WAYPOINT_CATALOG_PATH);
        // Minimal unit fixtures have no world; production catalogs retain schema,
        // finite positions, ordered rows and their original first-match behavior.
        if !path.exists() { return Ok(None); }
        let catalog: Value = serde_json::from_slice(&fs::read(&path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let migrating = catalog["schema"] != ffone_client::world_mission_indicators::CLIENT_NPC_WAYPOINT_CATALOG_SCHEMA;
        let mut catalog = ffone_client::world_mission_indicators::ClientNpcWaypointCatalog::into_native_document(catalog)?;
        let mut needed = BTreeSet::new();
        let mut sources = BTreeMap::new();
        // Appended authoring destinations follow the selected server's placement
        // coordinates when available, even when the task itself was not edited.
        for row in catalog["rows"].as_array().into_iter().flatten()
            .skip(ffone_client::world_mission_indicators::CLIENT_NPC_WAYPOINT_CATALOG_ROW_COUNT) {
            if let Some(id) = row["npcType"].as_i64() { needed.insert(id); }
        }
        for table in document["tables"].as_array().into_iter().flatten() {
            let old = self.base["tables"].as_array().into_iter().flatten().find(|old| old["name"] == table["name"]);
            let old_rows = old.and_then(|old| old["value"]["m_pMissionTable"]["m_pMissionData"].as_array());
            for task in table["value"]["m_pMissionTable"]["m_pMissionData"].as_array().into_iter().flatten() {
                if task["m_iHTaskID"].as_i64().is_none_or(|id| id <= 0) { continue; }
                let previous = old_rows.and_then(|rows| rows.iter().find(|row| row["m_iHTaskID"] == task["m_iHTaskID"]));
                for field in ["m_iHNPCID", "m_iHTerminatorNPCID", "m_iSTGrantWayPoint"] {
                    if previous.is_some_and(|previous| previous[field] == task[field]) { continue; }
                    if let Some(id) = task[field].as_i64().filter(|id| *id > 0) {
                        needed.insert(id);
                        sources.entry(id).or_insert(task);
                    }
                }
            }
        }
        let positions = self.publication.as_ref().map(|p| p.path.parent().unwrap().join("NPCs.json"))
            .filter(|p| p.exists()).map(|p| fs::read(p).map_err(|e| e.to_string())
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).map_err(|e| e.to_string()))).transpose()?;
        let mut placements: Vec<_> = positions.as_ref().and_then(|p| p["NPCs"].as_object())
            .into_iter().flatten().collect();
        placements.sort_by_key(|(id, _)| id.parse::<u64>().unwrap_or(u64::MAX));
        let rows = catalog["rows"].as_array_mut().ok_or("Missing NPC waypoint rows")?;
        let mut changed = migrating;
        for id in needed {
            let with_context = |error: String| sources.get(&id)
                .map(|task| schema::task_error(task, &error)).unwrap_or(error);
            let existing = rows.iter().position(|row| row["npcType"].as_i64() == Some(id));
            if existing.is_some_and(|i| i < ffone_client::world_mission_indicators::CLIENT_NPC_WAYPOINT_CATALOG_ROW_COUNT) { continue; }
            let Some((_, placement)) = placements.iter().find(|(_, p)| p["iNPCType"].as_i64() == Some(id)) else {
                // Unrelated edits may use a different server or work offline. Keep
                // valid authored coordinates; only newly assigned targets require
                // a placement on the selected server before publication.
                if existing.is_some() && !sources.contains_key(&id) { continue; }
                return Err(with_context(format!("Mission NPC has no world placement: {id}")));
            };
            let xyz: Vec<f64> = ["iX", "iZ", "iY"].iter().map(|field| placement[*field].as_f64()
                .filter(|n| n.is_finite()).map(|n| n * 0.01).ok_or_else(|| with_context("Invalid NPC position".to_owned()))).collect::<Result<_, _>>()?;
            if let Some(index) = existing {
                let position = serde_json::json!(xyz);
                if rows[index]["clientPosition"] != position {
                    rows[index]["clientPosition"] = position;
                    changed = true;
                }
            } else {
                rows.push(serde_json::json!({"npcType": id, "clientPosition": xyz}));
                changed = true;
            }
        }
        if !changed { return Ok(None); }
        ffone_client::world_mission_indicators::ClientNpcWaypointCatalog::from_json_bytes(
            &serde_json::to_vec(&catalog).map_err(|e| e.to_string())?)?;
        Ok(Some((path, catalog)))
    }

    pub(super) fn publish_files(&mut self, published: &Value, locales: &[(PathBuf, Value)],
        waypoint: Option<(PathBuf, Value)>) -> Result<(), String> {
        if self.publication.is_none() {
            let root = self.path.parent().and_then(Path::parent).and_then(Path::parent).ok_or("Missing asset root")?;
            let selected = fs::read(mission_server::settings_path(root)).ok()
                .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
                .is_some_and(|settings| settings["xdt_path"].is_string());
            if selected { return Err("Selected folder has no readable server xdt.json".into()); }
        }
        if let Some(server) = &self.publication { server.check()?; }
        let encode = |value: &Value| -> Result<Vec<u8>, String> {
            let mut bytes = serde_json::to_vec_pretty(value).map_err(|e|e.to_string())?;
            bytes.push(b'\n'); Ok(bytes)
        };
        // Serialize XDT once. Both destinations receive the exact same accepted bytes.
        let current = fs::read(&self.path).map_err(|e|e.to_string())?;
        let xdt = if serde_json::from_slice::<Value>(&current).ok().as_ref()==Some(published) { current } else { encode(published)? };
        let mut files = vec![(self.path.clone(), xdt.clone())];
        if let Some(server) = &self.publication { files.push((server.path.clone(), xdt)); }
        for (path, value) in locales { files.push((path.clone(), encode(value)?)); }
        if let Some((path, value)) = waypoint { files.push((path, encode(&value)?)); }
        // Stage every file before replacing any destination. A failed replacement
        // restores completed destinations; the editor keeps the drafts for retry.
        let mut staged = Vec::new();
        let mut destinations = BTreeSet::new();
        for (path, bytes) in &files {
            if !destinations.insert(path.clone()) { continue; }
            let backup = fs::read(path).map_err(|e| e.to_string())?;
            if backup==*bytes { continue; }
            let mut temp = tempfile::NamedTempFile::new_in(path.parent().unwrap()).map_err(|e| e.to_string())?;
            {
                let mut writer=std::io::BufWriter::with_capacity(64*1024,&mut temp);
                writer.write_all(bytes).map_err(|e| e.to_string())?;
                writer.flush().map_err(|e|e.to_string())?;
            }
            temp.as_file().sync_all().map_err(|e| e.to_string())?;
            staged.push((path.clone(), backup, temp));
        }
        let mut completed: Vec<(PathBuf, Vec<u8>)> = Vec::new();
        for (path, backup, temp) in staged {
            backups::retain(&path)?;
            if let Err(error) = temp.persist(&path) {
                let mut failures = Vec::new();
                for (path, bytes) in completed.into_iter().rev() {
                    if let Err(e) = fs::write(&path, bytes) { failures.push(format!("{}: {e}", path.display())); }
                }
                return Err(format!("Save failed ({}): {error}; rollback: {}", path.display(), failures.join(", ")));
            }
            completed.push((path, backup));
        }
        if let Some(server) = &mut self.publication { server.remember()?; }
        Ok(())
    }
}
