use std::{fs, process::Command};

use tempfile::TempDir;

const REGISTRY: &[u8] = br#"{"schema":"ffone.runtime-world.v1","entries":[]}"#;

#[test]
fn migrated_runtime_world_blocks_legacy_commands_before_they_write() {
    let temp = TempDir::new().unwrap();
    let asset_root = temp.path().join("assets/game");
    let registry_path = asset_root.join("_runtime/world.json");
    fs::create_dir_all(registry_path.parent().unwrap()).unwrap();
    fs::write(&registry_path, REGISTRY).unwrap();

    for command in [
        "plan-world",
        "publish-world",
        "publish-native-terrain-batch",
    ] {
        let output_path = temp.path().join(format!("{command}-output"));
        let missing_plan = temp.path().join("missing-plan.json");
        let mut invocation = Command::new(env!("CARGO_BIN_EXE_ffone-semantic-assets"));
        invocation
            .arg(command)
            .arg("--asset-root")
            .arg(&asset_root)
            .arg("--output")
            .arg(&output_path);
        if command == "publish-native-terrain-batch" {
            invocation.arg("--plan").arg(&missing_plan);
        }

        let result = invocation.output().unwrap();
        assert!(!result.status.success(), "{command} unexpectedly succeeded");
        let stderr = String::from_utf8(result.stderr).unwrap();
        assert!(stderr.contains(command), "unexpected stderr: {stderr}");
        assert!(
            stderr.contains("ffone.runtime-world.v1"),
            "unexpected stderr: {stderr}"
        );
        assert!(
            stderr.contains("registry-aware world workflow in FusionForge"),
            "unexpected stderr: {stderr}"
        );
        assert!(
            !output_path.exists(),
            "{command} wrote output before refusing the legacy workflow"
        );
        assert_eq!(fs::read(&registry_path).unwrap(), REGISTRY);
        assert_eq!(fs::read_dir(&asset_root).unwrap().count(), 1);
        assert_eq!(
            fs::read_dir(registry_path.parent().unwrap())
                .unwrap()
                .count(),
            1
        );
    }
}
