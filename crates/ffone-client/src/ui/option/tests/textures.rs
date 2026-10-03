use super::*;

#[test]
fn exact_semantic_texture_and_font_hashes_match_disk() {
    let root = asset_root();
    for contract in OPTION_TEXTURE_CONTRACTS {
        let path = root.join(contract.runtime_path);
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        assert_eq!(bytes.len() as u64, contract.bytes, "{}", path.display());
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            contract.sha256,
            "{}",
            path.display()
        );
    }
    for contract in OPTION_FONT_CONTRACTS {
        let path = root.join(contract.runtime_path);
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        assert_eq!(bytes.len() as u64, contract.bytes, "{}", path.display());
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            contract.sha256,
            "{}",
            path.display()
        );
    }
    for contract in OPTION_AUDIO_CONTRACTS {
        let path = root.join(contract.runtime_path);
        let bytes = fs::read(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        assert_eq!(bytes.len() as u64, contract.bytes, "{}", path.display());
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            contract.sha256,
            "{}",
            path.display()
        );
    }
}
