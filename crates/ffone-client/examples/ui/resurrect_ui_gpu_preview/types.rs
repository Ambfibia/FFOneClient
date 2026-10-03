use super::*;

#[derive(Resource)]
pub(super) struct PreviewOutput(pub(super) PathBuf);

pub(super) struct PreviewImage {
    pub(super) role: ResurrectTextureRole,
    pub(super) handle: Handle<Image>,
}

#[derive(Resource)]
pub(super) struct PreviewAssets {
    pub(super) images: Vec<PreviewImage>,
    pub(super) fonts: [Handle<Font>; 2],
}

impl PreviewAssets {
    pub(super) fn image(&self, role: ResurrectTextureRole) -> &Handle<Image> {
        &self
            .images
            .iter()
            .find(|image| image.role == role)
            .unwrap_or_else(|| panic!("missing preview handle for {role:?}"))
            .handle
    }
}

#[derive(SystemParam)]
pub(super) struct CaptureQueries<'w, 's> {
    pub(super) roots: Query<'w, 's, (&'static Node, &'static ComputedNode, &'static GlobalZIndex)>,
    pub(super) image_nodes: Query<
        'w,
        's,
        (
            &'static Node,
            &'static ComputedNode,
            &'static ImageNode,
            Option<&'static UiTransform>,
            Option<&'static ZIndex>,
        ),
        Without<Button>,
    >,
    pub(super) buttons: Query<
        'w,
        's,
        (
            &'static Node,
            &'static ComputedNode,
            &'static ZIndex,
            &'static Interaction,
            &'static ImageNode,
            &'static Children,
        ),
        With<Button>,
    >,
    pub(super) all_texts: Query<'w, 's, &'static Text>,
    pub(super) text_copy: Query<'w, 's, (&'static Text, &'static LocalizedText)>,
    pub(super) text_styles: Query<
        'w,
        's,
        (
            Entity,
            &'static Text,
            &'static LocalizedText,
            (&'static TextFont, &'static LineHeight),
            &'static TextLayout,
            &'static TextLayoutInfo,
            &'static ResurrectTextStyle,
            &'static ComputedNode,
            &'static UiTransform,
            &'static InheritedVisibility,
            &'static ChildOf,
        ),
    >,
    pub(super) computed_nodes: Query<'w, 's, &'static ComputedNode>,
}
