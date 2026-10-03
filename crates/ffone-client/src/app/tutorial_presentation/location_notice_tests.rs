use super::*;
use ffone_client::{
    gameplay_ui::CombatModeNotice, legacy_npc_nano_animation::LegacyNanoStandRandomStream,
};

#[test]
fn location_notice_tracks_entry_crossing_and_session_without_restarting_each_frame() {
    let mut app = App::new();
    app.insert_resource(State::new(ClientState::Tutorial))
        .init_resource::<GameplayUiModel>()
        .init_resource::<GameplayLoadingState>()
        .init_resource::<CombatModeNotice>()
        .insert_resource(LegacyNanoStandRandomStream::with_seed(7))
        .add_systems(Update, sync_world_location_notice);
    {
        let mut model = app.world_mut().resource_mut::<GameplayUiModel>();
        model.visible = true;
        model.player.owner = 1;
        model.minimap.map_name = "Sector V".to_owned();
    }
    let draws = |app: &App| {
        app.world()
            .resource::<LegacyNanoStandRandomStream>()
            .draw_count()
    };
    app.update();
    assert_eq!(draws(&app), 0, "tutorial suppresses location notices");
    app.insert_resource(State::new(ClientState::World));
    app.world_mut()
        .resource_mut::<GameplayLoadingState>()
        .visible = true;
    app.update();
    assert_eq!(draws(&app), 0, "loading must not consume the entry notice");
    app.world_mut()
        .resource_mut::<GameplayLoadingState>()
        .visible = false;
    app.update();
    assert_eq!(draws(&app), 1, "first world location is announced");
    for _ in 0..5 {
        app.update();
    }
    assert_eq!(draws(&app), 1, "idle frames must not restart the notice");
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .minimap
        .map_name = "Pokey Oaks North".to_owned();
    app.update();
    assert_eq!(
        draws(&app),
        2,
        "walking or warping into another zone announces it"
    );
    app.world_mut()
        .resource_mut::<GameplayUiModel>()
        .player
        .owner = 2;
    app.update();
    assert_eq!(draws(&app), 3, "a new character can enter the same zone");
    app.world_mut().resource_mut::<GameplayUiModel>().visible = false;
    app.update();
    app.world_mut().resource_mut::<GameplayUiModel>().visible = true;
    app.update();
    assert_eq!(draws(&app), 4, "reconnecting resets the location history");
}
