//! This example shows how to use the most basic global state machine.
//! The machine consists of a single state type that decides whether
//! a logo moves around the screen and changes color on each bounce.
//!
//! Global states are very similar to local states, they too are stored on an entity.
//! This special entity is marked with [`GlobalMarker`](bevy_state_v3::util::GlobalMarker),
//! but all state logic is shared between local and global states.

use bevy::{prelude::*, sprite::Anchor};
use bevy_state_v3::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        // Register machinery for the state type.
        // This is required for both global and local state, but only needs to be called once.
        // By providing an empty config we opt-out of default behavior like state transition and scoped entities.
        .register_state(StateConfig::<LogoState>::empty())
        // The best way to interact with global state is through commands.
        // We can initialize a new global state by not specifying a `local` target.
        .init_state(None, LogoState::Enabled)
        .add_systems(Startup, setup)
        .add_systems(Update, user_input)
        .add_systems(
            Update,
            // States come with run condition that work only(!) for global states.
            move_logo.run_if(in_state(LogoState::Enabled)),
        )
        .run();
}

/// We derive the [`State`] trait.
/// This creates a state with no dependencies, which is non-optional (always exists).
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

/// Create the camera and logo.
fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    println!();
    println!("Press `1` to toggle logo movement.");
    println!();

    // Add camera.
    commands.spawn(Camera2d);

    // Create logo with random position and velocity.
    let texture = assets.load("branding/bevy_logo_dark.png");
    commands.spawn((
        Sprite {
            image: texture,
            color: Color::hsv(0.0, 1.0, 1.0),
            anchor: Anchor::Center,
            ..default()
        },
        Velocity(Vec2::splat(5.0)),
    ));
}

/// Where the logo is going.
#[derive(Component)]
struct Velocity(Vec2);

/// Moves the logo around.
/// The logo bouncess off the screen edges.
/// On each bounce the color changes.
fn move_logo(
    mut logo: Single<(&mut Transform, &mut Velocity, &mut Sprite)>,
    camera: Single<&OrthographicProjection>,
) {
    let camera = camera;
    let (transform, velocity, sprite) = &mut *logo;

    transform.translation += velocity.0.extend(0.);
    let logo_pos = transform.translation.xy();

    // Detect collisions with screen edges.
    const LOGO_HALF_SIZE: Vec2 = Vec2::new(260., 65.);

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

    if flip_x | flip_y {
        // Rotate hue by golden angle for nice color variation.
        sprite.color = sprite.color.rotate_hue(137.507764);
    }
}
