//! Native equipment fitting uses the production item resolver and shared player rig.
use super::*;
use bevy::mesh::{
    VertexAttributeValues,
    skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
};
use ffone_client::{
    character_creation_data::{CharacterCreationData, ResolvedCreatorSelection},
    character_creation_ui::{CharacterAppearance, CharacterGender},
    player_preview::{
        NativePlayerPreviewCamera, NativePlayerPreviewModel, NativePlayerPreviewStatus,
    },
};
use ffone_protocol::{CharacterEquipSlot0104, ItemBase0104, Nano0104, PcAppearance0104};
use ffone_runtime_contracts::{AvatarItemCategory, AvatarItemLookup};

#[derive(Resource)]
pub(super) struct EquipmentLibrary {
    data: CharacterCreationData,
    bases: [ResolvedCreatorSelection; 2],
    outfits: [[ItemBase0104; 9]; 2],
    pub entries: BTreeMap<usize, AvatarItemLookup>,
    applied: Option<(usize, bool, bool)>,
    error: Option<LocalizedText>,
}

const CATEGORIES: [AvatarItemCategory; 7] = [
    AvatarItemCategory::Shirt,
    AvatarItemCategory::Pants,
    AvatarItemCategory::Shoes,
    AvatarItemCategory::Hat,
    AvatarItemCategory::Glasses,
    AvatarItemCategory::Back,
    AvatarItemCategory::Weapon,
];

pub(super) fn category_slug(category: AvatarItemCategory) -> &'static str {
    match category {
        AvatarItemCategory::Shirt => "shirt",
        AvatarItemCategory::Pants => "pants",
        AvatarItemCategory::Shoes => "shoes",
        AvatarItemCategory::Hat => "hat",
        AvatarItemCategory::Glasses => "glasses",
        AvatarItemCategory::Back => "back",
        AvatarItemCategory::Weapon => "weapon",
        _ => "unsupported",
    }
}

fn category_text(category: AvatarItemCategory) -> LocalizedText {
    let (key, fallback) = match category {
        AvatarItemCategory::Shirt => ("ui.editor.equipment.shirt", "Tops"),
        AvatarItemCategory::Pants => ("ui.editor.equipment.pants", "Pants"),
        AvatarItemCategory::Shoes => ("ui.editor.equipment.shoes", "Shoes"),
        AvatarItemCategory::Hat => ("ui.editor.equipment.hat", "Hats"),
        AvatarItemCategory::Glasses => ("ui.editor.equipment.glasses", "Glasses"),
        AvatarItemCategory::Back => ("ui.editor.equipment.back", "Back"),
        AvatarItemCategory::Weapon => ("ui.editor.equipment.weapon", "Weapons"),
        _ => unreachable!("only wearable categories enter the editor"),
    };
    LocalizedText::new(key, fallback)
}

fn slot(category: AvatarItemCategory) -> usize {
    (match category {
        AvatarItemCategory::Shirt => CharacterEquipSlot0104::UpperBody,
        AvatarItemCategory::Pants => CharacterEquipSlot0104::LowerBody,
        AvatarItemCategory::Shoes => CharacterEquipSlot0104::Foot,
        AvatarItemCategory::Hat => CharacterEquipSlot0104::Head,
        AvatarItemCategory::Glasses => CharacterEquipSlot0104::Face,
        AvatarItemCategory::Back => CharacterEquipSlot0104::Back,
        AvatarItemCategory::Weapon => CharacterEquipSlot0104::Hand,
        _ => unreachable!(),
    }) as usize
}

fn base_outfit(base: &ResolvedCreatorSelection) -> [ItemBase0104; 9] {
    let mut items = [ItemBase0104 {
        item_type: 0,
        item_id: 0,
        option: 0,
        time_limit: 0,
    }; 9];
    items[slot(AvatarItemCategory::Shirt)].item_id = base.equipped.upper_body_id;
    items[slot(AvatarItemCategory::Pants)].item_id = base.equipped.lower_body_id;
    items[slot(AvatarItemCategory::Shoes)].item_id = base.equipped.foot_id;
    items
}

impl EquipmentLibrary {
    pub(super) fn replace_tables(&mut self, mut replacement: Self) {
        replacement.outfits = self.outfits;
        *self = replacement;
    }
    pub fn capture_error(&self, model: &NativePlayerPreviewModel) -> Option<String> {
        if self.error.is_some() {
            return Some("equipment selection rejected".to_owned());
        }
        if let NativePlayerPreviewStatus::Blocked(error) = &model.status {
            Some(error.clone())
        } else {
            None
        }
    }
    pub fn open(root: &std::path::Path, catalog: &mut EditorCatalog) -> Result<Self, String> {
        let data = CharacterCreationData::open(root).map_err(|e| e.to_string())?;
        let male = data
            .resolve_creator(1, 1, "Editor", "Male", &CharacterAppearance::default())
            .map_err(|e| e.to_string())?;
        let female = data
            .resolve_creator(
                2,
                1,
                "Editor",
                "Female",
                &CharacterAppearance {
                    gender: CharacterGender::Girl,
                    ..default()
                },
            )
            .map_err(|e| e.to_string())?;
        let mut items: Vec<_> = data
            .avatar_items_document()
            .items
            .iter()
            .filter(|item| CATEGORIES.contains(&item.category))
            .cloned()
            .collect();
        items.sort_by_key(|item| (slot(item.category), item.item_number));
        let mut entries = BTreeMap::new();
        for item in items {
            let index = catalog.entries.len();
            let visual = &item.male;
            catalog.entries.push(EditorCatalogEntry {
                kind: CatalogKind::Equipment,
                display_name: item.name.clone(),
                semantic_id: format!(
                    "equipment/{}/{}",
                    category_slug(item.category),
                    item.item_number
                ),
                logical_name: visual.source_model_true_name.clone().unwrap_or_default(),
                glb: visual
                    .models
                    .first()
                    .map(|m| m.native_asset.path.clone())
                    .unwrap_or_default(),
                icon_path: item
                    .icon
                    .as_ref()
                    .and_then(|i| i.candidates.first())
                    .map(|i| i.path.clone()),
                npc_visual: None,
                hnpc_visual: None,
                animations: Vec::new(),
                network_id: Some(i64::from(item.item_number)),
                table_index: None,
                scale: None,
                height_server_units: None,
                level: Some(i64::from(item.level)),
                style: None,
                team_or_set: None,
                texture_main: visual.primary_texture.as_ref().map(|t| t.true_name.clone()),
                texture_sub: visual
                    .secondary_texture
                    .as_ref()
                    .map(|t| t.true_name.clone()),
                native_extension: false,
            });
            entries.insert(index, item);
        }
        Ok(Self {
            outfits: [base_outfit(&male), base_outfit(&female)],
            bases: [male, female],
            data,
            entries,
            applied: None,
            error: None,
        })
    }

    pub fn category_label(
        &self,
        index: usize,
        localization: &Localization,
        language: &Language,
    ) -> String {
        self.entries
            .get(&index)
            .map(|item| localization.text(language, &category_text(item.category)))
            .unwrap_or_default()
    }

    pub fn status_text(&self, preview: &NativePlayerPreviewModel) -> LocalizedText {
        if let Some(error) = &self.error {
            return error.clone();
        }
        match &preview.status {
            NativePlayerPreviewStatus::ReadyAnimated { .. } => {
                LocalizedText::new("ui.editor.equipment.ready", "Outfit ready")
            }
            NativePlayerPreviewStatus::Blocked(error) => {
                LocalizedText::new("ui.editor.status.error", "Asset error: {error}")
                    .with_arg("error", error.clone())
            }
            _ => LocalizedText::new("ui.editor.status.loading", "Loading native asset"),
        }
    }

    fn resolve(
        &self,
        female: bool,
        outfit: [ItemBase0104; 9],
    ) -> Result<ffone_client::player_preview::NativePlayerLook, String> {
        self.data
            .resolve_pc_appearance(&PcAppearance0104 {
                id: if female { 2 } else { 1 },
                style: self.bases[usize::from(female)].style.clone(),
                condition_bit_flag: 0,
                pc_state: 1,
                special_state: 0,
                level: 1,
                hp: 1000,
                map_number: 0,
                position: [0; 3],
                angle: 0,
                equipment: outfit,
                nano: Nano0104 {
                    id: 0,
                    skill_id: 0,
                    stamina: 0,
                },
                render_type: 0,
            })
            .map_err(|e| e.to_string())
    }
}

#[derive(Component)]
pub(super) struct DetailsSection(pub bool);
#[derive(Component)]
pub(super) struct CharacterSection;
#[derive(Component)]
pub(super) struct EquipmentSection;

#[derive(Component)]
pub(super) struct OutfitSummary;

#[derive(Component)]
pub(super) struct InspectorScroll;

pub(super) fn scroll_inspector(
    mut wheels: MessageReader<MouseWheel>,
    mut panels: Query<
        (
            &mut Node,
            &ComputedNode,
            &RelativeCursorPosition,
            &mut ScrollPosition,
        ),
        With<InspectorScroll>,
    >,
) {
    for (mut node, computed, cursor, mut position) in &mut panels {
        let overflow = Overflow::scroll_y();
        if node.overflow != overflow {
            node.overflow = overflow;
        }
        if !cursor.cursor_over() {
            continue;
        }
        for wheel in wheels.read() {
            let amount = match wheel.unit {
                MouseScrollUnit::Line => wheel.y * 32.0,
                MouseScrollUnit::Pixel => wheel.y,
            };
            let max = (computed.content_size().y - computed.size().y).max(0.0)
                * computed.inverse_scale_factor();
            position.y = (position.y - amount).clamp(0.0, max);
        }
    }
    wheels.clear();
}

pub(super) fn spawn_equipment_playback(parent: &mut ChildSpawnerCommands, fonts: &EditorFonts) {
    parent
        .spawn((
            EquipmentSection,
            Node {
                border_radius: BorderRadius::all(px(6)),
                height: px(EDITOR_PLAYBACK_HEIGHT),
                min_height: px(EDITOR_PLAYBACK_HEIGHT),
                width: percent(100),
                padding: UiRect::all(px(12)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                row_gap: px(12),
                ..default()
            },
            BackgroundColor(Color::srgba(0.012, 0.036, 0.049, 0.93)),
        ))
        .with_children(|panel| {
            panel
                .spawn(Node {
                    height: px(30),
                    column_gap: px(8),
                    ..default()
                })
                .with_children(|row| {
                    spawn_action_button(
                        row,
                        fonts,
                        EditorAction::ResetCamera,
                        "ui.editor.camera.reset",
                        "Fit model [R]",
                        150.0,
                    );
                    spawn_action_button_with_role(
                        row,
                        fonts,
                        EditorAction::ToggleTurntable,
                        "ui.editor.turntable.off",
                        "Rotate: off",
                        120.0,
                        DynamicTextRole::Turntable,
                    );
                });
            panel.spawn(editor_text(
                fonts,
                "ui.editor.equipment.camera_help",
                "↑/↓: select · Drag to orbit · Wheel to zoom · R to fit the whole character",
                12.0,
                Color::srgb(0.65, 0.8, 0.85),
                false,
            ));
        });
}

pub(super) fn spawn_equipment_controls(parent: &mut ChildSpawnerCommands, fonts: &EditorFonts) {
    parent.spawn((EquipmentSection, Node { display: Display::None, flex_direction: FlexDirection::Column, row_gap: px(8), flex_shrink: 0.0, ..default() }))
        .with_children(|panel| {
            panel.spawn(editor_text(fonts, "ui.editor.equipment.fit", "Try on a character", 16.0, Color::WHITE, true));
            panel.spawn(Node { height: px(30), column_gap: px(6), ..default() }).with_children(|row| {
                spawn_action_button(row, fonts, EditorAction::EquipmentGender(false), "ui.editor.equipment.male", "Male", 140.0);
                spawn_action_button(row, fonts, EditorAction::EquipmentGender(true), "ui.editor.equipment.female", "Female", 140.0);
            });
            panel.spawn(Node { flex_wrap: FlexWrap::Wrap, column_gap: px(5), row_gap: px(5), ..default() }).with_children(|row| {
                spawn_action_button(row, fonts, EditorAction::EquipmentCategory(None), "ui.editor.equipment.all", "All", 92.0);
                for (category, key, label) in [
                    (AvatarItemCategory::Shirt, "ui.editor.equipment.shirt", "Tops"),
                    (AvatarItemCategory::Pants, "ui.editor.equipment.pants", "Pants"),
                    (AvatarItemCategory::Shoes, "ui.editor.equipment.shoes", "Shoes"),
                    (AvatarItemCategory::Hat, "ui.editor.equipment.hat", "Hats"),
                    (AvatarItemCategory::Glasses, "ui.editor.equipment.glasses", "Glasses"),
                    (AvatarItemCategory::Back, "ui.editor.equipment.back", "Back"),
                    (AvatarItemCategory::Weapon, "ui.editor.equipment.weapon", "Weapons"),
                ] { spawn_action_button(row, fonts, EditorAction::EquipmentCategory(Some(category)), key, label, 92.0); }
            });
            spawn_action_button(panel, fonts, EditorAction::ResetOutfit, "ui.editor.equipment.reset", "Reset outfit", 220.0);
            panel.spawn(editor_text(fonts, "ui.editor.equipment.help", "Select an item to wear it. Each category replaces its slot; male and female outfits are kept separately.", 13.0, Color::srgb(0.65, 0.8, 0.85), false));
            panel.spawn((editor_text(fonts, "ui.editor.equipment.outfit", "Current outfit\n{items}", 12.0, Color::srgb(0.65, 0.8, 0.85), false), OutfitSummary));
        });
}

pub(super) fn handle_equipment_buttons(
    interactions: Query<(&Interaction, &EditorAction), Changed<Interaction>>,
    mut state: ResMut<EditorState>,
    catalog: Res<EditorCatalog>,
    mut library: ResMut<EquipmentLibrary>,
    mut model: ResMut<NativePlayerPreviewModel>,
    icons: Res<icon_generator::IconGenerator>,
) {
    if icons.active { return; }
    for (interaction, action) in &interactions {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match *action {
            EditorAction::CatalogSlot(_) if state.kind == CatalogKind::Equipment => {
                library.applied = None
            }
            EditorAction::EquipmentGender(female) => state.equipment_female = female,
            EditorAction::EquipmentCategory(category) => {
                state.equipment_category = category;
                if let Some(index) = state.filtered(&catalog).first().copied() {
                    state.select(&catalog, index);
                }
            }
            EditorAction::ResetOutfit => {
                let gender = usize::from(state.equipment_female);
                library.outfits[gender] = base_outfit(&library.bases[gender]);
                // Keep selection but do not immediately put it back on after reset.
                library.applied = Some((state.selected, state.equipment_female, false));
                library.error = None;
                if let Ok(look) = library.resolve(state.equipment_female, library.outfits[gender]) {
                    let _ = model.set_look(look);
                }
            }
            _ => {}
        }
    }
}

pub(super) fn sync_equipment_look(
    state: Res<EditorState>,
    mut library: ResMut<EquipmentLibrary>,
    mut model: ResMut<NativePlayerPreviewModel>,
    icons: Res<icon_generator::IconGenerator>,
    mut orbit: ResMut<OrbitCamera>,
) {
    let active = state.kind == CatalogKind::Equipment;
    if model.visible != active {
        model.visible = active;
    }
    if !active {
        if library.applied.take().is_some() {
            model.clear_look();
        }
        return;
    }
    let identity = (state.selected, state.equipment_female, icons.active);
    if library.applied == Some(identity) {
        return;
    }
    library.applied = Some(identity);
    library.error = None;
    let Some(item) = library.entries.get(&state.selected) else {
        model.clear_look();
        return;
    };
    let required = if state.equipment_female { 2 } else { 1 };
    if item.required_gender != 0 && item.required_gender != required {
        model.clear_look();
        library.error = Some(LocalizedText::new(
            "ui.editor.equipment.incompatible",
            "This item is unavailable for the selected gender.",
        ));
        return;
    }
    let gender = usize::from(state.equipment_female);
    let mut outfit = library.outfits[gender];
    let Ok(id) = i16::try_from(item.item_number) else {
        model.clear_look();
        return;
    };
    outfit[slot(item.category)].item_id = id;
    match library
        .resolve(state.equipment_female, outfit)
        .and_then(|mut look| {
            if icons.active {
                isolate_item(&mut look, item.category)?;
                orbit.fit_requested = true;
                model.yaw_degrees = 0.0;
            }
            Ok(look)
        })
        .and_then(|look| model.set_look(look))
    {
        Ok(()) if !icons.active => library.outfits[gender] = outfit,
        Ok(()) => {},
        Err(error) => {
            model.clear_look();
            library.error = Some(
                LocalizedText::new("ui.editor.status.error", "Asset error: {error}")
                    .with_arg("error", error),
            );
        }
    }
}

fn isolate_item(look: &mut ffone_client::player_preview::NativePlayerLook, category: AvatarItemCategory) -> Result<(), String> {
    use ffone_client::player_preview::NativePlayerPartKind as Part;
    let kind = match category {
        AvatarItemCategory::Shirt => Part::Shirt, AvatarItemCategory::Pants => Part::Pants,
        AvatarItemCategory::Shoes => Part::Shoes, AvatarItemCategory::Hat => Part::Hat,
        AvatarItemCategory::Glasses => Part::Glasses, AvatarItemCategory::Back => Part::Back,
        AvatarItemCategory::Weapon => Part::Weapon, _ => return Err("Unsupported icon category".into()),
    };
    look.parts.retain(|part| part.kind == kind);
    look.identity.push_str("/icon-only");
    look.weapon_animation_profile = None;
    look.validate()
}

pub(super) fn sync_equipment_camera(
    mut commands: Commands,
    state: Res<EditorState>,
    source: Single<(&Camera, &Transform), With<PreviewCamera>>,
    mut cameras: Query<
        (Entity, &mut Camera, &mut Transform, Option<&Msaa>),
        (With<NativePlayerPreviewCamera>, Without<PreviewCamera>),
    >,
    time: Res<Time>,
    mut model: ResMut<NativePlayerPreviewModel>,
) {
    for (entity, mut camera, mut transform, msaa) in &mut cameras {
        camera.order = 1;
        camera.viewport = source.0.viewport.clone();
        *transform = *source.1;
        if msaa != Some(&Msaa::Off) {
            commands.entity(entity).insert(Msaa::Off);
        }
    }
    if state.kind == CatalogKind::Equipment && state.turntable {
        model.yaw_degrees = (model.yaw_degrees + time.delta_secs() * 16.0).rem_euclid(360.0);
    }
}

pub(super) fn bind_section_visibility(
    state: Res<EditorState>,
    mut sections: Query<
        (
            &mut Node,
            Option<&EquipmentSection>,
            Option<&CharacterSection>,
            Option<&DetailsSection>,
        ),
        Or<(
            With<EquipmentSection>,
            With<CharacterSection>,
            With<DetailsSection>,
        )>,
    >,
) {
    for (mut node, equipment, character, details) in &mut sections {
        let visible = if equipment.is_some() {
            state.kind == CatalogKind::Equipment
        } else if character.is_some() {
            state.kind != CatalogKind::Equipment && (state.kind!=CatalogKind::Npc || state.npc_inspector==NpcInspectorTab::Animations)
        } else {
            details.is_some_and(|details| {
                !details.0 || (state.kind != CatalogKind::Equipment && if state.kind==CatalogKind::Npc {state.npc_inspector==NpcInspectorTab::Details}else{state.details_open})
            })
        };
        let display = if visible {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
}

pub(super) fn bind_outfit_summary(
    state: Res<EditorState>,
    library: Res<EquipmentLibrary>,
    localization: Res<Localization>,
    language: Res<Language>,
    mut summaries: Query<&mut LocalizedText, With<OutfitSummary>>,
) {
    if state.kind != CatalogKind::Equipment
        || !(state.is_changed() || library.is_changed() || language.is_changed())
    {
        return;
    }
    let outfit = &library.outfits[usize::from(state.equipment_female)];
    let items = CATEGORIES
        .iter()
        .filter_map(|&category| {
            let id = outfit[slot(category)].item_id;
            if id == 0 {
                return None;
            }
            library
                .entries
                .values()
                .find(|item| item.category == category && item.item_number == id as u32)
                .map(|item| {
                    format!(
                        "{}: {}",
                        localization.text(&language, &category_text(category)),
                        item.name
                    )
                })
        })
        .collect::<Vec<_>>()
        .join("\n");
    for mut text in &mut summaries {
        let next = LocalizedText::new("ui.editor.equipment.outfit", "Current outfit\n{items}")
            .with_arg("items", items.clone());
        if *text != next {
            *text = next;
        }
    }
}

/// Fit a bounding sphere with a margin using the tighter viewport FOV.
fn fit_distance(extent: Vec3, aspect: f32) -> f32 {
    let half_vertical = 22.5_f32.to_radians();
    let half_horizontal = (half_vertical.tan() * aspect.max(0.05)).atan();
    (extent.length() * 0.5 * 1.15 / half_vertical.min(half_horizontal).sin()).clamp(0.9, 120.0)
}

pub(super) fn fit_loaded_model(
    state: Res<EditorState>,
    status: Res<EditorRuntimeStatus>,
    player: Res<NativePlayerPreviewModel>,
    mut orbit: ResMut<OrbitCamera>,
    preview: Res<ModelPreview>,
    parents: Query<&ChildOf>,
    mesh_assets: Res<Assets<Mesh>>,
    bindposes: Res<Assets<SkinnedMeshInverseBindposes>>,
    joints: Query<&GlobalTransform>,
    meshes: Query<(
        Entity,
        &Mesh3d,
        Option<&SkinnedMesh>,
        &GlobalTransform,
        Option<&bevy::camera::visibility::RenderLayers>,
        &InheritedVisibility,
    )>,
    window: Single<&Window>,
) {
    if !orbit.fit_requested {
        return;
    }
    let equipment = state.kind == CatalogKind::Equipment;
    if !(if equipment {
        matches!(
            player.status,
            NativePlayerPreviewStatus::ReadyAnimated { .. }
        )
    } else {
        status.ready
    }) {
        return;
    }
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for (entity, mesh_handle, skin, global, layers, visible) in &meshes {
        if !visible.get() {
            continue;
        }
        let belongs = if equipment {
            layers.is_some_and(|layers| {
                layers.intersects(&bevy::camera::visibility::RenderLayers::layer(
                    ffone_client::player_preview::NATIVE_PLAYER_PREVIEW_RENDER_LAYER,
                ))
            })
        } else {
            belongs_to_root(entity, preview.root, &parents)
        };
        if !belongs {
            continue;
        }
        let Some(mesh) = mesh_assets.get(&mesh_handle.0) else {
            return;
        };
        let matrices = if let Some(skin) = skin {
            let Some(bindposes) = bindposes.get(&skin.inverse_bindposes) else {
                return;
            };
            if skin.joints.len() != bindposes.len() {
                return;
            }
            let matrices: Option<Vec<_>> = skin
                .joints
                .iter()
                .zip(bindposes.iter())
                .map(|(&joint, bind)| {
                    joints
                        .get(joint)
                        .ok()
                        .map(|global| global.to_matrix() * *bind)
                })
                .collect();
            let Some(matrices) = matrices else {
                return;
            };
            Some(matrices)
        } else {
            None
        };
        // Wait for every visible part. Partial bounds must not lock in a bad frame.
        let Some((mesh_min, mesh_max)) = posed_mesh_bounds(mesh, global, matrices.as_deref())
        else {
            return;
        };
        min = min.min(mesh_min);
        max = max.max(mesh_max);
    }
    if !min.is_finite() {
        return;
    }
    let Some((_, size)) = preview_viewport_logical_rect(Vec2::new(window.width(), window.height()))
    else {
        return;
    };
    let center = (min + max) * 0.5;
    orbit.target_y = center.y;
    orbit.target_xz = Vec2::new(center.x, center.z);
    orbit.distance = fit_distance(max - min, size.x / size.y);
    orbit.fit_requested = false;
}

/// Mirrors Bevy skinning: jointGlobal * inverseBind supplies world positions.
/// The skinned mesh node transform must not be applied a second time.
fn posed_mesh_bounds(
    mesh: &Mesh,
    global: &GlobalTransform,
    matrices: Option<&[Mat4]>,
) -> Option<(Vec3, Vec3)> {
    let VertexAttributeValues::Float32x3(positions) = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?
    else {
        return None;
    };
    let skin_attributes = if matrices.is_some() {
        let VertexAttributeValues::Uint16x4(indices) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_INDEX)?
        else {
            return None;
        };
        let VertexAttributeValues::Float32x4(weights) =
            mesh.attribute(Mesh::ATTRIBUTE_JOINT_WEIGHT)?
        else {
            return None;
        };
        if indices.len() != positions.len() || weights.len() != positions.len() {
            return None;
        }
        Some((indices, weights))
    } else {
        None
    };
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    let vertices: Box<dyn Iterator<Item = usize> + '_> = match mesh.indices() {
        Some(indices) => Box::new(indices.iter()),
        None => Box::new(0..positions.len()),
    };
    for index in vertices {
        let position = Vec3::from(*positions.get(index)?);
        let point = if let (Some(matrices), Some((indices, weights))) = (matrices, skin_attributes)
        {
            let mut world = Vec4::ZERO;
            for (&joint, &weight) in indices[index].iter().zip(weights[index].iter()) {
                if !weight.is_finite() || weight < 0.0 {
                    return None;
                }
                if weight > 0.0 {
                    world += (*matrices.get(usize::from(joint))? * position.extend(1.0)) * weight;
                }
            }
            if world.w <= 0.0 || !world.is_finite() {
                return None;
            }
            world.truncate()
        } else {
            global.transform_point(position)
        };
        if !point.is_finite() {
            return None;
        }
        min = min.min(point);
        max = max.max(point);
    }
    min.is_finite().then_some((min, max))
}

pub(super) fn belongs_to_root(
    mut entity: Entity,
    root: Option<Entity>,
    parents: &Query<&ChildOf>,
) -> bool {
    loop {
        if Some(entity) == root {
            return true;
        }
        let Ok(parent) = parents.get(entity) else {
            return false;
        };
        entity = parent.parent();
    }
}

#[cfg(test)]
#[path = "equipment/tests.rs"]
mod tests;
