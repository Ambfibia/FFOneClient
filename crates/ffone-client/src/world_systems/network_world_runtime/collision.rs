use super::*;

pub(super) const CHARACTER_COLLISION_SCHEMA_V1: &str = "ffone.native-character-collision.v1";

pub(super) const CHARACTER_COLLISION_SCHEMA_V2: &str = "ffone.native-character-collision.v2";

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct NativeCharacterCollisionContract0104 {
    pub(super) schema: String,
    pub(super) glb: String,
    pub(super) glb_blake3: String,
    #[serde(default)]
    pub(super) collider_glb: Option<String>,
    #[serde(default)]
    pub(super) collider_glb_blake3: Option<String>,
    #[serde(default)]
    pub(super) source: Value,
    #[serde(default)]
    pub(super) conversion: Value,
    pub(super) colliders: Vec<NativeCharacterCollider0104>,
}

impl NativeCharacterCollisionContract0104 {
    pub(super) fn validate(
        &self,
        path: &str,
        glb: &str,
        glb_blake3: &str,
        actual_collider_glb_blake3: Option<&str>,
    ) -> Result<(), String> {
        if !matches!(
            self.schema.as_str(),
            CHARACTER_COLLISION_SCHEMA_V1 | CHARACTER_COLLISION_SCHEMA_V2
        ) {
            return Err(format!(
                "character collision {path:?} must use {CHARACTER_COLLISION_SCHEMA_V1:?} or {CHARACTER_COLLISION_SCHEMA_V2:?}"
            ));
        }
        if self.glb != glb || self.glb_blake3 != glb_blake3 {
            return Err(format!(
                "character collision {path:?} does not bind the exact installed GLB"
            ));
        }
        let expected_path = glb
            .strip_suffix(".glb")
            .map(|stem| format!("{stem}.collision.json"))
            .ok_or_else(|| format!("character collision GLB path is not .glb: {glb:?}"))?;
        if path != expected_path {
            return Err(format!(
                "character collision sidecar must be adjacent to its GLB: expected {expected_path:?}, found {path:?}"
            ));
        }
        match (
            self.collider_glb.as_deref(),
            self.collider_glb_blake3.as_deref(),
            actual_collider_glb_blake3,
        ) {
            (None, None, None) => {}
            (Some(collider_glb), Some(expected_blake3), Some(actual_blake3)) => {
                let expected_collider_path = glb
                    .strip_suffix(".glb")
                    .map(|stem| format!("{stem}.collision.glb"))
                    .expect("visual GLB suffix already validated");
                if collider_glb != expected_collider_path
                    || Path::new(collider_glb).is_absolute()
                    || collider_glb.contains(['\\', ':'])
                    || collider_glb
                        .split('/')
                        .any(|part| part.is_empty() || part == "." || part == "..")
                {
                    return Err(format!(
                        "character collision {path:?} has invalid adjacent collider GLB {collider_glb:?}"
                    ));
                }
                if expected_blake3 != actual_blake3 {
                    return Err(format!(
                        "character collision {path:?} collider GLB hash mismatch"
                    ));
                }
            }
            _ => {
                return Err(format!(
                    "character collision {path:?} must provide both colliderGlb and colliderGlbBlake3, or neither"
                ));
            }
        }
        if self.schema == CHARACTER_COLLISION_SCHEMA_V1 {
            let source_alias = self
                .source
                .get("alias")
                .and_then(Value::as_str)
                .ok_or_else(|| format!("character collision {path:?} source.alias is missing"))?;
            if !matches!(source_alias, "primary" | "patched" | "alternate") {
                return Err(format!(
                    "character collision {path:?} has unknown source alias {source_alias:?}"
                ));
            }
            for pointer in [
                "/rawContainer/relativePath",
                "/rawContainer/sha256",
                "/serializedAsset/name",
                "/serializedAsset/sha256",
                "/exactContainerRoute",
                "/sourceGeometrySha256",
            ] {
                if self
                    .source
                    .pointer(pointer)
                    .and_then(Value::as_str)
                    .is_none_or(str::is_empty)
                {
                    return Err(format!(
                        "character collision {path:?} source{pointer} is missing"
                    ));
                }
            }
            if self
                .source
                .pointer("/meshCollider/componentClassId")
                .and_then(Value::as_i64)
                != Some(64)
            {
                return Err(format!(
                    "character collision {path:?} does not prove a Unity MeshCollider"
                ));
            }
            if self
                .conversion
                .get("command")
                .and_then(Value::as_str)
                .is_none_or(str::is_empty)
                || self
                    .conversion
                    .get("toolVersion")
                    .and_then(Value::as_str)
                    .is_none_or(str::is_empty)
            {
                return Err(format!(
                    "character collision {path:?} has incomplete conversion provenance"
                ));
            }
        } else if !self.source.is_null() || !self.conversion.is_null() {
            return Err(format!(
                "character collision {path:?} v2 must keep publication provenance outside runtime assets"
            ));
        }
        if self.colliders.is_empty() {
            return Err(format!(
                "character collision {path:?} has no collider entries"
            ));
        }
        let mut ids = std::collections::BTreeSet::new();
        for collider in &self.colliders {
            if collider.id.is_empty()
                || collider.name.is_empty()
                || collider.node.is_empty()
                || !ids.insert(collider.id.as_str())
                || collider.expected_vertex_count == 0
                || collider.expected_index_count == 0
                || !collider.expected_index_count.is_multiple_of(3)
                || collider.is_trigger
                || collider.convex
            {
                return Err(format!(
                    "character collision {path:?} has an invalid exact non-convex collider {:?}",
                    collider.id
                ));
            }
        }
        Ok(())
    }
}

pub(super) fn materialize_network_npc_collision_0104(
    event: On<WorldInstanceReady>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    pending: Query<&PendingNetworkNpcCollision0104>,
    children: Query<&Children>,
    names: Query<&Name>,
) {
    let scene = event.event().entity;
    let Ok(pending) = pending.get(scene) else {
        return;
    };
    for collider in &pending.contract.colliders {
        let mut stack = children
            .get(scene)
            .map(|children| children.iter().collect::<Vec<_>>())
            .unwrap_or_default();
        let mut matches = Vec::new();
        while let Some(entity) = stack.pop() {
            if names
                .get(entity)
                .is_ok_and(|name| name.as_str() == collider.node)
            {
                matches.push(entity);
            }
            if let Ok(descendants) = children.get(entity) {
                stack.extend(descendants.iter());
            }
        }
        let [node] = matches.as_slice() else {
            commands
                .entity(pending.gameplay_root)
                .insert(NetworkNpcVisualIssue0104 {
                    npc_type: pending.npc_type,
                    detail: format!(
                        "collision sidecar expected exactly one node {:?}, found {}",
                        collider.node,
                        matches.len()
                    ),
                });
            return;
        };
        spawn_pending_authored_model_collider(
            &mut commands,
            &asset_server,
            *node,
            collider.name.clone(),
            pending.glb_path.clone(),
            collider.mesh,
            collider.primitive,
            Transform::IDENTITY,
            collider.expected_vertex_count,
            collider.expected_index_count,
        );
    }
    commands
        .entity(scene)
        .remove::<PendingNetworkNpcCollision0104>();
}
