//! Shared placement document and native 2D/3D views of the same world.
use super::*;
use std::{path::Path, time::Instant};

#[path = "world_editor/atlas.rs"]
mod atlas;
#[path = "world_editor/ground.rs"]
mod ground;
#[path = "world_editor/input.rs"]
mod input;
#[path = "world_editor/instances.rs"]
mod instances;
#[path = "world_editor/map.rs"]
mod map;
#[path = "world_editor/model_templates.rs"]
mod model_templates;
#[path = "world_editor/markers.rs"]
mod markers;
#[path = "world_editor/model.rs"]
mod model;
#[path = "world_editor/objects.rs"]
mod objects;
#[path = "world_editor/operations.rs"]
mod operations;
#[path = "world_editor/picking.rs"]
mod picking;
#[path = "world_editor/manipulation.rs"]
mod manipulation;
#[path = "world_editor/npc_details.rs"]
mod npc_details;
#[path = "world_editor/collision.rs"]
mod collision;
#[path = "world_editor/preview.rs"]
mod preview;
#[path = "world_editor/stream.rs"]
mod stream;
#[path = "world_editor/session.rs"]
mod session;
#[path = "world_editor/terrain.rs"]
mod terrain;
#[path = "world_editor/terrain_create.rs"]
mod terrain_create;
#[path = "world_editor/squares.rs"]
mod squares;
#[path = "world_editor/grass.rs"]
mod grass;
#[path = "world_editor/type_picker.rs"]
mod type_picker;
#[path = "world_editor/type_preview.rs"]
mod type_preview;
#[path = "world_editor/routes.rs"]
mod routes;
#[cfg(test)]
#[path = "world_editor/tests.rs"]
mod tests;
#[path = "world_editor/view.rs"]
mod view;

pub(super) struct WorldEditorPlugin {
    root: PathBuf,
}
impl WorldEditorPlugin {
    pub(super) fn new(root: PathBuf) -> Self {
        Self { root }
    }
}
impl Plugin for WorldEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ffone_client::native_terrain::NativeTerrainPlugin)
            .insert_resource(WorldEditor::open(self.root.clone()))
            .init_resource::<preview::WorldPreview>()
            .init_resource::<stream::TileStreaming>()
            .init_resource::<objects::Search>()
            .init_resource::<type_preview::Preview>()
            .add_systems(
                Update,
                (
                    session::update,
                    input::buttons,
                    input::keyboard,
                    input::pointer,
                    stream::update,
                    objects::update,
                    type_preview::update,
                    preview::scene,
                    terrain::preview,
                    terrain::cursor,
                    routes::gizmos,
                    view::coordinate_marker,
                    manipulation::gizmos,
                    view::draw,
                )
                    .chain()
                    .after(handle_editor_buttons)
                    .before(LocalizationSet::Apply),
            )
            .add_systems(
                PostUpdate,
                preview::camera.after(TransformSystems::Propagate),
            )
            .add_systems(
                PostUpdate,
                preview::retire.before(TransformSystems::Propagate),
            );
    }
}

#[derive(Resource)]
pub(super) struct WorldEditor {
    root: PathBuf,
    open_mode: Option<bool>,
    session_request: Option<session::Request>,
    folder: Option<PathBuf>,
    sources: Vec<model::Source>,
    entities: Vec<model::Placement>,
    selected: Option<usize>,
    selected_point: Option<Vec3>,
    coordinate_clipboard: Option<Vec3>,
    last_entity_click: Option<(String, Instant)>,
    ignore_pointer_press: bool,
    show_collisions: bool,
    instance: u32,
    search: String,
    page: usize,
    center: Vec3,
    zoom: f32,
    yaw: f32,
    pitch: f32,
    distance: f32,
    snap: f32,
    placing: bool,
    placement_kind: usize,
    type_id: i64,
    type_picker: Option<type_picker::Picker>,
    type_image: Option<Handle<Image>>,
    object_template: Option<operations::Clipboard>,
    group_template: Option<Value>,
    routes: Option<routes::Editor>,
    focus: Option<Field>,
    edit: String,
    select_text: bool,
    drag: Option<input::Drag>,
    last_cursor: Option<Vec2>,
    undo: Vec<model::Edit>,
    redo: Vec<model::Edit>,
    changed_at: Option<Instant>,
    revision: u64,
    geometry_revision: u64,
    status: Option<LocalizedText>,
    maps: BTreeMap<String, map::Tile>,
    region: Option<[i32; 2]>,
    available_tiles: BTreeSet<[i32; 2]>,
    npc_map_icons: BTreeMap<i64, i32>,
    object_npc_types: BTreeSet<i64>,
    actor_enabled: [bool; 4],
    list_mode: u8,
    list_search: [String; 3],
    edit_objects: bool,
    clipboard: Option<operations::Clipboard>,
    instances: BTreeMap<u32, String>,
    instance_menu: Option<bool>,
    ground: BTreeMap<String, ground::Tile>,
    terrain_tool: Option<u8>,
    creating_terrain: bool,
    square_menu: bool,
    square_choices: Option<(u8, usize)>,
    music_choices: Vec<(String, String)>,
    square_names: BTreeMap<String, String>,
    grass_template: Option<operations::Clipboard>,
    grass_stroke: Option<usize>,
    grass_last: Option<Vec3>,
    brush_radius: f32,
    brush_strength: f32,
    brush_layer: usize,
    brush: Option<terrain::Stroke>,
    brush_cursor: Option<Vec3>,
    object_index: Vec<objects::Entry>,
    pending_object: Option<(String, String)>,
    map_picker: bool,
    atlas_center: Vec3,
    atlas_zoom: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Field {
    SquareName,
    SquareMusic,
    Search,
    Type,
    X,
    Y,
    Z,
    Angle,
    AngleX,
    AngleZ,
    InstanceName,
    EntityInstance,
    Snap,
    BrushRadius,
    BrushStrength,
    PickerSearch,
    RouteSpeed,
    RouteTarget,
    RouteStop,
    RouteX,
    RouteY,
    RouteZ,
}
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
enum Action {
    SquareSettings,
    SquareSkybox(u8),
    SquareShader(u8),
    SquareChoices(u8),
    SquareChoice(usize),
    SquarePage(bool),
    CreateTerrain,
    ShowCollisions,
    ToggleCollision,
    Select(usize),
    Field(Field),
    Apply,
    Cancel,
    Kind(usize),
    Place,
    Duplicate,
    Undo,
    Redo,
    SaveWork,
    RestoreWork,
    Publish,
    Server,
    Center,
    Page(bool),
    Tile,
    Map,
    Overview,
    ToggleKind(usize),
    List(u8),
    EditObjects(bool),
    Delete,
    Copy,
    Paste,
    CopyCoordinates,
    PasteCoordinates,
    Instances(bool),
    ChooseInstance(u32),
    NewInstance,
    Terrain(u8),
    TerrainLayer(usize),
    RemoteObject(usize),
    ChooseType(bool),
    PickerSelect(usize),
    PickerConfirm,
    PickerCancel,
    PickerPage(bool),
    Routes,
    RouteTab(bool),
    RouteKind(u8),
    RoutePage(bool),
    RouteNew,
    RouteSelect(usize),
    RouteSave,
    RoutePoint(usize),
    RouteRemovePoint,
    RouteLoop,
    RouteAssign,
}
#[derive(Component)]
struct Root;
#[derive(Component)]
struct Canvas;
#[derive(Component)]
struct ListViewport;
#[derive(Component)]
struct Marker;
#[derive(Component)]
struct InspectorScroll;

fn tr(l: &Localization, lang: &Language, key: &str, en: &str) -> String {
    l.text(
        lang,
        &LocalizedText::new(format!("ui.editor.world.{key}"), en),
    )
}

impl WorldEditor {
    fn error(&mut self, error: impl Into<String>) {
        self.status = Some(
            LocalizedText::new("ui.editor.world.error", "Error: {error}")
                .with_arg("error", error.into()),
        );
        self.revision += 1;
    }
    fn touch(&mut self) {
        self.changed_at = Some(Instant::now());
        self.revision += 1;
        self.geometry_revision += 1;
        self.status = None;
    }
    fn filtered(&self, catalog: &EditorCatalog) -> Vec<usize> {
        let needle = self.search.trim().to_lowercase();
        let mut result: Vec<_> = self
            .entities
            .iter()
            .enumerate()
            .filter(|(_, p)| p.instance == self.instance)
            .filter(|(_, p)| {
                if p.kind == 4 {
                    self.list_mode != 0
                } else {
                    self.list_mode != 1 && self.actor_enabled[p.kind]
                }
            })
            .filter(|(_, p)| {
                needle.is_empty()
                    || self.name(p, catalog).to_lowercase().contains(&needle)
                    || p.key.contains(&needle)
                    || p.type_id.to_string().contains(&needle)
            })
            .map(|(i, _)| i)
            .collect();
        if self.list_mode == 2 {
            result.retain(|i| self.entities[*i].position.distance(self.center) <= 100.);
            result.sort_by(|a, b| {
                self.entities[*a]
                    .position
                    .distance_squared(self.center)
                    .total_cmp(&self.entities[*b].position.distance_squared(self.center))
            });
        }
        result
    }
    fn displayed(&self, catalog: &EditorCatalog) -> Vec<usize> {
        let objects = if self.list_mode == 1 {
            &self.search
        } else {
            &self.list_search[1]
        };
        let needle = objects.trim().to_lowercase();
        self.entities
            .iter()
            .enumerate()
            .filter(|(_, p)| p.instance == self.instance)
            .filter(|(i, p)| {
                if p.kind == 4 {
                    self.selected == Some(*i)
                        || (!needle.is_empty()
                            && self.name(p, catalog).to_lowercase().contains(&needle))
                        || (needle.is_empty()
                            && (self.list_mode == 2 || self.edit_objects)
                            && p.position.distance(self.center) <= 100.)
                } else {
                    self.actor_enabled[p.kind]
                }
            })
            .map(|(i, _)| i)
            .collect()
    }
    fn pickable(&self) -> Vec<usize> {
        self.entities
            .iter()
            .enumerate()
            .filter(|(_, p)| p.instance == self.instance)
            .filter(|(_, p)| {
                if self.edit_objects {
                    p.kind == 4
                } else {
                    p.kind < 4 && self.actor_enabled[p.kind]
                }
            })
            .map(|(i, _)| i)
            .collect()
    }
    fn select_entity(&mut self, index: usize) {
        if let Some(p) = self.entities.get(index) {
            self.edit_objects = p.kind == 4;
            self.selected = Some(index);
            self.selected_point = None;
            self.terrain_tool = None;
            self.placing = false;
            self.focus = None;
            self.revision += 1;
        }
    }
    fn select_coordinates(&mut self, point: Option<Vec3>) -> Result<(), String> {
        self.selected = None;
        self.selected_point = point;
        self.drag = None;
        self.focus = None;
        self.status = None;
        self.revision += 1;
        if let Some(point) = point {
            self.selected_point = Some(self.snap_to_ground(point)?);
        }
        Ok(())
    }
    fn name(&self, p: &model::Placement, catalog: &EditorCatalog) -> String {
        if let Some(name) = &p.object_name {
            return name.clone();
        }
        catalog
            .entries
            .iter()
            .find(|entry| entry.kind == CatalogKind::Npc && entry.network_id == Some(p.type_id))
            .map(|e| e.display_name.clone())
            .unwrap_or_else(|| p.type_id.to_string())
    }
    fn selected(&self) -> Option<&model::Placement> {
        self.selected.and_then(|i| self.entities.get(i))
    }
    fn center_selected(&mut self) {
        if let Some((position, instance)) = self.selected().map(|p| (p.position, p.instance)) {
            self.center = position;
            self.instance = instance;
        }
        self.revision += 1;
    }
}
