use super::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PlayerItemModelCatalog {
    pub(super) schema: String,
    pub(super) models: Vec<PlayerItemModelRoute>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PlayerItemModelRoute {
    pub(super) category: String,
    pub(super) true_name: String,
    pub(super) source_route: String,
    pub(super) resource_set: String,
    pub(super) model: CharacterCreationAssetReference,
}

pub(super) fn gltf_node_path(
    nodes: &[serde_json::Value],
    parents: &[Option<usize>],
    mut index: usize,
) -> Result<String, String> {
    let mut names = Vec::new();
    loop {
        names.push(
            nodes[index]["name"]
                .as_str()
                .ok_or_else(|| format!("GLB node {index} has no true name"))?,
        );
        let Some(parent) = parents[index] else {
            break;
        };
        index = parent;
    }
    names.reverse();
    Ok(names.join("/"))
}
