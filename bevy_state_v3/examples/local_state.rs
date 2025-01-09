//! This example shows how to use the most basic local state machines.
//! The machines consists of a single state type that decides
//! whether a logo moves around the screen and changes color.

use bevy::{prelude::*, sprite::Anchor};
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
        .add_systems(Update, bounce_around)
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
                color: Color::oklch(0.5, 0.5, 0.0),
                anchor: Anchor::Center,
                ..default()
            },
            Transform::from_xyz(100.0, 0.0, 0.),
            Velocity(Vec2::splat(300.0)),
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
            color: Color::oklch(0.5, 0.5, 180.0),
            anchor: Anchor::Center,
            ..default()
        },
        Transform::from_xyz(-100.0, 0.0, 0.0),
        Velocity(Vec2::splat(-250.0)),
        ToggleOn(KeyCode::Digit2),
        // This time we add the state directly, by hand.
        LogoState::Enabled.into_data(),
    ));
}

/// Where the logo is going.
#[derive(Component)]
struct Velocity(Vec2);

/// Half of the logo size for collision checking.
const LOGO_HALF_SIZE: Vec2 = Vec2::new(260., 65.);

/// Make the logo bounce.
fn bounce_around(
    mut logos: Populated<
        (
            &StateData<LogoState>,
            &mut Sprite,
            &mut Transform,
            &mut Velocity,
        ),
        With<Sprite>,
    >,
    camera: Single<&Projection>,
    time: Res<Time>,
) {
    let Projection::Orthographic(camera) = &*camera else {
        return;
    };
    let delta = time.delta_secs();
    for (state, mut sprite, mut transform, mut velocity) in logos.iter_mut() {
        // Ignore logos which are in the disabled state.
        if *state.current() == LogoState::Disabled {
            continue;
        }

        transform.translation += velocity.0.extend(0.) * delta;

        let logo_pos = transform.translation.xy();

        // Check if the logo's extents are outside the screen.
        let outside_max = camera.area.max.cmplt(logo_pos + LOGO_HALF_SIZE);
        let outside_min = camera.area.min.cmpgt(logo_pos - LOGO_HALF_SIZE);

        // Clamp the logo to screen edges and reverse velocity if it hits an edge.
        transform.translation = transform
            .translation
            .xy()
            .clamp(
                camera.area.min + LOGO_HALF_SIZE,
                camera.area.max - LOGO_HALF_SIZE,
            )
            .extend(0.0);
        velocity.0 = Vec2::select(outside_max ^ outside_min, -velocity.0, velocity.0);

        if outside_min.any() || outside_max.any() {
            // Rotate hue by golden angle for nice color variation.
            sprite.color = sprite.color.rotate_hue(137.507764);
        }
    }
}
