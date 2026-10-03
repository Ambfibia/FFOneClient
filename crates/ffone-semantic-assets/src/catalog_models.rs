use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GlbProof {
    pub path: String,
    pub byte_exact_copy: bool,
    pub nodes: u64,
    pub meshes: u64,
    pub skins: u64,
    pub joints: u64,
    pub animations: u64,
    pub external_uris: Vec<String>,
    pub uri_safe: bool,
}

pub(super) fn glb_uris_are_safe(
    route: &SemanticRoute,
    uris: &[String],
    route_by_source: &BTreeMap<&str, &SemanticRoute>,
) -> Result<bool> {
    let source_parent = Path::new(&route.source_path)
        .parent()
        .unwrap_or(Path::new(""));
    let destination_parent = Path::new(&route.destination_path)
        .parent()
        .unwrap_or(Path::new(""));
    for uri in uris {
        if uri.starts_with("data:") {
            continue;
        }
        validate_relative_path(uri)?;
        let source_dependency = slash_path(&source_parent.join(native_path(uri)));
        let destination_dependency = slash_path(&destination_parent.join(native_path(uri)));
        let Some(dependency_route) = route_by_source.get(source_dependency.as_str()) else {
            return Ok(false);
        };
        if dependency_route.destination_path != destination_dependency {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn inspect_glb(path: &Path, destination: &str) -> Result<GlbProof> {
    let bytes = read_bytes(path)?;
    if bytes.len() < 20 || &bytes[0..4] != b"glTF" {
        return Err(SemanticAssetError::Verification(format!(
            "{} is not a GLB 2.0 file",
            path.display()
        )));
    }
    let version = u32::from_le_bytes(bytes[4..8].try_into().expect("four bytes"));
    let declared_len = u32::from_le_bytes(bytes[8..12].try_into().expect("four bytes")) as usize;
    if version != 2 || declared_len != bytes.len() {
        return Err(SemanticAssetError::Verification(format!(
            "{} has invalid GLB header",
            path.display()
        )));
    }
    let mut cursor = 12;
    let mut json = None;
    while cursor + 8 <= bytes.len() {
        let chunk_len =
            u32::from_le_bytes(bytes[cursor..cursor + 4].try_into().expect("four bytes")) as usize;
        let chunk_type = u32::from_le_bytes(
            bytes[cursor + 4..cursor + 8]
                .try_into()
                .expect("four bytes"),
        );
        cursor += 8;
        if cursor + chunk_len > bytes.len() {
            return Err(SemanticAssetError::Verification(format!(
                "{} has a truncated GLB chunk",
                path.display()
            )));
        }
        if chunk_type == 0x4E4F_534A {
            let text = std::str::from_utf8(&bytes[cursor..cursor + chunk_len])
                .map_err(|_| {
                    SemanticAssetError::Verification(format!(
                        "{} GLB JSON is not UTF-8",
                        path.display()
                    ))
                })?
                .trim_end_matches([' ', '\0']);
            json = Some(serde_json::from_str::<Value>(text).map_err(|source| {
                SemanticAssetError::Json {
                    path: path.to_owned(),
                    source,
                }
            })?);
        }
        cursor += chunk_len;
    }
    let json = json.ok_or_else(|| {
        SemanticAssetError::Verification(format!("{} has no GLB JSON chunk", path.display()))
    })?;
    let mut external_uris = Vec::new();
    collect_uris(&json, &mut external_uris);
    external_uris.sort();
    external_uris.dedup();
    let joints = json
        .get("skins")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|skin| skin.get("joints").and_then(Value::as_array))
        .map(|joints| joints.len() as u64)
        .sum();
    Ok(GlbProof {
        path: destination.to_owned(),
        byte_exact_copy: false,
        nodes: json_array_len(&json, "nodes"),
        meshes: json_array_len(&json, "meshes"),
        skins: json_array_len(&json, "skins"),
        joints,
        animations: json_array_len(&json, "animations"),
        external_uris,
        uri_safe: false,
    })
}
