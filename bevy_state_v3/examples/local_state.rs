//! This example shows how to use the most basic local state machines.
//! The machines consists of a single state type that decides
//! whether a logo moves around the screen and changes color.

use bevy::{
    color::palettes::tailwind::{BLUE_600, RED_600},
    prelude::*,
    sprite::Anchor,
};
use bevy_state_v3::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        // Register machinery for the state.
        // This is required for both global and local state, but only needs to be called once.
        // By providing an empty config we opt-out of state transition events.
        .register_state::<LogoState>(StateConfig::empty())
        .add_systems(Startup, setup)
        .add_systems(Update, user_input)
        // Because we are using local state, we cannot use global state to control whether the systems should run.
        // Each entity has to check it's own state and make the decision.
        .add_systems(Update, (bounce_around, cycle_color))
        .run();
}

#[derive(State, PartialEq, Debug, Clone)]
enum LogoState {
    Enabled,
    Disabled,
}

/// User controls.
fn user_input(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    logos: Populated<(Entity, &StateData<LogoState>, &ToggleOn)>,
) {
    for (entity, state, toggle_on) in logos.iter() {
        if input.just_pressed(toggle_on.0) {
            // Decide the next state based on current state.
            let next = match state.current() {
                LogoState::Enabled => LogoState::Disabled,
                LogoState::Disabled => LogoState::Enabled,
            };
            // Request a change for the state.
            // We target a specific entity to update a local state machine.
            commands.update_state(Some(entity), next);
            // We could also directly access `state.update` and set the next value,
            // since we're already querrying for it.
        }
    }
}

/// Half of the logo size for collision checking.
const LOGO_HALF_SIZE: Vec2 = Vec2::new(260., 65.);

/// Where the logo is going.
#[derive(Component)]
struct Velocity(Vec2);

/// Which key toggles the logo.
#[derive(Component)]
struct ToggleOn(KeyCode);

/// Create the camera and logo.
fn setup(mut commands: Commands, assets: Res<AssetServer>) {
    // Add camera.
    commands.spawn(Camera2d);

    // Create logo with random position and velocity.
    let texture = assets.load("branding/bevy_logo_dark.png");
    let entity = commands
        .spawn((
            Sprite {
                image: texture,
                color: RED_600.into(),
                anchor: Anchor::Center,
                ..default()
            },
            Transform::from_xyz(100.0, 0.0, 0.),
            Velocity(Vec2::splat(3.0)),
            ToggleOn(KeyCode::Digit1),
        ))
        .id();
    // Attach state to a local entity.
    commands.init_state(Some(entity), LogoState::Enabled);

    // Create another logo with random position and velocity.
    let texture = assets.load("branding/bevy_logo_dark.png");
    commands.spawn((
        Sprite {
            image: texture,
            color: BLUE_600.into(),
            anchor: Anchor::Center,
            ..default()
        },
        Transform::from_xyz(-100.0, 0.0, 0.0),
        Velocity(Vec2::splat(-2.0)),
        ToggleOn(KeyCode::Digit2),
        // This time we add the state directly, by hand.
        LogoState::Enabled.into_data(),
    ));
}

/// Make the logo bounce.
fn bounce_around(
    mut logos: Populated<(&StateData<LogoState>, &mut Transform, &mut Velocity), With<Sprite>>,
    camera: Single<&OrthographicProjection>,
) {
    let camera = camera;
    for (state, mut transform, mut velocity) in logos.iter_mut() {
        // Ignore logos which are in the disabled state.
        if *state.current() == LogoState::Disabled {
            continue;
        }

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
fn cycle_color(mut logos: Populated<(&StateData<LogoState>, &mut Sprite), With<Sprite>>) {
    for (state, mut sprite) in logos.iter_mut() {
        // Ignore logos which are in the disabled state.
        if *state.current() == LogoState::Disabled {
            continue;
        }

        sprite.color = sprite.color.rotate_hue(0.3);
    }
}
