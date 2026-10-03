use super::*;

#[test]
fn stationary_particle_does_not_dirty_its_transform_every_frame() {
    #[derive(Resource, Default)]
    struct ChangedTransformCount(usize);

    fn count_changed_transforms(
        particles: Query<(), (With<NativeParticle>, Changed<Transform>)>,
        mut count: ResMut<ChangedTransformCount>,
    ) {
        count.0 = particles.iter().count();
    }

    let asset_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
    let library = TutorialEffectLibrary::load(asset_root).unwrap();
    let compiled = compile_effect_plan(771, library.effects.get(&771).unwrap());
    let mut emitter = compiled
        .plan
        .unwrap_or_else(|| panic!("ES771 native blockers: {:#?}", compiled.blockers))
        .emitters
        .into_iter()
        .next()
        .expect("ES771 particle emitter");
    emitter.force = Vec3::ZERO;
    emitter.lifetime = 5.0;
    emitter.initial_size = 1.0;
    emitter.width_curve = AnimationCurve::default();
    emitter.height_curve = AnimationCurve::default();
    emitter.rotation_curve = AnimationCurve::default();

    let mut app = App::new();
    app.init_resource::<Time>()
        .init_resource::<ChangedTransformCount>()
        .add_systems(
            Update,
            (
                simulate_particles,
                count_changed_transforms.after(simulate_particles),
            ),
        );
    app.world_mut().spawn((
        Transform::IDENTITY,
        Visibility::Inherited,
        NativeParticle {
            stream_owner: None,
            plan: Arc::new(emitter),
            scale: 1.0,
            age: 0.0,
            velocity: Vec3::ZERO,
            stream_owned: false,
            alive: true,
        },
    ));
    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.1));

    app.update();
    assert_eq!(app.world().resource::<ChangedTransformCount>().0, 1);

    app.world_mut()
        .resource_mut::<Time>()
        .advance_by(std::time::Duration::from_secs_f32(0.1));
    app.update();
    assert_eq!(
        app.world().resource::<ChangedTransformCount>().0,
        0,
        "unchanged particle presentation must not retrigger transform propagation"
    );
}

#[test]
fn sword_trail_restarts_after_expiry_without_an_idle_animation_frame() {
    use crate::avatar_action::{schedule_legacy_action_frame, LegacyAvatarActionInput,
        LegacyAvatarActionContext, LegacyTargetSelection, LegacyAvatarClipBindings};
    let mut app=App::new();
    app.insert_resource(Time::<()>::default()).init_resource::<Assets<Mesh>>()
        .add_systems(Update, update_native_sword_trails);
    let owner=app.world_mut().spawn((LegacyAvatarActionState::default(), GlobalTransform::IDENTITY)).id();
    let attachment=app.world_mut().spawn(TutorialPlayerWeaponAttachment {
        rig_root:owner,item_id:1,exact_route:String::new(),socket_full_path:String::new()}).id();
    let top=app.world_mut().spawn(GlobalTransform::from_translation(Vec3::Y)).id();
    let bottom=app.world_mut().spawn(GlobalTransform::IDENTITY).id();
    let edges=vec![NativeSwordTrailEdge::default();RETROBUTION_SWORD_TRAIL_LENGTH];
    let mesh=app.world_mut().resource_mut::<Assets<Mesh>>().add(native_sword_trail_mesh(&edges));
    let trail=app.world_mut().spawn((Mesh3d(mesh), Visibility::Hidden, NativeSwordTrail {
        attachment,controller_root:owner,rig_root:owner,top_point:top,bottom_point:bottom,
        edges,emitting:false,attack_generation:None,elapsed_seconds:0.0})).id();
    for generation in 1..=3 {
        let mut action=app.world_mut().get_mut::<LegacyAvatarActionState>(owner).unwrap();
        let _=schedule_legacy_action_frame(LegacyAvatarActionInput {primary_held:true,..default()},
            &LegacyAvatarActionContext::default(),&LegacyTargetSelection::default(),
            &mut action,&LegacyAvatarClipBindings::default(),1.0);
        assert_eq!(action.attack_generation(),generation);
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(0.01));
        app.update();
        assert_eq!(*app.world().get::<Visibility>(trail).unwrap(),Visibility::Inherited);
        assert_eq!(app.world().get::<NativeSwordTrail>(trail).unwrap().attack_generation,Some(generation));
        app.world_mut().resource_mut::<Time>().advance_by(std::time::Duration::from_secs_f32(0.8));
        app.update();
        assert_eq!(*app.world().get::<Visibility>(trail).unwrap(),Visibility::Hidden);
        assert!(native_sword_attack_active(app.world().get::<LegacyAvatarActionState>(owner).unwrap()));
    }
    app.world_mut().despawn(attachment);app.update();
    assert!(app.world().get_entity(trail).is_err());
}
