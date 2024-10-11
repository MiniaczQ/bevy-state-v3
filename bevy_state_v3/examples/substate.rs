//! This example shows how to use hierarchy of two states: root and it's substate.

use bevy::{
    color::palettes::tailwind::{GRAY_300, GREEN_400, YELLOW_200},
    prelude::*,
};
use bevy_state_v3::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        .register_state(StateConfig::<InnerState>::empty())
        .register_state(StateConfig::<OuterState>::empty())
        .init_state(None, InnerState::Enabled)
        .init_state(None, Some(OuterState::Enabled))
        .add_systems(Startup, setup)
        .add_systems(Update, user_input)
        .add_systems(
            Update,
            (
                orbit_filtered::<InnerCircle>.run_if(in_state(InnerState::Enabled)),
                orbit_filtered::<OuterCircle>.run_if(in_state(Some(OuterState::Enabled))),
            ),
        )
        .run();
}

#[derive(State, Default, PartialEq, Debug, Clone)]
enum InnerState {
    #[default]
    Enabled,
    Disabled,
}

#[derive(State, Default, PartialEq, Debug, Clone)]
#[dependency(InnerState = InnerState::Enabled)]
enum OuterState {
    #[default]
    Enabled,
    Disabled,
}

/// User controls.
fn user_input(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    state: Global<(&StateData<InnerState>, &StateData<OuterState>)>,
) {
    let (logo_state, cycle_color_state) = *state;

    if input.just_pressed(KeyCode::Digit1) {
        let next = match logo_state.current() {
            InnerState::Enabled => InnerState::Disabled,
            InnerState::Disabled => InnerState::Enabled,
        };
        commands.update_state(None, next);
    }

    if input.just_pressed(KeyCode::Digit2) {
        if let Some(state) = cycle_color_state.current() {
            let next = match state {
                OuterState::Enabled => OuterState::Disabled,
                OuterState::Disabled => OuterState::Enabled,
            };
            commands.update_state(None, next);
        };
    }
}

/// Component for orbiting another entity.
#[derive(Component)]
struct OrbitEntity {
    parent: Entity,
    distance: f32,
    speed: f32,
    angle: f32,
}

impl OrbitEntity {
    pub fn new(parent: Entity, distance: f32, speed: f32, angle: f32) -> Self {
        Self {
            parent,
            distance,
            speed,
            angle,
        }
    }
}

// Marker components for filtering inner and outer circles.

#[derive(Component)]
struct InnerCircle;

#[derive(Component)]
struct OuterCircle;

/// Create the camera and circles.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    println!();
    println!("Press 1 to toggle motion of both outer circles.");
    println!("Press 2 to toggle motion of only the most outer circle.");
    println!();

    // Add camera.
    commands.spawn(Camera2d::default());

    // Add 3 circles:
    // - immovable center circle,
    // - inner circle that orbits the immovable circle,
    // - outer circle that orbits the inner circle.
    let innest = commands
        .spawn((
            Mesh2d(meshes.add(Circle::new(100.0))),
            MeshMaterial2d(materials.add(Color::from(YELLOW_200))),
        ))
        .id();
    let inner = commands
        .spawn((
            Mesh2d(meshes.add(Circle::new(20.0))),
            MeshMaterial2d(materials.add(Color::from(GREEN_400))),
            OrbitEntity::new(innest, 200.0, 2.0, 0.0),
            InnerCircle,
        ))
        .id();
    commands.spawn((
        Mesh2d(meshes.add(Circle::new(10.0))),
        MeshMaterial2d(materials.add(Color::from(GRAY_300))),
        OrbitEntity::new(inner, 40.0, 5.0, 0.0),
        OuterCircle,
    ));
}

/// Makes the filtered entity orbit it's parent.
fn orbit_filtered<M: Component>(
    mut queries: ParamSet<(
        Single<(&mut Transform, &mut OrbitEntity), With<M>>,
        Populated<&Transform>,
    )>,
    time: Res<Time>,
) {
    let parent = queries.p0().1.parent;
    let center = queries.p1().get(parent).unwrap().translation;
    let delta = time.delta_seconds();
    let (transform, orbit) = &mut *queries.p0();
    orbit.angle = (orbit.angle + orbit.speed * delta) % core::f32::consts::TAU;
    let offset = Quat::from_axis_angle(Vec3::Z, orbit.angle) * Vec3::new(orbit.distance, 0.0, 0.0);
    transform.translation = center + offset;
}
