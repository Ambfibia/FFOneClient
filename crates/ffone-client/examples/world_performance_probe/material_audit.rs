//! One-shot, read-only census after frame sampling. Equal current values do not
//! prove equal animation ownership or authorize sharing mutable materials.
use bevy::{asset::AssetId, prelude::*};
use ffone_client::legacy_model_material::LegacyModelMaterial;
use std::{collections::HashMap, path::Path};

pub(super) fn write(world: &mut World, output: &Path) {
    let mut uses: HashMap<AssetId<LegacyModelMaterial>, usize> = HashMap::new();
    for handle in world
        .query::<&MeshMaterial3d<LegacyModelMaterial>>()
        .iter(world)
    {
        *uses.entry(handle.0.id()).or_default() += 1;
    }
    let mut examples_by_id = HashMap::new();
    for (entity, handle) in world
        .query::<(Entity, &MeshMaterial3d<LegacyModelMaterial>)>()
        .iter(world)
    {
        examples_by_id.entry(handle.0.id()).or_insert_with(|| {
            use ffone_client::legacy_model_material::{
                LegacyMaterialPassCompanion, PendingLegacyModelMaterial,
            };
            let source = world
                .get::<LegacyMaterialPassCompanion>(entity)
                .map_or(entity, |pass| pass.source_mesh_entity);
            let kind = if world.get::<PendingLegacyModelMaterial>(source).is_some() {
                "model"
            } else {
                "other"
            };
            let mut cursor = entity;
            let mut hierarchy = Vec::new();
            for _ in 0..12 {
                if let Some(name) = world.get::<Name>(cursor) {
                    hierarchy.push(name.as_str().to_owned());
                }
                let Some(parent) = world.get::<ChildOf>(cursor) else {
                    break;
                };
                cursor = parent.parent();
            }
            serde_json::json!({"kind": kind, "hierarchy": hierarchy})
        });
    }
    let assets = world.resource::<Assets<LegacyModelMaterial>>();
    let mut groups: Vec<(&LegacyModelMaterial, usize, usize, serde_json::Value)> = Vec::new();
    let mut missing = 0;
    for (id, references) in &uses {
        let Some(material) = assets.get(*id) else {
            missing += 1;
            continue;
        };
        if let Some(group) = groups.iter_mut().find(|group| group.0 == material) {
            group.1 += 1;
            group.2 += references;
        } else {
            groups.push((material, 1, *references, examples_by_id[id].clone()));
        }
    }
    groups.sort_by_key(|group| std::cmp::Reverse(group.1));
    let duplicate_handles: usize = groups.iter().map(|group| group.1 - 1).sum();
    let examples: Vec<_> = groups.iter().filter(|group| group.1 > 1).take(20).map(|(material, handles, references, example)| {
        serde_json::json!({
            "distinctHandles": handles, "entityReferences": references,
            "baseTexturePath": material.base_texture.as_ref().and_then(|h| h.path()).map(ToString::to_string),
            "sortBias": material.sort_bias, "renderMode": format!("{:?}", material.render_mode),
            "gpuUvAnimation": material.gpu_uv_animation, "example": example, "materialValue": format!("{material:?}"),
        })
    }).collect();
    let report = serde_json::json!({
        "schema": "ffone.material-snapshot-audit.v1",
        "note": "Full current LegacyModelMaterial equality, including identical texture handles and pass state. Does not prove immutable ownership; equal animated poses are only candidates. Runs once after timing samples and never changes assets.",
        "boundHandles": uses.len(), "missingAssets": missing,
        "entityReferences": uses.values().sum::<usize>(),
        "uniqueCurrentValues": groups.len(), "duplicateCurrentValueHandles": duplicate_handles,
        "largestGroups": examples,
    });
    std::fs::write(
        output.join("material-duplicates.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
}
