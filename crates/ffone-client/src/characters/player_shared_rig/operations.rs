use super::*;

pub(super) fn append_native_unequipped_parts(
    contract: &mut PlayerSharedRigContract,
    locator: &AssetLocator,
    item_catalog: &PlayerItemModelCatalog,
) -> Result<(), String> {
    for &(gender, exact_route, true_name, category, clothes_index) in &UNEQUIPPED_PARTS {
        let rig = contract
            .genders
            .iter_mut()
            .find(|candidate| candidate.gender == gender)
            .ok_or_else(|| format!("native player-rig contract has no {gender:?} rig"))?;
        if rig
            .creator_parts
            .iter()
            .any(|part| part.exact_route == exact_route)
        {
            continue;
        }
        let matches = item_catalog
            .models
            .iter()
            .filter(|model| model.category == category && model.true_name == true_name)
            .collect::<Vec<_>>();
        let [resolved] = matches.as_slice() else {
            return Err(format!(
                "player item catalog resolves {category}/{true_name} {} times",
                matches.len()
            ));
        };
        if resolved.source_route.is_empty() || resolved.resource_set.is_empty() {
            return Err(format!(
                "player item catalog route {category}/{true_name} has incomplete ownership"
            ));
        }
        let glb = resolved.model.path.as_str();
        let bytes =
            locator.read_verified(glb, Some(resolved.model.bytes), &resolved.model.blake3)?;
        let skins = synthesize_native_skin_remaps(&bytes, true_name, rig)?;
        rig.creator_parts.push(PlayerRigPartContract {
            exact_route: exact_route.to_owned(),
            true_name: true_name.to_owned(),
            glb: glb.to_owned(),
            actor_skin_combiner_clothes_index: clothes_index,
            skins,
        });
    }
    Ok(())
}

/// Extends the clean creator-only rig contract with table-owned shirt, pants,
/// and shoes models that were already published and byte-certified by the
/// player-item catalog.
///
/// The clean client object dump proves exact `ActorWearIndexTable` ownership
/// for creator choices only. Inventory tables contain many more skinned
/// clothes. Their native GLBs preserve the exported joint order, so the
/// runtime can deterministically map every joint path to one verified actor
/// bone. Because this extension does not claim a recovered source-table
/// PathID, its synthesized remaps are explicitly marked as non-exact source
/// index parity even though the native asset bytes and route ownership remain
/// fail-closed.
pub(super) fn append_native_table_skinned_wearables(
    contract: &mut PlayerSharedRigContract,
    locator: &AssetLocator,
    item_catalog: &PlayerItemModelCatalog,
) -> Result<BTreeMap<(u8, String), String>, String> {
    let bytes = locator.read(CHARACTER_CREATION_AVATAR_ITEMS_PATH)?;
    let avatar_items: CharacterCreationAvatarItems = serde_json::from_slice(&bytes).map_err(
        |error| {
            format!(
                "invalid native avatar-item contract {CHARACTER_CREATION_AVATAR_ITEMS_PATH}: {error}"
            )
        },
    )?;
    if avatar_items.schema != CHARACTER_CREATION_AVATAR_ITEMS_SCHEMA {
        return Err(format!(
            "unsupported avatar-item schema {:?}; expected {:?}",
            avatar_items.schema, CHARACTER_CREATION_AVATAR_ITEMS_SCHEMA
        ));
    }

    let mut routes = BTreeMap::<
        (u8, String),
        (
            PlayerRigGender,
            &'static str,
            u8,
            String,
            CharacterCreationAssetReference,
        ),
    >::new();
    for item in &avatar_items.items {
        let (catalog_category, clothes_index) = match item.category {
            AvatarItemCategory::Shoes => ("shoes", 0),
            AvatarItemCategory::Pants => ("pants", 1),
            AvatarItemCategory::Shirt => ("shirt", 2),
            _ => continue,
        };
        for (gender, visual) in [
            (PlayerRigGender::Male, &item.male),
            (PlayerRigGender::Female, &item.female),
        ] {
            if visual.model_status == NativeLookupStatus::Missing {
                continue;
            }
            if !matches!(
                visual.model_status,
                NativeLookupStatus::VerifiedUnique | NativeLookupStatus::VerifiedVariants
            ) || visual.models.is_empty()
            {
                return Err(format!(
                    "table-owned {:?} item {} resolves {:?} models as {:?}",
                    item.category, item.item_number, gender, visual.model_status
                ));
            }
            for model in &visual.models {
                let gender_key = match gender {
                    PlayerRigGender::Male => 0,
                    PlayerRigGender::Female => 1,
                };
                let key = (gender_key, model.exact_route.clone());
                let value = (
                    gender,
                    catalog_category,
                    clothes_index,
                    model.true_name.clone(),
                    model.native_asset.clone(),
                );
                if let Some(previous) = routes.insert(key.clone(), value.clone())
                    && previous != value
                {
                    return Err(format!(
                        "table-owned {:?} route {:?} contradicts its native model ownership",
                        item.category, key.1
                    ));
                }
            }
        }
    }

    let mut extension_failures = BTreeMap::new();
    for ((_, exact_route), (gender, category, clothes_index, true_name, native_asset)) in routes {
        let rig = contract
            .genders
            .iter_mut()
            .find(|candidate| candidate.gender == gender)
            .ok_or_else(|| format!("native player-rig contract has no {gender:?} rig"))?;
        if let Some(existing) = rig
            .creator_parts
            .iter()
            .find(|part| part.exact_route == exact_route)
        {
            if existing.true_name != true_name
                || existing.glb != native_asset.path
                || existing.actor_skin_combiner_clothes_index != clothes_index
            {
                return Err(format!(
                    "table-owned route {exact_route:?} contradicts the certified {gender:?} player-rig part"
                ));
            }
            continue;
        }
        let ownership = item_catalog
            .models
            .iter()
            .filter(|model| {
                model.category == category
                    && model.true_name == true_name
                    && model.model == native_asset
            })
            .collect::<Vec<_>>();
        let [ownership] = ownership.as_slice() else {
            return Err(format!(
                "player item catalog resolves table-owned {category}/{true_name} {} times",
                ownership.len()
            ));
        };
        if ownership.source_route.is_empty() || ownership.resource_set.is_empty() {
            return Err(format!(
                "table-owned {category}/{true_name} has incomplete model ownership"
            ));
        }
        let glb = native_asset.path.as_str();
        let bytes = locator.read_verified(glb, Some(native_asset.bytes), &native_asset.blake3)?;
        let mut skins = match synthesize_native_skin_remaps(&bytes, &true_name, rig) {
            Ok(skins) => skins,
            Err(error) => {
                extension_failures.insert((player_rig_gender_key(gender), exact_route), error);
                continue;
            }
        };
        for skin in &mut skins {
            skin.exact_transform_index_parity = false;
        }
        rig.creator_parts.push(PlayerRigPartContract {
            exact_route,
            true_name,
            glb: glb.to_owned(),
            actor_skin_combiner_clothes_index: clothes_index,
            skins,
        });
    }
    Ok(extension_failures)
}

pub(super) fn append_native_hat_variants(
    contract: &mut PlayerSharedRigContract,
    locator: &AssetLocator,
    item_catalog: &PlayerItemModelCatalog,
) -> Result<(), String> {
    let variants = contract
        .genders
        .iter()
        .flat_map(|rig| {
            rig.creator_parts.iter().filter_map(move |part| {
                let (category, clothes_index) = if part.true_name.starts_with("m_face_")
                    || part.true_name.starts_with("f_face_")
                {
                    ("mask", 3)
                } else if part.true_name.starts_with("m_head_")
                    || part.true_name.starts_with("f_head_")
                {
                    ("head", 4)
                } else {
                    return None;
                };
                part.true_name.strip_suffix("_type01").map(|stem| {
                    (
                        rig.gender,
                        format!("wear/{stem}_type02.nif"),
                        format!("{stem}_type02"),
                        category,
                        clothes_index,
                    )
                })
            })
        })
        .collect::<Vec<_>>();

    for (gender, exact_route, true_name, category, clothes_index) in variants {
        let rig = contract
            .genders
            .iter_mut()
            .find(|candidate| candidate.gender == gender)
            .ok_or_else(|| format!("native player-rig contract has no {gender:?} rig"))?;
        if rig
            .creator_parts
            .iter()
            .any(|part| part.exact_route == exact_route)
        {
            continue;
        }
        let matches = item_catalog
            .models
            .iter()
            .filter(|model| model.category == category && model.true_name == true_name)
            .collect::<Vec<_>>();
        let [resolved] = matches.as_slice() else {
            return Err(format!(
                "player item catalog resolves required hat variant {category}/{true_name} {} times",
                matches.len()
            ));
        };
        if resolved.source_route.is_empty() || resolved.resource_set.is_empty() {
            return Err(format!(
                "required hat variant {category}/{true_name} has incomplete ownership"
            ));
        }
        let glb = resolved.model.path.as_str();
        let bytes =
            locator.read_verified(glb, Some(resolved.model.bytes), &resolved.model.blake3)?;
        let skins = synthesize_native_skin_remaps(&bytes, &true_name, rig)?;
        rig.creator_parts.push(PlayerRigPartContract {
            exact_route,
            true_name,
            glb: glb.to_owned(),
            actor_skin_combiner_clothes_index: clothes_index,
            skins,
        });
    }
    Ok(())
}

/// Adds the mixed Back-table branch which legacy `SetBack` sends through the
/// actor skin combiner instead of `AttachGO(back01)`.
///
/// Equip type zero remains a rigid socket attachment. Equip type one models
/// are full actor-skinned capes and occupy the optional sixth clothes slot;
/// attaching those roots to `back01` applies the backpack basis to an entire
/// duplicate actor hierarchy and produces the large above-head offset.
pub(super) fn append_native_skinned_backs(
    contract: &mut PlayerSharedRigContract,
    locator: &AssetLocator,
    item_catalog: &PlayerItemModelCatalog,
) -> Result<(), String> {
    let bytes = locator.read(CHARACTER_CREATION_AVATAR_ITEMS_PATH)?;
    let avatar_items: CharacterCreationAvatarItems = serde_json::from_slice(&bytes).map_err(
        |error| {
            format!(
                "invalid native avatar-item contract {CHARACTER_CREATION_AVATAR_ITEMS_PATH}: {error}"
            )
        },
    )?;
    if avatar_items.schema != CHARACTER_CREATION_AVATAR_ITEMS_SCHEMA {
        return Err(format!(
            "unsupported avatar-item schema {:?}; expected {:?}",
            avatar_items.schema, CHARACTER_CREATION_AVATAR_ITEMS_SCHEMA
        ));
    }

    let mut routes =
        BTreeMap::<(u8, String), (PlayerRigGender, String, CharacterCreationAssetReference)>::new();
    for item in avatar_items
        .items
        .iter()
        .filter(|item| item.category == AvatarItemCategory::Back && item.equip_type == 1)
    {
        for (gender, visual) in [
            (PlayerRigGender::Male, &item.male),
            (PlayerRigGender::Female, &item.female),
        ] {
            if visual.model_status == NativeLookupStatus::Missing {
                continue;
            }
            if visual.model_status != NativeLookupStatus::VerifiedUnique || visual.models.len() != 1
            {
                return Err(format!(
                    "skinned Back item {} resolves {:?} models as {:?}",
                    item.item_number, gender, visual.model_status
                ));
            }
            let model = &visual.models[0];
            let gender_key = match gender {
                PlayerRigGender::Male => 0,
                PlayerRigGender::Female => 1,
            };
            let key = (gender_key, model.exact_route.clone());
            let value = (gender, model.true_name.clone(), model.native_asset.clone());
            if let Some(previous) = routes.insert(key.clone(), value.clone())
                && previous != value
            {
                return Err(format!(
                    "skinned Back route {:?} contradicts its native model ownership",
                    key.1
                ));
            }
        }
    }

    for ((_, exact_route), (gender, true_name, native_asset)) in routes {
        let ownership = item_catalog
            .models
            .iter()
            .filter(|model| {
                model.category == "back"
                    && model.true_name == true_name
                    && model.model == native_asset
            })
            .collect::<Vec<_>>();
        let [ownership] = ownership.as_slice() else {
            return Err(format!(
                "player item catalog resolves skinned Back {true_name:?} {} times",
                ownership.len()
            ));
        };
        if ownership.source_route.is_empty() || ownership.resource_set.is_empty() {
            return Err(format!(
                "skinned Back {true_name:?} has incomplete model ownership"
            ));
        }
        let glb = native_asset.path.as_str();
        let bytes = locator.read_verified(glb, Some(native_asset.bytes), &native_asset.blake3)?;
        let rig = contract
            .genders
            .iter_mut()
            .find(|candidate| candidate.gender == gender)
            .ok_or_else(|| format!("native player-rig contract has no {gender:?} rig"))?;
        let skins = synthesize_native_skin_remaps(&bytes, &true_name, rig)?;
        rig.creator_parts.push(PlayerRigPartContract {
            exact_route,
            true_name,
            glb: glb.to_owned(),
            actor_skin_combiner_clothes_index: SKINNED_BACK_CLOTHES_INDEX,
            skins,
        });
    }
    Ok(())
}

pub(super) fn synthesize_native_skin_remaps(
    glb: &[u8],
    model_root: &str,
    rig: &PlayerGenderRigContract,
) -> Result<Vec<PlayerRigSkinRemap>, String> {
    if glb.len() < 20 || &glb[0..4] != b"glTF" {
        return Err(format!("{model_root} is not a GLB 2.0 file"));
    }
    let json_len = u32::from_le_bytes(glb[12..16].try_into().expect("four bytes")) as usize;
    let json_end = 20_usize
        .checked_add(json_len)
        .filter(|end| *end <= glb.len())
        .ok_or_else(|| format!("{model_root} GLB JSON chunk exceeds the file"))?;
    let document: serde_json::Value = serde_json::from_slice(&glb[20..json_end])
        .map_err(|error| format!("{model_root} GLB has invalid JSON metadata: {error}"))?;
    let nodes = document["nodes"]
        .as_array()
        .ok_or_else(|| format!("{model_root} GLB has no nodes"))?;
    let skins = document["skins"]
        .as_array()
        .ok_or_else(|| format!("{model_root} GLB has no skins"))?;
    let mut parents = vec![None; nodes.len()];
    for (parent, node) in nodes.iter().enumerate() {
        if let Some(children) = node["children"].as_array() {
            for child in children {
                let child = child
                    .as_u64()
                    .and_then(|value| usize::try_from(value).ok())
                    .ok_or_else(|| format!("{model_root} GLB has an invalid child index"))?;
                if child >= nodes.len() || parents[child].replace(parent).is_some() {
                    return Err(format!("{model_root} GLB hierarchy is not a tree"));
                }
            }
        }
    }
    let actor_nodes = rig
        .nodes
        .iter()
        .map(|node| (node.full_path.as_str(), node.actor_bone_index))
        .collect::<BTreeMap<_, _>>();
    let mut remaps = Vec::new();
    for node in nodes {
        let (Some(renderer), Some(skin_index)) = (
            node["name"].as_str(),
            node["skin"]
                .as_u64()
                .and_then(|value| usize::try_from(value).ok()),
        ) else {
            continue;
        };
        let skin = skins.get(skin_index).ok_or_else(|| {
            format!("{model_root}/{renderer} references missing skin {skin_index}")
        })?;
        let joints = skin["joints"]
            .as_array()
            .ok_or_else(|| format!("{model_root}/{renderer} skin has no joints"))?;
        let mut actor_bone_indices = Vec::with_capacity(joints.len());
        let mut actor_bone_paths = Vec::with_capacity(joints.len());
        let mut gltf_joint_paths = Vec::with_capacity(joints.len());
        for joint in joints {
            let joint = joint
                .as_u64()
                .and_then(|value| usize::try_from(value).ok())
                .filter(|index| *index < nodes.len())
                .ok_or_else(|| format!("{model_root}/{renderer} has an invalid joint index"))?;
            let gltf_path = gltf_node_path(nodes, &parents, joint)?;
            let suffix = gltf_path
                .find("/Bip01")
                .map(|offset| &gltf_path[offset..])
                .ok_or_else(|| {
                    format!("{model_root}/{renderer} joint {gltf_path:?} is outside Bip01")
                })?;
            let actor_root = match rig.gender {
                PlayerRigGender::Male => "m",
                PlayerRigGender::Female => "w",
            };
            let actor_path = format!("{actor_root}{suffix}");
            let actor_match = actor_nodes
                .get_key_value(actor_path.as_str())
                .map(|(path, index)| ((*path).to_owned(), *index))
                .or_else(|| {
                    // Several table-only legacy wearables serialize a compact
                    // skin hierarchy without the actor's `Bip01 NonAccum`
                    // node. Their ordered bone-name tail still identifies one
                    // exact actor joint; accept only a unique suffix match and
                    // keep diagnostic parity explicitly false at publication.
                    let compact_tail = gltf_path.split_once("/Bip01/").map(|(_, tail)| tail)?;
                    let matches = rig
                        .nodes
                        .iter()
                        .filter(|node| node.full_path.ends_with(compact_tail))
                        .collect::<Vec<_>>();
                    let [matched] = matches.as_slice() else {
                        return None;
                    };
                    Some((matched.full_path.clone(), matched.actor_bone_index))
                })
                .or_else(|| {
                    // Some late table-only wearables omit an intermediate
                    // actor node such as `Bip01 Pelvis`. The terminal joint
                    // name still resolves one exact actor bone. Never guess
                    // when that terminal name is repeated in the actor rig.
                    let joint_name = gltf_path.rsplit('/').next()?;
                    let matches = rig
                        .nodes
                        .iter()
                        .filter(|node| node.true_name == joint_name)
                        .collect::<Vec<_>>();
                    let [matched] = matches.as_slice() else {
                        return None;
                    };
                    Some((matched.full_path.clone(), matched.actor_bone_index))
                })
                .ok_or_else(|| {
                    format!(
                        "{model_root}/{renderer} joint {actor_path:?} is absent from the actor rig"
                    )
                })?;
            let (actor_path, actor_index) = actor_match;
            actor_bone_indices.push(actor_index);
            actor_bone_paths.push(actor_path);
            gltf_joint_paths.push(gltf_path);
        }
        remaps.push(PlayerRigSkinRemap {
            renderer_true_name: renderer.to_owned(),
            renderer_path_id: 0,
            actor_wear_index_table_path_id: 0,
            actor_bone_indices,
            actor_bone_paths,
            gltf_joint_paths,
            exact_transform_index_parity: true,
        });
    }
    if remaps.is_empty() {
        return Err(format!("{model_root} GLB contains no skinned renderers"));
    }
    Ok(remaps)
}

pub(super) fn exact_renderer_remap<'a>(
    mut entity: Entity,
    part_root: Entity,
    remaps: &'a [PlayerRigSkinRemap],
    parents: &Query<&ChildOf>,
    names: &Query<&Name>,
) -> Result<&'a PlayerRigSkinRemap, String> {
    loop {
        if let Ok(name) = names.get(entity) {
            let matches = remaps
                .iter()
                .filter(|remap| name.as_str() == remap.renderer_true_name)
                .collect::<Vec<_>>();
            if let [matched] = matches.as_slice() {
                // A GLB may nest one renderer below another renderer-named
                // node. The nearest owning node is the surface identity; a
                // parent match belongs to a different skin palette.
                return Ok(*matched);
            }
            if matches.len() > 1 {
                return Err(format!(
                    "skinned surface nearest renderer name {:?} matched {} identities",
                    name.as_str(),
                    matches.len()
                ));
            }
        }
        if entity == part_root {
            break;
        }
        let parent = parents
            .get(entity)
            .map_err(|_| "skinned surface escaped its part scene".to_owned())?;
        entity = parent.parent();
    }
    Err("skinned surface matched no renderer identity".to_owned())
}

pub(super) fn owning_scene_root(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    scene_roots: &Query<(), With<WorldAssetRoot>>,
) -> Option<Entity> {
    loop {
        if scene_roots.get(entity).is_ok() {
            return Some(entity);
        }
        entity = parents.get(entity).ok()?.parent();
    }
}
