//! Native file loading and binding; no converter or legacy input is required.
use super::*;
use bevy::asset::{AssetLoader, LoadContext, io::Reader};
use ffone_ui_layout::quit_menu::{QUIT_MENU_DOCUMENT_PATH, QuitMenuAction, QuitMenuDocument};

#[derive(Asset, TypePath, Clone, Debug)]
#[type_path = "ffone_client::quit_menu_ui::document"]
struct DocumentAsset(QuitMenuDocument);
#[derive(Default, TypePath)]
#[type_path = "ffone_client::quit_menu_ui::document"]
struct DocumentLoader;
impl AssetLoader for DocumentLoader {
    type Asset = DocumentAsset;
    type Settings = ();
    type Error = std::io::Error;
    async fn load(
        &self,
        reader: &mut dyn Reader,
        _: &(),
        _: &mut LoadContext<'_>,
    ) -> Result<DocumentAsset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        QuitMenuDocument::from_json(&bytes)
            .map(DocumentAsset)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
    }
    fn extensions(&self) -> &[&str] {
        &["ffquit.json"]
    }
}
#[derive(Resource)]
struct DocumentHandle(Handle<DocumentAsset>);

pub(super) fn install(app: &mut App) {
    app.init_asset::<DocumentAsset>()
        .init_asset_loader::<DocumentLoader>()
        .add_systems(crate::ui_startup::NativeGameplayUiStartup, request)
        .add_systems(
            Update,
            bind.after(QuitMenuUiSet::Visuals)
                .in_set(crate::ui_startup::NativeUiStartupSet),
        );
}
fn request(mut commands: Commands, server: Res<AssetServer>) {
    commands.insert_resource(DocumentHandle(server.load(QUIT_MENU_DOCUMENT_PATH)));
}
fn kind(action: QuitMenuAction) -> QuitMenuButtonKind {
    match action {
        QuitMenuAction::ChangeCharacter => QuitMenuButtonKind::ChangeCharacter,
        QuitMenuAction::QuitGame => QuitMenuButtonKind::QuitGame,
        QuitMenuAction::Cancel => QuitMenuButtonKind::Cancel,
    }
}

fn bind(
    mut commands: Commands,
    server: Res<AssetServer>,
    documents: Res<Assets<DocumentAsset>>,
    handle: Option<Res<DocumentHandle>>,
    assets: Option<ResMut<QuitMenuUiAssets>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    model: Res<QuitMenuUiModel>,
    mut previous: Local<Option<QuitMenuDocument>>,
    mut nodes: ParamSet<(
        Query<&mut Node, With<QuitMenuUiRoot>>,
        Query<(&mut Node, &mut UiTransform), With<QuitMenuDialog>>,
        Query<(&QuitMenuButton, &Interaction, &mut Node, &mut ImageNode)>,
        Query<
            (&mut ImageNode, Option<&QuitMenuBackdrop>),
            Or<(With<QuitMenuBackdrop>, With<QuitMenuDialog>)>,
        >,
    )>,
    mut labels: Query<
        (
            Entity,
            &QuitMenuButtonLabel,
            &mut TextFont,
            &mut LineHeight,
            &mut TextColor,
            &mut UiTransform,
            &mut TextLayout,
        ),
        Without<QuitMenuDialog>,
    >,
) {
    let Some(handle) = handle else {
        return;
    };
    let Some(DocumentAsset(document)) = documents.get(&handle.0) else {
        // Invalid or missing files remain a visible asset-loader error; no guessed UI.
        for mut root in &mut nodes.p0() {
            root.display = Display::None;
        }
        return;
    };
    let Some(mut assets) = assets else {
        return;
    };
    let changed = previous.as_ref() != Some(document);
    if changed {
        assets.backdrop = server.load(document.backdrop.clone());
        assets.dialog = server.load(document.dialog.clone());
        assets.font = server.load(document.font.clone());
        assets.button_normal = server.load(document.styles[0].normal.clone());
        assets.button_hover = server.load(document.styles[0].hover.clone());
        assets.button_active = document.styles[0]
            .active
            .as_ref()
            .map(|p| server.load(p.clone()));
        assets.cancel_normal = server.load(document.styles[1].normal.clone());
        assets.cancel_hover = server.load(document.styles[1].hover.clone());
        assets.cancel_active = document.styles[1]
            .active
            .as_ref()
            .map(|p| server.load(p.clone()));
    }
    if changed {
        for (mut image, backdrop) in &mut nodes.p3() {
            image.image = if backdrop.is_some() {
                assets.backdrop.clone()
            } else {
                assets.dialog.clone()
            };
        }
    }
    if let Ok(window) = windows.single() {
        let width = legacy_screen_extent(window.width());
        let height = legacy_screen_extent(window.height());
        let scale = model.ui_scale_override().unwrap_or_else(|| {
            (height as f32 / document.reference_height * document.scale_nudge).max(1.0)
        });
        let [w, h] = document.dialog_size;
        let layout = scaled_group_layout(
            QuitMenuUiRect::new(
                ((width - w as i32) / 2) as f32,
                ((height - h as i32) / 2) as f32,
                w,
                h,
            ),
            Vec2::new((width / 2) as f32, (height / 2) as f32),
            scale,
        );
        for (mut node, mut transform) in &mut nodes.p1() {
            apply_scaled_group(&mut node, &mut transform, layout);
        }
    }
    for (marker, interaction, mut node, mut image) in &mut nodes.p2() {
        let control = document
            .buttons
            .iter()
            .find(|b| kind(b.action) == marker.kind)
            .expect("validated action set");
        let style = &document.styles[control.style];
        let [x, y, w, h] = control.rect;
        if changed {
            node.left = px(x);
            node.top = px(y);
            node.width = px(w);
            node.height = px(h);
        }
        let [left, right, top, bottom] = style.padding;
        if changed {
            node.padding = UiRect {
                left: px(left),
                right: px(right),
                top: px(top),
                bottom: px(bottom),
            };
        }
        if changed {
            node.overflow = if style.clips_text {
                Overflow::clip()
            } else {
                Overflow::visible()
            };
        }
        let visual = if control.style == 0 {
            QuitMenuButtonVisual::Standard
        } else {
            QuitMenuButtonVisual::Cancel
        };
        let active = if model.visible && model.enabled {
            *interaction
        } else {
            Interaction::None
        };
        let [left, right, top, bottom] = style.border;
        let desired = assets.button_image(visual, active);
        if changed || image.image != desired {
            *image = sliced_image(
                desired,
                BorderRect {
                    min_inset: Vec2::new(left, top),
                    max_inset: Vec2::new(right, bottom),
                },
            );
            image.visual_box = match style.background_box {
                ffone_ui_layout::quit_menu::QuitMenuBackgroundBox::BorderBox => {
                    bevy::ui::VisualBox::BorderBox
                }
            };
        }
    }
    for (entity, marker, mut font, mut line, mut color, mut transform, mut layout) in &mut labels {
        let control = document
            .buttons
            .iter()
            .find(|b| kind(b.action) == marker.kind)
            .expect("validated action set");
        let style = &document.styles[control.style];
        if changed {
            commands.entity(entity).insert(LocalizedText::new(
                control.localization_key.clone(),
                control.fallback.clone(),
            ));
            font.font = assets.font.clone().into();
            font.font_size = document.font_size.into();
            *line = LineHeight::Px(document.line_height);
            let [x, y] = style.text_offset();
            *transform = UiTransform::from_translation(Val2::px(x, y));
            *layout = TextLayout::new(
                Justify::Center,
                if style.word_wrap {
                    LineBreak::WordBoundary
                } else {
                    LineBreak::NoWrap
                },
            );
        }
        // Production visual system handles interaction colors; use converted values.
        let interaction = nodes
            .p2()
            .iter()
            .find(|(b, _, _, _)| b.kind == marker.kind)
            .map(|(_, i, _, _)| *i)
            .unwrap_or(Interaction::None);
        let interaction = if model.visible && model.enabled {
            interaction
        } else {
            Interaction::None
        };
        let rgba = match interaction {
            Interaction::Hovered => style.hover_color,
            Interaction::Pressed => style.active_color,
            Interaction::None => style.normal_color,
        };
        let desired = Color::srgba(rgba[0], rgba[1], rgba[2], rgba[3]);
        if color.0 != desired {
            color.0 = desired;
        }
    }
    if changed {
        *previous = Some(document.clone());
    }
}

#[cfg(test)]
mod tests;
