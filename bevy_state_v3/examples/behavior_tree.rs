//! This example shows how state hierarchy can be composed to achieve simple enemy AI.
//! The enemy will start by rotating around, while looking for the player ship.
//! When player ship is found, it will be chased until out of sight, after
//! which the enemy will briefly rest, before looking for player again.

use std::time::Duration;

use bevy::{
    color::palettes::tailwind::{BLUE_300, RED_300},
    prelude::*,
    sprite::Anchor,
};
use bevy_state_v3::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        // Opt-out of default state transitions and state scoped entities.
        .register_state(StateConfig::<Behavior>::empty())
        .register_state(StateConfig::<Chase>::empty())
        .register_state(StateConfig::<Rest>::empty())
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (player_move, enemy_lookout, enemy_chase, enemy_rest).chain(),
        )
        .run();
}

/// Player marker component.
#[derive(Component)]
struct Player;

/// Enemy marker component.
#[derive(Component)]
struct Enemy;

/// Root state of enemy behavior.
#[derive(State, PartialEq, Debug, Clone)]
enum Behavior {
    /// Looking for player, no additional data.
    Lookout,
    /// Chasing player, substate stores the entity.
    Chase,
    /// Resting, substate stores timestamp when rest is over.
    Rest,
}

/// Vision defined by a circle/sphere sector.
#[derive(Component)]
struct Vision {
    /// Minimum value of dot product between `front` and `to_target` vectors to be within the vision angle.
    min_dot_product: f32,
    /// Maximum visibility distance.
    max_distance: f32,
}

impl Vision {
    /// Creates a new vision descriptor.
    fn new(half_angle: f32, max_distance: f32) -> Self {
        Self {
            min_dot_product: half_angle.to_radians().cos(),
            max_distance,
        }
    }

    /// Calculates whether `there` is visible from `here` while looking towards `front`.
    fn is_visible(&self, here: Vec3, front: Dir3, there: Vec3) -> bool {
        let delta = there - here;
        let distance = delta.length();
        let rcp = distance.recip();
        if !rcp.is_finite() || rcp == 0.0 {
            return false;
        }
        let direction = delta * rcp;
        distance < self.max_distance && self.min_dot_product < direction.dot(*front)
    }
}

/// Chase state specifies which entity the enemy is after.
/// In this example it'll always be the player.
#[derive(PartialEq, Debug, Clone)]
struct Chase {
    target: Entity,
}

/// Manually implementing state allows us to add additional requirements.
impl State for Chase {
    type Dependencies = Behavior;
    type Update = Option<Self>;
    type Repr = Option<Self>;

    fn update(
        state: &mut StateData<Self>,
        dependencies: StateSetData<'_, Self::Dependencies>,
    ) -> Self::Repr {
        let behavior = dependencies;
        let current = behavior.current();
        let next = state.update_mut().take();
        // We require ourselves to provide the initial state.
        match (current, next) {
            (Behavior::Chase, next) => {
                Some(next.expect("Changed state to `Chase` without specifying it's parameters."))
            }
            _ => None,
        }
    }
}

/// The resting state remembers when the resting period ends.
/// We want to store a timestamp rather than a timer, to not update the state
/// every frame. Alternatively, we can store the timer in custom [`State::Update`]
/// and keep the state type itself as a Zero Sized Type (ZST).
#[derive(PartialEq, Debug, Clone)]
struct Rest {
    until: Duration,
}

impl State for Rest {
    type Dependencies = Behavior;
    type Update = Option<Self>;
    type Repr = Option<Self>;

    fn update(
        state: &mut StateData<Self>,
        dependencies: StateSetData<'_, Self::Dependencies>,
    ) -> Self::Repr {
        let behavior = dependencies;
        let current = behavior.current();
        let next = state.update_mut().take();
        match (current, next) {
            (Behavior::Rest, next) => {
                Some(next.expect("Changed state to `Rest` without specifying it's parameters."))
            }
            _ => None,
        }
    }
}

fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    println!();
    println!("In this example the player ship moves towards the mouse cursor.");
    println!("The enemies are on the lookout for the player. They will chase him");
    println!("until he's out of sight, then rest a bit and go back to lookout.");
    println!();

    // Add camera.
    commands.spawn(Camera2d);

    // Create the player.
    let texture = assets.load("textures/simplespace/enemy_A.png");
    commands.spawn((
        Sprite {
            image: texture.clone(),
            color: BLUE_300.into(),
            anchor: Anchor::Center,
            ..default()
        },
        Transform::from_xyz(0.0, 200.0, 0.0),
        Player,
    ));

    // Create enemies.
    let texture = assets.load("textures/simplespace/ship_C.png");
    for i in 0..10 {
        let x = i as f32 * 100.0 - 450.0;
        commands.spawn((
            Sprite {
                image: texture.clone(),
                color: RED_300.into(),
                anchor: Anchor::Center,
                ..default()
            },
            Transform::from_xyz(x, -200.0, 0.0),
            Enemy,
            Vision::new(30.0, 400.0),
            // All states are attached directly, without the use of commands.
            Behavior::Lookout.into_data(),
            None::<Chase>.into_data(),
            None::<Rest>.into_data(),
        ));
    }
}

/// Time it takes to travel ~63.2% of the way towards the cursor.
const PLAYER_MOVE_TIME_CONSTANT: f32 = 0.3;

/// Player movement uses exponential smoothing between the player position and the cursor position.
/// https://en.wikipedia.org/wiki/Exponential_smoothing#Time_constant
fn player_move(
    mut transform: Single<&mut Transform, With<Player>>,
    window: Single<&Window>,
    camera: Single<(&Camera, &GlobalTransform)>,
    time: Res<Time>,
) {
    let Some(position) = window.cursor_position() else {
        return;
    };
    let Ok(ray) = camera.0.viewport_to_world(camera.1, position) else {
        return;
    };
    let target = ray.origin.with_z(0.0);
    let smoothing_factor = time.delta_secs() / PLAYER_MOVE_TIME_CONSTANT;
    let new_transform =
        smoothing_factor * target + (1.0 - smoothing_factor) * transform.translation;

    if (new_transform - transform.translation).length() > 0.1 {
        transform.translation = new_transform;
        look_to(&mut transform, target);
    }
}

/// How fast the enemy rotates in radians per second.
const ENEMY_ROTATION_SPEED: f32 = 1.0;

/// Rotate enemies and start chasing player if in sight.
fn enemy_lookout(
    mut enemies: Populated<
        (
            &mut Transform,
            &Vision,
            &mut StateData<Behavior>,
            &mut StateData<Chase>,
        ),
        (With<Enemy>, Without<Player>),
    >,
    player: Single<(Entity, &Transform), With<Player>>,
    time: Res<Time>,
) {
    let (player_entity, player_transform) = *player;
    let delta = time.delta_secs();

    for (mut transform, vision, mut behavior, mut chase) in enemies.iter_mut() {
        let Behavior::Lookout = behavior.current() else {
            continue;
        };

        transform.rotate_z(ENEMY_ROTATION_SPEED * delta);

        if vision.is_visible(
            transform.translation,
            transform.up(),
            player_transform.translation,
        ) {
            *behavior.update_mut() = Some(Behavior::Chase);
            *chase.update_mut() = Some(Chase {
                target: player_entity,
            })
        }
    }
}

/// The closest distance enemy will stay at from the target.
const CHASE_MIN_DISTANCE: f32 = 50.0;

/// How quickly can the enemy move per second.
const CHASE_SPEED: f32 = 100.0;

/// Chasing of the target entity.
/// The enemy will stop chasing and rest if:
/// - entity stops existing,
/// - entity gets out of sight.
fn enemy_chase(
    mut queries: ParamSet<(
        (
            Populated<(Entity, &StateData<Chase>), With<Enemy>>,
            Populated<&Transform>,
        ),
        Populated<(
            &mut Transform,
            &Vision,
            &mut StateData<Behavior>,
            &mut StateData<Rest>,
        )>,
    )>,
    time: Res<Time>,
) {
    let delta = time.delta_secs();
    let now = time.elapsed();

    let (enemies, transforms) = queries.p0();
    let mut targets = vec![];
    for (entity, chase) in enemies.iter() {
        let Some(Chase { target }) = *chase.current() else {
            continue;
        };
        let maybe_target = transforms.get(target).ok().map(|t| t.translation);
        targets.push((entity, maybe_target));
    }

    let mut enemies = queries.p1();
    for (entity, maybe_target) in targets {
        let (mut transform, vision, mut behavior, mut rest) = enemies.get_mut(entity).unwrap();

        // Rest if target no longer exists.
        let Some(target) = maybe_target else {
            *behavior.update_mut() = Some(Behavior::Rest);
            *rest.update_mut() = Some(Rest {
                until: now + Duration::from_secs(2),
            });
            continue;
        };

        // Rest if target is out of sight.
        if !vision.is_visible(transform.translation, transform.up(), target) {
            *behavior.update_mut() = Some(Behavior::Rest);
            *rest.update_mut() = Some(Rest {
                until: now + Duration::from_secs(2),
            });
            continue;
        }

        // Move and rotate towards target.
        let offset = target - transform.translation;
        let distance = offset.length();
        let rcp = distance.recip();
        if rcp.is_finite() && rcp > 0.0 {
            let direction = offset / distance;
            let step = (distance - CHASE_MIN_DISTANCE).min(CHASE_SPEED * delta);
            transform.translation += direction * step;
        }

        look_to(&mut transform, target);
    }
}

/// After resting enemies go back to lookout.
fn enemy_rest(
    mut enemies: Populated<(&mut StateData<Behavior>, &StateData<Rest>), With<Enemy>>,
    time: Res<Time>,
) {
    let now = time.elapsed();

    for (mut behavior, rest) in enemies.iter_mut() {
        let Some(Rest { until }) = *rest.current() else {
            continue;
        };

        if until < now {
            *behavior.update_mut() = Some(Behavior::Lookout);
        }
    }
}

/// Helper method for setting rotation based on a target position.
fn look_to(transform: &mut Transform, target: Vec3) {
    let Some(front) = (target - transform.translation).try_normalize() else {
        return;
    };
    transform.rotation = Quat::from_mat3(&Mat3 {
        x_axis: -front.cross(Vec3::Z),
        y_axis: front,
        z_axis: -Vec3::Z,
    });
}
