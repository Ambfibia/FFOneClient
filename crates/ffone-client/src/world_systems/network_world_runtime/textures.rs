use super::*;

pub(super) const NPC_TEXTURE_CATALOG_SCHEMA: &str = "ffone.xdt-npc-texture-catalog.v1";

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkNpcTextureOverride0104 {
    pub true_name: String,
    pub path: String,
    pub sha256: String,
    pub sampler: NativeSampler,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct NetworkNpcTextureCatalogDocument0104 {
    pub(super) schema: String,
    pub(super) textures: Vec<NetworkNpcTextureCatalogEntry0104>,
    #[serde(default)]
    pub(super) blocked: Vec<NetworkNpcTextureCatalogBlocked0104>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct NetworkNpcTextureCatalogEntry0104 {
    pub(super) true_name: String,
    pub(super) path: String,
    pub(super) sha256: String,
    pub(super) sampler: NativeSampler,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct NetworkNpcTextureCatalogBlocked0104 {
    pub(super) true_name: String,
    pub(super) slots: Vec<String>,
    pub(super) row_count: u64,
    pub(super) model_stems: Vec<String>,
    pub(super) npc_numbers: Vec<i64>,
    pub(super) status: String,
    pub(super) sampler_status: String,
}

#[derive(Default)]
pub(super) struct NetworkNpcTextureCatalog0104 {
    pub(super) by_true_name: BTreeMap<String, NetworkNpcTextureOverride0104>,
    pub(super) blocked: BTreeMap<String, NetworkNpcTextureCatalogBlocked0104>,
}

pub(super) fn parse_npc_texture_catalog(
    document: &Value,
    require_texture: &mut impl FnMut(&str) -> Result<(), String>,
) -> Result<NetworkNpcTextureCatalog0104, String> {
    let document: NetworkNpcTextureCatalogDocument0104 =
        serde_json::from_value(document.clone())
            .map_err(|error| format!("invalid XDT NPC texture catalog: {error}"))?;
    if document.schema != NPC_TEXTURE_CATALOG_SCHEMA {
        return Err(format!(
            "XDT NPC texture catalog must use {NPC_TEXTURE_CATALOG_SCHEMA:?}"
        ));
    }
    let mut catalog = NetworkNpcTextureCatalog0104::default();
    for entry in document.textures {
        if entry.true_name.trim().is_empty()
            || entry.true_name.eq_ignore_ascii_case("null")
            || entry.true_name.contains(['/', '\\'])
        {
            return Err(format!(
                "XDT NPC texture catalog has invalid trueName {:?}",
                entry.true_name
            ));
        }
        if entry.path.is_empty()
            || entry.path.contains('\\')
            || Path::new(&entry.path).is_absolute()
            || Path::new(&entry.path)
                .components()
                .any(|component| !matches!(component, std::path::Component::Normal(_)))
        {
            return Err(format!(
                "XDT NPC texture {:?} has invalid runtime path {:?}",
                entry.true_name, entry.path
            ));
        }
        if entry.sha256.len() != 64 || !entry.sha256.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(format!(
                "XDT NPC texture {:?} has invalid SHA-256",
                entry.true_name
            ));
        }
        if !entry.sampler.name.eq_ignore_ascii_case(&entry.true_name) {
            return Err(format!(
                "XDT NPC texture {:?} sampler belongs to {:?}",
                entry.true_name, entry.sampler.name
            ));
        }
        require_texture(&entry.path).map_err(|error| {
            format!(
                "XDT NPC texture {:?} path {:?} is unavailable: {error}",
                entry.true_name, entry.path
            )
        })?;
        let key = entry.true_name.to_ascii_lowercase();
        let value = NetworkNpcTextureOverride0104 {
            true_name: entry.true_name,
            path: entry.path,
            sha256: entry.sha256.to_ascii_lowercase(),
            sampler: entry.sampler,
        };
        if catalog.by_true_name.insert(key.clone(), value).is_some() {
            return Err(format!(
                "XDT NPC texture catalog has duplicate trueName {key:?}"
            ));
        }
    }
    for blocked in document.blocked {
        if blocked.true_name.trim().is_empty()
            || blocked.true_name.eq_ignore_ascii_case("null")
            || blocked.true_name.contains(['/', '\\'])
        {
            return Err(format!(
                "XDT NPC texture catalog has invalid blocked trueName {:?}",
                blocked.true_name
            ));
        }
        let key = blocked.true_name.to_ascii_lowercase();
        if catalog.by_true_name.contains_key(&key) {
            return Err(format!(
                "XDT NPC texture {key:?} is both resolved and blocked"
            ));
        }
        if catalog.blocked.insert(key.clone(), blocked).is_some() {
            return Err(format!(
                "XDT NPC texture catalog has duplicate blocked trueName {key:?}"
            ));
        }
    }
    Ok(catalog)
}

pub(super) fn optional_texture_name(mesh: &Value, field: &str) -> Option<String> {
    let name = mesh.get(field).and_then(Value::as_str)?.trim();
    (!name.is_empty() && !name.eq_ignore_ascii_case("null")).then(|| name.to_owned())
}

pub(super) fn resolve_npc_texture_override(
    textures: &NetworkNpcTextureCatalog0104,
    true_name: Option<&str>,
    slot: &str,
    npc_type: i32,
    context: &str,
    catalog: &mut NetworkNpcVisualCatalog0104,
) -> Option<NetworkNpcTextureOverride0104> {
    let true_name = true_name?;
    let key = true_name.to_ascii_lowercase();
    if let Some(texture) = textures.by_true_name.get(&key) {
        return Some(texture.clone());
    }
    if let Some(blocked) = textures.blocked.get(&key) {
        catalog.issue(
            npc_type,
            format!(
                "{context} {slot} texture {true_name:?} is blocked: status={}, samplerStatus={}, rows={}, slots={}, models={}, npcNumbers={}",
                blocked.status,
                blocked.sampler_status,
                blocked.row_count,
                blocked.slots.join(","),
                blocked.model_stems.join(","),
                blocked
                    .npc_numbers
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        );
    } else {
        catalog.issue(
            npc_type,
            format!(
                "{context} {slot} texture {true_name:?} has no audited native texture contract"
            ),
        );
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NetworkNpcTextureSlot0104 {
    Main,
    Sub,
}

impl NetworkNpcTextureSlot0104 {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Main => "main",
            Self::Sub => "sub",
        }
    }
}

/// Resolves one legacy material to the same XDT main/sub texture slot for
/// every NPC consumer. Keeping this decision shared prevents tutorial actors
/// from drifting away from the ordinary network-NPC/viewer path.
pub(crate) fn selected_network_npc_texture_override_0104<'a>(
    pending: &PendingLegacyModelMaterial,
    main_texture: Option<&'a NetworkNpcTextureOverride0104>,
    sub_texture: Option<&'a NetworkNpcTextureOverride0104>,
) -> Option<(NetworkNpcTextureSlot0104, &'a NetworkNpcTextureOverride0104)> {
    match legacy_npc_texture_role(
        &pending.true_name,
        pending.params.shader,
        main_texture.is_some(),
        sub_texture.is_some(),
    ) {
        Some(LegacyNpcTextureRole::Sub) => {
            sub_texture.map(|texture| (NetworkNpcTextureSlot0104::Sub, texture))
        }
        Some(LegacyNpcTextureRole::Main) => {
            main_texture.map(|texture| (NetworkNpcTextureSlot0104::Main, texture))
        }
        None => None,
    }
}

#[derive(Clone, Copy, Debug, Component)]
pub struct NetworkNpcTextureVariantBound0104;

/// Table-selected appearance for a cinematic-owned scene. Its camera basis,
/// animation and lifetime belong to the cinematic, while texture binding uses
/// the same material admission and pass handling as ordinary NPCs.
#[derive(Clone, Debug, Component)]
pub struct NpcSceneTextureOverrides0104 {
    pub npc_type: i32,
    pub main_texture: Option<NetworkNpcTextureOverride0104>,
    pub sub_texture: Option<NetworkNpcTextureOverride0104>,
}

impl From<&NetworkNpcVisualDefinition0104> for NpcSceneTextureOverrides0104 {
    fn from(definition: &NetworkNpcVisualDefinition0104) -> Self {
        Self {
            npc_type: definition.npc_type,
            main_texture: definition.main_texture.clone(),
            sub_texture: definition.sub_texture.clone(),
        }
    }
}

pub(super) type NpcTextureRoots0104<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'static NetworkNpcVisual0104>,
        Option<&'static NpcSceneTextureOverrides0104>,
    ),
    Or<(
        With<NetworkNpcVisual0104>,
        With<NpcSceneTextureOverrides0104>,
    )>,
>;

/// Applies `NpcMoveController.SetupNPC` table textures to ordinary server-owned
/// NPCs. Case-sensitive `main`/`sub` names remain authoritative; the shared
/// classifier also repairs the one audited clean-client FusionEffect material
/// whose serialized name incorrectly says `main` despite owning Texture2.
pub fn bind_network_npc_texture_variants_0104(
    mut commands: Commands,
    asset_server: Option<Res<AssetServer>>,
    materials: Option<ResMut<Assets<LegacyModelMaterial>>>,
    outline_materials: Option<ResMut<Assets<LegacyOutlineMaterial>>>,
    parents: Query<&ChildOf>,
    children: Query<&Children>,
    roots: NpcTextureRoots0104,
    surfaces: Query<
        (
            Entity,
            &Mesh3d,
            &MeshMaterial3d<LegacyModelMaterial>,
            &PendingLegacyModelMaterial,
            Option<&SkinnedMesh>,
            Option<&MeshTag>,
            Option<&VisibilityRange>,
            Option<&Visibility>,
        ),
        (
            With<LegacyMaterialApplied>,
            Without<NetworkNpcTextureVariantBound0104>,
        ),
    >,
    outline_companions: Query<
        &LegacyMaterialPassCompanion,
        With<MeshMaterial3d<LegacyOutlineMaterial>>,
    >,
    surface_companions: Query<
        (
            Entity,
            &MeshMaterial3d<LegacyModelMaterial>,
            &LegacyMaterialPassCompanion,
        ),
        Without<NetworkNpcTextureVariantBound0104>,
    >,
    bound_sources: Query<
        (
            &MeshMaterial3d<LegacyModelMaterial>,
            &NetworkNpcMaterialSurface0104,
        ),
        With<NetworkNpcTextureVariantBound0104>,
    >,
    primitive_extras: Query<&bevy::gltf::GltfExtras>,
) {
    let (Some(asset_server), Some(mut materials)) = (asset_server, materials) else {
        return;
    };
    let mut outline_materials = outline_materials;
    for (entity, mesh, material_handle, pending, skin, range_tag, range, visibility) in &surfaces {
        let Some((root, npc_type, main_texture, sub_texture)) =
            network_npc_visual_ancestor(entity, &parents, &roots)
        else {
            continue;
        };
        let selected = crate::legacy_model_material::native_npc_table_texture_writable(
            primitive_extras.get(entity).ok(),
        )
        .then(|| selected_network_npc_texture_override_0104(pending, main_texture, sub_texture))
        .flatten();
        let Some((slot, texture)) = selected else {
            commands.entity(entity).insert((
                NetworkNpcTextureVariantBound0104,
                NetworkNpcMaterialSurface0104 { root },
            ));
            continue;
        };
        let replacement = match load_legacy_main_texture_replacement_with_sampler(
            &asset_server,
            pending,
            texture.path.clone(),
            &texture.sampler,
        ) {
            Ok(replacement) => replacement,
            Err(error) => {
                let detail = format!(
                    "cannot bind XDT {} texture {:?} ({}) to material {:?}: {error}",
                    slot.as_str(),
                    texture.true_name,
                    texture.path,
                    pending.true_name
                );
                commands.entity(root).insert((
                    NetworkNpcVisualIssue0104 {
                        npc_type,
                        detail: detail.clone(),
                    },
                    NetworkNpcMaterialVisibilityBlocked0104 { detail },
                ));
                commands.entity(entity).insert((
                    NetworkNpcTextureVariantBound0104,
                    NetworkNpcMaterialSurface0104 { root },
                ));
                continue;
            }
        };
        let Some(mut material) = materials.get(&material_handle.0).cloned() else {
            continue;
        };
        let promote_fusion_matter =
            network_npc_texture_uses_fusion_matter_0104(slot, &texture.true_name)
                && pending.params.shader != LegacyShaderKind::FusionMatterLightDir;
        let promoted_params =
            promote_fusion_matter.then(|| network_npc_fusion_matter_params_0104(&pending.params));
        if let Some(params) = promoted_params.as_ref() {
            let Some(pass) = params.render_plan().passes.first().copied() else {
                continue;
            };
            let textures = LegacyModelTextures {
                base: Some(replacement.clone()),
                ..default()
            };
            let Some(promoted) = params.material_for_pass(pass, &textures) else {
                continue;
            };
            material = promoted;
        } else {
            material.base_texture = Some(replacement.clone());
        }
        commands.entity(entity).insert((
            MeshMaterial3d(materials.add(material)),
            NetworkNpcTextureVariantBound0104,
            NetworkNpcMaterialSurface0104 { root },
        ));

        let Some(params) = promoted_params else {
            continue;
        };
        let has_outline = children.get(entity).is_ok_and(|children| {
            children.iter().any(|child| {
                outline_companions.get(child).is_ok_and(|companion| {
                    companion.source_mesh_entity == entity
                        && companion.pass == LegacyPassKind::Outline
                })
            })
        });
        if has_outline {
            continue;
        }
        let Some(outline_materials) = outline_materials.as_deref_mut() else {
            continue;
        };
        let Some(outline_material) = params.outline_material() else {
            continue;
        };
        let mut companion = commands.spawn((
            Mesh3d(mesh.0.clone()),
            MeshMaterial3d(outline_materials.add(outline_material)),
            Transform::IDENTITY,
            Visibility::Inherited,
            LegacyOutlineCompanion {
                source_mesh_entity: entity,
            },
            LegacyMaterialPassCompanion {
                source_mesh_entity: entity,
                pass: LegacyPassKind::Outline,
            },
        ));
        if let Some(skin) = skin {
            companion.insert((skin.clone(), NoFrustumCulling));
        }
        if let Some(tag) = range_tag {
            companion.insert(tag.clone());
        }
        if let Some(range) = range {
            companion.insert(range.clone());
        }
        if let Some(visibility) = visibility {
            companion.insert(*visibility);
        }
        let companion = companion.id();
        commands
            .entity(entity)
            .add_child(companion)
            .insert(LegacyOutlineSource {
                width: params.outline_width,
                color: params.outline_color,
            });
    }

    // Transparent/cutout legacy shaders can materialize additional fill
    // passes as child entities. They intentionally do not carry
    // `PendingLegacyModelMaterial` or `LegacyMaterialApplied`: their source
    // renderer owns that metadata. Mirror the source's final XDT base texture
    // onto every such pass and give the shared reveal/AppearEffect path an
    // explicit completion proof. Treating these companions as independent
    // source renderers left tutorial actors hidden forever.
    for (entity, material_handle, companion) in &surface_companions {
        let Ok((source_handle, source_surface)) = bound_sources.get(companion.source_mesh_entity)
        else {
            continue;
        };
        let Some(source_base_texture) = materials
            .get(&source_handle.0)
            .map(|material| material.base_texture.clone())
        else {
            continue;
        };
        let Some(mut material) = materials.get(&material_handle.0).cloned() else {
            continue;
        };
        material.base_texture = source_base_texture;
        commands.entity(entity).insert((
            MeshMaterial3d(materials.add(material)),
            NetworkNpcTextureVariantBound0104,
            NetworkNpcMaterialSurface0104 {
                root: source_surface.root,
            },
        ));
    }
}

pub(super) fn network_npc_texture_uses_fusion_matter_0104(
    slot: NetworkNpcTextureSlot0104,
    texture_true_name: &str,
) -> bool {
    slot == NetworkNpcTextureSlot0104::Sub && texture_true_name == "spawn11_green"
}
