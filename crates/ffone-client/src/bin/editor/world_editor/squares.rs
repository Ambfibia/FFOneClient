use super::*;
impl WorldEditor {
    pub(super) fn open_square_choices(&mut self,mode:u8)->Result<(),String>{
        if mode==1&&self.music_choices.is_empty(){
            let catalog=ffone_client::semantic_audio::NativeAudioCatalog::open(&self.root,false)?;
            self.music_choices=catalog.assets().iter().filter(|a|a.category==ffone_client::semantic_audio::NativeAudioCategory::Music).map(|a|(a.true_name.clone(),a.logical_key.clone())).collect();
            self.music_choices.sort();self.music_choices.insert(0,("—".into(),String::new()));self.music_choices.insert(1,("stop".into(),"stop".into()));
        }
        self.square_choices=if self.square_choices.is_some_and(|(m,_)|m==mode){None}else{Some((mode,0))};self.focus=None;self.revision+=1;Ok(())
    }
    pub(super) fn square_values(&self,mode:u8)->Vec<(String,String)>{
        if mode==1{return self.music_choices.clone();}
        let names:BTreeSet<_>=(0..16).flat_map(|x|(0..16).filter_map(move |y|ffone_client::legacy_world_location::legacy_world_location_name((x as f32+0.5)*512.,(y as f32+0.5)*512.))).collect();
        std::iter::once(("—".into(),String::new())).chain(names.into_iter().map(|s|(s.into(),s.into()))).collect()
    }
    pub(super) fn choose_square_value(&mut self,index:usize)->Result<(),String>{
        let mode=self.square_choices.ok_or("Open square choices")?.0;
        let value=self.square_values(mode).get(index).ok_or("Choose a value")?.1.clone();
        self.change_square(if mode==0{"location"}else{"music"},Value::from(value))?;self.square_choices=None;Ok(())
    }
    pub(super) fn square_id(&self)->Option<String>{atlas::tile_at(self.selected().map(|p|p.position).or(self.selected_point).unwrap_or(self.center)).map(atlas::tile_id)}
    pub(super) fn square_settings(&self)->Value {
        self.square_id().and_then(|id|self.maps.get(&id)).and_then(|map|self.sources[map.scene].draft.get("squareSettings")).cloned().unwrap_or_else(||serde_json::json!({"location":"","music":"","skybox":"","terrainShader":null}))
    }
    pub(super) fn open_square_settings(&mut self)->Result<(),String>{
        if !self.square_menu {
            let id=self.square_id().ok_or("Select a square")?;
            if !self.maps.contains_key(&id){let documents=stream::read_tile(&self.root,&id,!self.sources.iter().any(|s|s.path==self.root.join("map/catalog.json")))?;self.install_tile(id,documents);}
        }
        self.square_menu=!self.square_menu;self.focus=None;self.revision+=1;Ok(())
    }
    pub(super) fn change_square(&mut self,key:&str,value:Value)->Result<(),String>{
        let id=self.square_id().ok_or("Select a square")?;
        let source=self.maps.get(&id).ok_or("Load the square")?.scene;
        let before=self.sources[source].draft.get("squareSettings").cloned();
        let mut after=self.square_settings();after[key]=value;
        let settings:ffone_client::world::NativeSquareSettings=serde_json::from_value(after.clone()).map_err(|e|e.to_string())?;
        settings.validate().map_err(|e|e.to_string())?;
        self.square_names.insert(id,settings.location);
        self.commit(vec![model::Patch{source,pointer:"/squareSettings".into(),before,after:Some(after)}])
    }
    pub(super) fn location_name(&self,point:Vec3)->String {
        self.square_id_for_point(point).and_then(|id|match self.maps.get(&id){Some(m)=>Some(self.sources[m.scene].draft["squareSettings"]["location"].as_str().unwrap_or_default()),None=>self.square_names.get(&id).map(String::as_str)}).filter(|s|!s.is_empty()).or_else(||ffone_client::legacy_world_location::legacy_world_location_name(-point.x,point.z)).unwrap_or("—").to_owned()
    }
    fn square_id_for_point(&self,point:Vec3)->Option<String>{atlas::tile_at(point).map(atlas::tile_id)}
    pub(super) fn prepare_square_names(&mut self,npc:i64){
        let ids:BTreeSet<_>=self.entities.iter().filter(|p|p.kind<3&&p.type_id==npc).filter_map(|p|atlas::tile_at(p.position)).map(atlas::tile_id).collect();
        for id in ids {
            if self.square_names.contains_key(&id){continue;}
            let name=terrain::read_source(&self.root.join(format!("map/tiles/{id}/scene.json"))).ok().and_then(|s|s["squareSettings"]["location"].as_str().map(str::to_owned)).unwrap_or_default();
            self.square_names.insert(id,name);
        }
    }
}
pub(super) fn draw(p:&mut ChildSpawnerCommands,f:&EditorFonts,e:&WorldEditor){
    use view::{button,field,label,value,WHITE,MUTED};
    value(p,f,e.square_id().unwrap_or_default(),18.);
    let settings=e.square_settings();
    field(p,f,e,Field::SquareName,"square_location",settings["location"].as_str().unwrap_or_default().to_owned());
    button(p,f,Action::SquareChoices(0),"choose","Choose",0.,e.square_choices.is_some_and(|(mode,_)|mode==0));
    field(p,f,e,Field::SquareMusic,"square_music",settings["music"].as_str().unwrap_or_default().to_owned());
    button(p,f,Action::SquareChoices(1),"choose","Choose",0.,e.square_choices.is_some_and(|(mode,_)|mode==1));
    if let Some((mode,page))=e.square_choices {
        for (i,(name,_)) in e.square_values(mode).into_iter().enumerate().skip(page*8).take(8){button(p,f,Action::SquareChoice(i),"value",&name,0.,false);}
        p.spawn(view::row()).with_children(|p|{button(p,f,Action::SquarePage(false),"previous","←",55.,false);button(p,f,Action::SquarePage(true),"next","→",55.,false);});
    }
    label(p,f,LocalizedText::new("ui.editor.world.square_music_help","Music: native track name or logical key; empty = zone default; stop = silence"),13.,MUTED);
    label(p,f,LocalizedText::new("ui.editor.world.square_skybox","Skybox"),16.,WHITE);
    for (mode,key,name) in [(0,"square_default","Default"),(1,"square_past","Past"),(2,"square_future","Future")] {
        button(p,f,Action::SquareSkybox(mode),key,name,0.,settings["skybox"]==["","past","future"][mode as usize]);
    }
    label(p,f,LocalizedText::new("ui.editor.world.square_shader","Terrain shader"),16.,WHITE);
    for (mode,key,name) in [(3,"square_default","Default"),(0,"square_vertexlit","Vertexlit"),(1,"square_lightmap","Lightmap"),(2,"square_realtime","Realtime")] {
        button(p,f,Action::SquareShader(mode),key,name,0.,settings["terrainShader"].as_u64()==(mode!=3).then_some(u64::from(mode)));
    }
    if e.focus.is_some(){button(p,f,Action::Apply,"apply","Apply",0.,false);}
}
