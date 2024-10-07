//! In this example states are used to model a simple behavioral tree of enemy entities.
//! The enemies can either stand and look around or move to selected position.
//! To increase performance, we opt-out of command-based updates and resort to manually setting it.

use bevy::{prelude::*, sprite::Anchor};
use bevy_state_v3::prelude::*;
use rand::{seq::SliceRandom, thread_rng, Rng, RngCore};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        // We opt-out of default behaviors like state transition events or scoped entities.
        .register_state::<BehaviorState>(StateConfig::empty())
        .register_state::<StandingState>(StateConfig::empty())
        .register_state::<MovingState>(StateConfig::empty())
        .add_systems(Startup, setup_enemies)
        .add_systems(Update, (enemies_standing, enemies_moving).chain())
        .run();
}

/// Marker for our enemies.
#[derive(Component)]
struct Enemy;

/// Root behavior state.
/// An enemy entity will stand and look around, once they find another enemy
/// entity, they'll try to move to the position in which they've seen them.
#[derive(State, PartialEq, Debug, Clone)]
enum BehaviorState {
    /// Looking around for other enemies.
    Standing,
    /// Moving to a target position.
    Moving,
}

/// Persistent update similar to that in `persistent_substate` example.
/// The main difference is focus on manually setting updates rather than using a command.
#[derive(Debug, Default)]
struct PersistentUpdate<S: State> {
    should_update: bool,
    value: S,
}

impl<S: State + Default> StateUpdate for PersistentUpdate<S> {
    fn should_update(&self) -> bool {
        self.should_update
    }

    fn post_update(&mut self) {
        self.should_update = false;
    }
}

impl<S: State> PersistentUpdate<S> {
    /// Sets update with provided state.
    pub fn set(&mut self, value: S) {
        self.should_update = true;
        self.value = value;
    }
}

/// Looking around with specific speed.
/// If another enemy is spotted, the state will change to moving.
#[derive(Default, PartialEq, Debug, Clone)]
struct StandingState {
    /// Rotation speed.
    looking_speed: f32,
    /// Size of the vision cone represented by cosine of the angle.
    vision_cos: f32,
}

impl State for StandingState {
    type Dependencies = BehaviorState;
    // By using a persistent update instead of `Option<S>` we ensure that
    // there is always a valid substate value we can use.
    type Update = PersistentUpdate<Self>;
    type Repr = Option<Self>;

    fn update(state: &mut StateData<Self>, behavior: StateDependencies<'_, Self>) -> Self::Repr {
        match behavior.current() {
            BehaviorState::Standing => Some(state.update().value.clone()),
            _ => None,
        }
    }
}

impl StandingState {
    /// Helper for creating random states.
    pub fn from_rng(rng: &mut dyn RngCore) -> Self {
        Self {
            looking_speed: rng.gen_range(0.2..=1.0) * [-1.0, 1.0].choose(rng).unwrap(),
            vision_cos: rng.gen_range(0.99..=0.999),
        }
    }
}

/// Moving towards target position.
/// Once at target position, go back to standing still and looking around.
#[derive(Default, PartialEq, Debug, Clone)]
struct MovingState {
    /// Target position.
    target: Vec2,
    /// Speed of movement.
    speed: f32,
}

impl State for MovingState {
    type Dependencies = BehaviorState;
    type Update = PersistentUpdate<Self>;
    type Repr = Option<Self>;

    fn update(state: &mut StateData<Self>, behavior: StateDependencies<'_, Self>) -> Self::Repr {
        match behavior.current() {
            BehaviorState::Moving => Some(state.update().value.clone()),
            _ => None,
        }
    }
}

impl MovingState {
    /// Helper for creating random states.
    pub fn from_rng(rng: &mut dyn RngCore, target: Vec2) -> Self {
        Self {
            target: target + Vec2::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)),
            speed: rng.gen_range(10.0..=30.0),
        }
    }
}

fn setup_enemies(mut commands: Commands, assets: Res<AssetServer>) {
    println!();
    println!("There is no human input in this example.");
    println!("The enemy ships will either look around for other ships");
    println!("or move to a location where they spoted a ship.");
    println!();

    // Add camera.
    commands.spawn(Camera2d);

    // Create enemies.
    let enemy_count = 500;
    let mut rng = thread_rng();
    let texture = assets.load("textures/simplespace/ship_C.png");
    for i in 0..enemy_count {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Hsla::hsl(i as f32 * 360.0 / enemy_count as f32, 0.5, 0.5).into(),
                    anchor: Anchor::Center,
                    ..default()
                },
                texture: texture.clone(),
                transform: Transform::from_xyz(
                    rng.gen_range(-1000.0..=1000.0),
                    rng.gen_range(-600.0..=600.0),
                    0.0,
                )
                .looking_to(Vec3::Z, Dir2::from_rng(&mut rng).extend(0.0)),
                ..default()
            },
            Enemy,
            // All states are attached directly.
            BehaviorState::Standing.into_data(),
            Some(StandingState::from_rng(&mut rng)).into_data(),
            None::<MovingState>.into_data(),
        ));
    }
}

/// Reservoir sampling allows us to select one random sample from an arbitrarly large set.
/// In this example, it is used to target one enemy out of many in the vision cone.
struct ReservoirSampler<'r, S> {
    rng: &'r mut dyn RngCore,
    sample: Option<S>,
    weight_sum: f32,
}

impl<'r, S> ReservoirSampler<'r, S> {
    /// Creates a new sampler with provided RNG.
    pub fn new(rng: &'r mut dyn RngCore) -> Self {
        Self {
            rng,
            sample: None,
            weight_sum: 0.0,
        }
    }

    /// Adds a single sample to the reservoir.
    /// The `weight` specifies how likely a sample is to be picked compared to other weights.
    pub fn add(&mut self, sample: S, weight: f32) {
        if self.sample.is_none() {
            // If this is the first sample, always select it.
            self.sample = Some(sample);
            self.weight_sum = weight;
        } else {
            // Every time a sample is added, we decide whether to pick it over the currently selected sample.
            self.weight_sum += weight;
            if self.rng.gen_bool((weight / self.weight_sum) as f64) {
                self.sample = Some(sample);
            }
        }
    }

    /// Consumes the reservoir and returns the sample.
    /// If no samples were added, this returns [`None`].
    pub fn take(self) -> Option<S> {
        self.sample
    }
}

/// Rotates standing enemies and makes them move once they select a target.
fn enemies_standing(
    mut queries: ParamSet<(
        Populated<(&mut Transform, &StateData<StandingState>), With<Enemy>>,
        Populated<(Entity, &Transform, &StateData<StandingState>), With<Enemy>>,
        Populated<(&mut StateData<BehaviorState>, &mut StateData<MovingState>)>,
    )>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    let mut rng = thread_rng();

    // First we rotate all standing enemies.
    let mut query = queries.p0();
    for (mut transform, state) in query.iter_mut() {
        if let Some(state) = state.current() {
            transform.rotation *= Quat::from_axis_angle(Vec3::Z, state.looking_speed * delta);
        }
    }

    // Then we detect which standing enemies see other enemies.
    let query = queries.p1();
    let mut updates = vec![];
    for (search, search_trs, state) in query.iter() {
        let Some(state) = state.current() else {
            continue;
        };
        let mut reservoir = ReservoirSampler::new(&mut rng);
        for (target, target_trs, _) in query.iter() {
            if search == target {
                continue;
            }
            let front = search_trs.up();
            let offset = target_trs.translation - search_trs.translation;
            let distance = offset.length();
            if distance < 1.0 {
                continue;
            }
            let direction = offset / distance;
            let cos = front.dot(direction);
            if state.vision_cos < cos {
                // Every enemy within vision is added to the reservoir to be potentially picked.
                reservoir.add(target_trs.translation.xy(), 1.0);
            }
        }
        // If found, the selected enemy's position becomes the target.
        if let Some(target) = reservoir.take() {
            updates.push((search, target));
        }
    }

    // Finally, to reduce command calling overhead, we set the state update manually.
    let mut query = queries.p2();
    for (entity, target) in updates {
        let (mut behavior, mut moving) = query.get_mut(entity).unwrap();
        *behavior.update_mut() = Some(BehaviorState::Moving);
        moving
            .update_mut()
            .set(MovingState::from_rng(&mut rng, target));
    }
}

/// Moves moving enemies until they reach their target position.
fn enemies_moving(
    mut queries: ParamSet<(
        Populated<(Entity, &mut Transform, &StateData<MovingState>), With<Enemy>>,
        Populated<(&mut StateData<BehaviorState>, &mut StateData<StandingState>)>,
    )>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    let mut rng = thread_rng();
    let mut query = queries.p0();
    let mut updates = vec![];
    for (entity, mut transform, state) in query.iter_mut() {
        if let Some(state) = state.current() {
            let offset = state.target.extend(0.0) - transform.translation;
            let distance = offset.length();
            let direction = offset / distance;
            let max_step = delta * state.speed;
            transform.rotation = Quat::from_mat3(&Mat3::from_cols(
                -direction.cross(Vec3::Z),
                direction,
                -Vec3::Z,
            ));
            transform.translation += direction * max_step.min(distance);

            if distance <= max_step {
                updates.push(entity);
            }
        }
    }

    // Again setting state update manually.
    let mut query = queries.p1();
    for entity in updates {
        let (mut behavior, mut moving) = query.get_mut(entity).unwrap();
        *behavior.update_mut() = Some(BehaviorState::Standing);
        moving.update_mut().set(StandingState::from_rng(&mut rng));
    }
}
