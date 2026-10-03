
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NanocomTextureEvidence {
    pub source_container: &'static str,
    pub source_path_id: i64,
    pub source_name: &'static str,
    pub width: u32,
    pub height: u32,
    pub runtime_path: &'static str,
    pub runtime_bytes: u64,
    pub runtime_sha256: &'static str,
}
