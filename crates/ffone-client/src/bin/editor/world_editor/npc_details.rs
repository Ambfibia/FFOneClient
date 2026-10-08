//! Every placement is listed independently, including portal/instance duplicates.
use super::*;

impl WorldEditor {
    pub(crate) fn npc_has_placements(&self,npc:i64)->bool{self.entities.iter().any(|p|p.kind<3&&p.type_id==npc)}
    pub(crate) fn prepare_npc_details(&mut self, folder: &Path, npc:i64) {
        if self.sources.is_empty() {
            if let Err(error) = self.load_server(folder) { self.error(error); }
        }
        self.prepare_square_names(npc);
    }
    pub(crate) fn npc_placements(&self, npc_type: i64, l: &Localization, lang: &Language) -> String {
        let placements: Vec<_> = self.entities.iter().filter(|p| p.kind < 3 && p.type_id == npc_type).collect();
        let mut lines = vec![l.text(lang, &LocalizedText::new("ui.editor.npc.placement_count", "World placements: {count}").with_arg("count", placements.len().to_string()))];
        for p in placements {
            let tile = atlas::tile_at(p.position).map(atlas::tile_id).unwrap_or_else(|| "—".into());
            let location = self.location_name(p.position);
            let instance = self.instances.get(&p.instance).map(String::as_str).unwrap_or("—");
            lines.push(l.text(lang, &LocalizedText::new("ui.editor.npc.placement_info", "{key}\nX {x} · Y {y} · Z {z}\nInstance {id}: {instance}\n{tile} · {location}")
                .with_arg("key", p.key.clone()).with_arg("x", format!("{:.2}", -p.position.x*100.))
                .with_arg("y", format!("{:.2}", p.position.z*100.)).with_arg("z", format!("{:.2}", p.position.y*100.))
                .with_arg("id", p.instance.to_string()).with_arg("instance", instance).with_arg("tile", tile).with_arg("location", location)));
        }
        lines.join("\n\n")
    }
}
