use super::*;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticDigests {
    pub hierarchy_sha256: String,
    pub geometry_sha256: String,
    pub skins_sha256: String,
    pub standard_animations_sha256: String,
    pub animation_metadata_sha256: String,
    pub materials_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SemanticRoundtripProof {
    pub schema: String,
    pub status: String,
    pub matched: bool,
    pub canonical_encoding: String,
    pub floating_point_contract: String,
    pub covered_scopes: Vec<String>,
    pub excluded_scopes: Vec<String>,
    pub source: SemanticDigests,
    pub emitted: SemanticDigests,
}

#[derive(Debug, PartialEq)]
pub(super) enum CanonicalJsonNumber {
    Signed(i64),
    Unsigned(u64),
    Float(u64),
    Invalid,
}

#[derive(Debug, Eq, PartialEq)]
pub(super) struct SemanticSections {
    pub(super) hierarchy: Vec<u8>,
    pub(super) geometry: Vec<u8>,
    pub(super) skins: Vec<u8>,
    pub(super) standard_animations: Vec<u8>,
    pub(super) animation_metadata: Vec<u8>,
    pub(super) materials: Vec<u8>,
}

impl SemanticSections {
    pub(super) fn from_model(model: &NativeModel) -> Result<Self> {
        Ok(Self {
            hierarchy: hierarchy_from_model(model)?,
            geometry: geometry_from_model(model)?,
            skins: skins_from_model(model)?,
            standard_animations: animations_from_model(model)?,
            animation_metadata: animation_metadata_from_model(model)?,
            materials: materials_from_model(model)?,
        })
    }

    pub(super) fn from_glb(glb: &[u8]) -> Result<Self> {
        let parsed = ParsedGlb::parse(glb)?;
        Ok(Self {
            hierarchy: hierarchy_from_glb(&parsed)?,
            geometry: geometry_from_glb(&parsed)?,
            skins: skins_from_glb(&parsed)?,
            standard_animations: animations_from_glb(&parsed)?,
            animation_metadata: animation_metadata_from_glb(&parsed)?,
            materials: materials_from_glb(&parsed)?,
        })
    }

    pub(super) fn digests(&self) -> SemanticDigests {
        SemanticDigests {
            hierarchy_sha256: sha256(&self.hierarchy),
            geometry_sha256: sha256(&self.geometry),
            skins_sha256: sha256(&self.skins),
            standard_animations_sha256: sha256(&self.standard_animations),
            animation_metadata_sha256: sha256(&self.animation_metadata),
            materials_sha256: sha256(&self.materials),
        }
    }
}

#[derive(Default)]
pub(super) struct Canonical(pub(super) Vec<u8>);

impl Canonical {
    pub(super) fn tag(&mut self, value: &str) {
        self.string(value);
    }

    pub(super) fn len(&mut self, value: usize) -> Result<()> {
        self.u64(u64::try_from(value).map_err(|_| ModelError::Overflow("semantic item count"))?);
        Ok(())
    }

    pub(super) fn string(&mut self, value: &str) {
        self.u64(value.len() as u64);
        self.0.extend_from_slice(value.as_bytes());
    }

    pub(super) fn bool(&mut self, value: bool) {
        self.u8(u8::from(value));
    }

    pub(super) fn u8(&mut self, value: u8) {
        self.0.push(value);
    }

    pub(super) fn u16(&mut self, value: u16) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn u32(&mut self, value: u32) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn u64(&mut self, value: u64) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn i32(&mut self, value: i32) {
        self.0.extend_from_slice(&value.to_le_bytes());
    }

    pub(super) fn f32(&mut self, value: f32) {
        self.u32(value.to_bits());
    }

    pub(super) fn f64(&mut self, value: f64) {
        self.u64(value.to_bits());
    }

    pub(super) fn optional_u32(&mut self, value: Option<u32>) {
        self.bool(value.is_some());
        if let Some(value) = value {
            self.u32(value);
        }
    }

    pub(super) fn optional_i32(&mut self, value: Option<i32>) {
        self.bool(value.is_some());
        if let Some(value) = value {
            self.i32(value);
        }
    }

    pub(super) fn optional_f64(&mut self, value: Option<f64>) {
        self.bool(value.is_some());
        if let Some(value) = value {
            self.f64(value);
        }
    }
}

pub(super) struct Accessors<'document, 'binary> {
    pub(super) document: &'document Value,
    pub(super) binary: &'binary [u8],
}

impl<'document, 'binary> Accessors<'document, 'binary> {
    pub(super) fn new(glb: &'document ParsedGlb<'binary>) -> Result<Self> {
        if optional_root_array(&glb.document, "buffers")?.len() != 1 {
            return invalid("semantic proof requires exactly one GLB buffer");
        }
        Ok(Self {
            document: &glb.document,
            binary: glb.binary,
        })
    }

    pub(super) fn info(&self, index: u32, expected_type: &str) -> Result<AccessorInfo> {
        let accessors = required_array(self.document, "accessors", "GLB accessors")?;
        let accessor = indexed(accessors, index, "accessor")?;
        if accessor.get("sparse").is_some()
            || accessor.get("normalized").and_then(Value::as_bool) == Some(true)
            || required_string(accessor.get("type"), "accessor type")? != expected_type
        {
            return invalid("semantic proof rejects sparse/normalized or mistyped accessors");
        }
        let view_index = required_u32(accessor.get("bufferView"), "accessor bufferView")?;
        let views = required_array(self.document, "bufferViews", "GLB bufferViews")?;
        let view = indexed(views, view_index, "bufferView")?;
        if optional_u32(view.get("buffer"))?.unwrap_or(0) != 0 {
            return invalid("semantic proof accessor references a non-BIN buffer");
        }
        let view_offset = optional_usize(view.get("byteOffset"))?.unwrap_or(0);
        let view_length = required_usize(view.get("byteLength"), "bufferView byteLength")?;
        let view_end = checked_add(view_offset, view_length, "bufferView end")?;
        if view_end > self.binary.len() {
            return invalid("semantic proof bufferView exceeds BIN chunk");
        }
        let component_type = required_u32(accessor.get("componentType"), "componentType")?;
        let component_count = match expected_type {
            "SCALAR" => 1,
            "VEC2" => 2,
            "VEC3" => 3,
            "VEC4" => 4,
            "MAT4" => 16,
            _ => return invalid("semantic proof requested an unknown accessor type"),
        };
        let count = required_usize(accessor.get("count"), "accessor count")?;
        let byte_stride = optional_usize(view.get("byteStride"))?;
        let component_size = component_size(component_type)?;
        let element_size = component_size
            .checked_mul(component_count)
            .ok_or(ModelError::Overflow("accessor element size"))?;
        let stride = byte_stride.unwrap_or(element_size);
        if stride < element_size || stride % component_size != 0 {
            return invalid("semantic proof accessor byteStride is invalid");
        }
        let accessor_offset = optional_usize(accessor.get("byteOffset"))?.unwrap_or(0);
        let occupied = if count == 0 {
            accessor_offset
        } else {
            checked_add(
                accessor_offset,
                checked_add(
                    (count - 1)
                        .checked_mul(stride)
                        .ok_or(ModelError::Overflow("accessor stride"))?,
                    element_size,
                    "accessor final element",
                )?,
                "accessor occupied bytes",
            )?
        };
        if occupied > view_length {
            return invalid("semantic proof accessor exceeds its bufferView");
        }
        Ok(AccessorInfo {
            view_offset,
            accessor_offset,
            component_type,
            component_count,
            count,
            stride,
        })
    }

    pub(super) fn read_f32(&self, index: u32, kind: &str) -> Result<Vec<Vec<f32>>> {
        let info = self.info(index, kind)?;
        if info.component_type != 5_126 {
            return invalid("semantic proof expected FLOAT accessor data");
        }
        let mut rows = Vec::with_capacity(info.count);
        for element in 0..info.count {
            let mut row = Vec::with_capacity(info.component_count);
            for component in 0..info.component_count {
                let offset = info.component_offset(element, component, 4)?;
                row.push(f32::from_le_bytes(read_array::<4>(self.binary, offset)?));
            }
            rows.push(row);
        }
        Ok(rows)
    }

    pub(super) fn read_joints(&self, index: u32) -> Result<Vec<[u16; 4]>> {
        let info = self.info(index, "VEC4")?;
        if info.component_type != 5_123 {
            return invalid("semantic proof expected UNSIGNED_SHORT JOINTS_0");
        }
        let mut values = Vec::with_capacity(info.count);
        for element in 0..info.count {
            let mut row = [0; 4];
            for (component, value) in row.iter_mut().enumerate() {
                let offset = info.component_offset(element, component, 2)?;
                *value = u16::from_le_bytes(read_array::<2>(self.binary, offset)?);
            }
            values.push(row);
        }
        Ok(values)
    }

    pub(super) fn read_indices(&self, index: u32) -> Result<Vec<u32>> {
        let info = self.info(index, "SCALAR")?;
        if !matches!(info.component_type, 5_123 | 5_125) {
            return invalid("semantic proof indices must be UNSIGNED_SHORT or UNSIGNED_INT");
        }
        let size = component_size(info.component_type)?;
        let mut values = Vec::with_capacity(info.count);
        for element in 0..info.count {
            let offset = info.component_offset(element, 0, size)?;
            values.push(if size == 2 {
                u32::from(u16::from_le_bytes(read_array::<2>(self.binary, offset)?))
            } else {
                u32::from_le_bytes(read_array::<4>(self.binary, offset)?)
            });
        }
        Ok(values)
    }
}

#[derive(Clone, Copy)]
pub(super) struct AccessorInfo {
    pub(super) view_offset: usize,
    pub(super) accessor_offset: usize,
    pub(super) component_type: u32,
    pub(super) component_count: usize,
    pub(super) count: usize,
    pub(super) stride: usize,
}

impl AccessorInfo {
    pub(super) fn component_offset(self, element: usize, component: usize, size: usize) -> Result<usize> {
        checked_add(
            checked_add(
                checked_add(
                    self.view_offset,
                    self.accessor_offset,
                    "accessor base offset",
                )?,
                element
                    .checked_mul(self.stride)
                    .ok_or(ModelError::Overflow("accessor element offset"))?,
                "accessor element offset",
            )?,
            component
                .checked_mul(size)
                .ok_or(ModelError::Overflow("accessor component offset"))?,
            "accessor component offset",
        )
    }
}
