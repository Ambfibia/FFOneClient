use super::*;
use bevy::{ecs::system::RunSystemOnce, time::TimeUpdateStrategy};
use std::time::Duration;

fn fixture() -> (App, Entity) {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Image>()
        // Below `Time<Virtual>`'s 250 ms clamp.
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            200,
        )))
        .init_resource::<MinimapNewMailAlarm>()
        .init_resource::<MissionUiModel>()
        .add_systems(Update, bind);
    app.world_mut()
        .run_system_once(|mut commands: Commands, assets: Res<AssetServer>| {
            commands
                .spawn(Node::default())
                .with_children(|parent| spawn(parent, assets.load(TEXTURE_PATH)));
        })
        .unwrap();
    let icon = app
        .world_mut()
        .query_filtered::<Entity, With<MinimapNewMailIcon>>()
        .single(app.world())
        .unwrap();
    (app, icon)
}

fn display(app: &App, icon: Entity) -> Display {
    app.world().get::<Node>(icon).unwrap().display
}

#[test]
fn new_mail_icon_blinks_for_two_seconds_only_while_the_menu_is_closed() {
    let (mut app, icon) = fixture();
    app.update();
    assert_eq!(display(&app, icon), Display::None);
    let node = app.world().get::<Node>(icon).unwrap();
    assert_eq!(
        (node.left, node.top, node.width, node.height),
        (px(145.5), px(158.0), px(19.0), px(14.0))
    );
    assert_eq!(app.world().get::<ZIndex>(icon).unwrap().0, 1);

    app.world_mut().resource_mut::<MinimapNewMailAlarm>().arm();
    app.update();
    assert_eq!(display(&app, icon), Display::Flex);
    let elapsed = app.world().resource::<Time>().elapsed_secs();
    assert_eq!(
        app.world().get::<ImageNode>(icon).unwrap().color.alpha(),
        minimap_new_mail_alpha(elapsed)
    );

    // `!bViewMenu && bNewMail`: the open Nanocom menu hides the icon
    // without pausing the alarm.
    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .nanocom_main_menu_visible = true;
    app.update();
    assert_eq!(display(&app, icon), Display::None);
    app.world_mut()
        .resource_mut::<MissionUiModel>()
        .nanocom_main_menu_visible = false;
    for _ in 0..6 {
        app.update();
    }
    // 1.6 seconds counted down.
    assert_eq!(display(&app, icon), Display::Flex);
    for _ in 0..3 {
        app.update();
    }
    assert!(!app.world().resource::<MinimapNewMailAlarm>().active());
    assert_eq!(display(&app, icon), Display::None);
}

#[test]
fn new_mail_alpha_is_the_absolute_cosine_of_global_time() {
    assert_eq!(minimap_new_mail_alpha(0.0), 1.0);
    assert!(minimap_new_mail_alpha(0.25) < 1e-6);
    assert!((minimap_new_mail_alpha(0.5) - 1.0).abs() < 1e-6);
    assert!((minimap_new_mail_alpha(0.125) - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6);
}

#[test]
fn new_mail_icon_texture_matches_native_contract() {
    use sha2::Digest;
    let bytes = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../assets/game")
            .join(TEXTURE_PATH),
    )
    .unwrap();
    assert_eq!(bytes.len(), 393);
    assert_eq!(
        format!("{:x}", sha2::Sha256::digest(&bytes)),
        "7f2ec8c023456f7d12d33b91cc46bbd5241a6f66746fd9cf72ef871410cc0fd5"
    );
    assert_eq!(
        (
            u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
            u32::from_be_bytes(bytes[20..24].try_into().unwrap())
        ),
        (19, 14)
    );
}
