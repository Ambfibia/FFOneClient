//! Production regression: relocated native textures remain reachable from GLB
//! image slots and native material/mip metadata, with no retired root references.
use std::{
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

fn normalized(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    for part in path.components() {
        match part {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            _ => result.push(part.as_os_str()),
        }
    }
    result
}

fn visit(value: &serde_json::Value, parent: &Path, root: &Path, count: &mut usize) {
    match value {
        serde_json::Value::Object(values) => {
            for value in values.values() {
                visit(value, parent, root, count);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                visit(value, parent, root, count);
            }
        }
        serde_json::Value::String(uri) if uri.ends_with(".png") && uri.contains("/textures/") => {
            let resolved = normalized(&parent.join(uri));
            assert!(
                !resolved.starts_with(root.join("textures")),
                "retired texture route: {uri}"
            );
            assert!(
                resolved.starts_with(root),
                "texture escapes the native root: {uri}"
            );
            assert!(
                resolved.is_file(),
                "missing native texture: {}",
                resolved.display()
            );
            *count += 1;
        }
        _ => {}
    }
}

#[test]
fn production_glb_texture_and_mip_routes_survive_domain_relocation() {
    let root = normalized(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game"));
    assert!(!root.join("textures").exists());
    let mut directories = vec![root.clone()];
    let mut count = 0;
    while let Some(directory) = directories.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                directories.push(path);
            } else if path.extension().is_some_and(|extension| extension == "glb") {
                let mut file = fs::File::open(&path).unwrap();
                let mut header = [0; 20];
                file.read_exact(&mut header).unwrap();
                let length = u32::from_le_bytes(header[12..16].try_into().unwrap()) as usize;
                let mut json = vec![0; length];
                file.read_exact(&mut json).unwrap();
                let value = serde_json::from_slice(&json).unwrap();
                visit(&value, path.parent().unwrap(), &root, &mut count);
            }
        }
    }
    assert!(
        count > 1000,
        "production shared texture references were not inspected"
    );
}
