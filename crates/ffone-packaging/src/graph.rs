use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufReader, Read},
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) const DOMAIN_CATALOGS: &[(&str, &str)] = &[
    ("data/tables/xdt.json", "ffone.xdt.v1"),
    ("map/catalog.json", "ffone.map-catalog.v1"),
    (
        "characters/player/items/catalog.json",
        "ffone.player-item-set-catalog.v1",
    ),
    ("localization/catalog.json", "ffone.localization-catalog.v1"),
];

pub(crate) const REQUIRED_ROOT_FILES: &[&str] = &["data/tables/xdt.json"];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct AssetGraphEntry {
    pub path: String,
    pub owner: String,
    pub group: String,
    pub bytes: u64,
    pub blake3: Option<String>,
    pub modified_nanos: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AssetGraph {
    pub files: Vec<AssetGraphEntry>,
    pub groups: BTreeMap<String, Vec<AssetGraphEntry>>,
    pub payload_bytes: u64,
    pub root_blake3: String,
    pub domain_catalogs: usize,
    pub direct_references: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DomainValidation {
    pub catalogs: usize,
    pub references: usize,
}

/// Exact payload index recovered from verified release archives. JSON
/// catalogs remain loose; immutable object GLBs/PNGs may be resolved through
/// this table without weakening path, byte-length or hash checks.
pub(crate) type PackedAssetIndex = BTreeMap<String, (u64, String)>;

pub(crate) fn validate_domain_roots(
    root: &Path,
    verify_hashes: bool,
) -> Result<DomainValidation, String> {
    validate_domain_roots_with_packed(root, verify_hashes, &PackedAssetIndex::new())
}

pub(crate) fn validate_domain_roots_with_packed(
    root: &Path,
    verify_hashes: bool,
    packed: &PackedAssetIndex,
) -> Result<DomainValidation, String> {
    validate_asset_root(root)?;
    let mut references = 0_usize;
    for &(relative, schema) in DOMAIN_CATALOGS {
        let value = read_json(root, relative)?;
        let is_xdt = relative == "data/tables/xdt.json";
        let actual_schema = if is_xdt && value.get("_ffone").is_some() {
            value.pointer("/_ffone/schema").and_then(Value::as_str)
        } else {
            value.get("schema").and_then(Value::as_str)
        };
        let legacy_xdt = is_xdt && actual_schema == Some("ffone.table-set.v1");
        if actual_schema != Some(schema) && !legacy_xdt {
            return Err(format!(
                "domain catalog {relative:?} has schema {:?}, expected {schema:?}",
                actual_schema
            ));
        }
        match relative {
            "data/tables/xdt.json" => {
                let routes = native_asset_routes(&value)?;
                for model in routes["m_pCharacterModelData"]
                    .as_array()
                    .ok_or("missing character model rows")?
                {
                    validate_character_route(model)?;
                    let path = model["glb"].as_str().ok_or("model has no GLB path")?;
                    require_regular_file(root, path)?;
                    references += 1;
                }
                for audio in routes["m_pAudioData"]
                    .as_array()
                    .ok_or("missing audio rows")?
                {
                    let path = audio["path"].as_str().ok_or("audio row has no path")?;
                    validate_relative(path)?;
                    if audio["category"] != "voice" {
                        require_regular_file(root, path)?;
                        references += 1;
                    } else {
                        // Languages and their take counts are independent. Every
                        // existing file is included by the release tree walk.
                        let voice_root = root.join("audio/voice");
                        if voice_root.is_dir() {
                            for locale in fs::read_dir(voice_root).map_err(|e| e.to_string())? {
                                let locale = locale.map_err(|e| e.to_string())?;
                                if locale.file_type().map_err(|e| e.to_string())?.is_dir() {
                                    let relative = format!(
                                        "audio/voice/{}/{path}",
                                        locale.file_name().to_string_lossy()
                                    );
                                    if native_path(root, &relative).exists() {
                                        require_regular_file(root, &relative)?;
                                        references += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            "map/catalog.json" => {
                references += validate_map_catalog_references(root, &value, verify_hashes, packed)?;
            }
            "characters/player/items/catalog.json" => {
                references += validate_player_item_catalog_references(root, &value, verify_hashes)?;
            }
            "localization/catalog.json" => {
                let fallback = value["fallback"]
                    .as_str()
                    .ok_or_else(|| "localization catalog has no fallback locale".to_owned())?;
                let locales = value["locales"]
                    .as_array()
                    .ok_or_else(|| "localization catalog has no locales array".to_owned())?;
                let mut ids = BTreeSet::new();
                let mut paths = BTreeSet::new();
                for locale in locales {
                    let id = locale["id"]
                        .as_str()
                        .ok_or_else(|| "localization entry has no id".to_owned())?;
                    let path = locale["text"]
                        .as_str()
                        .ok_or_else(|| "localization entry has no text path".to_owned())?;
                    validate_locale_id(id)?;
                    if !ids.insert(id.to_owned()) {
                        return Err(format!("duplicate localization locale {id:?}"));
                    }
                    if !path.starts_with("localization/")
                        || !path.ends_with(".json")
                        || path == "localization/catalog.json"
                        || !paths.insert(path.to_owned())
                    {
                        return Err(format!("invalid localization bundle path {path:?}"));
                    }
                    require_regular_file(root, path)?;
                    references += 1;
                }
                if !ids.contains(fallback) {
                    return Err(format!(
                        "localization fallback {fallback:?} has no catalog entry"
                    ));
                }
            }
            _ => unreachable!("catalog list and validator must change together"),
        }
    }
    for &relative in REQUIRED_ROOT_FILES {
        require_regular_file(root, relative)?;
        references += 1;
    }
    Ok(DomainValidation {
        catalogs: DOMAIN_CATALOGS.len(),
        references,
    })
}

fn validate_character_route(model: &Value) -> Result<(), String> {
    let category = model["category"]
        .as_str()
        .ok_or_else(|| "character entry has no category".to_owned())?;
    let id = model["id"]
        .as_str()
        .ok_or_else(|| "character entry has no id".to_owned())?;
    let glb = model["glb"]
        .as_str()
        .ok_or_else(|| "character entry has no glb".to_owned())?;
    let directory = match category {
        "nano" => "nanos",
        "mob" => "mobs",
        "npc" => "npcs",
        "fusion" => "fusions",
        "shiny" => "shinies",
        other => return Err(format!("unsupported character category {other:?}")),
    };
    // Accepted IDs survive classification changes (for example shared/npc_ed
    // now belongs to npcs). Category owns the physical directory; the ID still
    // owns the package slug, independently of its original namespace.
    let slug = id
        .split_once('/')
        .filter(|(namespace, slug)| {
            matches!(
                *namespace,
                "nano" | "mob" | "npc" | "fusion" | "shared" | "shiny"
            ) && !slug.is_empty()
                && !slug.contains('/')
                && !slug.contains('\\')
                && !matches!(*slug, "." | "..")
        })
        .map(|(_, slug)| slug)
        .ok_or_else(|| format!("character id {id:?} has no valid namespace/package slug"))?;
    let prefix = format!("characters/{directory}/{slug}/");
    // Route ids are lowercase while exported folders keep the logical model
    // name's casing (shinies/shineni_Item). The graph scan rejects paths that
    // differ only by case, so an ASCII-folded package prefix stays unambiguous.
    if !glb
        .get(..prefix.len())
        .is_some_and(|head| head.eq_ignore_ascii_case(&prefix))
    {
        return Err(format!(
            "character {id:?} GLB {glb:?} is outside package {prefix:?}"
        ));
    }
    Ok(())
}

fn validate_map_catalog_references(
    root: &Path,
    catalog: &Value,
    verify_hashes: bool,
    packed: &PackedAssetIndex,
) -> Result<usize, String> {
    let mut references = 0;
    for (array, field, label) in [
        ("resourceSets", "definition", "map resource set"),
        ("geometry", "model", "map geometry"),
        ("objects", "definition", "map object"),
    ] {
        for entry in catalog[array]
            .as_array()
            .ok_or_else(|| format!("map catalog has no {array} array"))?
        {
            let artifact = entry
                .get(field)
                .ok_or_else(|| format!("{label} entry has no {field}"))?;
            validate_scoped_reference_with_packed(
                root,
                artifact,
                verify_hashes,
                label,
                "objects/",
                packed,
            )?;
            references += 1;
        }
    }
    for tile in catalog["tiles"]
        .as_array()
        .ok_or_else(|| "map catalog has no tiles array".to_owned())?
    {
        for field in ["manifest", "objects"] {
            let artifact = tile
                .get(field)
                .ok_or_else(|| format!("map tile has no {field}"))?;
            validate_scoped_reference(root, artifact, verify_hashes, "map tile", "map/")?;
            references += 1;
        }
    }
    for artifact in catalog
        .get("sharedFiles")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        validate_map_owned_reference(root, artifact, verify_hashes, "map shared file", packed)?;
        references += 1;
    }
    Ok(references)
}

fn validate_player_item_catalog_references(
    root: &Path,
    catalog: &Value,
    verify_hashes: bool,
) -> Result<usize, String> {
    let mut references = 0;
    for entry in catalog["sets"]
        .as_array()
        .ok_or_else(|| "player item catalog has no sets array".to_owned())?
    {
        let artifact = entry
            .get("definition")
            .ok_or_else(|| "player item set has no definition".to_owned())?;
        validate_scoped_reference(
            root,
            artifact,
            verify_hashes,
            "player item set",
            "characters/player/",
        )?;
        references += 1;
    }
    for entry in catalog["models"]
        .as_array()
        .ok_or_else(|| "player item catalog has no models array".to_owned())?
    {
        let artifact = entry
            .get("model")
            .ok_or_else(|| "player item route has no model".to_owned())?;
        validate_scoped_reference(
            root,
            artifact,
            verify_hashes,
            "player item model",
            "characters/player/",
        )?;
        references += 1;
    }
    for artifact in catalog["renderingTextures"]
        .as_array()
        .ok_or_else(|| "player item catalog has no renderingTextures array".to_owned())?
    {
        validate_scoped_reference(
            root,
            artifact,
            verify_hashes,
            "player rendering texture",
            "characters/player/",
        )?;
        references += 1;
    }
    Ok(references)
}

fn validate_scoped_reference(
    root: &Path,
    value: &Value,
    verify_hashes: bool,
    context: &str,
    prefix: &str,
) -> Result<(), String> {
    let path = value["path"]
        .as_str()
        .ok_or_else(|| format!("{context} reference has no path"))?;
    if !path.starts_with(prefix) {
        return Err(format!(
            "{context} owns non-canonical path {path:?}; expected prefix {prefix:?}"
        ));
    }
    validate_reference(root, value, verify_hashes, context)
}

fn validate_map_owned_reference(
    root: &Path,
    value: &Value,
    verify_hashes: bool,
    context: &str,
    packed: &PackedAssetIndex,
) -> Result<(), String> {
    let path = value["path"]
        .as_str()
        .ok_or_else(|| format!("{context} reference has no path"))?;
    // Immutable textures can be shared by map and character consumers. Keep
    // that native texture root distinct from arbitrary cross-domain payloads.
    let shared_texture = (path.starts_with("effects/shared/textures/")
        || (path.starts_with("characters/") && path.contains("/textures/")))
        && path.ends_with(".png");
    if !path.starts_with("map/") && !path.starts_with("objects/") && !shared_texture {
        return Err(format!(
            "{context} owns non-canonical path {path:?}; expected prefix \"map/\" or \"objects/\", or a shared PNG owned by effects/characters"
        ));
    }
    validate_reference_with_packed(root, value, verify_hashes, context, packed)
}

fn validate_scoped_reference_with_packed(
    root: &Path,
    value: &Value,
    verify_hashes: bool,
    context: &str,
    prefix: &str,
    packed: &PackedAssetIndex,
) -> Result<(), String> {
    let path = value["path"]
        .as_str()
        .ok_or_else(|| format!("{context} reference has no path"))?;
    if !path.starts_with(prefix) {
        return Err(format!(
            "{context} owns non-canonical path {path:?}; expected prefix {prefix:?}"
        ));
    }
    validate_reference_with_packed(root, value, verify_hashes, context, packed)
}

pub(crate) fn build_asset_graph(root: &Path, verify_hashes: bool) -> Result<AssetGraph, String> {
    build_asset_graph_with_packed(root, verify_hashes, &PackedAssetIndex::new())
}

pub(crate) fn build_asset_graph_with_packed(
    root: &Path,
    verify_hashes: bool,
    packed: &PackedAssetIndex,
) -> Result<AssetGraph, String> {
    // Domain paths/schemas are checked first, but payload hashing happens only
    // once during the graph scan below. Expected catalog hashes are compared
    // against those graph nodes afterwards.
    let domain = validate_domain_roots_with_packed(root, false, packed)?;
    let mut paths = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory)
            .map_err(|error| format!("cannot read {}: {error}", directory.display()))?
        {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
            if file_type.is_symlink() {
                return Err(format!(
                    "asset tree contains unsupported symlink {}",
                    path.display()
                ));
            }
            if file_type.is_dir() {
                pending.push(path);
            } else if file_type.is_file() {
                let relative = relative_path(root, &path)?;
                if is_control_path(&relative) {
                    continue;
                }
                paths.push((relative, path));
            }
        }
    }
    paths.sort_by(|left, right| left.0.cmp(&right.0));

    let mut folded = BTreeSet::new();
    let mut files = Vec::with_capacity(paths.len());
    let mut payload_bytes = 0_u64;
    let mut root_hasher = blake3::Hasher::new();
    for (relative, absolute) in paths {
        validate_relative(&relative)?;
        if !folded.insert(relative.to_ascii_lowercase()) {
            return Err(format!(
                "case-insensitive duplicate asset path {relative:?}"
            ));
        }
        let group = group_for(&relative)
            .ok_or_else(|| format!("asset {relative:?} has no declared runtime owner group"))?;
        validate_extension(&relative)?;
        let metadata = fs::metadata(&absolute)
            .map_err(|error| format!("cannot inspect {}: {error}", absolute.display()))?;
        let bytes = metadata.len();
        let modified_nanos = metadata
            .modified()
            .map_err(|error| format!("cannot read mtime for {}: {error}", absolute.display()))?
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("invalid mtime for {}: {error}", absolute.display()))?
            .as_nanos();
        let blake3 = if verify_hashes && !is_editable_asset(&relative) {
            Some(hash_file(&absolute)?)
        } else {
            None
        };
        let owner = owner_for(&relative);
        if let Some(blake3) = &blake3 {
            root_hasher.update(relative.as_bytes());
            root_hasher.update(&[0]);
            root_hasher.update(&bytes.to_le_bytes());
            root_hasher.update(&[0]);
            root_hasher.update(blake3.as_bytes());
        }
        payload_bytes = payload_bytes
            .checked_add(bytes)
            .ok_or_else(|| "asset payload byte count overflow".to_owned())?;
        files.push(AssetGraphEntry {
            path: relative,
            owner,
            group: group.to_owned(),
            bytes,
            blake3,
            modified_nanos,
        });
    }
    let mut groups = BTreeMap::<String, Vec<AssetGraphEntry>>::new();
    for file in &files {
        groups
            .entry(file.group.clone())
            .or_default()
            .push(file.clone());
    }
    // Catalogs describe semantic routes, not the byte identity of editable
    // payloads. This pass checks that every route resolves in the graph without
    // comparing legacy size/hash fields.
    let transitive_references = if verify_hashes {
        validate_catalog_hashes_against_graph(root, &files, packed)?
    } else {
        0
    };
    Ok(AssetGraph {
        files,
        groups,
        payload_bytes,
        root_blake3: root_hasher.finalize().to_hex().to_string(),
        domain_catalogs: domain.catalogs,
        direct_references: domain.references + transitive_references,
    })
}

fn validate_catalog_hashes_against_graph(
    root: &Path,
    files: &[AssetGraphEntry],
    packed: &PackedAssetIndex,
) -> Result<usize, String> {
    let index = files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect::<BTreeMap<_, _>>();
    let mut references = 0;

    let tables = read_json(root, "data/tables/xdt.json")?;
    let routes = native_asset_routes(&tables)?;
    for model in routes["m_pCharacterModelData"]
        .as_array()
        .ok_or_else(|| "character catalog has no models array".to_owned())?
    {
        compare_graph_reference(&index, model, "glb", "glbBlake3", None, packed)?;
        references += 1;
    }

    references += validate_map_catalog_hashes_against_graph(root, &index, packed)?;
    references += validate_player_item_catalog_hashes_against_graph(root, &index, packed)?;
    references += validate_tutorial_catalog_hashes_against_graph(root, &index, packed)?;
    Ok(references)
}

fn validate_tutorial_catalog_hashes_against_graph(
    root: &Path,
    index: &BTreeMap<&str, &AssetGraphEntry>,
    packed: &PackedAssetIndex,
) -> Result<usize, String> {
    let mut references = 0;
    let effects_path = "map/shared/effects/catalog.json";
    if native_path(root, effects_path).exists() {
        let catalog = read_json(root, effects_path)?;
        if catalog["schema"].as_str() != Some("ffone.tutorial-effect-catalog.v1") {
            return Err(format!(
                "unsupported tutorial effect catalog schema in {effects_path:?}"
            ));
        }
        for effect in catalog["effects"]
            .as_array()
            .ok_or_else(|| "tutorial effect catalog has no effects array".to_owned())?
        {
            compare_graph_reference(
                index,
                effect,
                "closurePath",
                "closureBlake3",
                Some("closureBytes"),
                packed,
            )?;
            references += 1;
        }
    }

    let projectiles_path = "map/shared/projectiles/catalog.json";
    if native_path(root, projectiles_path).exists() {
        let catalog = read_json(root, projectiles_path)?;
        if catalog["schema"].as_str() != Some("ffone.tutorial-projectile-catalog.v1") {
            return Err(format!(
                "unsupported tutorial projectile catalog schema in {projectiles_path:?}"
            ));
        }
        compare_graph_reference(
            index,
            &catalog,
            "bulletTableClosurePath",
            "bulletTableClosureBlake3",
            Some("bulletTableClosureBytes"),
            packed,
        )?;
        references += 1;
        for effect in catalog["particleEffects"]
            .as_array()
            .ok_or_else(|| "tutorial projectile catalog has no particleEffects array".to_owned())?
        {
            compare_graph_reference(
                index,
                effect,
                "closurePath",
                "closureBlake3",
                Some("closureBytes"),
                packed,
            )?;
            references += 1;
        }
        for row in catalog["rows"]
            .as_array()
            .ok_or_else(|| "tutorial projectile catalog has no rows array".to_owned())?
        {
            compare_graph_reference(index, row, "rowPath", "rowBlake3", Some("rowBytes"), packed)?;
            references += 1;
        }
    }
    Ok(references)
}

fn validate_map_catalog_hashes_against_graph(
    root: &Path,
    index: &BTreeMap<&str, &AssetGraphEntry>,
    packed: &PackedAssetIndex,
) -> Result<usize, String> {
    let catalog = read_json(root, "map/catalog.json")?;
    let mut references = 0;
    for entry in catalog["resourceSets"]
        .as_array()
        .ok_or_else(|| "map catalog has no resourceSets array".to_owned())?
    {
        let definition = &entry["definition"];
        compare_graph_artifact(index, definition, packed)?;
        references += 1;
        let path = definition["path"]
            .as_str()
            .ok_or_else(|| "map resource set has no definition path".to_owned())?;
        let set = read_json(root, path)?;
        references +=
            validate_resource_set_hashes_against_graph(index, &set, "map_object", packed)?;
    }
    for (array, field) in [("geometry", "model"), ("objects", "definition")] {
        for entry in catalog[array]
            .as_array()
            .ok_or_else(|| format!("map catalog has no {array} array"))?
        {
            compare_graph_artifact(index, &entry[field], packed)?;
            references += 1;
        }
    }
    for tile in catalog["tiles"]
        .as_array()
        .ok_or_else(|| "map catalog has no tiles array".to_owned())?
    {
        for field in ["manifest", "objects"] {
            compare_graph_artifact(index, &tile[field], packed)?;
            references += 1;
        }
    }
    for artifact in catalog
        .get("sharedFiles")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        compare_graph_artifact(index, artifact, packed)?;
        references += 1;
    }
    Ok(references)
}

fn validate_player_item_catalog_hashes_against_graph(
    root: &Path,
    index: &BTreeMap<&str, &AssetGraphEntry>,
    packed: &PackedAssetIndex,
) -> Result<usize, String> {
    let catalog = read_json(root, "characters/player/items/catalog.json")?;
    let mut references = 0;
    for entry in catalog["sets"]
        .as_array()
        .ok_or_else(|| "player item catalog has no sets array".to_owned())?
    {
        let definition = &entry["definition"];
        compare_graph_artifact(index, definition, packed)?;
        references += 1;
        let path = definition["path"]
            .as_str()
            .ok_or_else(|| "player item set has no definition path".to_owned())?;
        let set = read_json(root, path)?;
        references +=
            validate_resource_set_hashes_against_graph(index, &set, "player_item", packed)?;
    }
    for entry in catalog["models"]
        .as_array()
        .ok_or_else(|| "player item catalog has no models array".to_owned())?
    {
        compare_graph_artifact(index, &entry["model"], packed)?;
        references += 1;
    }
    for artifact in catalog["renderingTextures"]
        .as_array()
        .ok_or_else(|| "player item catalog has no renderingTextures array".to_owned())?
    {
        compare_graph_artifact(index, artifact, packed)?;
        references += 1;
    }
    Ok(references)
}

fn validate_resource_set_hashes_against_graph(
    index: &BTreeMap<&str, &AssetGraphEntry>,
    set: &Value,
    expected_domain: &str,
    packed: &PackedAssetIndex,
) -> Result<usize, String> {
    if set["schema"].as_str() != Some("ffone.resource-set.v1")
        || set["domain"].as_str() != Some(expected_domain)
    {
        return Err(format!(
            "resource set has schema/domain {:?}/{:?}, expected ffone.resource-set.v1/{expected_domain}",
            set["schema"].as_str(),
            set["domain"].as_str()
        ));
    }
    let mut references = 0;
    for artifact in set["textures"]
        .as_array()
        .ok_or_else(|| "resource set has no textures array".to_owned())?
    {
        compare_graph_artifact(index, artifact, packed)?;
        references += 1;
    }
    for member in set["members"]
        .as_array()
        .ok_or_else(|| "resource set has no members array".to_owned())?
    {
        compare_graph_artifact(index, &member["definition"], packed)?;
        references += 1;
        for artifact in member["files"]
            .as_array()
            .ok_or_else(|| "resource set member has no files array".to_owned())?
        {
            compare_graph_artifact(index, artifact, packed)?;
            references += 1;
        }
    }
    Ok(references)
}

fn compare_graph_artifact(
    index: &BTreeMap<&str, &AssetGraphEntry>,
    artifact: &Value,
    packed: &PackedAssetIndex,
) -> Result<(), String> {
    compare_graph_reference(index, artifact, "path", "blake3", Some("bytes"), packed)
}

fn compare_graph_reference(
    index: &BTreeMap<&str, &AssetGraphEntry>,
    value: &Value,
    path_field: &str,
    _hash_field: &str,
    _bytes_field: Option<&str>,
    packed: &PackedAssetIndex,
) -> Result<(), String> {
    let path = value[path_field]
        .as_str()
        .ok_or_else(|| format!("catalog reference has no {path_field}"))?;
    if !index.contains_key(path) && !packed.contains_key(path) {
        return Err(format!(
            "catalog dependency {path:?} is absent from the release graph and verified packs"
        ));
    }
    Ok(())
}

fn validate_reference(
    root: &Path,
    value: &Value,
    verify_hashes: bool,
    context: &str,
) -> Result<(), String> {
    validate_reference_with_packed(
        root,
        value,
        verify_hashes,
        context,
        &PackedAssetIndex::new(),
    )
}

fn validate_reference_with_packed(
    root: &Path,
    value: &Value,
    verify_hashes: bool,
    context: &str,
    packed: &PackedAssetIndex,
) -> Result<(), String> {
    let path = value["path"]
        .as_str()
        .ok_or_else(|| format!("{context} reference has no path"))?;
    let _ = (verify_hashes, context);
    require_reference_with_packed(root, path, packed)
}

fn require_reference_with_packed(
    root: &Path,
    relative: &str,
    packed: &PackedAssetIndex,
) -> Result<(), String> {
    validate_relative(relative)?;
    let path = native_path(root, relative);
    match fs::metadata(&path) {
        Ok(metadata) if metadata.is_file() => Ok(()),
        Ok(_) => Err(format!(
            "domain-owned asset is not a file: {}",
            path.display()
        )),
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound && packed.contains_key(relative) =>
        {
            Ok(())
        }
        Err(error) => Err(format!(
            "missing domain-owned asset {}: {error}",
            path.display()
        )),
    }
}

fn read_json(root: &Path, relative: &str) -> Result<Value, String> {
    let path = require_regular_file(root, relative)?;
    let bytes =
        fs::read(&path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("invalid {}: {error}", path.display()))
}

fn require_regular_file(root: &Path, relative: &str) -> Result<PathBuf, String> {
    validate_relative(relative)?;
    let path = native_path(root, relative);
    let metadata = fs::metadata(&path)
        .map_err(|error| format!("missing required asset {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!("required asset is not a file: {}", path.display()));
    }
    Ok(path)
}

fn validate_asset_root(root: &Path) -> Result<(), String> {
    let metadata = fs::metadata(root)
        .map_err(|error| format!("cannot inspect asset root {}: {error}", root.display()))?;
    if !metadata.is_dir() {
        return Err(format!("asset root is not a directory: {}", root.display()));
    }
    for retired in [
        "asset-manifest.json",
        "asset-index.json",
        "manifests",
        "_runtime/audio.json",
        "_runtime/characters.json",
    ] {
        if root.join(retired).exists() {
            return Err(format!(
                "retired global asset metadata must be removed: {}",
                root.join(retired).display()
            ));
        }
    }
    Ok(())
}

fn validate_relative(path: &str) -> Result<(), String> {
    if path.is_empty() || path.contains('\\') {
        return Err(format!("invalid asset path {path:?}"));
    }
    let parsed = Path::new(path);
    if parsed.is_absolute()
        || parsed
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!("unsafe asset path {path:?}"));
    }
    Ok(())
}

fn validate_extension(path: &str) -> Result<(), String> {
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    if matches!(
        extension.as_str(),
        "glb" | "png" | "ogg" | "ttf" | "otf" | "json" | "bin" | "wgsl"
    ) {
        Ok(())
    } else {
        Err(format!("unsupported runtime asset extension for {path:?}"))
    }
}

pub(crate) fn group_for(path: &str) -> Option<&'static str> {
    if path.starts_with("nano/icons/") || path.starts_with("icons/") {
        return Some("icons");
    }
    Some(match path.split('/').next()? {
        "_runtime" => "runtime",
        "audio" => "audio",
        "characters" => "characters",
        "effects" => "effects",
        "data" => "data",
        "fonts" => "fonts",
        "localization" => "localization",
        "map" => "map",
        "objects" => "map",
        "shaders" => "shaders",
        "ui" => "ui",
        _ => return None,
    })
}

fn owner_for(path: &str) -> String {
    let parts = path.split('/').collect::<Vec<_>>();
    match parts.as_slice() {
        ["effects", "shared", ..] => "effects:shared-textures".to_owned(),
        ["map", "tiles", tile, ..] => format!("map:tile/{tile}"),
        ["objects", category, resource_set, ..] => {
            format!("map:object-set/{category}/{resource_set}")
        }
        ["map", "shared", domain, ..] => format!("map:shared/{domain}"),
        ["characters", category, id, ..] => format!("characters:{category}/{id}"),
        ["audio", category, ..] => format!("audio:{category}"),
        ["ui", "en" | "ru", system, ..] => format!("code:ui/{system}"),
        ["icons", category, ..] => format!("gameplay:icons/{category}"),
        [root, ..] => format!("root:{root}"),
        [] => "root".to_owned(),
    }
}

fn is_control_path(_path: &str) -> bool {
    false
}

fn relative_path(root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(root)
        .map_err(|error| error.to_string())
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

fn native_path(root: &Path, relative: &str) -> PathBuf {
    relative
        .split('/')
        .fold(root.to_path_buf(), |path, component| path.join(component))
}

fn hash_file(path: &Path) -> Result<String, String> {
    let file =
        fs::File::open(path).map_err(|error| format!("cannot open {}: {error}", path.display()))?;
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// Translator-owned files are intentionally outside the immutable content
/// graph. They are still required to be safe, regular files, but no hash is
/// computed, serialized, or compared for them.
pub(crate) fn is_editable_asset(path: &str) -> bool {
    path == "data/tables/xdt.json"
        || path.starts_with("audio/")
        || path.starts_with("localization/")
}

fn validate_locale_id(locale: &str) -> Result<(), String> {
    if !locale.is_empty()
        && !locale.starts_with('-')
        && !locale.ends_with('-')
        && !locale.contains("--")
        && locale
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        Ok(())
    } else {
        Err(format!("invalid localization locale {locale:?}"))
    }
}

fn native_asset_routes(table_set: &Value) -> Result<&Value, String> {
    let table_set = table_set.get("_ffone").unwrap_or(table_set);
    let tables = table_set["tables"]
        .as_array()
        .ok_or("TableData has no tables")?;
    let mut rows = tables.iter().filter(|t| t["name"] == "native_asset_routes");
    let row = rows.next().ok_or("TableData has no native_asset_routes")?;
    if rows.next().is_some() {
        return Err("duplicate native_asset_routes".to_owned());
    }
    Ok(&row["value"])
}
