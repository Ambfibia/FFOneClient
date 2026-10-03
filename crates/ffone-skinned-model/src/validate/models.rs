use super::*;

pub(super) fn model_node_path(model: &NativeModel, node: usize) -> String {
    let mut segments = Vec::new();
    let mut cursor = Some(node as u32);
    while let Some(index) = cursor {
        let node = &model.nodes[index as usize];
        segments.push(node.name.as_str());
        cursor = node.parent;
    }
    segments.reverse();
    segments.join("/")
}
