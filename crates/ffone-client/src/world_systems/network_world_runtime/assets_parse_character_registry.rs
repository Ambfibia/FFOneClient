use super::*;

pub(super) const CHARACTER_REGISTRY_SCHEMA: &str = "ffone.semantic-character-registry.v2";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NetworkNpcVisualCatalogIssue0104 {
    pub npc_type: i32,
    pub detail: String,
}

#[derive(Clone, Debug)]
pub(super) struct RegistryModel0104 {
    pub(super) logical_name: String,
    pub(super) category: String,
    pub(super) glb: String,
    pub(super) collision: Option<String>,
}

pub(super) fn parse_character_registry(
    registry: &Value,
) -> Result<BTreeMap<String, Vec<RegistryModel0104>>, String> {
    if registry.get("schema").and_then(Value::as_str) != Some(CHARACTER_REGISTRY_SCHEMA) {
        return Err(format!(
            "native character registry must use {CHARACTER_REGISTRY_SCHEMA:?}"
        ));
    }
    let models = registry
        .get("models")
        .and_then(Value::as_array)
        .ok_or_else(|| "native character registry has no models array".to_owned())?;
    let mut by_logical_name = BTreeMap::<String, Vec<RegistryModel0104>>::new();
    for (index, model) in models.iter().enumerate() {
        let context = format!("registry.models[{index}]");
        let category = required_string(model, "category", &context)?;
        if category == "nano" || category == "player" || category == "tutorial" {
            continue;
        }
        let logical_name = required_string(model, "logicalName", &context)?;
        let legacy_aliases = match model.get("legacyAliases") {
            None => Vec::new(),
            Some(value) => value
                .as_array()
                .ok_or_else(|| format!("{context}.legacyAliases must be an array"))?
                .iter()
                .enumerate()
                .map(|(alias_index, alias)| {
                    alias.as_str().map(str::to_owned).ok_or_else(|| {
                        format!("{context}.legacyAliases[{alias_index}] must be a string")
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        };
        let glb = required_string(model, "glb", &context)?;
        let collision = model
            .get("collision")
            .map(|value| {
                value
                    .as_str()
                    .filter(|path| {
                        path.ends_with(".collision.json")
                            && !Path::new(path).is_absolute()
                            && !path.contains(['\\', ':'])
                            && !path
                                .split('/')
                                .any(|part| part.is_empty() || part == "." || part == "..")
                    })
                    .map(str::to_owned)
                    .ok_or_else(|| {
                        format!(
                            "{context}.collision must be a normalized relative .collision.json path"
                        )
                    })
            })
            .transpose()?;
        let stem = Path::new(glb)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| format!("{context}.glb has no UTF-8 file stem"))?;
        if stem != logical_name {
            return Err(format!(
                "{context} logicalName {logical_name:?} does not match GLB root stem {stem:?}"
            ));
        }
        let package_route = Path::new(glb)
            .parent()
            .and_then(Path::file_name)
            .and_then(|route| route.to_str())
            .ok_or_else(|| format!("{context}.glb has no UTF-8 package route"))?;
        let owns_canonical_route = package_route == logical_name || legacy_aliases.is_empty();
        let candidate = RegistryModel0104 {
            logical_name: logical_name.to_owned(),
            category: category.to_owned(),
            glb: glb.to_owned(),
            collision,
        };
        for name in owns_canonical_route
            .then(|| logical_name.to_owned())
            .into_iter()
            .chain(legacy_aliases)
        {
            if name.is_empty() || name.contains(['/', '\\']) || name == "." || name == ".." {
                return Err(format!(
                    "{context} contains invalid logicalName/legacyAlias {name:?}"
                ));
            }
            let candidates = by_logical_name.entry(name.clone()).or_default();
            if candidates.iter().any(|existing| {
                existing.category == candidate.category && existing.glb != candidate.glb
            }) {
                return Err(format!(
                    "registry maps logical name/alias {name:?}/{category} to contradictory GLBs"
                ));
            }
            if !candidates.iter().any(|existing| {
                existing.category == candidate.category && existing.glb == candidate.glb
            }) {
                candidates.push(candidate.clone());
            }
        }
    }
    Ok(by_logical_name)
}

/// Allows a state-level loading barrier to request the same lazy catalog
/// before the first world or tutorial NPC entity exists.
#[derive(Debug, Default, Resource)]
pub struct NetworkNpcVisualCatalogRequest0104 {
    pub requested: bool,
}

#[derive(Debug, Default, Resource)]
pub struct NetworkNpcVisualCatalogState0104 {
    pub attempted: bool,
    pub catalog: Option<NetworkNpcVisualCatalog0104>,
    pub(super) hnpc: Option<Arc<HnpcRuntimeCatalog>>,
    pub error: Option<String>,
}

pub(super) fn load_network_npc_visual_catalog_0104(
    locator: Option<Res<AssetLocator>>,
    rig_catalog: Option<Res<NativePlayerRigCatalog>>,
    request: Res<NetworkNpcVisualCatalogRequest0104>,
    mut state: ResMut<NetworkNpcVisualCatalogState0104>,
    npcs: Query<(), With<NetworkNpc0104>>,
    shinies: Query<(), With<crate::entity_lifecycle::NetworkShiny0104>>,
    tutorial_actors: Query<(), With<TutorialActor>>,
) {
    // Parsing the complete NPC/XDT/animation catalog is gameplay work. Do not
    // put it on the login critical path; the first lifecycle batch requests it
    // before visual resolution in the same Update schedule.
    if state.attempted
        || (!request.requested
            && npcs.is_empty()
            && shinies.is_empty()
            && tutorial_actors.is_empty())
    {
        return;
    }
    let (Some(locator), Some(rig_catalog)) = (locator, rig_catalog) else {
        return;
    };
    state.attempted = true;
    match NetworkNpcVisualCatalog0104::open(&locator) {
        Ok(mut catalog) => {
            let hnpc = match HnpcRuntimeCatalog::open(&locator, &rig_catalog) {
                Ok(hnpc) => Some(Arc::new(hnpc)),
                Err(error) => {
                    error!("failed to load native HNPC appearance catalog: {error}");
                    state.error = Some(error);
                    None
                }
            };
            if let Some(hnpc) = hnpc.as_ref() {
                let unresolved = catalog
                    .hnpc_definitions
                    .values()
                    .filter_map(|definition| {
                        let appearance = hnpc.appearance(definition.appearance_index)?;
                        appearance.look.is_none().then_some((
                            definition.npc_type,
                            definition.appearance_index,
                            "has no wearable parts",
                        ))
                    })
                    .collect::<Vec<_>>();
                for (npc_type, appearance_index, detail) in unresolved {
                    catalog.issue(
                        npc_type,
                        format!("HNPC appearance {appearance_index} {detail}"),
                    );
                }
                let missing = catalog
                    .hnpc_definitions
                    .values()
                    .filter(|definition| hnpc.appearance(definition.appearance_index).is_none())
                    .map(|definition| (definition.npc_type, definition.appearance_index))
                    .collect::<Vec<_>>();
                for (npc_type, appearance_index) in missing {
                    catalog.issue(
                        npc_type,
                        format!(
                            "m_iHNpcNum={appearance_index} is outside the production HNPC catalog"
                        ),
                    );
                }
                info!(
                    "loaded {} native HNPC appearances for {} server HNPC definitions",
                    hnpc.len(),
                    catalog.hnpc_len()
                );
            }
            for issue in catalog.issues.iter().take(20) {
                warn!(
                    "network NPC catalog skipped type {}: {}",
                    issue.npc_type, issue.detail
                );
            }
            if catalog.issues.len() > 20 {
                warn!(
                    "server NPC catalog has {} additional unresolved rows; inspect NetworkNpcVisualCatalogState0104 for the complete typed diagnostics",
                    catalog.issues.len() - 20
                );
            }
            info!(
                "loaded {} direct and {} HNPC server visual definitions ({} unresolved rows)",
                catalog.len(),
                catalog.hnpc_len(),
                catalog.issues.len()
            );
            state.catalog = Some(catalog);
            state.hnpc = hnpc;
        }
        Err(error) => {
            error!("failed to load server NPC visual catalog: {error}");
            state.error = Some(error);
        }
    }
}
