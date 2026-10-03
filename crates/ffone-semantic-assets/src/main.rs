use std::{
    env, fs,
    path::{Path, PathBuf},
};

use ffone_semantic_assets::{
    Result, RoutePlan, SemanticAssetError, audit_asset_tree, build_retro_world_plan,
    ensure_legacy_world_publication_allowed, publish_native_terrain_batch, publish_route_plan,
    reconcile_retro_world_native_terrain, validate_route_plan, write_json,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("semantic asset organizer failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut args = env::args_os().skip(1);
    let command = args
        .next()
        .and_then(|value| value.into_string().ok())
        .unwrap_or_else(|| "help".to_owned());
    let options = Options::parse(args)?;

    match command.as_str() {
        "audit" => {
            let audit = audit_asset_tree(&options.asset_root, &options.manifest)?;
            let output = options
                .output
                .unwrap_or_else(|| PathBuf::from("work/semantic-assets/semantic-tree-audit.json"));
            write_json(&output, &audit)?;
            println!(
                "audit: entries={} verified={} semantic={} hashed={} blocked={} report={}",
                audit.counts.manifest_entries,
                audit.counts.verified_entries,
                audit.counts.semantic_entries,
                audit.counts.hashed_filename_entries,
                audit.counts.blocked_entries,
                output.display()
            );
        }
        "plan-world" => {
            ensure_legacy_world_publication_allowed(&options.asset_root, "plan-world")?;
            let plan = build_retro_world_plan(&options.asset_root)?;
            let proofs = validate_route_plan(&options.asset_root, &options.manifest, &plan)?;
            let output = options
                .output
                .unwrap_or_else(|| PathBuf::from("work/semantic-assets/world-route-plan.json"));
            write_json(&output, &plan)?;
            println!(
                "world plan: routes={} glb_proofs={} report={}",
                plan.routes.len(),
                proofs.len(),
                output.display()
            );
        }
        "publish-world" => {
            ensure_legacy_world_publication_allowed(&options.asset_root, "publish-world")?;
            let output_root = options
                .output
                .unwrap_or_else(|| PathBuf::from("work/semantic-assets"));
            let pre_audit = audit_asset_tree(&options.asset_root, &options.manifest)?;
            write_json(
                &output_root.join("semantic-tree-prepublication-audit.json"),
                &pre_audit,
            )?;
            let plan = build_retro_world_plan(&options.asset_root)?;
            write_json(&output_root.join("world-route-plan.json"), &plan)?;
            let report = publish_route_plan(&options.asset_root, &options.manifest, &plan)?;
            write_json(&output_root.join("world-publication-report.json"), &report)?;
            let final_audit = audit_asset_tree(&options.asset_root, &options.manifest)?;
            write_json(
                &output_root.join("semantic-tree-final-audit.json"),
                &final_audit,
            )?;
            println!(
                "world publication: status={} committed={} reused={} manifest_verified={}/{} report={}",
                report.status,
                report.committed_files.len(),
                report.reused_identical_files.len(),
                final_audit.counts.verified_entries,
                final_audit.counts.manifest_entries,
                output_root.display()
            );
        }
        "validate-plan" => {
            let publication_plan = options.publication_plan.ok_or_else(|| {
                SemanticAssetError::InvalidPlan("validate-plan requires --plan PATH".to_owned())
            })?;
            let plan = read_route_plan(&publication_plan)?;
            let proofs = validate_route_plan(&options.asset_root, &options.manifest, &plan)?;
            println!(
                "semantic plan valid: routes={} glb_proofs={} plan={}",
                plan.routes.len(),
                proofs.len(),
                publication_plan.display()
            );
        }
        "publish-plan" => {
            let publication_plan = options.publication_plan.ok_or_else(|| {
                SemanticAssetError::InvalidPlan("publish-plan requires --plan PATH".to_owned())
            })?;
            let output_root = options
                .output
                .unwrap_or_else(|| PathBuf::from("work/semantic-assets/generic-publication"));
            let pre_audit = audit_asset_tree(&options.asset_root, &options.manifest)?;
            write_json(
                &output_root.join("semantic-tree-prepublication-audit.json"),
                &pre_audit,
            )?;
            let plan = read_route_plan(&publication_plan)?;
            validate_route_plan(&options.asset_root, &options.manifest, &plan)?;
            let report = publish_route_plan(&options.asset_root, &options.manifest, &plan)?;
            write_json(&output_root.join("publication-report.json"), &report)?;
            let final_audit = audit_asset_tree(&options.asset_root, &options.manifest)?;
            write_json(
                &output_root.join("semantic-tree-final-audit.json"),
                &final_audit,
            )?;
            println!(
                "semantic publication: status={} routes={} committed={} reused={} manifest_verified={}/{} report={}",
                report.status,
                plan.routes.len(),
                report.committed_files.len(),
                report.reused_identical_files.len(),
                final_audit.counts.verified_entries,
                final_audit.counts.manifest_entries,
                output_root.display()
            );
        }
        "reconcile-native-terrain" => {
            let output_root = options
                .output
                .unwrap_or_else(|| PathBuf::from("work/semantic-assets"));
            let report =
                reconcile_retro_world_native_terrain(&options.asset_root, &options.manifest)?;
            write_json(
                &output_root.join("native-terrain-reconciliation-report.json"),
                &report,
            )?;
            let final_audit = audit_asset_tree(&options.asset_root, &options.manifest)?;
            write_json(
                &output_root.join("semantic-tree-final-audit.json"),
                &final_audit,
            )?;
            println!(
                "native terrain reconciliation: status={} published={} removed_manifest={} removed_files={} manifest_verified={}/{} report={}",
                report.status,
                report.published_files.len(),
                report.removed_manifest_paths.len(),
                report.removed_legacy_runtime_files.len(),
                final_audit.counts.verified_entries,
                final_audit.counts.manifest_entries,
                output_root.display()
            );
        }
        "publish-native-terrain-batch" => {
            ensure_legacy_world_publication_allowed(
                &options.asset_root,
                "publish-native-terrain-batch",
            )?;
            let publication_plan = options.publication_plan.ok_or_else(|| {
                ffone_semantic_assets::SemanticAssetError::InvalidPlan(
                    "publish-native-terrain-batch requires --plan PATH".to_owned(),
                )
            })?;
            let output_root = options
                .output
                .unwrap_or_else(|| PathBuf::from("work/semantic-assets"));
            let report = publish_native_terrain_batch(
                &options.asset_root,
                &options.manifest,
                &publication_plan,
            )?;
            write_json(
                &output_root.join("native-terrain-batch-publication-report.json"),
                &report,
            )?;
            let final_audit = audit_asset_tree(&options.asset_root, &options.manifest)?;
            write_json(
                &output_root.join("semantic-tree-final-audit.json"),
                &final_audit,
            )?;
            println!(
                "native terrain batch: status={} entries={} files={} blocked_placement_data={} manifest_verified={}/{} report={}",
                report.status,
                report.entries_published,
                report.published_files.len(),
                report.blocked_placements_retained_as_data,
                final_audit.counts.verified_entries,
                final_audit.counts.manifest_entries,
                output_root.display()
            );
        }
        _ => {
            println!(
                "Usage:\n  cargo run -p ffone-semantic-assets -- audit [--asset-root PATH] [--manifest PATH] [--output FILE]\n  cargo run -p ffone-semantic-assets -- plan-world [options]\n  cargo run -p ffone-semantic-assets -- publish-world [--output DIRECTORY]\n  cargo run -p ffone-semantic-assets -- validate-plan --plan FILE\n  cargo run -p ffone-semantic-assets -- publish-plan --plan FILE [--output DIRECTORY]\n  cargo run -p ffone-semantic-assets -- reconcile-native-terrain [--output DIRECTORY]\n  cargo run -p ffone-semantic-assets -- publish-native-terrain-batch --plan FILE [--output DIRECTORY]\n\nDefaults assume the command is run from FFOneClient."
            );
        }
    }
    Ok(())
}

fn read_route_plan(path: &Path) -> Result<RoutePlan> {
    let bytes = fs::read(path).map_err(|source| SemanticAssetError::Io {
        path: path.to_owned(),
        source,
    })?;
    serde_json::from_slice(&bytes).map_err(|source| SemanticAssetError::Json {
        path: path.to_owned(),
        source,
    })
}

struct Options {
    asset_root: PathBuf,
    manifest: PathBuf,
    output: Option<PathBuf>,
    publication_plan: Option<PathBuf>,
}

impl Options {
    fn parse(args: impl Iterator<Item = std::ffi::OsString>) -> Result<Self> {
        let mut asset_root = PathBuf::from("assets/game");
        let mut manifest = None;
        let mut output = None;
        let mut publication_plan = None;
        let mut args = args.peekable();
        while let Some(flag) = args.next() {
            let value = args.next().ok_or_else(|| {
                ffone_semantic_assets::SemanticAssetError::InvalidPlan(format!(
                    "missing value for {}",
                    flag.to_string_lossy()
                ))
            })?;
            match flag.to_string_lossy().as_ref() {
                "--asset-root" => asset_root = value.into(),
                "--manifest" => manifest = Some(value.into()),
                "--output" => output = Some(value.into()),
                "--plan" => publication_plan = Some(value.into()),
                unknown => {
                    return Err(ffone_semantic_assets::SemanticAssetError::InvalidPlan(
                        format!("unknown option {unknown}"),
                    ));
                }
            }
        }
        let manifest = manifest.unwrap_or_else(|| asset_root.join("asset-manifest.json"));
        ensure_manifest_under_asset_root(&asset_root, &manifest)?;
        Ok(Self {
            asset_root,
            manifest,
            output,
            publication_plan,
        })
    }
}

fn ensure_manifest_under_asset_root(asset_root: &Path, manifest: &Path) -> Result<()> {
    if manifest.file_name().is_none() || asset_root.as_os_str().is_empty() {
        return Err(ffone_semantic_assets::SemanticAssetError::InvalidPlan(
            "asset root and manifest must be concrete paths".to_owned(),
        ));
    }
    Ok(())
}
