use super::*;

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkNpcVisualDefinition0104 {
    pub npc_type: i32,
    pub logical_name: String,
    pub category: String,
    pub glb: String,
    pub table_scale: f32,
    /// Exact XDT m_iHeight, retained in protocol centiunits for the legacy
    /// NpcMoveController downward ground probe.
    pub height_server_units: i32,
    pub main_texture: Option<NetworkNpcTextureOverride0104>,
    pub sub_texture: Option<NetworkNpcTextureOverride0104>,
    pub walk_animation_speed: f32,
    pub run_animation_speed: f32,
    pub(super) collision_path: Option<String>,
    pub(super) collision_contract: Option<Arc<NativeCharacterCollisionContract0104>>,
    pub(super) animation_effect_events: Arc<[NetworkNpcAnimationEffectEvent0104]>,
    pub(super) animation_sound_events: Arc<[NetworkNpcAnimationSoundEvent0104]>,
    /// First authored `end` event per clip (`NpcAnimation.EndAnimation`).
    pub(super) animation_ends: Arc<BTreeMap<String, f32>>,
    pub(super) material_animation_clips: Arc<[LegacyMaterialAnimationClip]>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NetworkHnpcVisualDefinition0104 {
    pub npc_type: i32,
    pub appearance_index: usize,
    pub height_server_units: i32,
    pub walk_animation_speed: f32,
    pub run_animation_speed: f32,
    pub idle_clips: Option<[String; 3]>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct NetworkNpcVisualCatalog0104 {
    pub(super) skill_animations: BTreeMap<i32, NetworkNpcSkillAnimations0104>,
    pub(super) shinies: BTreeMap<i32, Result<shiny::ShinyVisualDefinition, String>>,
    pub(super) definitions: BTreeMap<i32, NetworkNpcVisualDefinition0104>,
    pub(super) hnpc_definitions: BTreeMap<i32, NetworkHnpcVisualDefinition0104>,
    pub issues: Vec<NetworkNpcVisualCatalogIssue0104>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct NetworkNpcSkillAnimations0104 {
    pub(super) slots: [(i32, i32); 4],
    pub(super) mega_spell: bool,
}

impl NetworkNpcSkillAnimations0104 {
    pub(crate) fn hit(&self, skill_id: i16) -> Option<NetworkNpcCombatClip0104> {
        let (_, animation) = self
            .slots
            .iter()
            .find(|(id, _)| *id > 0 && *id == i32::from(skill_id))?;
        match animation {
            0 => Some(NetworkNpcCombatClip0104::Skill),
            1 => Some(NetworkNpcCombatClip0104::Skill1),
            2 => Some(NetworkNpcCombatClip0104::Skill2),
            3 => Some(NetworkNpcCombatClip0104::Skill3),
            _ => None,
        }
    }

    pub(crate) fn ready(&self, skill_id: i16) -> NetworkNpcCombatClip0104 {
        if self.mega_spell && self.slots[0].0 == i32::from(skill_id) {
            NetworkNpcCombatClip0104::MegaReady
        } else {
            NetworkNpcCombatClip0104::SkillReady
        }
    }
}

impl NetworkNpcVisualCatalog0104 {
    pub(crate) fn skill_animations(&self, npc_type: i32) -> Option<&NetworkNpcSkillAnimations0104> {
        self.skill_animations.get(&npc_type)
    }
    pub fn open(locator: &AssetLocator) -> Result<Self, String> {
        let table_set: Value = locator.read_table_set()?;
        let registry =
            ffone_client_foundation::asset_tables::character_models_from_document(&table_set)?;
        let texture_catalog: Value = locator.read_json(NPC_TEXTURE_CATALOG_PATH)?;
        let mut catalog = Self::from_documents(
            &table_set,
            &registry,
            &texture_catalog,
            |path| locator.require_file(path).map(|_| ()),
            |path| locator.require_file(path).map(|_| ()),
        )?;
        catalog.shinies = shiny::definitions(&table_set, &registry);
        let mut animation_metadata_by_glb = BTreeMap::<
            String,
            (
                Arc<[NetworkNpcAnimationEffectEvent0104]>,
                Arc<[NetworkNpcAnimationSoundEvent0104]>,
                Arc<BTreeMap<String, f32>>,
                Arc<[LegacyMaterialAnimationClip]>,
                String,
            ),
        >::new();
        let mut collision_by_path =
            BTreeMap::<String, Arc<NativeCharacterCollisionContract0104>>::new();
        for definition in catalog.definitions.values_mut() {
            let (events, sound_events, animation_ends, material_clips, glb_blake3) =
                if let Some(metadata) = animation_metadata_by_glb.get(&definition.glb) {
                    metadata.clone()
                } else {
                    let bytes = locator.read(&definition.glb)?;
                    let events: Arc<[NetworkNpcAnimationEffectEvent0104]> =
                        parse_network_npc_animation_effect_events(&bytes)?.into();
                    let sound_events: Arc<[NetworkNpcAnimationSoundEvent0104]> =
                        parse_network_npc_animation_sound_events(&bytes)?.into();
                    let animation_ends = Arc::new(parse_network_npc_animation_end_events(&bytes)?);
                    let material_clips: Arc<[LegacyMaterialAnimationClip]> =
                        parse_legacy_material_animation_clips(&bytes)?.into();
                    let glb_blake3 = blake3::hash(&bytes).to_hex().to_string();
                    animation_metadata_by_glb.insert(
                        definition.glb.clone(),
                        (
                            events.clone(),
                            sound_events.clone(),
                            animation_ends.clone(),
                            material_clips.clone(),
                            glb_blake3.clone(),
                        ),
                    );
                    (
                        events,
                        sound_events,
                        animation_ends,
                        material_clips,
                        glb_blake3,
                    )
                };
            definition.animation_effect_events = events;
            definition.animation_sound_events = sound_events;
            definition.animation_ends = animation_ends;
            definition.material_animation_clips = material_clips;
            if let Some(path) = definition.collision_path.as_deref() {
                let contract = if let Some(contract) = collision_by_path.get(path) {
                    contract.clone()
                } else {
                    let contract: NativeCharacterCollisionContract0104 = locator.read_json(path)?;
                    let collider_glb_blake3 = contract
                        .collider_glb
                        .as_deref()
                        .map(|collider_glb| {
                            locator
                                .read(collider_glb)
                                .map(|bytes| blake3::hash(&bytes).to_hex().to_string())
                        })
                        .transpose()?;
                    contract.validate(
                        path,
                        &definition.glb,
                        &glb_blake3,
                        collider_glb_blake3.as_deref(),
                    )?;
                    let contract = Arc::new(contract);
                    collision_by_path.insert(path.to_owned(), contract.clone());
                    contract
                };
                definition.collision_contract = Some(contract);
            }
        }
        Ok(catalog)
    }

    #[must_use]
    pub fn get(&self, npc_type: i32) -> Option<&NetworkNpcVisualDefinition0104> {
        self.definitions.get(&npc_type)
    }
    pub fn shiny_models(&self) -> impl Iterator<Item = (i32, &str)> {
        self.shinies.iter().filter_map(|(id,visual)|visual.as_ref().ok().map(|v|(*id,v.glb.as_str())))
    }

    #[must_use]
    pub fn get_hnpc(&self, npc_type: i32) -> Option<&NetworkHnpcVisualDefinition0104> {
        self.hnpc_definitions.get(&npc_type)
    }

    #[must_use]
    pub fn hnpc_len(&self) -> usize {
        self.hnpc_definitions.len()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty() && self.hnpc_definitions.is_empty()
    }

    pub(super) fn from_documents(
        table_set: &Value,
        registry: &Value,
        texture_catalog: &Value,
        mut require_model: impl FnMut(&str) -> Result<(), String>,
        mut require_texture: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<Self, String> {
        let registry_models = parse_character_registry(registry)?;
        let texture_catalog = parse_npc_texture_catalog(texture_catalog, &mut require_texture)?;
        let tables = table_set
            .get("tables")
            .and_then(Value::as_array)
            .ok_or_else(|| "TableData table-set has no tables array".to_owned())?;
        let matches = tables
            .iter()
            .filter(|table| table.get("name").and_then(Value::as_str) == Some(CONSOLIDATED_TABLE))
            .collect::<Vec<_>>();
        let [table] = matches.as_slice() else {
            return Err(format!(
                "TableData must contain exactly one {CONSOLIDATED_TABLE:?} table"
            ));
        };
        let npc_table = table
            .pointer("/value/m_pNpcTable")
            .ok_or_else(|| "consolidated TableData has no m_pNpcTable".to_owned())?;
        let npc_rows = npc_table
            .get("m_pNpcData")
            .and_then(Value::as_array)
            .ok_or_else(|| "m_pNpcTable has no m_pNpcData array".to_owned())?;
        let mesh_rows = npc_table
            .get("m_pNpcMeshData")
            .and_then(Value::as_array)
            .ok_or_else(|| "m_pNpcTable has no m_pNpcMeshData array".to_owned())?;

        let skill_types: BTreeMap<i64, i64> = table
            .pointer("/value/m_pSkillTable/m_pSkillData")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|skill| {
                Some((
                    skill.get("m_iSkillNumber")?.as_i64()?,
                    skill.get("m_iSkillType")?.as_i64()?,
                ))
            })
            .collect();
        let mut catalog = Self::default();
        for (row_index, row) in npc_rows.iter().enumerate() {
            let context = format!("m_pNpcData[{row_index}]");
            let npc_type = required_i32(row, "m_iNpcNumber", &context)?;
            if npc_type <= 0 {
                continue;
            }
            let mut slots = [(0, 0); 4];
            for (index, (skill, animation)) in [
                ("m_iMegaType", "m_iMegaAni"),
                ("m_iActiveSkill1", "m_iActiveSkill1Ani"),
                ("m_iActiveSkill2", "m_iActiveSkill2Ani"),
                ("m_iSupportSkill", "m_iSupportSkillAni"),
            ]
            .into_iter()
            .enumerate()
            {
                slots[index] = (
                    i32::try_from(optional_i64(row, skill, &context)?.unwrap_or(0))
                        .map_err(|_| format!("{context}.{skill} must fit i32"))?,
                    i32::try_from(optional_i64(row, animation, &context)?.unwrap_or(0))
                        .map_err(|_| format!("{context}.{animation} must fit i32"))?,
                );
            }
            let mega_spell = slots[0].0 > 0
                && skill_types
                    .get(&i64::from(slots[0].0))
                    .is_some_and(|skill_type| *skill_type != 23);
            catalog.skill_animations.insert(
                npc_type,
                NetworkNpcSkillAnimations0104 { slots, mega_spell },
            );
            if optional_i64(row, "m_iHNpc", &context)?.unwrap_or_default() != 0 {
                let appearance_index = usize::try_from(required_i64(row, "m_iHNpcNum", &context)?)
                    .map_err(|_| format!("{context}.m_iHNpcNum must be non-negative"))?;
                let height_server_units = required_i32(row, "m_iHeight", &context)?;
                if height_server_units <= 0 {
                    catalog.issue(npc_type, format!("{context}.m_iHeight must be positive"));
                    continue;
                }
                let definition = NetworkHnpcVisualDefinition0104 {
                    npc_type,
                    appearance_index,
                    height_server_units,
                    idle_clips: hnpc_idle_clips(row, mesh_rows, &context)?,
                    walk_animation_speed: positive_speed_or_one(required_f32(
                        row,
                        "m_fWalkAnimationSpeed",
                        &context,
                    )?),
                    run_animation_speed: positive_speed_or_one(required_f32(
                        row,
                        "m_fRunAnimationSpeed",
                        &context,
                    )?),
                };
                if let Some(previous) = catalog
                    .hnpc_definitions
                    .insert(npc_type, definition.clone())
                    && previous != definition
                {
                    return Err(format!(
                        "contradictory HNPC visual definitions for NPC type {npc_type}"
                    ));
                }
                continue;
            }
            let npc_class = optional_i64(row, "m_iNpcType", &context)?.unwrap_or_default();
            // Primary NpcMoveController.SetupNPC keeps these server-owned
            // roots alive as location probes but disables their model, print
            // name, and damage presentation. Most of the rows point at the
            // shared ObjectNPC1 cube, so resolving them through the ordinary
            // visual catalog leaks that hidden marker into the world.
            if npc_class >= 100 {
                continue;
            }
            let mesh_index = usize::try_from(required_i64(row, "m_iMesh", &context)?)
                .map_err(|_| format!("{context}.m_iMesh must be non-negative"))?;
            let Some(mesh) = mesh_rows.get(mesh_index) else {
                catalog.issue(
                    npc_type,
                    format!("{context}.m_iMesh={mesh_index} is outside m_pNpcMeshData"),
                );
                continue;
            };
            let logical_name = required_string(
                mesh,
                "m_pstrMMeshModelString",
                &format!("m_pNpcMeshData[{mesh_index}]"),
            )?;
            if logical_name.is_empty() || logical_name.eq_ignore_ascii_case("null") {
                catalog.issue(
                    npc_type,
                    format!("{context} has no renderable male mesh identity"),
                );
                continue;
            }
            let table_scale = required_f32(row, "m_fScale", &context)?;
            if !table_scale.is_finite() || table_scale <= 0.0 {
                catalog.issue(
                    npc_type,
                    format!("{context}.m_fScale must be finite and positive"),
                );
                continue;
            }
            let height_server_units = required_i32(row, "m_iHeight", &context)?;
            if height_server_units <= 0 {
                catalog.issue(npc_type, format!("{context}.m_iHeight must be positive"));
                continue;
            }

            let main_texture_name = optional_texture_name(mesh, "m_pstrMTextureString");
            let sub_texture_name = optional_texture_name(mesh, "m_pstrMTextureString2");
            let main_texture = resolve_npc_texture_override(
                &texture_catalog,
                main_texture_name.as_deref(),
                "main",
                npc_type,
                &context,
                &mut catalog,
            );
            let sub_texture = resolve_npc_texture_override(
                &texture_catalog,
                sub_texture_name.as_deref(),
                "sub",
                npc_type,
                &context,
                &mut catalog,
            );

            let Some(candidates) = registry_models.get(logical_name) else {
                catalog.issue(
                    npc_type,
                    format!("XDT mesh {logical_name:?} has no semantic character-registry entry"),
                );
                continue;
            };
            let Some(model) = select_registry_model(candidates) else {
                catalog.issue(
                    npc_type,
                    format!(
                        "XDT mesh {logical_name:?} resolves to ambiguous registry categories: {}",
                        candidates
                            .iter()
                            .map(|candidate| candidate.category.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                );
                continue;
            };
            if let Err(error) = require_model(&model.glb) {
                catalog.issue(
                    npc_type,
                    format!("semantic GLB {:?} is unavailable: {error}", model.glb),
                );
                continue;
            }
            let definition = NetworkNpcVisualDefinition0104 {
                npc_type,
                // The XDT value may be an audited legacy alias. Scene root
                // normalization must still target the GLB's true Unity root.
                logical_name: model.logical_name.clone(),
                category: model.category.clone(),
                glb: model.glb.clone(),
                table_scale,
                height_server_units,
                main_texture,
                sub_texture,
                walk_animation_speed: positive_speed_or_one(required_f32(
                    row,
                    "m_fWalkAnimationSpeed",
                    &context,
                )?),
                run_animation_speed: positive_speed_or_one(required_f32(
                    row,
                    "m_fRunAnimationSpeed",
                    &context,
                )?),
                collision_path: model.collision.clone(),
                collision_contract: None,
                animation_effect_events: Arc::from([]),
                animation_sound_events: Arc::from([]),
                animation_ends: Arc::default(),
                material_animation_clips: Arc::from([]),
            };
            if let Some(previous) = catalog.definitions.insert(npc_type, definition.clone())
                && previous != definition
            {
                return Err(format!(
                    "contradictory visual definitions for NPC type {npc_type}"
                ));
            }
        }
        Ok(catalog)
    }

    pub(super) fn issue(&mut self, npc_type: i32, detail: String) {
        self.issues
            .push(NetworkNpcVisualCatalogIssue0104 { npc_type, detail });
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct NativeCharacterCollider0104 {
    pub(super) id: String,
    pub(super) name: String,
    pub(super) node: String,
    pub(super) mesh: usize,
    pub(super) primitive: usize,
    pub(super) expected_vertex_count: usize,
    pub(super) expected_index_count: usize,
    pub(super) is_trigger: bool,
    pub(super) convex: bool,
}

#[derive(Default, Resource)]
pub struct NetworkPcVisualDataState0104 {
    pub attempted: bool,
    pub(super) data: Option<Arc<CharacterCreationData>>,
    pub error: Option<String>,
}

#[derive(Default, Resource)]
pub(super) struct NetworkPcVisualGeneration0104(pub(super) u64);

#[derive(Default, Resource)]
pub(super) struct NetworkHnpcVisualGeneration0104(pub(super) u64);

#[derive(Clone, Debug, Component)]
pub struct NetworkPcVisual0104 {
    pub pc_id: i32,
    pub rig_root: Entity,
    pub generation: u64,
    pub(super) request: PendingPcVisual0104,
}

#[derive(Clone, Debug, PartialEq, Eq, Component)]
pub struct NetworkPcVisualIssue0104 {
    pub pc_id: i32,
    pub detail: String,
}

#[derive(Clone, Debug, Component)]
pub(super) struct NetworkPcRig0104 {
    pub(super) pc_root: Entity,
    pub(super) pc_id: i32,
    pub(super) look: NativePlayerLook,
}

/// One exact appearance-owned remote-player part. Shared-skin scenes and the
/// legacy rigid Hat/Glasses/Back `AttachGO` models use the same marker so the
/// material binder and hidden-until-ready gate validate the complete look.
/// Weapon stays outside this contract because its packet-driven animation and
/// attachment owner already exist separately below.
#[derive(Clone, Debug, Component)]
pub(super) struct NetworkPcAppearancePart0104 {
    pub(super) rig_root: Entity,
    pub(super) kind: NativePlayerPartKind,
    pub(super) exact_route: String,
    pub(super) glb: String,
}

#[derive(Clone, Debug, Component)]
pub(super) struct PendingNetworkPcAppearanceAttachment0104 {
    pub(super) rig_root: Entity,
    pub(super) socket_full_path: String,
    pub(super) socket_local_scale_override: Option<Vec3>,
}

#[derive(Clone, Debug, Component)]
pub(super) struct PendingNetworkPcWeaponAttachment0104 {
    pub(super) rig_root: Entity,
    pub(super) socket_full_path: String,
    pub(super) socket_local_scale_override: Option<Vec3>,
}

#[derive(Clone, Debug, Component)]
pub(super) struct NetworkPcWeaponAttachment0104;

#[derive(Clone, Debug, Component)]
pub struct NetworkHnpcVisual0104 {
    pub npc_type: i32,
    pub appearance_index: usize,
    pub rig_root: Entity,
    pub generation: u64,
    pub walk_animation_speed: f32,
    pub run_animation_speed: f32,
}

#[derive(Clone, Debug, Component)]
pub(super) struct NetworkHnpcRig0104 {
    pub(super) npc_root: Entity,
    pub(super) npc_type: i32,
    pub(super) look: NativePlayerLook,
    pub(super) idle_clips: Option<[String; 3]>,
    pub(super) animation_ends: Arc<BTreeMap<String, f32>>,
    pub(super) animation_sounds: Arc<[NetworkNpcAnimationSoundEvent0104]>,
}

#[derive(Clone, Debug, Component)]
pub(super) struct NetworkHnpcPart0104 {
    pub(super) rig_root: Entity,
    pub(super) exact_route: String,
    pub(super) glb: String,
}

#[derive(Clone, Debug, Component)]
pub(super) struct PendingNetworkHnpcAttachment0104 {
    pub(super) rig_root: Entity,
    pub(super) socket_full_path: String,
    pub(super) socket_local_scale_override: Option<Vec3>,
}

/// Material and attachment completion for consumers that own their own NPC lifecycle.
pub struct NativeHnpcAppearancePlugin;

impl Plugin for NativeHnpcAppearancePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                bind_network_hnpc_attachments_0104,
                bind_network_hnpc_materials_0104,
                finalize_network_hnpc_visuals_0104,
            )
                .chain(),
        );
    }
}

#[derive(Clone, Debug, Component)]
pub struct NetworkNpcVisual0104 {
    pub npc_type: i32,
    pub logical_name: String,
    pub glb_path: String,
    pub table_scale: f32,
    pub visual_container: Entity,
    pub scene: Entity,
    pub(super) gltf: Handle<Gltf>,
    pub(super) main_texture: Option<NetworkNpcTextureOverride0104>,
    pub(super) sub_texture: Option<NetworkNpcTextureOverride0104>,
    pub(super) walk_animation_speed: f32,
    pub(super) run_animation_speed: f32,
    pub(super) animation_effect_events: Arc<[NetworkNpcAnimationEffectEvent0104]>,
    pub(super) animation_sound_events: Arc<[NetworkNpcAnimationSoundEvent0104]>,
    pub(super) animation_ends: Arc<BTreeMap<String, f32>>,
    pub(super) material_animation_clips: Arc<[LegacyMaterialAnimationClip]>,
}

/// Active NPC branch of Retrobution's shared `AppearEffect`.
///
/// Primary `NpcMoveController.SetModel` invokes this for ordinary world NPCs
/// and tutorial NPCs alike, after the model and its XDT texture variants are
/// ready.
#[derive(Clone, Copy, Debug, Default, PartialEq, Component)]
pub struct NetworkNpcAppearEffect0104 {
    pub(super) elapsed_seconds: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
pub struct NetworkNpcAppearOutlineBound0104;

/// Per-root vertical state and exact XDT probe height used by the ordinary
/// world replacement for legacy NpcMoveController grounding.
#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub struct NetworkNpcGrounding0104 {
    pub(crate) probe_height: f32,
    pub(crate) vertical_velocity: f32,
}

impl NetworkNpcGrounding0104 {
    #[must_use]
    pub(super) fn from_server_height(height_server_units: i32) -> Self {
        debug_assert!(height_server_units > 0);
        Self {
            probe_height: protocol_distance_to_native(height_server_units),
            vertical_velocity: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Component)]
pub struct NetworkNpcVisualIssue0104 {
    pub npc_type: i32,
    pub detail: String,
}

#[derive(Clone, Debug, Component)]
pub(super) struct PendingNetworkNpcCollision0104 {
    pub(super) gameplay_root: Entity,
    pub(super) npc_type: i32,
    pub(super) glb_path: String,
    pub(super) contract: Arc<NativeCharacterCollisionContract0104>,
}

/// Keep locomotion through short gaps between streamed path segments. The root
/// still stops exactly at the packet destination; only the idle transition waits.
#[derive(Component)]
pub struct NetworkNpcMotionSettled0104 {
    pub(super) since: f64,
}

#[derive(Default, Component)]
pub(super) struct NetworkNpcHighAnimations0104 {
    pub(super) reset_revision: u64,
    pub(super) applied: [Option<NetworkNpcAnimationApplied0104>; 2],
    pub(super) requests: [u64; 2],
    pub(super) cursors: [Option<NetworkNpcIdleEventCursor0104>; 2],
}

/// `NpcAnimation.MakeUperLayer` plays every melee/wound state with
/// `AnimationBlendMode.Additive`: Unity adds the difference between the
/// current frame and the clip's first frame to the low layer. Several clean
/// clips (road golem, ghost and tireman wound) are authored from the bind
/// pose, so playing them as full-body states showed that T-pose. Bevy's Add
/// node composes its children in ascending node order: `reference` samples
/// the clip at 0 s with weight -1 before `clip`, giving
/// `clip(t) * clip(0)^-1 * low`. Published delta clips start at identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct NetworkNpcAdditivePair0104 {
    pub(super) reference: AnimationNodeIndex,
    pub(super) clip: AnimationNodeIndex,
}
