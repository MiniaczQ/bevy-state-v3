//! This example shows how to use state scoped entities.
//! State scoped entities disappear when their selected state is exited.
//! They are created by marking any entity with the [`StateScoped`] component.
//! For this to work correctly, the state we use must be appropriately configured.

use bevy::{prelude::*, sprite::Anchor};
use bevy_state_v3::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        // Enable (despawning of) state scoped entities.
        .register_state::<MyState>(StateConfig::empty().with_state_scoped(true))
        .init_state(None, MyState::Spawning)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                user_input,
                spawn_logos.run_if(in_state(MyState::Spawning)),
                bounce_around,
            )
                .chain(),
        )
        .run();
}

#[derive(State, Default, PartialEq, Debug, Clone)]
enum MyState {
    #[default]
    Spawning,
    Existing,
    Disabled,
}

/// User controls.
fn user_input(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    state: Global<&StateData<MyState>>,
) {
    if input.just_pressed(KeyCode::Space) {
        match state.current() {
            MyState::Spawning => commands.update_state(None, MyState::Existing),
            MyState::Existing => commands.update_state(None, MyState::Disabled),
            MyState::Disabled => commands.update_state(None, MyState::Spawning),
        };
    }
}

/// Half of the logo size for collision checking.
const LOGO_HALF_SIZE: Vec2 = Vec2::new(260., 65.);

/// Where the logo is going.
#[derive(Component)]
struct Velocity(Vec2);

/// Create the camera and logo.
fn setup(mut commands: Commands) {
    println!();
    println!("Press SPACE to cycle between");
    println!("- spawning entities,");
    println!("- keeping entities,");
    println!("- removing entities.");
    println!();

    // Add camera.
    commands.spawn(Camera2d);
}

/// Spawns a new logo every 3 seconds.
fn spawn_logos(
    mut commands: Commands,
    assets: Res<AssetServer>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
    mut index: Local<u32>,
) {
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(0.3, TimerMode::Repeating));
    timer.tick(time.delta());

    if !timer.finished() {
        return;
    }

    // Create logo with random position and velocity.
    let t = *index as f32;
    let angle = t * 137.507764;
    let texture = assets.load("branding/bevy_logo_dark.png");
    commands.spawn((
        Sprite {
            image: texture,
            color: Color::oklch(0.5, 0.5, angle),
            anchor: Anchor::Center,
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 0.0),
        Velocity(Vec2::from(angle.to_radians().sin_cos()) * 300.0),
        StateScoped::<MyState>(MyState::Existing),
    ));
    *index += 1;
}

/// Make the logo bounce.
fn bounce_around(
    mut logos: Populated<(&mut Sprite, &mut Transform, &mut Velocity), With<Sprite>>,
    camera: Single<&Projection>,
    time: Res<Time>,
) {
    let Projection::Orthographic(camera) = &*camera else {
        return;
    };
    let delta = time.delta_secs();
    for (mut sprite, mut transform, mut velocity) in logos.iter_mut() {
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
