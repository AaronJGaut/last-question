use bevy::{
    core::FixedTimestep,
    prelude::*,
};

const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Component)]
struct Label(String);

#[derive(Component)]
struct Transform(Vec3);

#[derive(Component)]
struct Velocity(Vec3);

fn physics_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.0 += velocity.0 * TIME_STEP;
    }
}

fn debug_system(query: Query<(&Label, &Transform)>) {
    for (label, transform) in query.iter() {
        println!("{}: {}", label.0, transform.0);
    }
}

fn startup_system(mut commands: Commands) {
    commands
        .spawn()
        .insert(Label("Test point 1".to_string()))
        .insert(Transform(Vec3::new(0.0, 0.0, 0.0)))
        .insert(Velocity(Vec3::new(0.1, 0.0, 0.0)));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(physics_system)
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1f64))
                .with_system(debug_system)
        )
        .run();
}
