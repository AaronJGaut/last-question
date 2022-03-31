use bevy::{
    app::AppExit,
    core::FixedTimestep,
    prelude::*,
    sprite::collide_aabb::{collide, Collision},
    window::WindowMode
};

use last_question::pixel_perfect::{PixelPerfectPlugin, WorldCamera};
use last_question::tile;

const INPUT_TIME_STEP: f32 = 1.0 / 300.0;
const PHYSICS_TIME_STEP: f32 = 1.0 / 120.0;
const GRAVITY: f32 = 30.;

#[derive(Component)]
struct Label(String);

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Gravity(f32);

#[derive(Component)]
struct Player;

enum Direction {
    Left,
    Right,
    Neutral,
}

#[derive(Component)]
struct Mobility {
    on_ground: bool,
    jump_speed: f32,
    walk_speed: f32,
    walk_direction: Direction,
}

#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum PhysicsSystem {
    Gravity,
    Velocity,
    Collision,
    Camera,
}

fn physics_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * PHYSICS_TIME_STEP;
    }
}

fn gravity_system(mut query: Query<(&mut Velocity, &Gravity)>) {
    for (mut velocity, gravity) in query.iter_mut() {
        velocity.0.y -= gravity.0 * PHYSICS_TIME_STEP;
    }
}

fn input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity, &mut Mobility), With<Player>>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    let (mut transform, mut velocity, mut mobility) = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::R) {
        transform.translation = Vec3::new(0., 1., 0.);
        velocity.0 = Vec3::new(0., 0., 0.);
    }

    if keyboard_input.just_pressed(KeyCode::A) {
        mobility.walk_direction = Direction::Left;
    }
    if keyboard_input.just_released(KeyCode::A)
        && matches!(mobility.walk_direction, Direction::Left)
    {
        mobility.walk_direction = if keyboard_input.pressed(KeyCode::D) {
            Direction::Right
        } else {
            Direction::Neutral
        };
    }

    if keyboard_input.just_pressed(KeyCode::D) {
        mobility.walk_direction = Direction::Right;
    }
    if keyboard_input.just_released(KeyCode::D)
        && matches!(mobility.walk_direction, Direction::Right)
    {
        mobility.walk_direction = if keyboard_input.pressed(KeyCode::A) {
            Direction::Left
        } else {
            Direction::Neutral
        };
    }

    velocity.0.x = mobility.walk_speed
        * match mobility.walk_direction {
            Direction::Left => -1.0,
            Direction::Right => 1.0,
            Direction::Neutral => 0.0,
        };

    if keyboard_input.just_pressed(KeyCode::Space) {
        if mobility.on_ground {
            mobility.on_ground = false;
            velocity.0.y = mobility.jump_speed;
        }
    }
    if keyboard_input.just_released(KeyCode::Space) {
        if velocity.0.y > 0.0 {
            velocity.0.y = 0.0;
        }
    }

    if keyboard_input.pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}

fn player_solid_collision_system(
    mut player_query: Query<(&mut Velocity, &mut Transform, &mut Mobility), With<Player>>,
    collider_query: Query<&Transform, (With<tile::SolidCollider>, Without<Player>)>,
) {
    let (mut player_vel, mut player_tran, mut jump) = player_query.single_mut();
    jump.on_ground = false;
    for solid_tran in collider_query.iter() {
        let collision = collide(
            player_tran.translation,
            player_tran.scale.truncate(),
            solid_tran.translation,
            solid_tran.scale.truncate(),
        );
        if let Some(collision) = collision {
            let mean_scale = 0.5 * (player_tran.scale + solid_tran.scale);
            match collision {
                Collision::Left => {
                    if player_vel.0.x > 0.0 {
                        player_vel.0.x = 0.0;
                    }
                    player_tran.translation.x = solid_tran.translation.x - mean_scale.x;
                }
                Collision::Right => {
                    if player_vel.0.x < 0.0 {
                        player_vel.0.x = 0.0;
                    }
                    player_tran.translation.x = solid_tran.translation.x + mean_scale.x;
                }
                Collision::Top => {
                    if player_vel.0.y < 0.0 {
                        player_vel.0.y = 0.0;
                    }
                    player_tran.translation.y = solid_tran.translation.y + mean_scale.y;
                    jump.on_ground = true;
                }
                Collision::Bottom => {
                    if player_vel.0.y > 0.0 {
                        player_vel.0.y = 0.0;
                    }
                    player_tran.translation.y = solid_tran.translation.y - mean_scale.y;
                }
                Collision::Inside => {}
            }
        }
    }
}

fn update_camera_system(
    mut camera_query: Query<(&mut Transform, &WorldCamera), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let (mut camera_transform, _camera) = camera_query.single_mut();
    let player_transform = player_query.single();
    camera_transform.translation = player_transform.translation;
}

fn startup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn()
        .insert(Label("Player".to_string()))
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0., 1., 0.),
                scale: Vec3::new(1., 1.8, 1.),
                ..default()
            },
            sprite: Sprite {
                color: Color::rgb(0., 1., 0.),
                ..default()
            },
            ..default()
        })
        .insert(Velocity(Vec3::ZERO))
        .insert(Player)
        .insert(Gravity(GRAVITY))
        .insert(Mobility {
            walk_speed: 15.,
            // Last factor is peak jump height under normal gravity
            jump_speed: (2. * GRAVITY * 5.8).sqrt(),
            on_ground: false,
            walk_direction: Direction::Neutral,
        });

    for (x, y) in [
        (-5, 0), (-4, 0), (-3, 0), (-2, 0), (-1, 0), (0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0),
        (5, 1), (5, 2), (5, 3), (5, 4), (5, 5), (5, 6), (-5, 1), (-5, 2), (-5, 3), (-5, 4), (-5, 5),
        (-5, 6), (2, 5), (3, 5),
    ] {
        commands.spawn_bundle(tile::SolidTile::from_spec(tile::TileSpec {
            pos: IVec2::new(x, y),
            appearance: tile::TileAppearance::Texture(asset_server.load("tile.png")),
        }));
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            resizable: false,
            mode: WindowMode::BorderlessFullscreen,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PixelPerfectPlugin)
        .add_startup_system(startup_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(INPUT_TIME_STEP as f64))
                .with_system(input_system),
        )
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(PHYSICS_TIME_STEP as f64))
                .with_system(gravity_system.label(PhysicsSystem::Gravity))
                .with_system(
                    physics_system
                        .label(PhysicsSystem::Velocity)
                        .after(PhysicsSystem::Gravity),
                )
                .with_system(
                    player_solid_collision_system
                        .label(PhysicsSystem::Collision)
                        .after(PhysicsSystem::Velocity),
                )
                .with_system(
                    update_camera_system
                        .label(PhysicsSystem::Camera)
                        .after(PhysicsSystem::Collision),
                ),
        )
        .run();
}
