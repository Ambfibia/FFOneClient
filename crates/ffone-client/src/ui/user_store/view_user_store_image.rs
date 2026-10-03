use super::*;

pub(super) fn spawn_user_store_image(
    parent: &mut ChildSpawnerCommands,
    element: UserStoreUiElement0104,
    rect: UserStoreUiRect,
    image: Handle<Image>,
    border: Option<BorderRect>,
) {
    let image_node = match border {
        Some(border) => sliced_image(image, border),
        None => stretched_image(image),
    };
    parent.spawn((element, rect.node(), image_node, Pickable::IGNORE));
}
