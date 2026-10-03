//! Exhaustive in-runtime GPU audit for every wearable avatar item.
//!
//! Each gender-valid Shirt/Pants/Shoes/Hat/Glasses/Back row is resolved
//! through `CharacterCreationData`, presented by the production
//! `NativePlayerPreview`, rendered for several frames, and captured. The JSON
//! report proves which table texture actually reached each visible material;
//! a PNG merely existing on disk is not sufficient for acceptance.

use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use bevy::{
    asset::AssetPlugin,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::{PresentMode, WindowResolution},
};
use ffone_client::{
    character_creation_data::{CharacterCreationData, ResolvedCreatorSelection},
    character_creation_ui::{CharacterAppearance, CharacterGender as UiGender},
    legacy_model_material::LegacyModelMaterialPlugin,
    player_appearance_material::ActorSkinTextureRole,
    player_preview::{
        NativePlayerLook, NativePlayerPartKind, NativePlayerPreviewAudit, NativePlayerPreviewModel,
        NativePlayerPreviewPlugin, NativePlayerPreviewStage, NativePlayerPreviewStatus,
    },
    player_shared_rig::{
        NativePlayerRigAssetCache, NativePlayerRigCatalog, NativePlayerSharedRigPlugin,
    },
};
use ffone_protocol::{CharacterEquipSlot0104, ItemBase0104, Nano0104, PcAppearance0104};
use ffone_runtime_contracts::{AvatarItemCategory, NativeLookupStatus, PlayerRigGender};
use serde::Serialize;

fn main() {
    let cli = Cli::parse().unwrap_or_else(|error| {
        eprintln!("{error}");
        std::process::exit(2);
    });
    let data = Arc::new(
        CharacterCreationData::open(&cli.asset_root)
            .expect("open production character-creation data"),
    );
    let male = data
        .resolve_creator(1, 1, "Audit", "Male", &CharacterAppearance::default())
        .expect("resolve male audit base");
    let female = data
        .resolve_creator(
            2,
            1,
            "Audit",
            "Female",
            &CharacterAppearance {
                gender: UiGender::Girl,
                ..CharacterAppearance::default()
            },
        )
        .expect("resolve female audit base");
    let jobs = build_jobs(&data, &cli);
    println!(
        "avatar equipment GPU audit queued {} gendered renders",
        jobs.len()
    );
    let catalog = NativePlayerRigCatalog::open(&cli.asset_root)
        .expect("open production native player-rig catalog");

    App::new()
        .insert_resource(ClearColor(Color::srgb(0.06, 0.08, 0.11)))
        .insert_resource(AuditData { data, male, female })
        .insert_resource(AuditState {
            cli: cli.clone(),
            jobs,
            cursor: 0,
            phase: Phase::Start,
            results: Vec::new(),
            capture_complete: None,
            transient_retries: 0,
            started: Instant::now(),
        })
        .insert_resource(catalog)
        .add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: cli.asset_root.to_string_lossy().into_owned(),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "FFOne exhaustive avatar equipment GPU audit".into(),
                        resolution: WindowResolution::new(384, 512),
                        present_mode: PresentMode::AutoNoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((
            LegacyModelMaterialPlugin,
            NativePlayerSharedRigPlugin,
            NativePlayerPreviewPlugin,
        ))
        .add_systems(
            Update,
            drive_audit.after(ffone_client::player_preview::NativePlayerPreviewSet::Rebuild),
        )
        .run();
}

mod constants;
mod materials;
mod types;
mod state;
mod validation;
mod operations;
mod output;
mod input;
mod systems;
mod textures;

use constants::{
    REPORT_SCHEMA, MIN_CAPTURE_VISIBLE_PIXELS, MAX_TRANSIENT_RETRIES, ITEM_TIMEOUT,
    REPORT_CHECKPOINT_INTERVAL
};
use materials::{READY_RENDER_FRAMES, FIRST_CAPTURE_RENDER_FRAMES};
use types::{
    GenderFilter, Cli, Job, SurfaceEvidence, ItemResult, Phase, CaptureOutcome,
    PendingItemCapture
};
use state::{ScreenshotMode, status_label};
use validation::{AuditGender, AuditData, AuditState, drive_audit};
use operations::{
    build_jobs, blocked_result, resolution_failure, item_result, finish_result, is_wearable,
    gender_allowed, category_slot, category_label, role_label
};
use output::{save_item_capture, write_report};
use input::{resolve_job_look, parse_category};
use systems::evaluate_ready_job;
use textures::{meaningful_source_texture, source_texture_matches};
