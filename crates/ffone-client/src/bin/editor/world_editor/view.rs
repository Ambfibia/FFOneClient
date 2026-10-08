use super::*;
pub(super) const WHITE: Color = Color::srgb(0.86, 0.94, 1.);
pub(super) const MUTED: Color = Color::srgb(0.55, 0.70, 0.84);

pub(super) fn label(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    text: LocalizedText,
    size: f32,
    color: Color,
) {
    let mut bundle = editor_text(f, "ui.editor.world.value", "{value}", size, color, false);
    bundle.localized = text;
    p.spawn(bundle);
}
pub(super) fn value(p: &mut ChildSpawnerCommands, f: &EditorFonts, text: impl Into<String>, size: f32) {
    label(
        p,
        f,
        LocalizedText::new("ui.editor.world.value", "{value}").with_arg("value", text.into()),
        size,
        WHITE,
    );
}
pub(super) fn row() -> Node {
    Node {
        width: percent(100),
        min_height: px(32),
        flex_shrink: 0.,
        align_items: AlignItems::Center,
        column_gap: px(6),
        ..default()
    }
}
fn stack() -> Node {
    Node {
        flex_direction: FlexDirection::Column,
        min_width: px(0),
        min_height: px(0),
        row_gap: px(6),
        ..default()
    }
}
pub(super) fn button(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    a: Action,
    key: &str,
    en: &str,
    width: f32,
    selected: bool,
) {
    let mut image = sliced_image(
        f.button.clone(),
        ffone_client::option_ui::OPTION_BIG_LABEL_BORDER,
    );
    image.color = if selected {
        Color::srgb(0.6, 0.85, 1.)
    } else {
        Color::srgb(0.3, 0.48, 0.68)
    };
    p.spawn((
        Button,
        a,
        image,
        Node {
            width: if width == 0. { percent(100) } else { px(width) },
            height: px(32),
            min_width: px(0),
            flex_shrink: 0.,
            padding: UiRect::horizontal(px(8)),
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..default()
        },
    ))
    .with_children(|p| {
        label(
            p,
            f,
            if key=="value" {LocalizedText::new("ui.editor.world.value","{value}").with_arg("value",en)} else {LocalizedText::new(format!("ui.editor.world.{key}"), en)},
            14.,
            WHITE,
        )
    });
}
pub(super) fn field(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &WorldEditor,
    field: Field,
    name: &str,
    current: String,
) {
    p.spawn(row()).with_children(|p| {
        p.spawn(Node {
            width: px(100),
            flex_shrink: 0.,
            ..default()
        })
        .with_children(|p| {
            label(
                p,
                f,
                LocalizedText::new(format!("ui.editor.world.field.{name}"), name),
                14.,
                MUTED,
            )
        });
        p.spawn((
            Button,
            Action::Field(field),
            Node {
                flex_grow: 1.,
                min_width: px(0),
                height: px(32),
                padding: UiRect::all(px(6)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(if e.focus == Some(field) {
                Color::srgb(0.06, 0.23, 0.38)
            } else {
                Color::srgb(0.025, 0.06, 0.12)
            }),
        ))
        .with_children(|p| {
            value(
                p,
                f,
                if e.focus == Some(field) {
                    format!("{}│", e.edit)
                } else {
                    current
                },
                15.,
            )
        });
    });
}
pub(super) fn rect(size: Vec2) -> (Vec2, Vec2) {
    let min = Vec2::new(310., EDITOR_HEADER_HEIGHT + 114.);
    (
        min,
        Vec2::new(
            (size.x - min.x - 318.).max(1.),
            (size.y - min.y - 48.).max(1.),
        ),
    )
}
pub(super) fn page_size(height: f32) -> usize {
    ((height - EDITOR_HEADER_HEIGHT - 344.) / 40.)
        .floor()
        .max(1.) as usize
}
pub(super) fn list_capacity(node:Option<&ComputedNode>,height:f32,three_d:bool,objects:bool)->usize {
    node.filter(|n|n.size().y>2.).map(|n|((n.size().y*n.inverse_scale_factor+6.)/40.).floor().max(1.) as usize)
        .unwrap_or_else(||page_size(height-if three_d{38.}else{0.}+if objects{38.}else{0.}))
}
pub(super) fn screen(e: &WorldEditor, position: Vec3, size: Vec2) -> Vec2 {
    size * 0.5 + Vec2::new(-(position.x - e.center.x), -(position.z - e.center.z)) * e.zoom
}
pub(super) fn draw(
    mut commands: Commands,
    fonts: Option<Res<EditorFonts>>,
    e: Res<WorldEditor>,
    state: Res<EditorState>,
    catalog: Res<EditorCatalog>,
    language: Res<Language>,
    localization: Res<Localization>,
    server: Res<AssetServer>,
    preview: Res<preview::WorldPreview>,
    search: Res<objects::Search>,
    window: Single<&Window>,
    roots: Query<Entity, With<Root>>,
    layout: Query<(&ComputedNode,Option<&Canvas>,Option<&ListViewport>),Or<(With<Canvas>,With<ListViewport>)>>,
    scrolls: Query<&ScrollPosition, With<InspectorScroll>>,
    mut shown: Local<(Option<bool>, u64, u32, u32, [u32; 3])>,
    mut images: Local<markers::Images>,
) {
    let (_, estimated) = rect(Vec2::new(window.width(), window.height()));
    let canvas_size = layout
        .iter()
        .find(|(n,canvas,_)|canvas.is_some() && n.size().min_element()>1.)
        .map(|(n,_,_)| n.size() * n.inverse_scale_factor)
        .unwrap_or(estimated);
    let page_size=list_capacity(layout.iter().find(|(_,_,list)|list.is_some()).map(|(n,_,_)|n),window.height(),state.world_open==Some(true),e.list_mode==1);
    let inspector_scroll = scrolls.iter().next().map(|s| s.0).unwrap_or_default();
    let next = (
        state.world_open,
        e.revision,
        window.width() as u32,
        window.height() as u32,
        [canvas_size.x as u32, canvas_size.y as u32,page_size as u32],
    );
    if *shown == next && !language.is_changed() {
        return;
    }
    let Some(f) = fonts else {
        return;
    };
    *shown = next;
    for entity in &roots {
        commands.entity(entity).despawn();
    }
    let Some(three_d) = state.world_open else {
        return;
    };
    let raster = !three_d || e.map_picker;
    let filtered = e.filtered(&catalog);
    let remote = e.remote_objects();
    let displayed = e.displayed(&catalog);
    commands.spawn((Root,Node {position_type:PositionType::Absolute,left:px(0),top:px(EDITOR_HEADER_HEIGHT),width:percent(100),bottom:px(0),padding:UiRect::all(px(12)),..stack()},GlobalZIndex(6),BackgroundColor(if three_d {Color::NONE}else{Color::srgb(0.015,0.035,0.065)})))
    .with_children(|root| {
        root.spawn(row()).with_children(|bar| {
            button(bar,&f,Action::SaveWork,"save_work","Save work",110.,false);
            button(bar,&f,Action::RestoreWork,"restore_work","Open backup",120.,false);
            button(bar,&f,Action::Publish,"publish","Rewrite",105.,false);
            button(bar,&f,Action::Undo,"undo","Undo",80.,false);
            button(bar,&f,Action::Redo,"redo","Redo",80.,false);
            button(bar,&f,Action::Center,"center","Focus selection",145.,false);
            button(bar,&f,if three_d {Action::Map}else{Action::Tile},if three_d {"map"}else{"tile"},if three_d {"Select area on map"}else{"Open nearby map tile"},170.,e.map_picker);
            if raster {button(bar,&f,Action::Overview,"overview","Entire map",130.,false);}
            value(bar,&f,format!("{} · Δ{}",e.entities.len(),e.sources.iter().filter(|s|s.base!=s.draft).count()),14.);
        });
        root.spawn(row()).with_children(|bar| {
            button(bar,&f,Action::Server,"server","Select server folder",210.,false);
            value(bar,&f,e.folder.as_ref().map(|p|p.display().to_string()).unwrap_or_else(||tr(&localization,&language,"no_server","Select RustyFusion TableData folder")),14.);
        });
        root.spawn(row()).with_children(|bar| {
            button(bar,&f,Action::EditObjects(false),"edit_actors","Edit NPCs",145.,!e.edit_objects&&e.terrain_tool.is_none());
            button(bar,&f,Action::EditObjects(true),"edit_objects","Edit objects",145.,e.edit_objects&&e.terrain_tool.is_none());
            button(bar,&f,Action::Terrain(0),"terrain","Terrain editor",145.,e.terrain_tool.is_some());
            button(bar,&f,Action::Routes,"routes","Path editor",145.,e.routes.is_some());
        });
        root.spawn(Node {flex_grow:1.,min_height:px(0),column_gap:px(10),..default()}).with_children(|body| {
            body.spawn((Node {width:px(288),flex_shrink:0.,padding:UiRect::all(px(8)),..stack()},BackgroundColor(Color::srgb(0.035,0.065,0.11))))
            .with_children(|p| {
                button(p,&f,Action::Instances(false),"instances","Select instance",0.,e.instance_menu==Some(false));
                value(p,&f,format!("{} · {}",e.instance,e.instances.get(&e.instance).map(String::as_str).unwrap_or("?")),14.);
                p.spawn(row()).with_children(|p| {
                    button(p,&f,Action::List(0),"actors","NPCs",70.,e.list_mode==0);
                    button(p,&f,Action::List(1),"objects","Objects",86.,e.list_mode==1);
                    button(p,&f,Action::List(2),"nearby","Nearby",86.,e.list_mode==2);
                });
                if e.list_mode!=1 {p.spawn(row()).with_children(|p|for (kind,key,name) in [(0,"npc","NPC"),(1,"mob","Mob"),(2,"group","Group"),(3,"shiny","Shiny")] {
                    button(p,&f,Action::ToggleKind(kind),key,name,62.,e.actor_enabled[kind]);
                });}
                field(p,&f,&e,Field::Search,"search",e.search.clone());
                if e.list_mode==1 && search.loading(){label(p,&f,LocalizedText::new("ui.editor.world.search_loading","Indexing world objects…"),13.,MUTED);}
                if e.instance_menu.is_some() {
                    let needle=e.search.to_lowercase();
                    let instances:Vec<_>=e.instances.iter().filter(|(id,name)|needle.is_empty()||id.to_string().contains(&needle)||name.to_lowercase().contains(&needle)).collect();
                    p.spawn(row()).with_children(|p|{
                        button(p,&f,Action::Page(false),"previous","←",45.,false);
                        button(p,&f,Action::Page(true),"next","→",45.,false);
                    });
                    for (id,name) in instances.iter().skip(e.page*8).take(8) {
                        p.spawn((Button,Action::ChooseInstance(**id),Node {height:px(29),width:percent(100),overflow:Overflow::clip(),..default()},BackgroundColor(Color::srgb(0.05,0.15,0.25))))
                            .with_children(|p|value(p,&f,format!("{id} · {name}"),13.));
                    }
                    button(p,&f,Action::NewInstance,"new_instance","Create new instance",0.,false);
                    if e.focus==Some(Field::InstanceName) {field(p,&f,&e,Field::InstanceName,"instance_name",e.edit.clone());}
                    return;
                }
                p.spawn(row()).with_children(|p| {
                    button(p,&f,Action::Page(false),"previous","←",45.,false);
                    button(p,&f,Action::Page(true),"next","→",45.,false);
                    value(p,&f,format!("{}/{}",e.page+1,(filtered.len()+remote.len()).div_ceil(page_size).max(1)),14.);
                });
                p.spawn((ListViewport,Node {flex_grow:1.,overflow:Overflow::clip(),..stack()})).with_children(|p| {
                    for &i in filtered.iter().skip(e.page*page_size).take(page_size) {
                        let entry=&e.entities[i];
                        p.spawn((Button,Action::Select(i),Node {width:percent(100),height:px(34),min_height:px(34),padding:UiRect::axes(px(6),px(3)),overflow:Overflow::clip(),..default()},BackgroundColor(if e.selected==Some(i){Color::srgb(0.08,0.25,0.41)}else{Color::srgb(0.03,0.08,0.14)})))
                        .with_children(|p|value(p,&f,format!("{} · {}",entry.key.rsplit('/').next().unwrap_or_default(),e.name(entry,&catalog)),14.));
                    }
                    let offset=(e.page*page_size).saturating_sub(filtered.len());
                    let count=page_size.saturating_sub(filtered.len().saturating_sub(e.page*page_size).min(page_size));
                    for &i in remote.iter().skip(offset).take(count) {
                        let entry=&e.object_index[i];
                        p.spawn((Button,Action::RemoteObject(i),Node{width:percent(100),height:px(34),overflow:Overflow::clip(),..default()},BackgroundColor(Color::srgb(0.03,0.08,0.14)))).with_children(|p|value(p,&f,format!("{} · {}",entry.tile,entry.name),14.));
                    }
                });
            });
            body.spawn(Node {flex_grow:1.,min_width:px(0),..stack()}).with_children(|p| {
                label(p,&f,LocalizedText::new(if e.map_picker {"ui.editor.world.help_map"}else if three_d {"ui.editor.world.help3d"}else{"ui.editor.world.help2d"},if e.map_picker {"Click an area: load this tile and 8 neighbours · wheel: zoom"}else if three_d {"LMB: select / move · RMB: orbit · MMB: pan · wheel: zoom"}else{"LMB: select / move · RMB/MMB: pan · wheel: zoom"}),14.,MUTED);
                let canvas=p.spawn((Canvas,RelativeCursorPosition::default(),Node {flex_grow:1.,min_height:px(0),overflow:Overflow::clip(),..default()},BackgroundColor(if !raster {Color::NONE}else{Color::srgb(0.015,0.045,0.075)}))).id();
                if raster {
                    p.commands().entity(canvas).with_children(|p| {
                        atlas::images(p,&e,canvas_size,&server,&mut images.atlas);
                        atlas::grid(p,&e,&f,canvas_size);
                        if e.map_picker {
                            return;
                        }
                        let spacing=1000.*0.01*e.zoom;
                        if spacing>=12. {
                            for axis in 0..2 {
                                let offset=if axis==0 {-e.center.x*e.zoom}else{e.center.z*e.zoom};
                                let start=(canvas_size[axis]*0.5+offset).rem_euclid(spacing);
                                for n in 0..=(canvas_size[axis]/spacing) as usize {
                                    let at=start+n as f32*spacing;
                                    p.spawn((Node {position_type:PositionType::Absolute,left:px(if axis==0{at}else{0.}),top:px(if axis==1{at}else{0.}),width:if axis==0{px(1)}else{percent(100)},height:if axis==1{px(1)}else{percent(100)},..default()},BackgroundColor(Color::srgba(0.05,0.12,0.18,0.3))));
                                }
                            }
                        }
                        for &i in &remote {
                            let at=screen(&e,e.object_index[i].position,canvas_size);
                            if at.x<0. || at.y<0. || at.x>canvas_size.x || at.y>canvas_size.y {continue;}
                            p.spawn((Node{position_type:PositionType::Absolute,left:px(at.x-5.),top:px(at.y-5.),width:px(10),height:px(10),..default()},BackgroundColor(Color::srgb(0.9,0.7,0.2))));
                        }
                        routes::draw_2d(p,&f,&e,canvas_size);
                        if let Some(point)=e.selected_point.filter(|_|e.selected.is_none()&&e.terrain_tool.is_none()) {
                            let at=screen(&e,point,canvas_size);
                            for (offset,dimensions) in [(Vec2::new(-8.,-1.),Vec2::new(16.,2.)),(Vec2::new(-1.,-8.),Vec2::new(2.,16.))] {
                                p.spawn((Node{position_type:PositionType::Absolute,left:px(at.x+offset.x),top:px(at.y+offset.y),width:px(dimensions.x),height:px(dimensions.y),..default()},BackgroundColor(Color::srgb(0.2,0.85,1.))));
                            }
                        }
                        for &i in &displayed {
                            let entry=&e.entities[i];let at=screen(&e,entry.position,canvas_size);
                            if at.x<0. || at.y<0. || at.x>canvas_size.x || at.y>canvas_size.y {continue;}
                            let selected=e.selected==Some(i);
                            if entry.kind==4 && e.zoom<0.2 && !selected {continue;}
                            markers::draw(p,entry,at,selected,&e,&server,&mut images);
                            if selected { p.spawn(Node {position_type:PositionType::Absolute,left:px(at.x+10.),top:px(at.y+8.),..default()}).with_children(|p|value(p,&f,e.name(entry,&catalog),14.)); }
                        }
                    });
                }
            });
            body.spawn((InspectorScroll,ScrollPosition(inspector_scroll),RelativeCursorPosition::default(),Node {width:px(296),flex_shrink:0.,padding:UiRect::all(px(10)),overflow:Overflow::scroll_y(),..stack()},BackgroundColor(Color::srgb(0.035,0.065,0.11))))
            .with_children(|p| {
                if e.routes.is_some(){routes::draw(p,&f,&e);return;}
                button(p,&f,Action::SquareSettings,"square_settings","Square settings",0.,e.square_menu);
                if e.square_menu{squares::draw(p,&f,&e);return;}
                label(p,&f,if e.selected.is_none()&&e.selected_point.is_some(){LocalizedText::new("ui.editor.world.selected_coordinates","Selected coordinates")}else{LocalizedText::new("ui.editor.world.entity","Entity placement")},18.,WHITE);
                if three_d {button(p,&f,Action::ShowCollisions,"show_collisions","Show collisions",0.,e.show_collisions);}
                if let Some(region)=e.region {
                    label(p,&f,LocalizedText::new("ui.editor.world.loaded","{tile} · tiles {loaded}/{total}").with_arg("tile",atlas::tile_id(region)).with_arg("loaded",if three_d {preview.loaded_count()}else{e.maps.len()}.to_string()).with_arg("total",e.desired_tiles().len().to_string()),14.,WHITE);
                }
                if let Some(entry)=e.selected() {
                    value(p,&f,e.name(entry,&catalog),16.);value(p,&f,&entry.key,12.);
                    if entry.kind!=4 {label(p,&f,LocalizedText::new("ui.editor.world.type_id","NPC type ID: {id}").with_arg("id",entry.type_id.to_string()),14.,WHITE);}
                    if entry.kind==4{value(p,&f,e.sources[entry.source].draft.pointer(&entry.pointer).and_then(|r|r["object"].as_str()).unwrap_or_default(),12.);}
                    button(p,&f,Action::ChooseType(true),"choose","Choose",0.,false);
                    for (field,name,current) in [(Field::X,"X",-entry.position.x*100.),(Field::Y,"Y",entry.position.z*100.),(Field::Z,"Z",entry.position.y*100.),(Field::Angle,"angle",entry.angle),(Field::EntityInstance,"instance",entry.instance as f32)] {
                        if entry.kind==4 && matches!(field,Field::Angle|Field::EntityInstance) {continue;}
                        if field==Field::EntityInstance {button(p,&f,Action::Instances(true),"entity_instance","Entity instance",0.,false);continue;}
                        field_row(p,&f,&e,field,name,format!("{current:.2}"));
                    }
                    if entry.kind==4 {let rotation=e.object_rotation(entry);for (axis,name,number) in [(Field::AngleX,"angle_x",rotation.x),(Field::Angle,"angle_y",rotation.y),(Field::AngleZ,"angle_z",rotation.z)] {field_row(p,&f,&e,axis,name,format!("{number:.2}"));}}
                    if entry.kind==4 {button(p,&f,Action::ToggleCollision,"collision","Collision",0.,e.object_collision(entry));}
                    button(p,&f,Action::Duplicate,"duplicate","Duplicate entity",0.,false);
                    p.spawn(row()).with_children(|p| {
                        button(p,&f,Action::Copy,"copy","Copy",83.,false);
                        button(p,&f,Action::Paste,"paste","Paste",83.,false);
                        button(p,&f,Action::Delete,"delete","Delete",83.,false);
                    });
                }
                if let Some(point)=e.selected_point.filter(|_|e.selected.is_none()) {
                    for (axis,current) in [("X",-point.x*100.),("Y",point.z*100.),("Z",point.y*100.)] {
                        p.spawn(row()).with_children(|p| {
                            p.spawn(Node{width:px(100),flex_shrink:0.,..default()}).with_children(|p|value(p,&f,axis,14.));
                            value(p,&f,format!("{current:.2}"),15.);
                        });
                    }
                }
                if let Some(tool)=e.terrain_tool {
                    button(p,&f,Action::CreateTerrain,"create_terrain","Create terrain · click an empty square",0.,e.creating_terrain);
                    label(p,&f,LocalizedText::new("ui.editor.world.terrain","Terrain editor"),18.,WHITE);
                    for (mode,key,name) in [(0,"terrain_height","Height"),(1,"terrain_flatten","Flatten"),(2,"terrain_paint","Paint textures"),(3,"terrain_paths","Path textures"),(4,"terrain_grass","Grass")] {button(p,&f,Action::Terrain(mode),key,name,0.,tool==mode);}
                    if tool==4 { value(p,&f,e.grass_template.as_ref().and_then(|t|t.row["object"].as_str()).unwrap_or("—"),14.);button(p,&f,Action::ChooseType(false),"grass_model","Choose grass model",0.,false); }
                    field(p,&f,&e,Field::BrushRadius,"brush_radius",e.brush_radius.to_string());
                    field(p,&f,&e,Field::BrushStrength,"brush_strength",e.brush_strength.to_string());
                    label(p,&f,LocalizedText::new("ui.editor.world.brush_help","LMB: paint · Shift: lower height · RMB: camera · Ctrl+Z/Y: undo/redo"),13.,MUTED);
                    if tool==2 || tool==3 {if let Some(tile)=atlas::tile_at(e.brush_cursor.unwrap_or(e.center)).and_then(|at|e.ground.get(&atlas::tile_id(at))) {
                        for (i,layer) in tile.descriptor["splat"]["layers"].as_array().into_iter().flatten().enumerate() {
                            let name=layer["trueTextureName"].as_str().unwrap_or("?");
                            if tool==3 && !terrain::path_texture(name){continue;}
                            p.spawn((Button,Action::TerrainLayer(i),Node{width:percent(100),min_height:px(58),column_gap:px(8),align_items:AlignItems::Center,..default()},BackgroundColor(if i==e.brush_layer{Color::srgb(0.08,0.25,0.41)}else{Color::srgb(0.03,0.08,0.14)}))).with_children(|p| {
                                if let Some(path)=layer["albedo"]["path"].as_str(){let image=images.terrain.entry(path.to_owned()).or_insert_with(||server.load(path.to_owned()));p.spawn((Node{width:px(48),height:px(48),flex_shrink:0.,..default()},ImageNode::new(image.clone())));}
                                value(p,&f,name,14.);
                            });
                        }
                    }}
                    if e.focus.is_some(){button(p,&f,Action::Apply,"apply","Apply",0.,false);}
                    return;
                }
                if e.selected().is_some()||e.selected_point.is_some() {
                    button(p,&f,Action::CopyCoordinates,"copy_coordinates","Copy coordinates",0.,false);
                }
                if e.selected().is_some()&&e.coordinate_clipboard.is_some() {
                    button(p,&f,Action::PasteCoordinates,"paste_coordinates","Paste coordinates",0.,false);
                }
                field(p,&f,&e,Field::Snap,"snap",e.snap.to_string());
                label(p,&f,LocalizedText::new("ui.editor.world.new","Place an entity"),18.,WHITE);
                p.spawn(row()).with_children(|p| {
                    for (i,key,en) in [(0,"npc","NPC"),(1,"mob","Mob"),(2,"group","Group"),(3,"shiny","Shiny")] {
                        button(p,&f,Action::Kind(i),key,en,62.,e.placement_kind==i);
                    }
                });
                button(p,&f,Action::Kind(4),"world_model","World model",0.,e.placement_kind==4);
                if e.placement_kind==4{value(p,&f,e.object_template.as_ref().and_then(|t|t.row["object"].as_str()).unwrap_or("—").to_string(),14.);}else{field(p,&f,&e,Field::Type,"type",e.type_id.to_string());}
                button(p,&f,Action::ChooseType(false),"choose","Choose",0.,false);
                button(p,&f,Action::Place,if e.placing {"placing"}else{"place"},if e.placing {"Click in the world · Esc cancels"}else{"Place in the world"},0.,e.placing);
                if e.focus.is_some() { p.spawn(row()).with_children(|p| {
                    button(p,&f,Action::Apply,"apply","Apply",110.,false);
                    button(p,&f,Action::Cancel,"cancel","Cancel",110.,false);
                }); }
            });
        });
        type_picker::draw(root,&f,&e);
        if let Some(status)=&e.status {label(root,&f,status.clone(),14.,WHITE);}
        else {label(root,&f,LocalizedText::new("ui.editor.world.coordinates","Server XY plane · Z = height · angle in degrees · instance = iMapNum · Ctrl+S saves work"),13.,MUTED);}
    });
}
pub(super) fn coordinate_marker(e:Res<WorldEditor>,state:Res<EditorState>,mut gizmos:Gizmos) {
    if state.world_open!=Some(true)||e.map_picker||e.selected.is_some()||e.terrain_tool.is_some()||e.routes.is_some(){return;}
    if let Some(point)=e.selected_point {
        let center=point+Vec3::Y*0.15;
        let radius=(e.distance*0.006).clamp(0.5,6.);
        let color=Color::srgb(0.2,0.85,1.);
        gizmos.line(center-Vec3::X*radius,center+Vec3::X*radius,color);
        gizmos.line(center-Vec3::Z*radius,center+Vec3::Z*radius,color);
        gizmos.line(center,center+Vec3::Y*radius,color);
    }
}
fn field_row(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    e: &WorldEditor,
    a: Field,
    name: &str,
    current: String,
) {
    field(p, f, e, a, name, current);
}
