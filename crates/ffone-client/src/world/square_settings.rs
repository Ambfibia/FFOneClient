//! Authored per-square presentation; legacy defaults apply when an override is absent.
use super::*;
#[derive(Component, Debug, Clone, Default, PartialEq, Eq, serde::Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all="camelCase")]
pub struct NativeSquareSettings {
    #[serde(default)] pub location: String,
    #[serde(default)] pub music: String,
    /// Empty, past, or future: the two native skybox families.
    #[serde(default)] pub skybox: String,
    /// Native terrain render families: 0 vertexlit, 1 lightmap, 2 realtime.
    #[serde(default)] pub terrain_shader: Option<u8>,
}
impl NativeSquareSettings {
    pub fn validate(&self)->Result<(),NativeWorldSceneError>{
        if !matches!(self.skybox.as_str(),""|"past"|"future")||self.terrain_shader.is_some_and(|n|n>2) {
            return Err(NativeWorldSceneError::new("Invalid square skybox or terrain shader"));
        }
        Ok(())
    }
}
pub fn square_at(position:Vec3)->[i32;2]{[(-position.x/512.).floor() as i32,(position.z/512.).floor() as i32]}
pub(super) fn update_shader(roots:Query<(&NativeWorldSceneRoot,&NativeSquareSettings)>,terrains:Query<(&MeshMaterial3d<crate::native_terrain::NativeTerrainMaterial>,&GlobalTransform)>,mut materials:ResMut<Assets<crate::native_terrain::NativeTerrainMaterial>>) {
    for (handle,transform) in &terrains {
        let p=transform.translation()+Vec3::new(-256.,0.,256.);
        if let Some(mode)=roots.iter().find(|(root,_)|root.tile==square_at(p)).and_then(|(_,s)|s.terrain_shader) {
            if let Some(mut material)=materials.get_mut(&handle.0){material.uniform.metadata.y=f32::from(mode);}
        }
    }
}
