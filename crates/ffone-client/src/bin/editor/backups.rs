//! Three recoverable pre-save snapshots beside each authored file.
use std::{fs, path::Path};

pub(super) fn retain(path: &Path) -> Result<(), String> {
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return Ok(());
    };
    if !matches!(
        name,
        "xdt.json" | "en.json" | "ru.json" | "client-npc-waypoints.json" | "NPCs.json" | "paths.json"
    ) || !path.exists()
    {
        return Ok(());
    }
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    let folder = path
        .parent()
        .ok_or("Missing backup directory")?
        .join(".ffone-backups")
        .join(name);
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    if fs::read(folder.join("1.bak")).ok().as_deref() == Some(bytes.as_slice()) {
        return Ok(());
    }
    for index in (1..3).rev() {
        let previous = folder.join(format!("{index}.bak"));
        if previous.exists() {
            fs::copy(previous, folder.join(format!("{}.bak", index + 1)))
                .map_err(|e| e.to_string())?;
        }
    }
    fs::write(folder.join("1.bak"), bytes).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keeps_three_distinct_pre_save_versions() {
        let folder = tempfile::tempdir().unwrap();
        let path = folder.path().join("xdt.json");
        for version in 0..5 {
            fs::write(&path, version.to_string()).unwrap();
            retain(&path).unwrap();
            retain(&path).unwrap();
        }
        let backups = folder.path().join(".ffone-backups/xdt.json");
        for (index, version) in [(1, "4"), (2, "3"), (3, "2")] {
            assert_eq!(
                fs::read_to_string(backups.join(format!("{index}.bak"))).unwrap(),
                version
            );
        }
        assert_eq!(fs::read_dir(backups).unwrap().count(), 3);
    }
}
