use super::*;

pub(super) fn validate_contract(contract: &PlayerSharedRigContract) -> Result<(), String> {
    if contract.schema != PLAYER_SHARED_RIG_SCHEMA {
        return Err(format!(
            "unsupported native player-rig schema {:?}; expected {:?}",
            contract.schema, PLAYER_SHARED_RIG_SCHEMA
        ));
    }
    if !contract.creator_preview_ready
        || contract.fake_animation_used
        || contract.unity_runtime_required
    {
        return Err(format!(
            "native player-rig contract is not production ready: creatorPreviewReady={}, fakeAnimationUsed={}, unityRuntimeRequired={}",
            contract.creator_preview_ready,
            contract.fake_animation_used,
            contract.unity_runtime_required
        ));
    }
    if contract.genders.len() != 2 {
        return Err(format!(
            "native player-rig contract has {} genders, expected 2",
            contract.genders.len()
        ));
    }
    let mut male_seen = false;
    let mut female_seen = false;
    for gender in &contract.genders {
        let seen = match gender.gender {
            PlayerRigGender::Male => &mut male_seen,
            PlayerRigGender::Female => &mut female_seen,
        };
        if std::mem::replace(seen, true) {
            return Err(format!(
                "native player-rig contract repeats {:?}",
                gender.gender
            ));
        }
        if gender.nodes.is_empty() || gender.nodes[0].parent_actor_bone_index.is_some() {
            return Err(format!(
                "{:?} shared skeleton has no unique root",
                gender.gender
            ));
        }
        for (index, node) in gender.nodes.iter().enumerate() {
            if node.actor_bone_index as usize != index {
                return Err(format!(
                    "{:?} actorBones index drift at {}: {}",
                    gender.gender, index, node.actor_bone_index
                ));
            }
            if node
                .parent_actor_bone_index
                .is_some_and(|parent| parent as usize >= index)
            {
                return Err(format!(
                    "{:?} actor bone {:?} has non-ancestral parent {:?}",
                    gender.gender, node.full_path, node.parent_actor_bone_index
                ));
            }
        }
        let stand1 = gender
            .clips
            .iter()
            .filter(|clip| clip.name == "stand1")
            .collect::<Vec<_>>();
        if stand1.len() != 1
            || stand1[0].runtime_status != "native-bevy-ready"
            || !gender.stand1_runtime_ready
        {
            return Err(format!(
                "{:?} shared skeleton has no certified native stand1",
                gender.gender
            ));
        }
        let expected_parts = match gender.gender {
            PlayerRigGender::Male => 54,
            PlayerRigGender::Female => 51,
        };
        let expected_choices = match gender.gender {
            PlayerRigGender::Male => 118,
            PlayerRigGender::Female => 113,
        };
        if gender.creator_parts.len() != expected_parts
            || gender.creator_choices.len() != expected_choices
            || gender.default_creator_part_routes.len() != 5
        {
            return Err(format!(
                "{:?} creator coverage drifted: parts={}, choices={}, defaults={}",
                gender.gender,
                gender.creator_parts.len(),
                gender.creator_choices.len(),
                gender.default_creator_part_routes.len()
            ));
        }
        let mut routes = BTreeSet::new();
        let mut glbs = BTreeSet::new();
        let mut published_clothes_slots = BTreeSet::new();
        for part in &gender.creator_parts {
            if !routes.insert(part.exact_route.as_str()) || part.skins.is_empty() {
                return Err(format!(
                    "{:?} creator part route {:?} is duplicate or skinless",
                    gender.gender, part.exact_route
                ));
            }
            if !glbs.insert(part.glb.as_str()) {
                return Err(format!(
                    "{:?} creator part GLB {:?} is not unique",
                    gender.gender, part.glb
                ));
            }
            if part.actor_skin_combiner_clothes_index > 4 {
                return Err(format!(
                    "{:?} creator part {:?} has invalid ActorSkinCombiner clothes slot {}",
                    gender.gender, part.exact_route, part.actor_skin_combiner_clothes_index
                ));
            }
            published_clothes_slots.insert(part.actor_skin_combiner_clothes_index);
            for skin in &part.skins {
                if !skin.exact_transform_index_parity
                    || skin.actor_bone_indices.is_empty()
                    || skin.actor_bone_indices.len() != skin.actor_bone_paths.len()
                    || skin.actor_bone_indices.len() != skin.gltf_joint_paths.len()
                {
                    return Err(format!(
                        "{:?} creator part {:?} renderer {:?} has no exact transform-index parity",
                        gender.gender, part.exact_route, skin.renderer_true_name
                    ));
                }
            }
        }
        let expected_clothes_slots = (0_u8..=4).collect::<BTreeSet<_>>();
        if published_clothes_slots != expected_clothes_slots {
            return Err(format!(
                "{:?} creator parts do not cover exact ActorSkinCombiner clothes slots 0..=4",
                gender.gender
            ));
        }
        let defaults = gender
            .default_creator_part_routes
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if defaults.len() != gender.default_creator_part_routes.len()
            || !defaults.iter().all(|route| routes.contains(route))
        {
            return Err(format!(
                "{:?} default creator routes are duplicate or outside complete coverage",
                gender.gender
            ));
        }
        let default_clothes_slots = gender
            .default_creator_part_routes
            .iter()
            .filter_map(|route| {
                gender
                    .creator_parts
                    .iter()
                    .find(|part| part.exact_route == *route)
                    .map(|part| part.actor_skin_combiner_clothes_index)
            })
            .collect::<BTreeSet<_>>();
        if default_clothes_slots != expected_clothes_slots {
            return Err(format!(
                "{:?} default creator parts do not select each ActorSkinCombiner clothes slot exactly once",
                gender.gender
            ));
        }
        let mut choice_keys = BTreeSet::new();
        for choice in &gender.creator_choices {
            if !choice_keys.insert((choice.appearance_category, choice.creation_index)) {
                return Err(format!(
                    "{:?} repeats creator choice {:?}/{}",
                    gender.gender, choice.appearance_category, choice.creation_index
                ));
            }
            let matches = gender
                .creator_parts
                .iter()
                .filter(|part| part.exact_route == choice.exact_route && part.glb == choice.glb)
                .collect::<Vec<_>>();
            let [part] = matches.as_slice() else {
                return Err(format!(
                    "{:?} creator choice {:?}/{} does not resolve exactly one certified route+GLB",
                    gender.gender, choice.appearance_category, choice.creation_index
                ));
            };
            let expected_clothes_index =
                actor_skin_combiner_clothes_index(choice.appearance_category);
            if part.actor_skin_combiner_clothes_index != expected_clothes_index {
                return Err(format!(
                    "{:?} creator choice {:?}/{} resolves ActorSkinCombiner clothes slot {}, expected {}",
                    gender.gender,
                    choice.appearance_category,
                    choice.creation_index,
                    part.actor_skin_combiner_clothes_index,
                    expected_clothes_index
                ));
            }
        }
    }
    Ok(())
}
