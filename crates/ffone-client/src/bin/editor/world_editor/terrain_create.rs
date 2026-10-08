//! New native squares are staged alongside existing world edits, with one undo action.
use super::*;
use model::{Patch, Source};
use ffone_client::native_terrain::{NativeTerrain, NativeHeightmapCollider};

fn digest(bytes: &[u8]) -> Value { Value::from(format!("blake3:{}",blake3::hash(bytes).to_hex())) }
pub(super) fn update_height_metadata(descriptor: &mut Value, samples: &[u16]) -> Result<(), String> {
    let width=descriptor["dimensions"]["width"].as_u64().ok_or("Missing terrain width")? as usize;
    let height=descriptor["dimensions"]["height"].as_u64().ok_or("Missing terrain height")? as usize;
    let raw:Vec<_>=samples.iter().flat_map(|v|v.to_le_bytes()).collect();
    let transpose:Vec<_>=(0..width).flat_map(|x|(0..height).flat_map(move |z|samples[z*width+x].to_le_bytes())).collect();
    descriptor["heightmap"]["canonicalOrderRawBlake3"]=digest(&raw);
    descriptor["heightmap"]["sourceOrderRawBlake3"]=digest(&transpose);
    descriptor["heightmap"]["rawMin"]=Value::from(*samples.iter().min().ok_or("Empty heightmap")?);
    descriptor["heightmap"]["rawMax"]=Value::from(*samples.iter().max().unwrap());
    let scale=descriptor["scale"]["heightScale"].as_f64().unwrap()/32767.;
    descriptor["nativeGeometry"]["localBounds"]["min"][1]=Value::from(*samples.iter().min().unwrap() as f64*scale);
    descriptor["nativeGeometry"]["localBounds"]["max"][1]=Value::from(*samples.iter().max().unwrap() as f64*scale);
    Ok(())
}
fn identity() -> Value { serde_json::json!({"translation":[0.,0.,0.],"rotation":[0.,0.,0.,1.],"scale":[1.,1.,1.]}) }

impl WorldEditor {
    fn stage_document(&mut self, path: PathBuf, after: Value, patches: &mut Vec<Patch>) -> Result<usize,String> {
        let i=if let Some(i)=self.sources.iter().position(|s|s.path==path){i}else{
            let base=if path.exists(){terrain::read_source(&path)?}else{Value::Null};
            self.sources.push(Source{path,base:base.clone(),draft:base}); self.sources.len()-1
        };
        patches.push(Patch{source:i,pointer:String::new(),before:Some(self.sources[i].draft.clone()),after:Some(after)});
        Ok(i)
    }
    pub(super) fn create_terrain(&mut self, point: Vec3) -> Result<(), String> {
        let at=atlas::tile_at(point).ok_or("Invalid square")?;
        let id=atlas::tile_id(at);
        if self.ground.contains_key(&id) || ground::Tile::load(&self.root,&id).is_ok() { return Err("This square already has terrain".into()); }
        self.finish_brush()?;
        // An existing square without ground still owns objects and scene settings.
        if !self.maps.contains_key(&id) && self.root.join(format!("map/tiles/{id}/scene.json")).is_file() {
            let documents=stream::read_tile(&self.root,&id,!self.sources.iter().any(|s|s.path==self.root.join("map/catalog.json")))?;
            self.install_tile(id.clone(),documents);
        }
        self.scan_tiles();
        let mut candidates:Vec<_>=self.available_tiles.iter().copied().collect();
        candidates.sort_by_key(|t|(t[0]-at[0]).pow(2)+(t[1]-at[1]).pow(2));
        let (template_id,template)=candidates.into_iter().find_map(|t|{
            let id=atlas::tile_id(t); ground::Tile::load(&self.root,&id).ok().map(|tile|(id,tile))
        }).ok_or("No native terrain template is available")?;
        let template_path=template.path.strip_prefix(&self.root).map_err(|e|e.to_string())?.to_string_lossy().replace('\\',"/");
        for t in atlas::neighbours(at).into_iter().filter(|t|*t!=at) {
            let tid=atlas::tile_id(t);
            if !self.ground.contains_key(&tid) { if let Ok(tile)=ground::Tile::load(&self.root,&tid){self.ground.insert(tid,tile);} }
        }
        let origin=Vec3::new(-at[0] as f32*512.,0.,at[1] as f32*512.);
        let width=template.width; let height=template.height;
        // Project onto all available edges. Exact edge samples take precedence over interpolation.
        let mut boundary=Vec::new();
        for z in 0..height { for x in 0..width {
            if x!=0&&z!=0&&x+1!=width&&z+1!=height{continue;}
            let p=origin+Vec3::new(-(x as f32)*template.spacing.x,0.,z as f32*template.spacing.y);
            if let Some(y)=self.ground.values().find_map(|tile|tile.height_at(p)) { boundary.push((x,z,y)); }
        }}
        let mut heights=vec![point.y;width*height];
        for z in 0..height { for x in 0..width {
            let mut sum=0.;let mut total=0.;
            for &(bx,bz,y) in &boundary {
                let distance=(x.abs_diff(bx).pow(2)+z.abs_diff(bz).pow(2)) as f32;
                if distance==0.{sum=y;total=1.;break;}
                let weight=1./distance;sum+=y*weight;total+=weight;
            }
            if total>0.{heights[z*width+x]=sum/total;}
        }}
        let base=heights.iter().copied().fold(f32::INFINITY,f32::min)-1.;
        let top=heights.iter().copied().fold(f32::NEG_INFINITY,f32::max);
        let range=(top-base+10.).max(600.);
        let samples:Vec<_>=heights.iter().map(|y|((y-base)/range*32767.).round().clamp(0.,65535.) as u16).collect();
        let mut descriptor=template.descriptor.clone();
        for key in ["source","sceneInstance","detailAndTrees","gameplayAttributes","lightmap"] { descriptor.as_object_mut().unwrap().remove(key); }
        descriptor["trueName"]=Value::from(format!("Terrain_{id}"));
        descriptor["scale"]["heightScale"]=Value::from(range);
        descriptor["heightmap"]["vertexShifts"]=serde_json::json!([]);
        update_height_metadata(&mut descriptor,&samples)?;
        let mut weights=template.weights.clone();
        let resolution=descriptor["splat"]["resolution"].as_u64().unwrap() as usize;
        let names:Vec<_>=descriptor["splat"]["layers"].as_array().unwrap().iter().map(|l|l["trueTextureName"].as_str().unwrap_or_default()).collect();
        for z in 0..resolution{for x in 0..resolution{
            let p=origin+Vec3::new(-(x as f32+0.5)/resolution as f32*512.,0.,(z as f32+0.5)/resolution as f32*512.);
            let nearest=self.ground.values().filter_map(|tile|{
                let local=tile.transform.inverse().transform_point3(p);
                let q=Vec3::new(local.x.clamp(-512.,0.),local.y,local.z.clamp(0.,512.));
                Some((tile,q,(q.x-local.x).powi(2)+(q.z-local.z).powi(2)))
            }).min_by(|a,b|a.2.total_cmp(&b.2));
            let Some((tile,q,_))=nearest else{continue;};
            let sr=tile.descriptor["splat"]["resolution"].as_u64().unwrap() as usize;
            let sx=(-q.x/512.*sr as f32).floor().clamp(0.,(sr-1) as f32) as usize;
            let sz=(q.z/512.*sr as f32).floor().clamp(0.,(sr-1) as f32) as usize;
            let mut values=vec![0u8;weights.len()*4];
            for (i,name) in names.iter().enumerate(){
                if let Some(source)=tile.descriptor["splat"]["layers"].as_array().unwrap().iter().position(|l|l["trueTextureName"].as_str()==Some(*name)){
                    values[i]=tile.weights[source/4][(sz*sr+sx)*4+source%4];
                }
            }
            if values.iter().any(|v|*v>0){terrain::normalize_weights(&mut values,0);for (i,raw) in weights.iter_mut().enumerate(){raw[(z*resolution+x)*4..(z*resolution+x)*4+4].copy_from_slice(&values[i*4..i*4+4]);}}
        }}
        let path=self.root.join(format!("map/tiles/{id}/terrain/terrain.json"));
        let mut patches=Vec::new();
        self.stage_document(path.parent().unwrap().join("heightmap.png"),terrain::pixels(width as u32,height as u32,"gray16",&samples),&mut patches)?;
        let res=descriptor["splat"]["resolution"].as_u64().unwrap() as u32;
        for (map,raw) in descriptor["splat"]["weightMaps"].as_array().unwrap().iter().zip(&weights) {
            self.stage_document(path.parent().unwrap().join(map["path"].as_str().unwrap()),terrain::pixels(res,res,"rgba8",raw),&mut patches)?;
        }
        let mut environment:Value=terrain::read_source(&template.path.parent().unwrap().join("environment/environment.json"))?;
        environment["tileId"]=Value::from(format!("{:02}_{:02}",at[0],at[1]));
        environment["ambience"]["gridCoordinates"]=serde_json::json!(at);
        let env_path=self.root.join(format!("map/tiles/{id}/terrain/environment/environment.json"));
        let env_hash=blake3::hash(&terrain::source_bytes(&environment)?).to_hex().to_string();
        descriptor["environment"]["blake3"]=Value::from(format!("blake3:{env_hash}"));
        self.stage_document(env_path,environment,&mut patches)?;
        self.stage_document(path.clone(),descriptor.clone(),&mut patches)?;
        let mut scene=if let Some(map)=self.maps.get(&id){self.sources[map.scene].draft.clone()}else{
            let mut s=terrain::read_source(&self.root.join(format!("map/tiles/{template_id}/scene.json")))?;
            for key in ["provenance","nativeTerrain"]{s.as_object_mut().unwrap().remove(key);}
            s["name"]=Value::from(id.clone());s["tile"]=serde_json::json!(at);s["root"]=identity();
            for key in ["models","visuals","colliders"]{s[key]=serde_json::json!([]);}s
        };
        let root:ffone_client::world::AuthoredWorldTransform=serde_json::from_value(scene["root"].clone()).map_err(|e|e.to_string())?;
        let global=Transform::from_translation(origin+Vec3::Y*base);
        let local=Transform::from_matrix(root.try_to_bevy("new terrain root").map_err(|e|e.to_string())?.to_matrix().inverse()*global.to_matrix());
        scene["nativeTerrain"]=serde_json::json!({"editorAuthored":true,"editorTemplate":template_path,"name":format!("Terrain_{id}"),"trueName":format!("Terrain_{id}"),"path":format!("map/tiles/{id}/terrain/terrain.json"),"blake3":"0".repeat(64),"transform":{"translation":local.translation.to_array(),"rotation":local.rotation.to_array(),"scale":local.scale.to_array()},"environment":{"path":format!("map/tiles/{id}/terrain/environment/environment.json"),"schema":"ffone.native-terrain-environment.v1","status":"complete","blake3":env_hash}});
        let scene_source=self.stage_document(self.root.join(format!("map/tiles/{id}/scene.json")),scene,&mut patches)?;
        let existing=self.maps.get(&id).map(|m|m.objects);
        let objects=if let Some(objects)=existing{objects}else{
            let mut objects=terrain::read_source(&self.root.join(format!("map/tiles/{template_id}/objects.json")))?;
            objects["tileId"]=Value::from(id.clone());objects["objects"]=serde_json::json!([]);
            let index=self.stage_document(self.root.join(format!("map/tiles/{id}/objects.json")),objects,&mut patches)?;
            let mut behaviour=terrain::read_source(&self.root.join(format!("map/tiles/{template_id}/behaviour.json")))?;
            for value in behaviour.as_object_mut().unwrap().values_mut(){if value.is_array(){*value=serde_json::json!([]);}}
            behaviour["tile"]=serde_json::json!(at);behaviour["id"]=Value::from(id.clone());
            self.stage_document(self.root.join(format!("map/tiles/{id}/behaviour.json")),behaviour,&mut patches)?;
            index
        };
        let mut manifest=terrain::read_source(&self.root.join(format!("map/tiles/{template_id}/tile.json")))?;
        if let Some(source)=self.sources.iter().find(|s|s.path==self.root.join(format!("map/tiles/{id}/tile.json"))) { manifest=source.draft.clone(); }
        manifest["id"]=Value::from(id.clone());manifest["grid"]=serde_json::json!(at);
        if existing.is_none(){manifest["files"]=serde_json::json!([]);}
        for key in ["scene","objects","behaviour","terrain"] {
            manifest[key]=serde_json::json!({"path":format!("map/tiles/{id}/{}",if key=="terrain"{"terrain/terrain.json".into()}else{format!("{key}.json")}),"bytes":0,"blake3":"0".repeat(64)});
        }
        self.stage_document(self.root.join(format!("map/tiles/{id}/tile.json")),manifest,&mut patches)?;
        let mut catalog=self.sources.iter().find(|s|s.path==self.root.join("map/catalog.json")).map(|s|s.draft.clone()).unwrap_or(terrain::read_source(&self.root.join("map/catalog.json"))?);
        if !catalog["tiles"].as_array().unwrap().iter().any(|t|t["tileId"]==id) {
            catalog["tiles"].as_array_mut().unwrap().push(serde_json::json!({"tileId":id,"manifest":{"path":format!("map/tiles/{id}/tile.json"),"bytes":0,"blake3":"0".repeat(64)},"objects":{"path":format!("map/tiles/{id}/objects.json"),"bytes":0,"blake3":"0".repeat(64)}}));
        }
        self.stage_document(self.root.join("map/catalog.json"),catalog,&mut patches)?;
        self.maps.insert(id.clone(),map::Tile{id:id.clone(),objects,scene:scene_source,native_scene:None});
        self.commit(patches)?;
        self.region=Some(at);self.creating_terrain=false;self.selected_point=Some(point);self.geometry_revision+=1;
        Ok(())
    }
    pub(super) fn restore_authored_terrain(&mut self) -> Result<(),String> {
        let removed:Vec<_>=self.sources.iter().filter(|s|s.path.ends_with("terrain/terrain.json")&&s.draft.is_null()).filter_map(|s|s.path.parent().map(Path::to_path_buf)).collect();
        for source in &mut self.sources {if removed.iter().any(|folder|source.path.starts_with(folder)){source.draft=Value::Null;}}
        let scenes:Vec<_>=self.sources.iter().enumerate().filter(|(_,s)|s.path.ends_with("scene.json")&&s.draft["nativeTerrain"]["editorAuthored"]==true).map(|(i,s)|(i,s.draft.clone())).collect();
        for (scene_index,scene) in scenes {
            let rel=scene["nativeTerrain"]["path"].as_str().ok_or("Missing authored descriptor")?;
            let path=self.root.join(rel);
            let id=scene["name"].as_str().ok_or("Missing square name")?.to_owned();
            if self.ground.contains_key(&id){continue;}
            let Some(source)=self.sources.iter().find(|s|s.path==path&&!s.draft.is_null())else{continue;};
            let descriptor=source.draft.clone();
            let template=scene["nativeTerrain"]["editorTemplate"].as_str().ok_or("Missing terrain template")?;
            let bytes=fs::read(self.root.join(template)).map_err(|e|e.to_string())?;
            let native=NativeTerrain::open(&self.root,template,&blake3::hash(&bytes).to_hex().to_string()).map_err(|e|e.to_string())?;
            let parent=path.parent().unwrap();
            let read=|rel:&str|->Result<Value,String>{let path=parent.join(rel); self.sources.iter().find(|s|s.path==path).map(|s|Ok(s.draft.clone())).unwrap_or_else(||terrain::read_source(&path))};
            let samples:Vec<u16>=serde_json::from_value(read("heightmap.png")?["pixels"].clone()).map_err(|e|e.to_string())?;
            let weights=descriptor["splat"]["weightMaps"].as_array().unwrap().iter().map(|w|serde_json::from_value(read(w["path"].as_str().unwrap())?["pixels"].clone()).map_err(|e|e.to_string())).collect::<Result<Vec<Vec<u8>>,String>>()?;
            let typed=serde_json::from_value(descriptor.clone()).map_err(|e|e.to_string())?;
            let collider=NativeHeightmapCollider::from_samples(&typed,&samples).map_err(|e|e.to_string())?;
            let authored=native.with_editor_descriptor(typed,&samples,&weights).map_err(|e|e.to_string())?;
            let root:ffone_client::world::AuthoredWorldTransform=serde_json::from_value(scene["root"].clone()).map_err(|e|e.to_string())?;
            let local:ffone_client::world::AuthoredWorldTransform=serde_json::from_value(scene["nativeTerrain"]["transform"].clone()).map_err(|e|e.to_string())?;
            let transform=root.try_to_bevy("authored root").map_err(|e|e.to_string())?.mul_transform(local.try_to_bevy("authored terrain").map_err(|e|e.to_string())?).to_matrix();
            let width=descriptor["dimensions"]["width"].as_u64().unwrap() as usize;let height=descriptor["dimensions"]["height"].as_u64().unwrap() as usize;
            let spacing=Vec2::new(descriptor["scale"]["sampleSpacingX"].as_f64().unwrap() as f32,descriptor["scale"]["sampleSpacingZ"].as_f64().unwrap() as f32);
            let height_scale=descriptor["scale"]["heightScale"].as_f64().unwrap() as f32/32767.;
            self.ground.insert(id.clone(),ground::Tile{authored:Some(authored),descriptor,samples,path,weights,revision:1,transform,width,height,spacing,height_scale,collider});
            if let Some(objects)=self.sources.iter().position(|s|s.path==self.root.join(format!("map/tiles/{id}/objects.json"))){self.maps.entry(id.clone()).or_insert(map::Tile{id,objects,scene:scene_index,native_scene:None});}
        }
        let sources=&self.sources;
        self.ground.retain(|_,t|t.authored.is_none()||sources.iter().any(|s|s.path==t.path&&!s.draft.is_null()));
        Ok(())
    }
}
