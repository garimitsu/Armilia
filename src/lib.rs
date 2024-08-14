use avian2d::PhysicsPlugins;
use avian2d::prelude::*;
use bevy::prelude::*;

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins {});
        app.add_plugins(PhysicsPlugins::default());
        app.add_plugins(PhysicsDebugPlugin::default());
        app.insert_resource(Gravity(Vec2::ZERO));
        app.add_plugins(startup);
        app.add_systems(FixedUpdate, player_controller);
        app.add_systems(FixedUpdate, decay_system);
        app.add_systems(FixedUpdate, hitbox_system);
        app.observe(spawn_enemy);
    }
}

fn startup(app: &mut App) {
    app.add_systems(Startup, spawn_camera);
    app.add_systems(Startup, spawn_player);
    app.add_systems(Startup, setup);
}

fn setup(mut commands: Commands) {
    commands.trigger(SpawnEnemy {});
}

/// Spawn default camera.
/// TODO: Placeholder
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Camera"),
        Camera2dBundle::default(),
        // Render all UI to this camera.
        IsDefaultUiCamera,
    ));
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Hurtbox;

#[derive(Component)]
struct Hitbox;


mod layer {
    use avian2d::prelude::*;

    #[derive(PhysicsLayer, Default)]
    pub enum AvianLayers {
        #[default]
        Physics,
        Hurt,
        Hit
    }

    use AvianLayers::*;

    pub fn hitbox() -> CollisionLayers {
        CollisionLayers::new(Hit, Hurt)
    }

    pub fn hurtbox() -> CollisionLayers {
        CollisionLayers::new(Hurt, Hit)
    }

    pub fn physics() -> CollisionLayers {
        CollisionLayers::new(Physics, Physics)
    }

}

fn hitbox_system(hitboxes: Query<(&Hitbox, &CollidingEntities)>, hurtboxes: Query<&Parent, With<Hurtbox>>, mut commands: Commands) {
    for (_, collisions) in hitboxes.iter() {
        for c in &collisions.0 {
            let mut entity = *c;
            while let Ok(p) = hurtboxes.get(entity) {
                entity = p.get();
            }
            if let Some(ec) = commands.get_entity(entity) {
                ec.despawn_recursive();
            }
        }
    }
}

#[derive(Event)]
struct SpawnEnemy;

fn spawn_enemy(_: Trigger<SpawnEnemy>, mut commands: Commands) {
    let hurtbox = commands.spawn((Collider::circle(14.), Transform::default(), Sensor::default(), Hurtbox {}, layer::hurtbox())).id();
    let mut enemy = commands.spawn( (Collider::rectangle(32., 32.), RigidBody::Kinematic, Transform::from_translation(Vec3::new(0., 128., 0.)), layer::physics()));
    enemy.add_child(hurtbox);
}

fn spawn_player(mut commands: Commands) {
    let e = commands.spawn((Collider::circle(4.), Transform::from_translation(Vec3::new(0., 12., 0.)))).id();
    let mut p = commands.spawn(
        (Transform::from_rotation(Quat::from_rotation_z(45f32.to_radians())),
            LockedAxes::ROTATION_LOCKED,
        RigidBody::Dynamic,
            Collider::rectangle(32., 32.),
            Player {}, layer::physics()
        )
    );
    p.add_child(e);
}

const PLAYER_SPEED : f32 = 100.;

fn player_controller(mut player: Query<(Entity, &mut LinearVelocity, &mut Rotation), With<Player>>, input: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if input.just_pressed(KeyCode::KeyR) {
        commands.trigger(SpawnEnemy);
        return;
    }


    let mut movement = Vec2::ZERO;
    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        movement.y += 1.;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        movement.x -= 1.;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        movement.y -= 1.;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        movement.x += 1.;
    }

    if movement.length() > 0. {
        movement = movement.normalize();
    }

    if let Ok((entity, mut velocity, mut rotation)) = player.get_single_mut() {
        velocity.0 = movement * PLAYER_SPEED;
        if movement.length() > 0. {
            let mut angle = Vec3::new(0.,1.,0.).angle_between(movement.extend(0.)).to_degrees();
            if movement.x > 0. { angle = 360.-angle }
            *rotation = Rotation::degrees(angle);
        }
        if input.just_pressed(KeyCode::KeyF) {
            let e = commands.spawn((
                Transform::from_translation(Vec3::new(0.,32.,0.)),
                Collider::rectangle(8.,32.),
                Sensor::default(),
                Decay { counter: 32},
                Hitbox {},
                layer::hitbox()
            )).id();
            commands.entity(entity).add_child(e);
        }

    }
}

#[derive(Component)]
struct Decay {
    pub counter: usize
}

fn decay_system(mut query: Query<(Entity,&mut Decay)>, mut commands: Commands) {
    for (e, mut decay) in query.iter_mut() {
        decay.counter -= 1;
        if decay.counter == 0 {
            commands.entity(e).despawn();
        }
    }
}