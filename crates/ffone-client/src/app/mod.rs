//! Native client application runtime and composition boundary.
//!
//! The binary entry point delegates here. Bevy application construction and
//! schedule wiring live in [`schedule`], while feature runtime systems remain
//! in this module until their owning plugins absorb them in later stages.
#[cfg(test)]
use ffone_client::movement::LegacyInputState;
#[cfg(test)]
use ffone_client::movement::MovementIntentQueue;
#[cfg(test)]
use ffone_client::user_equip_ui::UserEquipUiActionOutcome;
#[cfg(test)]
use ffone_client::user_equip_ui::apply_user_equip_ui_action;
#[cfg(test)]
use ffone_protocol::GM_SET_VALUE_SPEED_0104;
#[cfg(test)]
use ffone_protocol::ItemChestOpenSuccess0104;
#[cfg(test)]
use ffone_protocol::PcItemDeleteSuccess0104;
#[cfg(test)]
use race::RACE_INSTANCE_MAP_INFO_BASE_SIZE;
#[cfg(test)]
use runtime_status::RuntimePlayerStatus;
#[cfg(test)]
use runtime_status::apply_runtime_frame;
#[cfg(test)]
use world_nano::ordinary_world_nano_skill_supported;
#[cfg(test)]
use world_nano::requested_world_nano_slot;

mod bank_delete;
mod barber;
mod bootstrap;
mod character_flow;
mod gameplay_event_audio;
mod gameplay_ui_actions;
mod guide_nanocom;
mod hotkeys;
mod gamepad;
mod gamepad_ui;
mod gamepad_probe;
use gamepad::{GamepadActionState, sample_gamepad_actions};
mod mission_dialogue;
mod mission_escort;
mod movement_buffs;
mod nano_recall;
mod network_ingress;
mod npc_skill_presentation;
mod performance_probe;
mod schedule;
mod service_portrait;
mod session_lifecycle;
mod shared_redeem;
mod shiny_pickup;
mod social_ingress;
mod state;
#[cfg(test)]
mod tests;
mod transportation_portrait;
mod tutorial_choreography;
mod tutorial_presentation;
mod tutorial_runtime;
mod tutorial_warp;
mod user_settings;
mod vehicle_transition;
mod vendor_chest;
mod warp_presentation;
mod window_render_sync;
mod world_equipment;
mod world_skill_effects;

use character_flow::*;
use gameplay_event_audio::gameplay_event_audio;
use gameplay_ui_actions::*;
use hotkeys::*;
use social_ingress::*;
use tutorial_choreography::*;
use tutorial_presentation::*;
use tutorial_runtime::*;
use tutorial_warp::*;
use user_settings::*;
use warp_presentation::*;
use world_equipment::*;

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    env, fs,
    path::{Component, Path, PathBuf},
    process::{Command, ExitCode},
    sync::Arc,
    thread,
};

use bevy::{
    asset::{Asset, AssetApp, AssetPath, AssetPlugin, embedded_path, io::AssetSourceBuilder},
    audio::{AudioSink, AudioSinkPlayback},
    camera::visibility::RenderLayers,
    ecs::system::{NonSendMarker, SystemParam},
    input::InputSystems,
    prelude::*,
    reflect::TypePath,
    render::{
        Render, RenderApp, RenderPlugin, RenderSystems,
        camera::ExtractedCamera,
        render_resource::{AsBindGroup, ShaderType, TextureFormat},
        settings::{Backends, InstanceFlags, RenderCreation, WgpuSettings},
        view::{ViewTarget, create_surfaces},
    },
    shader::ShaderRef,
    window::{
        CursorGrabMode, CursorOptions, MonitorSelection, PresentMode, PrimaryWindow, WindowMode,
        WindowPosition, WindowResolution,
    },
};
#[cfg(windows)]
use bevy_winit::WINIT_WINDOWS;
#[cfg(test)]
use ffone_client::avatar_action::{
    LegacyAvatarActionInput, LegacyFocusedTarget, LegacyTargetSelection,
};
#[cfg(test)]
use ffone_client::race_ui::{mode::RaceRequestIntent, rank::RaceRankPhase};
#[cfg(test)]
use ffone_client::skill_buff_ui::SkillBuffTargetUi;
#[cfg(test)]
use ffone_client::tutorial_nano_gameplay::{
    TUTORIAL_BUTTERCUP_NANO_ID, TUTORIAL_BUTTERCUP_SKILL_ID,
};
#[cfg(test)]
use ffone_client::upsell_ui::UpsellUiMode;
#[cfg(test)]
use ffone_client::user_equip_ui::UserEquipCloseSource;
#[cfg(test)]
use ffone_client::vendor_ui::VendorSystemMessageId0104;
use ffone_client::{
    assets::AssetLocator,
    avatar_action::{
        LegacyAvatarActionContext, LegacyAvatarActionPlugin, LegacyAvatarActionSet,
        LegacyAvatarActionState, LegacyAvatarTargetFeed, LegacyMoveMode, LegacyTargetKind,
        LegacyVehiclePresentationFamily, LegacyWeaponTargetMode, select_legacy_targets,
    },
    bank_runtime::BankProductionRuntime0104,
    bank_ui::{
        BankLifecyclePhase, BankModalState, BankModeProjection0104, BankUiOutbox0104, BankUiPlugin,
        BankUiSet, BankUiState,
    },
    buddy_ui::{
        BUDDY_MAX_SLOTS, BuddyChatWindowStyle, BuddyConfirmation, BuddyEntry, BuddyInvite,
        BuddyInviteDisposition, BuddyPresence, BuddyStateUpdate, BuddyTarget, BuddyUiAction,
        BuddyUiModel, BuddyUiNotice, BuddyUiOutbox, BuddyUiPlugin, BuddyUiSet,
    },
    cashmall_ui::{
        CashmallLifecyclePhase0104, CashmallOpenSource0104, CashmallUiOutbox0104,
        CashmallUiPlugin0104, CashmallUiSet0104, CashmallUiState0104,
    },
    character_creation_data::{
        CharacterCreationData, CharacterCreationDataResource, ResolvedCreatorSelection,
    },
    character_creation_ui::{
        AppearanceField, CHARACTER_CREATION_PREVIEW_MAX_DISTANCE,
        CHARACTER_CREATION_PREVIEW_MIN_DISTANCE, CharacterAppearance,
        CharacterCreationCameraAction, CharacterCreationCapability, CharacterCreationPreviewStatus,
        CharacterCreationScreen, CharacterCreationUiAction, CharacterCreationUiModel,
        CharacterCreationUiOutbox, CharacterCreationUiSet, CharacterGender, CustomCharacterName,
        GeneratedCharacterName, NativeCharacterCreationUiPlugin,
    },
    character_scene::LegacyCharacterSceneStatus,
    character_selection_portraits::{
        CharacterSelectionPortraitStatus, CharacterSelectionPortraitsModel,
        CharacterSelectionPortraitsSet, GameplayPlayerPortraitModel,
        NativeCharacterSelectionPortraitsPlugin,
    },
    character_selection_ui::{
        CharacterLocationBackground, CharacterPreviewStatus, CharacterSelectionCapability,
        CharacterSelectionUiAction, CharacterSelectionUiModel, CharacterSelectionUiOutbox,
        CharacterSelectionUiSet, CharacterSlotUi, NativeCharacterSelectionUiPlugin,
        OccupiedCharacterSlotUi, resolve_character_selection_location,
    },
    combi_runtime::{CombiOpenContext0104, CombiProductionRuntime0104},
    combi_ui::{
        CombiModeProjection0104, CombiPhase0104, CombiUiOutbox0104, CombiUiPlugin, CombiUiSet0104,
        CombiUiState0104,
    },
    coordinates::{
        LegacyUnityHeadingDegrees, ProtocolPosition, ProtocolYawDegrees,
        native_model_forward_child_rotation, native_to_unity_vector, unity_to_native_vector,
    },
    email_runtime::{
        EmailItemFeaturePolicy0104, EmailOpenContext0104, EmailProductionRuntime0104,
        email_buddies_from_buddy_ui_0104, email_inventory_authority_0104,
    },
    email_ui::{
        EmailNetworkInbox0104, EmailNetworkRuntime0104, EmailTransportOutbox, EmailUiAudioCue,
        EmailUiAudioOutbox,
        EmailUiModel, EmailUiOutbox, EmailUiPlugin, EmailUiSet,
    },
    enchant_runtime::{EnchantOpenContext0104, EnchantProductionRuntime0104},
    enchant_ui::{
        EnchantModeProjection0104, EnchantPhase0104, EnchantUiOutbox0104, EnchantUiPlugin0104,
        EnchantUiSet0104,
    },
    entity_lifecycle::{
        NetworkEntityLifecycle0104Plugin, NetworkEntityLifecycleIngress0104, NetworkNpc0104,
        NetworkNpcAppearance0104, NetworkPcAppearance0104, NetworkRemotePc0104,
        NetworkSessionEpoch0104, consume_network_entity_lifecycle_0104,
    },
    game_guide_ui::{GameGuideUiModel, GameGuideUiPlugin},
    gameplay_audio::{
        GameplayAudioPlugin, GameplayAudioRuntime, GameplayAudioSet, LegacyNpcVoiceCue,
    },
    gameplay_nano_portraits::{GameplayNanoPortraitCatalog, GameplayNanoPortraitPlugin},
    gameplay_ui::{
        ChatChannel, ChatLineKind, ChatLineUi, CurrentObjectiveProgressUi, CurrentObjectiveUi,
        GameplayHud, GameplayUiAction, GameplayUiAudioOutbox, GameplayUiModel, GameplayUiOutbox,
        GameplayUiPlugin, GameplayUiSet, MinimapMarkerIcon, NanoSlotUi, NanoWheelTransientUi,
        NpcBarkerBubbleRuntime, NpcServiceKind, PlayerFreeChatBubbleRuntime, PlayerStatusUi,
        QuickChatItem, QuickChatMenuMode, gameplay_ui_scale, minimap_marker, minimap_marker_alpha,
        minimap_marker_sized, minimap_tiles, minimap_waypoint, normal_world_minimap_marker_style,
    },
    group_runtime::{GroupLeaveOwner0104, GroupOwnerReachability0104, GroupProductionRuntime0104},
    group_ui::{GroupUiModel, GroupUiPlugin},
    guide_runtime::{
        GuideInitIntent, GuideNpcServiceRoute, GuidePostChangeIntent, GuideRuntime,
        GuideServerProfile, guide_npc_service_route,
    },
    guide_ui::{
        GUIDE_COMPUTRESS_ICON_PATH, GuideUiAudioOutbox, GuideUiModel, GuideUiOutbox, GuideUiPlugin,
        GuideUiSet,
    },
    inventory_runtime::InventoryRuntime0104,
    launcher_ui::{LauncherUiModel, LauncherUiPlugin, LauncherUiSet},
    legacy_environment::{LegacyAvatarEnvironmentState, LegacyEnvironmentSet},
    legacy_glow::LegacyGlowPlugin,
    legacy_model_material::{
        LegacyMaterialApplied, LegacyModelMaterial, LegacyModelMaterialPlugin,
    },
    legacy_world_location::{legacy_world_location_name, legacy_world_location_name_or_last},
    localization::{
        DEFAULT_LANGUAGE, Language, Localization, LocalizationPlugin, LocalizationSet,
        LocalizedText, LocalizedVoice, VoiceLanguage, localized_tutorial_instruction,
        localized_tutorial_literal, localized_tutorial_scene_text,
    },
    login_ui::{LoginUiSet, NativeLoginUiPlugin},
    mission_ui::{
        MissionJournalUi, MissionUiModel, MissionUiPlugin, MissionUiSet, NpcInteractionUi,
        PendingMissionUiRequest,
    },
    movement::{
        LegacyCameraKeyInput, LegacyMovementPlugin, LegacyMovementSet, LegacyOrbitCamera, LegacyPlayerController,
        LegacyWorldColliderPending,
    },
    nano_free_tuning_runtime::{NanoFreeTuningBank0104, project_correlated_nano_tune_reply},
    nano_free_tuning_ui::{
        NanoFreeTuningModel, NanoFreeTuningPresentationSet, NanoFreeTuningUiCommandOutbox,
        NanoFreeTuningUiPlugin,
    },
    nanocom_message_ui::{
        NANOCOM_NUMBUH_TWO_ICON_PATH, NanocomMessageKind, NanocomMessageResolution,
        NanocomMessageUiModel, NanocomMessageUiOutbox, NanocomMessageUiPlugin, NanocomMessageUiSet,
    },
    network::{
        CharacterEntryLocation0104, CharacterOperationStage, CharacterSummary, NanoTunePending0104,
        NetworkBridge, NetworkCommand, NetworkEvent, WorldReady,
    },
    network_world_runtime::{
        NetworkHnpcVisual0104, NetworkNpcTextureVariantBound0104, NetworkNpcVisual0104,
        NpcSceneTextureOverrides0104, bind_network_npc_texture_variants_0104,
    },
    option_ui::{
        InputSettings, LegacyInputBinding, LegacyOptionAction, LegacyPhysicalKey,
        OptionOpenAudioRoute, OptionUiAssetGate, OptionUiModel, OptionUiOutbox, OptionUiPlugin,
        OptionUiSet, legacy_physical_key,
    },
    overheat_ui::OverheatUiPlugin,
    pc2pc_ui::{Pc2pcOfferRuntime0104, Pc2pcPendingOffer0104, Pc2pcUiModel0104, Pc2pcUiPlugin},
    player_preview::{
        NATIVE_PLAYER_CREATION_CAMERA_DISTANCE, NATIVE_PLAYER_INVENTORY_CAMERA_DISTANCE,
        NATIVE_PLAYER_SELECTION_CAMERA_DISTANCE, NativePlayerPreviewModel,
        NativePlayerPreviewPlugin, NativePlayerPreviewSet, NativePlayerPreviewStage,
        NativePlayerPreviewStatus, prewarm_native_player_look,
    },
    player_shared_rig::{
        NativePlayerRigAssetCache, NativePlayerRigCatalog, NativePlayerSharedRigPlugin,
    },
    quick_slot_ui::{QuickSlotUiModel, QuickSlotUiPlugin, QuickSlotUiSet},
    quit_menu_runtime::{
        QuitMenuExitReply, QuitMenuRuntime, clean_pc_exit_code_message, decode_quit_menu_exit_reply,
    },
    quit_menu_ui::{QuitMenuUiModel, QuitMenuUiOutbox, QuitMenuUiPlugin, QuitMenuUiSet},
    race_ui::{
        hud::{RaceHudPlugin, RaceHudSet},
        mode::{
            RaceEcomType, RaceModeModel, RaceModePresentationSet, RaceModeUiCommandOutbox,
            RaceModeUiPlugin, RaceRewardPresentation,
        },
        rank::{
            RaceRankCatalog, RaceRankModel, RaceRankPresentationSet, RaceRankUiCommandOutbox,
            RaceRankUiPlugin,
        },
    },
    remote::{RemoteAnimation, RemoteAnimationState, RemotePlayerPlugin, decode_remote_frame},
    resurrect_ui::{
        ResurrectUiContext, ResurrectUiModel, ResurrectUiOutbox, ResurrectUiPlugin, ResurrectUiSet,
    },
    rule_runtime::{RuleOpenRequest, RuleRuntime, rule_npc_service_route},
    rule_ui::{RuleUiModel, RuleUiOutbox, RuleUiPlugin, RuleUiSet},
    semantic_audio::NativeAudioCatalog,
    server_selection_ui::ServerSelectionUiPlugin,
    skill_buff_ui::{SkillBuffUiCatalog, SkillBuffUiModel, SkillBuffUiPlugin, SkillBuffUiSet},
    system_message_ui::{
        SystemMessageButtonType, SystemMessageChoice, SystemMessageRequest, SystemMessageUiAction,
        SystemMessageUiAudioOutbox, SystemMessageUiModel, SystemMessageUiOutbox,
        SystemMessageUiPlugin, SystemMessageUiSet,
    },
    transportation_ui::{
        TRANSPORTATION_FAILURE_PACKET_ID, TRANSPORTATION_REGISTRATION_FAILURE_PACKET_ID,
        TRANSPORTATION_REGISTRATION_SUCCESS_PACKET_ID, TRANSPORTATION_SUCCESS_PACKET_ID,
        TransportationCatalog, TransportationModel, TransportationOpenContext, TransportationPhase,
        TransportationPlayerSnapshot, TransportationPresentationSet,
        TransportationRegistrationReply0104, TransportationService, TransportationTarget,
        TransportationUiCommandOutbox, TransportationUiPlugin, TransportationWarpReply0104,
        TransportationWorldPoint, decode_transportation_registration_reply_0104,
        decode_transportation_warp_reply_0104,
    },
    tutorial::{
        CombatStage, InfectionStage, MinimapStage, MissionStage, MovementStage, NanoPowerStage,
        TutorialEvent, TutorialInputLock, TutorialScene, TutorialStage,
    },
    tutorial_actors::{
        TutorialActor, TutorialActorAnimationAssets, TutorialActorCombatConfig,
        TutorialActorCommandQueue, TutorialActorEventQueue, TutorialActorIssueQueue,
        TutorialActorPoseState, TutorialActorRegistry, TutorialActorSet,
        TutorialActorStandRandomStream, TutorialNpcObservationSnapshot, TutorialTargetingProfile,
        advance_tutorial_actor_combat_animation, advance_tutorial_actor_motion,
        apply_tutorial_actor_animation_playback, apply_tutorial_actor_commands,
        cleanup_tutorial_actors, ground_tutorial_actors, produce_tutorial_avatar_target_feed,
        refresh_tutorial_npc_observation, spawn_tutorial_actor_visuals,
    },
    tutorial_auxiliary_choreography::{
        TutorialAuxiliaryAction, TutorialAuxiliaryEmission, TutorialAuxiliarySequence,
        TutorialBooleanFlag, TutorialScreenAxis as AuxiliaryScreenAxis,
        TutorialScreenPivot as AuxiliaryScreenPivot, TutorialScreenPoint, TutorialSubtitleChannel,
        TutorialText, TutorialWaypointTarget, tutorial_auxiliary_definition,
    },
    tutorial_choreography::{
        BlockingWait, CameraAction, CameraMode, ChoreographyAction, ClientVec3, EffectAction,
        EffectLocaleGate, EntityRef, EquipmentAction, LoopAction, NanoAction, NpcAction,
        NpcCommand, PlayerAction, PositionExpr, ProgressAction, ProjectileAction, RotationExpr,
        TutorialCursorDirection as ChoreographyCursorDirection, TutorialPictureAction,
        TutorialScreenAxis as ChoreographyScreenAxis, TutorialSoundAction,
        tutorial_scene_choreography,
    },
    tutorial_choreography_formula::{
        TutorialCameraCaptureStore, resolve_position as resolve_typed_choreography_position,
        resolve_rotation as resolve_typed_choreography_rotation,
    },
    tutorial_choreography_runtime::{
        ChoreographyCompletion, ChoreographyPlaybackEvent, FrameSequenceSample, RigAnimationTarget,
        TutorialChoreographyIssue, TutorialChoreographyIssueQueue, TutorialChoreographyPlayer,
        TutorialChoreographyPresentation, TutorialChoreographyRuntimePlugin,
    },
    tutorial_cinematic_title::{TutorialCinematicTitlePlugin, TutorialCinematicTitleSet},
    tutorial_effects_runtime::{
        TutorialEffectLibrary, TutorialEffectPlacement, TutorialEffectRuntime,
        TutorialEffectRuntimeCommand, TutorialEffectRuntimeSet, TutorialEffectsRuntimePlugin,
        TutorialProjectileMotion, cleanup_tutorial_effect_runtime, process_tutorial_effect_runtime,
    },
    tutorial_ep_barrier::TutorialEpBarrierPlugin,
    tutorial_logic::{
        LegacySpawnPosition, SceneCompletion, TutorialDialogue, TutorialIntent,
        TutorialLocaleBranch, TutorialNpcSpawn, TutorialObservation, TutorialProgressReset,
        evaluate_progress,
    },
    tutorial_mission_content::{
        GameplaySkillUiDefinition, GameplayWarpEligibility, GameplayWarpEligibilityInput,
        GameplayWarpGroupMember, GameplayWarpServerPosition, GameplayWarpWorldPosition,
        TutorialMissionContent, TutorialMissionType, TutorialWarpTarget,
    },
    tutorial_nano_gameplay::{
        TUTORIAL_BUTTERCUP_INITIAL_STAMINA, TutorialNanoGameplayCommandQueue,
        TutorialNanoGameplayLoadout, TutorialNanoGameplayPlugin, TutorialNanoGameplaySet,
        TutorialNanoGameplayState, cleanup_tutorial_nano_gameplay,
    },
    tutorial_nano_presentation::{
        TutorialNanoPresentationCommandQueue, TutorialNanoPresentationPlugin,
        TutorialNanoPresentationSet, TutorialNanoPresentationSpawn,
        cleanup_tutorial_nano_presentation,
    },
    tutorial_nanocom_message::{
        TUTORIAL_NANOCOM_MESSAGE_TYPE, TUTORIAL_NANOCOM_NPC_TYPE, TutorialNanocomMessageContext,
        TutorialNanocomMessagePlugin, TutorialNanocomMessageSet,
    },
    tutorial_native_mechanics::{TutorialNativeIntentApplication, TutorialNativeMechanics},
    tutorial_overlay_ui::{
        TutorialArrowCue, TutorialArrowDirection, TutorialCueScalePivot, TutorialIllustration,
        TutorialIllustrationCue, TutorialOverlayUiModel, TutorialOverlayUiPlugin, TutorialUi,
    },
    tutorial_player_presentation::{
        PlayerWeaponAnimationCatalog, TutorialPlayerClip, TutorialPlayerEquipmentRequest,
        TutorialPlayerPresentationCommandQueue, tutorial_player_gender_from_protocol,
    },
    tutorial_player_rig_runtime::{
        TutorialPlayerAnimationApplied, TutorialPlayerFallbackVisual,
        TutorialPlayerRigRuntimePlugin, TutorialSelectedPlayerRig, TutorialSelectedPlayerRigActive,
        TutorialSelectedPlayerRigCandidate, TutorialSelectedPlayerRigStatus,
        spawn_tutorial_selected_player_rig,
    },
    tutorial_voice_subtitles::{
        TutorialVoiceSubtitlePlugin, TutorialVoiceSubtitleSet, TutorialVoiceSubtitleState,
        is_english_locale,
    },
    ui_startup::NativeUiStartupPhase,
    upsell_ui::{UpsellUiAudioOutbox, UpsellUiModel, UpsellUiOutbox, UpsellUiPlugin, UpsellUiSet},
    user_equip_runtime::{
        UserEquipPendingRequest0104, UserEquipProductionRuntime0104,
        prepare_user_equip_chest_open_0104, prepare_user_equip_delete_0104,
    },
    user_equip_ui::{
        UserEquipAvatarPreviewPresentation, UserEquipItemModeProjection, UserEquipModalState,
        UserEquipNanoStationAction, UserEquipUiAction, UserEquipUiOutbox, UserEquipUiPlugin,
        UserEquipUiSet, UserEquipUiState,
    },
    user_store_runtime::UserStoreProductionRuntime0104,
    user_store_ui::{UserStoreUiPlugin0104, UserStoreUiSet0104, UserStoreUiState0104},
    vendor_runtime::{
        ActiveVendorSourceNpc0104, VendorOutboundRequest0104, VendorProductionEvent0104,
        VendorProductionRuntime0104,
    },
    vendor_ui::{
        VendorLifecyclePhase, VendorModalState, VendorModeProjection0104, VendorUiCommand0104,
        VendorUiOutbox0104, VendorUiPlugin, VendorUiSet, VendorUiState,
    },
    world::{
        EXTENDED_WORLD_CAMERA_FAR_NATIVE, NativeWorldCatalog, NativeWorldPlugin, NativeWorldScope,
        NativeWorldSet, NativeWorldStreamingStatus, load_native_world_scenes,
    },
    world_audio::{
        RetrobutionInstanceAudioState, RetrobutionWorldAudioCatalog, RetrobutionWorldAudioPlugin,
    },
    world_behaviour::{
        NativeWorldBehaviourRoot, WorldBehaviourPlugin, consume_world_launcher_outbox,
        process_world_trigger_uses,
    },
    world_map::{
        WorldMapOpenContext, WorldMapPhase, WorldMapPresentation, WorldMapPresentationAssetStatus,
        WorldMapPresentationPlugin, WorldMapPresentationSet,
    },
    world_mission_indicators::ClientNpcWaypointCatalog,
    world_mission_runtime::{
        WorldMissionAcceptRejection, WorldMissionRuntime, WorldMissionServerEvent0104,
    },
    world_nano_cooldown::WorldNanoCooldownRuntime,
    world_targeting::produce_world_avatar_target_feed,
};
#[cfg(test)]
use ffone_client::{
    combi_runtime::CombiReplyPacket0104,
    combi_ui::{CombiSelectionSlot0104, CombiSuccessReply0104, CombiUiCommand0104},
    enchant_runtime::EnchantReplyPacket0104,
    enchant_ui::EnchantAttachmentSlot0104,
};
#[cfg(test)]
use ffone_client::{
    gameplay_ui::QuickChatMenuUi,
    tutorial_player_presentation::{
        TutorialPlayerAnimationDispatch, TutorialPlayerPresentationCommand,
    },
};
use ffone_net::{PresentNpcTypesGameplayFrame0104, WorldBootstrapPacket};
#[cfg(test)]
use ffone_protocol::PcAttackNpcsSuccess0104;
use ffone_protocol::{
    AllGroupFreeChatPacket0104, AllGroupFreeChatRequest0104, AllGroupFreeChatSuccess0104,
    AllGroupMenuChatPacket0104, AllGroupMenuChatRequest0104, AvatarEmoteChat0104,
    BuddyAcceptRequest0104, BuddyBaseInfo0104, BuddyFindNameAcceptRequest0104,
    BuddyFindNameRequest0104, BuddyFreeChatPacket0104, BuddyFreeChatRequest0104,
    BuddyFreeChatSuccess0104, BuddyLifecyclePacket0104, BuddyMenuChatPacket0104,
    BuddyMenuChatRequest0104, BuddyRemoveRequest0104, BuddySetBlockRequest0104,
    BuddyStateRequest0104, BuddyStateSuccess0104, BuddyWarpRequest0104, CharacterCreateRequest0104,
    CharacterNameCheckRequest0104, CharacterNameSaveSuccess0104, DecodedFrame, FixedUtf16,
    FreeChatPacket0104, FreeChatRequest0104, FreeChatSuccess0104, InventoryPacket0104,
    ItemBase0104, ItemMoveSuccessPacket0104, ItemReward0104, ItemUsePacket0104, MenuChatPacket0104,
    MenuChatRequest0104, NpcInteractionRequest0104, PcItemDeleteRequest0104,
    PcNanoCreatePacket0104, PcRegenRequest0104, PcWarheadFirePacket0104, PresentNpcTypesPacket0104,
    TimeBuffDotDamageTick0104, WirePayload, decode_all_group_freechat_packet_0104,
    decode_all_group_menu_chat_packet_0104, decode_avatar_emote_chat_0104,
    decode_buddy_freechat_packet_0104, decode_buddy_lifecycle_packet_0104,
    decode_buddy_menu_chat_packet_0104, decode_freechat_packet_0104,
    decode_gm_set_value_reply_0104, decode_inventory_packet_0104, decode_item_use_packet_0104,
    decode_menu_chat_packet_0104, decode_pc_nano_create_packet_0104,
    decode_pc_warhead_fire_packet_0104, decode_server_message_0104, packet,
};
#[cfg(windows)]
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    window::Icon,
};
#[cfg(windows)]
use winsafe::{HMONITOR, POINT, co};

pub(crate) use bootstrap::entry;
use network_ingress::{NetworkIngressPlugin, poll_network};
use schedule::run;
use session_lifecycle::{
    NetworkSessionBoundary, NetworkSessionLifecyclePlugin, NetworkSessionLifecycleSet,
    NetworkSessionReset,
};
use state::ClientState;

mod asset_residency;
mod bank;
mod cashmall;
mod character_loading;
mod chat_commands;
mod combi;
mod dexter_ship_drive;
mod dexter_ship_scene;
mod email_catalog;
mod email_outputs;
mod email_production;
mod enchant;
mod group_pc2pc;
mod player_interaction;
mod guide;
mod input_gates;
mod launcher;
mod loading_screen;
mod local_avatar;
mod local_inventory;
mod login;
mod mission_indicators;
mod modal_gates;
mod nano_free_tuning;
mod nano_free_tuning_production;
mod network_smoke;
mod npc_warp;
mod option_runtime;
mod quick_slots;
mod quit_menu;
mod race;
mod resurrect;
mod rule;
mod runtime_status;
mod sky;
mod transportation;
mod tutorial_ambience;
mod tutorial_combat;
mod tutorial_indicators;
mod tutorial_mission_flow;
mod tutorial_scene_audio;
mod tutorial_session;
mod tutorial_startup;
mod user_equip;
mod vendor;
mod world_combat;
mod world_intents;
mod world_map;
mod world_mission;
mod world_nano;
mod world_npc_interaction;
mod world_scene;
use asset_residency::{
    AssetResidency, SharedTutorialEffectLibrary, TutorialEffectLibraryLoadStatus,
    begin_tutorial_effect_library_load, poll_tutorial_effect_library, sync_asset_residency_groups,
};
use bank::{
    BankSystemMessageRuntime, apply_bank_network_frame, consume_bank_system_message_outbox,
    consume_bank_ui_outbox, reset_bank_session, reset_bank_shell, sync_bank_ui_context,
};
use cashmall::{
    consume_cashmall_production_effects_0104, consume_upsell_audio_outbox,
    consume_upsell_ui_outbox, consume_user_store_outbox_0104,
    guard_user_store_before_interaction_0104, guard_user_store_production_boundary_0104,
    is_cashmall_hidden_chat_command_0104, reset_cashmall_session_0104, reset_upsell_shell,
    reset_user_store_session_0104, route_cashmall_production_input_0104,
    sync_cashmall_production_context_0104, sync_upsell_ui_context,
};
use character_loading::{
    activate_character_creation_ui, activate_character_selection_ui, activate_gameplay_ui,
    begin_character_creation_asset_prewarm, begin_character_creation_loading,
    begin_character_selection_loading, defer_native_ui, drive_character_creation_asset_prewarm,
    reset_character_selection_asset_lease,
    gate_character_creation_loading, gate_character_selection_loading,
    initialize_character_creation_data, release_character_creation_asset_lease,
};
use chat_commands::{
    FlightCommand, GmNanoRequestError, GmSpeedRequestError, build_gm_nano_request,
    build_gm_speed_request, decode_local_gm_speed_reply_0104, is_server_chat_command_0104,
    parse_gm_nano_command, parse_gm_speed_command, parse_local_flight_command,
};
use combi::{
    CombiNetworkFrameInbox0104, CombiProductionCatalog0104, CombiProductionShell0104,
    RuntimeCombiItemCatalog0104, combi_player_authority_0104, consume_combi_network_frames_0104,
    consume_combi_production_outputs_0104, consume_combi_system_message_outbox_0104,
    drive_combi_production_0104, reset_combi_session_0104, reset_combi_shell_0104,
    sync_combi_presentation_0104,
};
use dexter_ship_drive::{
    apply_dexter_ship_animations, cleanup_dexter_ship_cutscene, drive_dexter_ship_cutscene,
};
use dexter_ship_scene::{
    DEXTER_SHIP_RENDER_TEXTURE_SIZE, DexterShipActor, DexterShipActorRole,
    DexterShipCutsceneRuntime, DexterShipHologram, DexterShipSceneReady, dexter_ship_dexter_clip,
    spawn_dexter_ship_cutscene,
};
use email_catalog::{
    EmailProductionCatalog0104, available_email_task_ids_0104, email_guide_messages_0104,
    enqueue_login_guide_nanocom_0104,
};
use email_outputs::{
    consume_email_production_outputs_0104, consume_email_system_message_outbox_0104,
    drive_email_production_post_interaction_0104, drive_email_production_pre_interaction_0104,
    poll_email_update_check_0104,
};
use email_production::{
    EmailProductionShell0104, active_email_task_ids_0104, consume_email_network_frames_0104,
    email_player_authority_0104, reset_email_session_0104, reset_email_shell_0104,
};
use enchant::{
    EnchantNetworkFrameInbox0104, EnchantProductionShell0104, consume_enchant_network_frames_0104,
    consume_enchant_production_outputs_0104, consume_enchant_system_message_outbox_0104,
    consume_enchant_text_input, drive_enchant_production_0104, enchant_player_authority_0104,
    reset_enchant_session_0104, reset_enchant_shell_0104, sync_enchant_presentation_0104,
};
use group_pc2pc::{
    BUDDY_NANOCOM_MESSAGE_ID_BASE, BUDDY_SYSTEM_MESSAGE_ID_BASE, GroupRuntimeIntegration0104,
    WorldPc2pcOfferPrompt, apply_group_production_frame_0104, consume_group_nanocom_outbox_0104,
    consume_group_system_message_outbox_0104, consume_world_pc2pc_offer_prompt,
    drain_world_pc2pc_offer_outbox, open_world_pc2pc_accepted_session, reset_group_session_0104,
    reset_world_pc2pc_session, world_remote_pc_display_name,
};
use guide::{
    ActiveGuideSourceNpc, GUIDE_WARP_FAILURE_MESSAGE_ID, GuideMentorReplyOutcome,
    GuideProductionRuntime, advance_pending_guide_warp, apply_guide_mentor_reply_transactional,
    capture_guide_escape, consume_guide_system_message_outbox, consume_guide_ui_audio_outbox,
    consume_guide_ui_outbox, reset_guide_shell, switch_guide_special_state, sync_guide_ui_context,
};
use input_gates::{
    apply_tutorial_npc_subtarget_camera, apply_world_npc_subtarget_camera,
    open_tutorial_exit_dialog_on_shortcut, sync_legacy_gameplay_cursor, sync_tutorial_action_gate,
    sync_tutorial_input_gate,
};
use launcher::{
    drive_launcher_production, prepare_launcher_production_context, read_launcher_configured_aim,
};
use loading_screen::{
    GameplayLoadingState, ResourceLoadingScope, spawn_gameplay_loading_screen,
    sync_gameplay_loading_screen,
};
use local_avatar::{
    advance_skyway_traversal, drive_local_vehicle_toggle, reset_local_vehicle_presentation,
    sync_local_avatar_presentation,
};
use local_inventory::{
    LocalInventoryRuntime, WorldPlayerApparel0104, WorldPlayerApparelCandidate0104,
    WorldPlayerEquipmentProjection,
};
use login::{
    ActiveLoginCredentials, ClientConfig, LoadedCharacterCreationData, PendingLogin, begin_login,
    consume_login_ui_effects, gate_login_loading, handle_login_ui_requests, sync_login_ui,
};
use mission_indicators::{
    TutorialMissionIndicatorRuntime, TutorialNanoConditionEffectRuntime,
    WorldMissionBarkerRequestRuntime, WorldMissionIndicatorRuntime, WorldMissionWaypointRuntime,
    reset_world_mission_barker_request, sync_world_mission_indicators, sync_world_mission_waypoint,
    sync_world_npc_game_icons, tutorial_current_objective_ui, tutorial_minimap_marker_icon,
    tutorial_npc_mission_symbol,
};
use modal_gates::{
    GameplayModalModels, GameplayUiModalInputs, sync_guide_chat_input_gate,
    sync_nanocom_foreign_modal_suppression,
};
use nano_free_tuning::{
    NanoFreeTuningOpenTrigger, NanoFreeTuningPreviewAnimation, NanoFreeTuningProductionRuntime,
    open_pending_nano_free_tuning, reset_nano_free_tuning_session, reset_nano_free_tuning_shell,
};
use nano_free_tuning_production::{
    apply_nano_create_commit, consume_nano_free_tuning_production,
    emit_nano_free_tuning_preview_animation_sounds, sync_nano_free_tuning_preview_animation,
};
use network_smoke::run_network_smoke;
use npc_warp::{
    AuthoritativeWarpReply0104, NORMAL_NPC_WARP_DELAY_SECONDS, NORMAL_NPC_WARP_EFFECT_ID,
    NORMAL_NPC_WARP_FAILURE_MESSAGE_ID, NORMAL_NPC_WARP_PRESENTATION,
    NORMAL_NPC_WARP_SOUND_TRUE_NAME, NormalNpcWarpIdentity, NormalNpcWarpRuntime,
    PendingNormalNpcWarp, WarpDepartureClock, advance_pending_normal_npc_warp,
    apply_authoritative_world_teleport, consume_normal_warp_system_message_outbox,
    decode_authoritative_warp_reply_0104, normal_npc_warp_item_slot, open_unpaid_past_warp_upsell,
    render_normal_npc_warp_window_in_fade, validate_normal_npc_warp_ui_action,
};
#[cfg(test)]
use option_runtime::option_sound_gain;
use option_runtime::{
    OptionProductionRuntime, apply_option_camera_input_settings, clear_option_action_press,
    consume_option_ui_outbox, option_action_just_pressed, option_binding_just_pressed,
    reset_option_session,
    route_option_mode_input, sync_option_buddy_projection, sync_option_live_runtime,
    sync_option_settings_to_live_owners, sync_retrobution_audio_mix,
};
use quick_slots::{
    apply_quick_slot_frame, apply_skill_buff_frame, apply_special_state_frame,
    consume_quick_slot_ui_outbox, sync_quick_slot_ui_context, sync_skill_buff_ui_context,
};
use quit_menu::{
    capture_quit_menu_open_intent, close_quit_menu_from_gamepad, complete_quit_menu_destination, consume_quit_menu_audio_outbox,
    consume_quit_menu_ui_outbox, open_quit_menu_from_nanocom, reset_quit_menu_shell,
    sync_quit_menu_context,
};
use race::{
    RACE_RANK_HTTP_GAP, RACE_RANK_HTTP_TRANSPORT_AVAILABLE, RaceNetworkFrameInbox,
    RaceProductionRuntime, consume_race_mode_ui_commands, consume_race_network_frames,
    consume_race_production_outputs, consume_race_rank_ui_commands,
    consume_race_system_message_outbox, decode_race_instance_map_info_0104,
    open_race_fail_from_warp_away, open_race_mode_from_npc, race_service_allowed, read_race_i32,
    reset_race_session, reset_race_shell, sync_race_presentation_input, sync_race_world_rings,
    sync_race_hud, cancel_expired_race,
    warp_away_xcom_index,
};
use resurrect::{
    LocalResurrectPacket0104, consume_resurrect_ui_outbox, decode_local_resurrect_packet_0104,
    reset_resurrect_shell, sync_resurrect_ui_context,
};
use rule::{consume_rule_runtime, reset_rule_session, sync_rule_ui_context};
use runtime_status::{
    RuntimeNanoSlot, RuntimeStatus, apply_pc_regen_success_to_runtime,
    apply_runtime_frame_with_authority, client_state_for_world_scope,
    legacy_avatar_max_fusion_matter, legacy_avatar_max_hp, native_world_scope,
    resolve_runtime_nano_slots, resolve_runtime_resurrection_item_slot,
};
use sky::{
    LEGACY_REFERENCE_GLOW_ENABLED, ensure_and_update_legacy_fusion_star,
    ensure_and_update_legacy_skybox,
};
use transportation::{
    SkywayTraversalMotion, TransportationProductionRuntime, TransportationSkywayFrame0104,
    advance_transportation_production, consume_transportation_model_outbox,
    consume_transportation_system_message_outbox, consume_transportation_ui_commands,
    decode_transportation_skyway_frame_0104, reset_transportation_session,
    reset_transportation_shell, sync_transportation_presentation_input,
    transportation_move_ok_voice_set, transportation_npc_service,
};
use tutorial_ambience::{
    LegacyWorldAmbienceCache, TutorialAmbientRuntime, TutorialDome, TutorialLoopAudio,
    TutorialSceneAudio, TutorialVoiceAudio, animate_tutorial_dome_material,
    apply_legacy_world_ambience, cleanup_tutorial_ambient, drive_tutorial_ambient,
    legacy_world_ambience_active, tutorial_ambient_after_chapter_init,
    tutorial_ambient_after_confirmed_warp,
};
use tutorial_combat::{
    collect_tutorial_actor_events, collect_tutorial_avatar_actions,
    collect_tutorial_nano_gameplay_events, report_tutorial_runtime_issues,
    tutorial_named_descendant_position,
};
use tutorial_indicators::{
    cleanup_pending_tutorial_actor_effects, cleanup_tutorial_mission_indicators,
    cleanup_tutorial_nano_condition_effects, sync_tutorial_mission_interaction,
    sync_tutorial_nano_condition_effects, sync_tutorial_world_mission_indicators,
    tutorial_actor_scene_is_terminal, tutorial_descendants_named,
};
use tutorial_mission_flow::{
    append_tutorial_stage_instruction, apply_tutorial_dialogue_intent, apply_tutorial_warp,
    consume_tutorial_mission_outbox, scan_tutorial_location_tasks, stop_tutorial_dialogue,
    tutorial_auxiliary_for_dialogue, tutorial_auxiliary_for_stage,
};
use tutorial_scene_audio::{
    cleanup_tutorial_effect_runtime_unless_world_ready, cleanup_tutorial_runtime,
    horizontal_distance, play_tutorial_scene_timeline,
    recover_tutorial_completion_to_character_selection, request_tutorial_completion,
    shortest_angle_delta, spawn_tutorial_voice, start_tutorial_voice_subtitle, stop_tutorial_voice,
    sync_tutorial_voice_subtitle_context, tutorial_client_position_to_native,
    tutorial_target_in_center,
};
use tutorial_session::{
    PendingTutorialActorEffect, PendingTutorialActorEffects, TUTORIAL_CHAT_HISTORY_LIMIT,
    TutorialActorDrive, TutorialAuxiliaryDriveState, TutorialAuxiliaryPresentation,
    TutorialChoreographyDrive, TutorialChoreographyExecution, TutorialChoreographyFrame,
    TutorialExitDrive, TutorialLogicRuntime, TutorialMissionRuntime,
    TutorialProjectileRandomStream, TutorialSession,
};
use tutorial_startup::{
    TUTORIAL_MOVEMENT_MARKER_EFFECT_ID, TUTORIAL_START_ANGLE, TUTORIAL_START_UNITY_POSITION,
    enable_player_after_native_collider_ready, preload_tutorial_effects,
    prime_tutorial_startup_pose, start_tutorial_sequence, suppress_locked_tutorial_ui_shortcuts,
    sync_tutorial_native_stage,
};
use user_equip::{
    LocalVehiclePresentationRuntime, UserEquipAvatarPresentationRuntime,
    UserEquipSystemMessageRuntime0104, apply_user_equip_frame,
    consume_user_equip_system_message_outbox, consume_user_equip_ui_outbox,
    reset_user_equip_session, sync_user_equip_ui_context,
};
use vendor::{
    VendorSystemMessageRuntime, apply_vendor_frame, consume_vendor_system_message_outbox,
    consume_vendor_ui_audio_outbox, consume_vendor_ui_outbox, reset_vendor_session,
    reset_vendor_shell, sync_vendor_ui_context,
};
use world_combat::{
    WorldCombatLifecycle, advance_world_combat_lifecycle, animate_local_player_damage,
    collect_world_npc_attack_visuals, frame_observes_local_combat, local_player_damage_audio_edge,
    local_player_npc_attack_result_from_frame, queue_local_player_damage_audio,
    validated_world_weapon_bullet_links,
};
use world_intents::{
    flush_movement_intents, flush_world_gameplay_intents, sync_tutorial_network_npc_visibility,
    toggle_player_interaction,
};
use world_map::{
    WorldMapProductionRuntime, WorldMapServerClock, apply_world_map_present_reply,
    consume_world_map_outbox, handle_world_map_input, open_world_map_for_ready_player,
    reset_world_map_session, reset_world_map_shell, sync_world_map_projection,
    world_map_npc_sources, world_map_player_from_native,
};
use world_mission::{
    MissionDeleteConfirmationRuntime, MissionNanocomEdge, apply_world_mission_network_frame,
    consume_mission_delete_system_message_outbox, enqueue_mission_nanocom,
    localized_mission_delete_confirmation, reset_world_mission_session, sync_world_mission_ui,
};
use world_nano::{
    WorldNanoAuthorityInbox0104, activate_world_nano_from_keyboard, advance_world_nano_cooldowns,
    apply_world_nano_authority_inbox, apply_world_nano_response_frame,
    apply_world_npc_skill_response_frame, reset_world_nano_and_mission_presentation,
    sync_world_nano_presentation,
};
use world_npc_interaction::{
    bank_service_kind, collect_world_npc_interactions, combi_service_allowed_0104,
    enchant_service_allowed_0104, guide_service_entry,
};
use world_scene::{
    project_nano_tune_correlation_fault, recover_failed_world_ready_connection,
    rollback_failed_world_ready_spawn, setup_scene, spawn_native_world_slice,
    sync_local_infection_status_effect, update_legacy_avatar_environment,
};

#[cfg(test)]
mod skyway_traversal_motion_tests;

mod assets_register_dexter_hologram_asset;
mod models;
mod constants;
mod animation;
mod operations;
mod materials;
mod types;
mod systems;
mod projects;
mod state_client_state_uses_world_nano_aut;
mod audio_consume_gameplay_ui_audio_outbox;
mod commands;

use assets_register_dexter_hologram_asset::{
    CHARACTER_ASSET_SOURCE, DEFAULT_CHARACTER_ASSET_ROOT, dexter_hologram_asset_path,
    register_dexter_hologram_asset
};
use models::DEFAULT_CHARACTER_MODEL;
use constants::{
    DEFAULT_CHARACTER_ROOT, HELP, NATIVE_UI_REFERENCE_WIDTH, NATIVE_UI_REFERENCE_HEIGHT,
    LOCAL_INFECTION_EFFECT_NAME, LOCAL_INFECTION_AURA_EFFECT_ID,
    LOCAL_INFECTION_DAMAGE_EFFECT_ID, LOCAL_INFECTION_PROTECTION_EFFECT_ID
};
use animation::DEFAULT_CHARACTER_ANIMATION;
#[cfg(windows)]
use operations::{set_primary_window_icon, open_external_url};
#[cfg(test)]
use operations::centered_outer_window_position;
use operations::{
    release_unextracted_view_targets, enforce_primary_good_msaa,
    native_ui_window_scale_factor, shared_gameplay_world_active, world_nano_authority_active
};
#[cfg(target_os = "macos")]
use operations::open_external_url;
#[cfg(all(not(windows), not(target_os = "macos")))]
use operations::open_external_url;
use materials::{stable_native_render_plugin, DexterHologramMaterial, DexterHologramUniform};
pub use types::ReleaseUnextractedViewTargetsPlugin;
use types::{
    LocalPlayer, LocalNetworkIdentity, LocalCharacterScene, WorldSliceEntity,
    TutorialSuppressedNetworkNpc
};
use systems::sync_native_ui_window_scale;
#[cfg(test)]
use systems::native_ui_scale_factor_update;
use projects::NetworkLifecycleSession;
use state_client_state_uses_world_nano_aut::{
    LocalInfectionPresentationState, client_state_sends_movement_intents,
    client_state_uses_world_nano_authority, client_state_uses_ordinary_world_action_collector
};
use audio_consume_gameplay_ui_audio_outbox::{
    consume_gameplay_ui_audio_outbox, consume_system_message_ui_audio_outbox
};
use commands::ordinary_world_action_collector_active;
