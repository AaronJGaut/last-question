use bevy::{
    core::FixedTimestep,
    prelude::*,
    app::{AppExit},
};

const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Component)]
struct Label(String);

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Player;

fn physics_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * TIME_STEP;
    }
}

fn input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &Player)>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    let mut velocity = query.single_mut().0;
    let mut direction = Vec3::ZERO;
    if keyboard_input.pressed(KeyCode::W) {
        direction.y += 1.0;
    }
    if keyboard_input.pressed(KeyCode::A) {
        direction.x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::S) {
        direction.y -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) {
        direction.x += 1.0;
    }
    if keyboard_input.pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
    direction = direction.normalize_or_zero();
    velocity.0 = 50.0 * direction;
}

fn debug_system(query: Query<(&Label, &Transform)>) {
    for (label, transform) in query.iter() {
        println!("{}: {}", label.0, transform.translation);
    }
}

fn startup_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn()
        .insert(Label("Test point 1".to_string()))
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(5.0, 5.0, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.0, 1.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity(Vec3::ZERO))
        .insert(Player);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(input_system)
                .with_system(physics_system)
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(1f64))
                .with_system(debug_system)
        )
        .run();
}
