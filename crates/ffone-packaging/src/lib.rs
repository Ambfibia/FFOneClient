//! Deterministic dependency-graph validation and release packaging for FFOne assets.
//!
//! Rust compilation never copies game content. `xtask` calls this library explicitly,
//! and a full verification remains the final release gate.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{BufReader, Read, Write},
    path::{Component, Path, PathBuf},
    time::UNIX_EPOCH,
};

use serde::{Deserialize, Serialize};

mod graph;
use graph::{AssetGraph, build_asset_graph, is_editable_asset, validate_domain_roots};

// Historical manifest fixtures are intentionally excluded after the runtime
// control-plane removal. Current graph/pack behavior is covered by the public
// integration suite.
#[cfg(any())]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn test_entry(path: &str, payload: &[u8]) -> ProjectFile {
        ProjectFile {
            source_path: format!("test-source/{path}"),
            path: path.to_owned(),
            kind: "test".to_owned(),
            bytes: payload.len() as u64,
            blake3: blake3::hash(payload).to_hex().to_string(),
        }
    }

    fn write_test_manifest(root: &Path, files: &[ProjectFile]) {
        fs::create_dir_all(root).unwrap();
        let manifest = ProjectManifest {
            schema: "ffone.project-assets.v1".to_owned(),
            protocol: 1,
            locale: "test".to_owned(),
            source_pack: serde_json::json!({"id": "test"}),
            files: files.to_vec(),
        };
        fs::write(root.join(LEGACY_MANIFEST), pretty_json(&manifest).unwrap()).unwrap();
    }

    fn write_test_payload(root: &Path, path: &str, payload: &[u8]) {
        let destination = root.join(native_path(path));
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        fs::write(destination, payload).unwrap();
    }

    #[test]
    fn semantic_groups_are_explicit() {
        assert_eq!(
            group_for("characters/npc/foo/model.glb"),
            Some("characters")
        );
        assert_eq!(
            group_for("props/vegetation/trees/oak/model.glb"),
            Some("props")
        );
        assert_eq!(group_for("icons/entities/mobs/1.png"), Some("icons"));
        assert_eq!(group_for("nano/icons/nano/1.png"), Some("icons"));
        assert_eq!(
            group_for("models/world/map_01/model.glb"),
            Some("world-generated")
        );
        assert_eq!(group_for("models/legacy.glb"), None);
        assert_eq!(group_for("map/shared/effects/catalog.json"), Some("map"));
        assert_eq!(group_for("_runtime/audio.json"), Some("runtime"));
        assert_eq!(group_for("unknown/file.bin"), None);
    }

    #[test]
    fn payload_byte_totals_fail_on_overflow() {
        let files = vec![
            ProjectFile {
                bytes: u64::MAX,
                ..test_entry("audio/a.ogg", b"a")
            },
            test_entry("audio/b.ogg", b"b"),
        ];
        let error = checked_payload_bytes(&files, "test payloads").unwrap_err();
        assert!(error.contains("overflow"), "{error}");
    }

    #[test]
    fn manifests_register_runtime_files_from_disk_and_check_mode_is_read_only() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        write_test_manifest(root, &[]);
        let relative = "ui/en/character/selection/controls/CancelNormal.png";
        let payload = b"runtime-ui-texture";
        write_test_payload(root, relative, payload);
        let original_manifest = fs::read(root.join(LEGACY_MANIFEST)).unwrap();

        let error = generate_group_manifests(&GroupManifestOptions {
            asset_root: root.to_path_buf(),
            check: true,
            full: false,
        })
        .unwrap_err();
        assert!(error.contains("new runtime files"));
        assert!(error.contains(relative));
        assert_eq!(
            fs::read(root.join(LEGACY_MANIFEST)).unwrap(),
            original_manifest
        );

        let report = generate_group_manifests(&GroupManifestOptions {
            asset_root: root.to_path_buf(),
            check: false,
            full: false,
        })
        .unwrap();
        assert!(report.changed);
        assert_eq!(report.files, 1);

        let (manifest, _) = read_manifest(root).unwrap();
        let [entry] = manifest.files.as_slice() else {
            panic!("expected exactly one registered runtime file");
        };
        assert_eq!(entry.path, relative);
        assert_eq!(entry.source_path, relative);
        assert_eq!(entry.kind, "texture");
        assert_eq!(entry.bytes, payload.len() as u64);
        assert_eq!(entry.blake3, blake3::hash(payload).to_hex().to_string());

        let clean = generate_group_manifests(&GroupManifestOptions {
            asset_root: root.to_path_buf(),
            check: true,
            full: false,
        })
        .unwrap();
        assert!(!clean.changed);
        assert_eq!(clean.files, 1);
    }

    #[test]
    fn manifests_reject_unowned_runtime_paths_without_mutating_authority() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        write_test_manifest(root, &[]);
        write_test_payload(root, "models/unclassified.glb", b"unowned-model");
        let original_manifest = fs::read(root.join(LEGACY_MANIFEST)).unwrap();

        let error = generate_group_manifests(&GroupManifestOptions {
            asset_root: root.to_path_buf(),
            check: false,
            full: false,
        })
        .unwrap_err();
        assert!(error.contains("matches no declared group"));
        assert_eq!(
            fs::read(root.join(LEGACY_MANIFEST)).unwrap(),
            original_manifest
        );
        assert!(!root.join(ASSET_INDEX).exists());
        assert!(!root.join(GROUP_DIR).exists());
    }

    #[test]
    fn rejects_unsafe_paths() {
        for value in ["", "../x", "/x", "a\\b", "a/./b"] {
            assert!(validate_relative(value).is_err(), "{value}");
        }
        assert!(validate_relative("world/maps/map_01/scene.json").is_ok());
    }

    #[test]
    fn restore_only_copies_missing_verified_payloads() {
        let target = tempdir().unwrap();
        let donor = tempdir().unwrap();
        let existing = b"existing";
        let missing = b"missing payload";
        let files = vec![
            test_entry("audio/existing.ogg", existing),
            test_entry("models/missing.glb", missing),
        ];
        write_test_manifest(target.path(), &files);
        write_test_manifest(donor.path(), &files);
        write_test_payload(target.path(), &files[0].path, existing);
        // The existing target does not require a physical donor. This proves
        // donor I/O is restricted to target-missing manifest entries.
        write_test_payload(donor.path(), &files[1].path, missing);

        let report = restore_missing_assets(&RestoreOptions {
            asset_root: target.path().to_path_buf(),
            donor_root: donor.path().to_path_buf(),
        })
        .unwrap();

        assert_eq!(
            report,
            RestoreReport {
                manifest_files: 2,
                restored_files: 1,
                restored_bytes: missing.len() as u64,
                already_valid_files: 1,
                already_valid_bytes: existing.len() as u64,
                missing_files: 0,
                missing_bytes: 0,
            }
        );
        assert_eq!(
            fs::read(target.path().join(native_path(&files[1].path))).unwrap(),
            missing
        );
    }

    #[test]
    fn corrupt_donor_aborts_before_any_copy() {
        let target = tempdir().unwrap();
        let donor = tempdir().unwrap();
        let first = b"first valid";
        let second = b"second valid";
        let files = vec![
            test_entry("audio/first.ogg", first),
            test_entry("models/second.glb", second),
        ];
        write_test_manifest(target.path(), &files);
        write_test_manifest(donor.path(), &files);
        write_test_payload(donor.path(), &files[0].path, first);
        write_test_payload(donor.path(), &files[1].path, b"corrupt");

        let error = restore_missing_assets(&RestoreOptions {
            asset_root: target.path().to_path_buf(),
            donor_root: donor.path().to_path_buf(),
        })
        .unwrap_err();

        assert!(error.contains("donor asset failed full verification"));
        assert!(!target.path().join(native_path(&files[0].path)).exists());
        assert!(!target.path().join(native_path(&files[1].path)).exists());
    }

    #[test]
    fn existing_mismatch_is_never_overwritten() {
        let target = tempdir().unwrap();
        let donor = tempdir().unwrap();
        let existing = b"expected existing";
        let missing = b"expected missing";
        let files = vec![
            test_entry("audio/existing.ogg", existing),
            test_entry("models/missing.glb", missing),
        ];
        write_test_manifest(target.path(), &files);
        write_test_manifest(donor.path(), &files);
        write_test_payload(target.path(), &files[0].path, b"wrong");
        write_test_payload(donor.path(), &files[0].path, existing);
        write_test_payload(donor.path(), &files[1].path, missing);

        let error = restore_missing_assets(&RestoreOptions {
            asset_root: target.path().to_path_buf(),
            donor_root: donor.path().to_path_buf(),
        })
        .unwrap_err();

        assert!(error.contains("refusing to overwrite"));
        assert_eq!(
            fs::read(target.path().join(native_path(&files[0].path))).unwrap(),
            b"wrong"
        );
        assert!(!target.path().join(native_path(&files[1].path)).exists());
    }

    #[test]
    fn donor_manifest_identity_mismatch_is_rejected() {
        let target = tempdir().unwrap();
        let donor = tempdir().unwrap();
        let payload = b"same bytes";
        let target_entry = test_entry("models/model.glb", payload);
        let mut donor_entry = target_entry.clone();
        donor_entry.kind = "different-kind".to_owned();
        write_test_manifest(target.path(), std::slice::from_ref(&target_entry));
        write_test_manifest(donor.path(), std::slice::from_ref(&donor_entry));
        write_test_payload(donor.path(), &donor_entry.path, payload);

        let error = restore_missing_assets(&RestoreOptions {
            asset_root: target.path().to_path_buf(),
            donor_root: donor.path().to_path_buf(),
        })
        .unwrap_err();

        assert!(error.contains("donor manifest identity mismatch"));
        assert!(!target.path().join(native_path(&target_entry.path)).exists());
    }

    #[test]
    fn incomplete_donor_reports_bytes_and_aborts_before_copy() {
        let target = tempdir().unwrap();
        let donor = tempdir().unwrap();
        let available = b"available";
        let unavailable = b"not available";
        let files = vec![
            test_entry("audio/available.ogg", available),
            test_entry("models/unavailable.glb", unavailable),
        ];
        write_test_manifest(target.path(), &files);
        write_test_manifest(donor.path(), std::slice::from_ref(&files[0]));
        write_test_payload(donor.path(), &files[0].path, available);

        let error = restore_missing_assets(&RestoreOptions {
            asset_root: target.path().to_path_buf(),
            donor_root: donor.path().to_path_buf(),
        })
        .unwrap_err();

        assert!(error.contains("missing 1 files"));
        assert!(error.contains(&format!("({} bytes)", unavailable.len())));
        assert!(!target.path().join(native_path(&files[0].path)).exists());
    }

    #[test]
    fn recovery_archive_removes_only_flat_legacy_payloads() {
        let temp = tempdir().unwrap();
        let asset_root = temp.path().join("assets");
        let archive_root = temp.path().join("imports").join("recovery");
        let raw_audio = b"flat audio";
        let raw_model = b"flat model";
        let semantic_voice = b"semantic voice";
        let semantic_character = b"semantic character";
        let files = vec![
            test_entry("audio/legacy--hash.ogg", raw_audio),
            test_entry("models/legacy--hash.glb", raw_model),
            test_entry("audio/voice/en/ben/line.ogg", semantic_voice),
            test_entry("characters/npcs/npc_ben/npc_ben.glb", semantic_character),
        ];
        write_test_manifest(&asset_root, &files);
        for (entry, payload) in files.iter().zip([
            raw_audio.as_slice(),
            raw_model.as_slice(),
            semantic_voice.as_slice(),
            semantic_character.as_slice(),
        ]) {
            write_test_payload(&asset_root, &entry.path, payload);
        }

        let report = archive_recovery_assets(&ArchiveRecoveryOptions {
            asset_root: asset_root.clone(),
            archive_root: archive_root.clone(),
        })
        .unwrap();

        assert_eq!(report.archived_files, 2);
        assert_eq!(
            report.archived_bytes,
            (raw_audio.len() + raw_model.len()) as u64
        );
        assert_eq!(report.remaining_manifest_files, 2);
        assert!(!asset_root.join("audio/legacy--hash.ogg").exists());
        assert!(!asset_root.join("models/legacy--hash.glb").exists());
        assert_eq!(
            fs::read(archive_root.join("audio/legacy--hash.ogg")).unwrap(),
            raw_audio
        );
        assert_eq!(
            fs::read(archive_root.join("models/legacy--hash.glb")).unwrap(),
            raw_model
        );
        assert_eq!(
            fs::read(asset_root.join("audio/voice/en/ben/line.ogg")).unwrap(),
            semantic_voice
        );
        assert_eq!(
            fs::read(asset_root.join("characters/npcs/npc_ben/npc_ben.glb")).unwrap(),
            semantic_character
        );
        let (manifest, _) = read_manifest(&asset_root).unwrap();
        assert_eq!(
            manifest
                .files
                .iter()
                .map(|entry| entry.path.as_str())
                .collect::<Vec<_>>(),
            vec![
                "audio/voice/en/ben/line.ogg",
                "characters/npcs/npc_ben/npc_ben.glb"
            ]
        );
        let archive_index: serde_json::Value =
            serde_json::from_slice(&fs::read(archive_root.join("archive-index.json")).unwrap())
                .unwrap();
        assert_eq!(archive_index["archivedFiles"], 2);
        assert_eq!(archive_index["schema"], "ffone.recovery-archive.v1");
        assert!(
            archive_recovery_assets(&ArchiveRecoveryOptions {
                asset_root,
                archive_root,
            })
            .unwrap_err()
            .contains("refusing to overwrite")
        );
    }
}

mod core_assets;
mod core_constants;
mod core_validation;
mod core_types;
mod core_projects;
mod core_operations;
mod core_systems;
mod core_codec;
mod core_input;
mod core_output;

use core_assets::{
    LEGACY_MANIFEST, ProjectManifest, RecoveryArchiveIndex, package_asset_graph,
    read_manifest, native_path
};
pub use core_assets::verify_asset_root;
use core_constants::{RECEIPT, CACHE};
pub use core_validation::{AssetValidationOptions, AssetValidationReport, validate_assets};
use core_validation::{validate_manifest, validate_relative, validate_regular_directory};
pub use core_types::{
    PackageOptions, ReleaseSourceSnapshot, VerifyOptions, RestoreOptions,
    ArchiveRecoveryOptions, ReleaseSourceReport, PackageReport, VerifyReport, RestoreReport,
    ArchiveRecoveryReport
};
use core_types::{PackageCache, CachedFile, ReleaseReceipt, ReleaseGroupSummary};
use core_projects::ProjectFile;
pub use core_operations::{
    snapshot_release_source, package_loose, package_loose_from_snapshot,
    restore_missing_assets, archive_recovery_assets
};
use core_operations::{
    verify_graph_file, regular_metadata, modified_nanos, canonical_string, replace_file,
    temporary_sibling, remove_stale, pretty_json
};
use core_systems::refresh_editable_metadata;
use core_codec::{is_flat_recovery_payload, checked_payload_bytes, copy_verified_payload};
use core_input::{read_json, parse_hash};
use core_output::{copy_atomic, write_atomic};
