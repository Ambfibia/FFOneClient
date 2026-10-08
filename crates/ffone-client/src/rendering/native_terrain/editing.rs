//! Isolated authoring copies retain accepted layer assets and update geometry in place.
use super::*;

impl NativeTerrain {
    /// An authored square reuses verified layer images, while supplying its own height basis.
    pub fn with_editor_descriptor(&self, descriptor: NativeTerrainDescriptor, samples: &[u16], weights: &[Vec<u8>]) -> Result<Self, NativeTerrainError> {
        validate_descriptor(&descriptor)?;
        let mut next = self.clone();
        next.descriptor = descriptor;
        next.grass_layers = Vec::new().into();
        next.gameplay_attributes = None;
        next.lightmap = None;
        next.lightmap_mips = None;
        next.with_editor_data(samples, weights)
    }
    pub fn with_editor_data(
        &self,
        samples: &[u16],
        weights: &[Vec<u8>],
    ) -> Result<Self, NativeTerrainError> {
        let geometry = build_geometry(&self.descriptor, samples)?;
        if weights.len() != self.weight_maps.len()
            || weights.iter().any(|v| {
                v.len()
                    != self.descriptor.splat.resolution as usize
                        * self.descriptor.splat.resolution as usize
                        * 4
            })
        {
            return Err(NativeTerrainError::new(
                "Editor terrain weight dimensions differ from the descriptor",
            ));
        }
        let mut next = self.clone();
        next.geometry = Arc::new(geometry);
        next.height_samples = samples.to_vec().into();
        let resolution = self.descriptor.splat.resolution;
        let mut chains: Vec<Vec<VerifiedRgbaImage>> = weights
            .iter()
            .map(|v| {
                vec![VerifiedRgbaImage {
                    width: resolution,
                    height: resolution,
                    pixels: v.clone().into(),
                }]
            })
            .collect();
        let mut size = resolution;
        while size > 1 {
            let smaller = (size / 2).max(1);
            for chain in &mut chains {
                let source = &chain.last().unwrap().pixels;
                let mut pixels = vec![0; smaller as usize * smaller as usize * 4];
                for y in 0..smaller as usize {
                    for x in 0..smaller as usize {
                        for c in 0..4 {
                            let sum: u32 = (0..2)
                                .flat_map(|dy| (0..2).map(move |dx| (dx, dy)))
                                .map(|(dx, dy)| {
                                    u32::from(
                                        source[((y * 2 + dy).min(size as usize - 1)
                                            * size as usize
                                            + (x * 2 + dx).min(size as usize - 1))
                                            * 4
                                            + c],
                                    )
                                })
                                .sum();
                            pixels[(y * smaller as usize + x) * 4 + c] = ((sum + 2) / 4) as u8;
                        }
                    }
                }
                chain.push(VerifiedRgbaImage {
                    width: smaller,
                    height: smaller,
                    pixels: pixels.into(),
                });
            }
            size = smaller;
        }
        next.weight_maps = chains
            .iter()
            .map(|c| c[0].clone())
            .collect::<Vec<_>>()
            .into();
        next.weight_map_mips = chains.into();
        Ok(next)
    }
    pub fn editor_mesh(&self) -> Mesh {
        self.geometry.to_mesh()
    }
    pub fn editor_weight_image(&self) -> Image {
        weight_array_image(self)
    }
    pub fn editor_weight_mips(&self) -> Vec<Vec<(u32, u32, Vec<u8>)>> {
        self.weight_map_mips
            .iter()
            .map(|chain| {
                chain
                    .iter()
                    .map(|i| (i.width, i.height, i.pixels.to_vec()))
                    .collect()
            })
            .collect()
    }
}
