//! This example shows how to use the most basic global state machine.
//! The machine consists of a single state type that decides
//! whether a logo moves around the screen and changes color.

use bevy::{prelude::*, sprite::Anchor};
use bevy_state_v3::prelude::*;
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        // Register machinery for the state.
        // This is required for both global and local state, but only needs to be called once.
        // By providing an empty config we opt-out of state transition events.
        .register_state::<LogoState>(StateConfig::empty())
        // By targeting no specific entity, we create a global state.
        // We provide the initial state value.
        // Because we're not using transition events or state hierarchy, update suppresion doesn't matter.
        .init_state(None, LogoState::Enabled)
        .add_systems(Startup, setup)
        .add_systems(Update, user_input)
        .add_systems(
            Update,
            // We can use global state to determine when certain systems run.
            (bounce_around, cycle_color).run_if(in_state(LogoState::Enabled)),
        )
        .run();
}

#[derive(State, Default, PartialEq, Debug, Clone)]
enum LogoState {
    #[default]
    Enabled,
    Disabled,
}

/// User controls.
fn user_input(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    state: Global<&StateData<LogoState>>,
) {
    if input.just_pressed(KeyCode::Digit1) {
        // Decide the next state based on current state.
        let next = match state.current() {
            LogoState::Enabled => LogoState::Disabled,
            LogoState::Disabled => LogoState::Enabled,
        };
        // Request a change for the state.
        commands.update_state(None, next);
    }
}

/// Half of the logo size for collision checking.
const LOGO_HALF_SIZE: Vec2 = Vec2::new(260., 65.);

/// Where the logo is going.
#[derive(Component)]
struct Velocity(Vec2);

/// Create the camera and logo.
fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    // Add camera.
    commands.spawn(Camera2d);

    // Create logo with random position and velocity.
    let mut rng = rand::thread_rng();
    let texture = assets.load("branding/bevy_logo_dark.png");
    commands.spawn((
        Sprite {
            image: texture,
            color: Color::hsv(rng.gen_range(0.0..=1.0), 1.0, 1.0),
            anchor: Anchor::Center,
            ..default()
        },
        Transform::from_xyz(
            rng.gen_range(-200.0..=200.),
            rng.gen_range(-200.0..=200.),
            0.,
        ),
        Velocity(Dir2::from_rng(&mut rng) * rng.gen_range(0.0..=10.)),
    ));
}

/// Make the logo bounce.
fn bounce_around(
    mut logos: Populated<(&mut Transform, &mut Velocity), With<Sprite>>,
    camera: Single<&OrthographicProjection>,
) {
    let camera = camera;
    for (mut transform, mut velocity) in logos.iter_mut() {
        transform.translation += velocity.0.extend(0.);
        let logo_pos = transform.translation.xy();

        let mut flip_x = false;
        let x_max = camera.area.max.x - LOGO_HALF_SIZE.x;
        if x_max < logo_pos.x {
            transform.translation.x = x_max;
            flip_x = !flip_x;
        }
        let x_min = camera.area.min.x + LOGO_HALF_SIZE.x;
        if logo_pos.x < x_min {
            transform.translation.x = x_min;
            flip_x = !flip_x;
        }
        if flip_x {
            velocity.0.x *= -1.;
        }

        let mut flip_y = false;
        let y_max = camera.area.max.y - LOGO_HALF_SIZE.y;
        if y_max < logo_pos.y {
            transform.translation.y = y_max;
            flip_y = !flip_y;
        }
        let y_min = camera.area.min.y + LOGO_HALF_SIZE.y;
        if logo_pos.y < y_min {
            transform.translation.y = y_min;
            flip_y = !flip_y;
        }
        if flip_y {
            velocity.0.y *= -1.;
        }
    }
}

/// Make the logo rainbow.
fn cycle_color(mut logos: Populated<&mut Sprite, With<Sprite>>) {
    for mut sprite in logos.iter_mut() {
        sprite.color = sprite.color.rotate_hue(0.3);
    }
}
