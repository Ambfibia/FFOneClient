use super::*;
impl WorldEditor {
    pub(super) fn grass_brush(&mut self,point:Vec3)->Result<(),String>{
        let template=self.grass_template.clone().ok_or("Choose a grass model")?;
        if self.grass_last.is_some_and(|last|last.distance(point)<(self.brush_radius*0.65).max(0.5)){return Ok(());}
        if self.grass_stroke.is_none(){self.grass_stroke=Some(self.undo.len());}
        self.grass_last=Some(point);
        let count=self.brush_strength.round().clamp(1.,24.) as usize;
        for i in 0..count {
            let theta=i as f32*2.399963;
            let radius=self.brush_radius*((i as f32+0.5)/count as f32).sqrt();
            let position=self.snap_to_ground(point+Vec3::new(theta.cos()*radius,0.,theta.sin()*radius))?;
            self.paste_object(template.clone(),position)?;
        }
        Ok(())
    }
    pub(super) fn finish_grass(&mut self){
        self.grass_last=None;
        let Some(start)=self.grass_stroke.take()else{return;};
        let changes:Vec<_>=self.undo.drain(start..).collect();
        let mut merged:BTreeMap<(usize,String),model::Patch>=BTreeMap::new();
        for edit in changes{for patch in edit.patches{merged.entry((patch.source,patch.pointer.clone())).and_modify(|p|p.after=patch.after.clone()).or_insert(patch);}}
        let patches:Vec<_>=merged.into_values().filter(|p|p.before!=p.after).collect();
        if !patches.is_empty(){self.undo.push(model::Edit{patches,selection:self.selected().map(|p|p.key.clone())});}
    }
}
