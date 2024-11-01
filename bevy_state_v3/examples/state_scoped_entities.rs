//! This example shows how to use the most basic global state machine.
//! The machine consists of a single state type that decides
//! whether a logo moves around the screen and changes color.

use bevy::{prelude::*, sprite::Anchor};
use bevy_state_v3::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        .register_state::<MyState>(StateConfig::empty().with_despawn_state_scoped(true))
        .init_state(None, MyState::Enabled)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                user_input,
                (spawn_logos, bounce_around).run_if(in_state(MyState::Enabled)),
            )
                .chain(),
        )
        .run();
}

#[derive(State, Default, PartialEq, Debug, Clone)]
enum MyState {
    #[default]
    Enabled,
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
            MyState::Enabled => commands.update_state(None, MyState::Disabled),
            MyState::Disabled => commands.update_state(None, MyState::Enabled),
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
    let timer = timer.get_or_insert_with(|| Timer::from_seconds(1.0, TimerMode::Repeating));
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
        Velocity(Vec2::from(angle.to_radians().sin_cos()) * 3.0),
        StateScoped::<MyState>(MyState::Enabled),
    ));
    *index += 1;
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
