use bevy::{
    core::FixedTimestep,
    prelude::*,
    app::{AppExit},
    sprite::collide_aabb::{collide, Collision},
};

const TIME_STEP: f32 = 1.0 / 60.0;

#[derive(Component)]
struct Label(String);

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Gravity(f32);

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Mobility {
    on_ground: bool,
    jump_speed: f32,
    walk_speed: f32,
}

#[derive(Component)]
struct SolidCollider;

#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
enum PhysicsSystem {
    Gravity,
    Input,  // TODO move out of physics
    Velocity,
    Collision,
}


fn physics_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * TIME_STEP;
    }
}

fn gravity_system(mut query: Query<(&mut Velocity, &Gravity)>) {
    for (mut velocity, gravity) in query.iter_mut() {
        velocity.0.y -= gravity.0 * TIME_STEP;
    }
}

fn input_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Velocity, &Player, &mut Mobility)>,
    mut app_exit_events: EventWriter<AppExit>,
) {
    let (mut velocity, player, mut mobility) = query.single_mut();

    let mut x = 0.0;
    if keyboard_input.pressed(KeyCode::A) {
        x -= 1.0;
    }
    if keyboard_input.pressed(KeyCode::D) {
        x += 1.0;
    }
    velocity.0.x = mobility.walk_speed * x;

    if keyboard_input.pressed(KeyCode::Space) {
        if mobility.on_ground {
            mobility.on_ground = false;
            velocity.0.y = mobility.jump_speed;
        }
    }

    if keyboard_input.pressed(KeyCode::Escape) {
        app_exit_events.send(AppExit);
    }
}

fn player_solid_collision_system(
    mut player_query: Query<(&mut Velocity, &mut Transform, &Player, &mut Mobility)>,
    collider_query: Query<(&Transform, &SolidCollider), Without<Player>>,
) {
    let (mut player_vel, mut player_tran, player, mut jump) = player_query.single_mut();
    for (solid_tran, solid_collider) in collider_query.iter() {
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
            }
        }
    }
}

fn debug_overlay_system(query: Query<(&Transform, &Velocity, &Player)>) {
    let (transform, velocity, player) = query.single();
    println!("{}: {}", "Player vel", velocity.0);
}

fn startup_system(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn()
        .insert(Label("Player".to_string()))
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: Vec3::new(20.0, 40.0, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.0, 1.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Velocity(Vec3::ZERO))
        .insert(Player)
        .insert(Gravity(1000.0))
        .insert(Mobility {
            walk_speed: 300.0,
            jump_speed: 500.0,
            on_ground: false,
        });

    commands
        .spawn()
        .insert(Label("Floor".to_string()))
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, -200.0, 0.0),
                scale: Vec3::new(500.0, 20.0, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.0, 1.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(SolidCollider);

    commands
        .spawn()
        .insert(Label("Platform".to_string()))
        .insert_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(250.0, 0.0, 0.0),
                scale: Vec3::new(20.0, 400.0, 1.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::rgb(0.0, 1.0, 1.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(SolidCollider);

}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(startup_system)
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(TIME_STEP as f64))
                .with_system(
                    gravity_system
                        .label(PhysicsSystem::Gravity)
                )
                .with_system(
                    input_system
                        .label(PhysicsSystem::Input)
                        .after(PhysicsSystem::Gravity)
                )
                .with_system(
                    physics_system
                        .label(PhysicsSystem::Velocity)
                        .after(PhysicsSystem::Input)
                )
                .with_system(
                    player_solid_collision_system
                        .label(PhysicsSystem::Collision)
                        .after(PhysicsSystem::Velocity)
                )
        )
        .run();
}
