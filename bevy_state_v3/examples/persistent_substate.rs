//! This example shows how to use hierarchy made of multiple states; a root state and it's substate.

use bevy::{prelude::*, sprite::Anchor};
use bevy_state_v3::{prelude::*, state::StateUpdate};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        .register_state::<LogoState>(StateConfig::empty())
        .register_state::<CycleColorState>(StateConfig::empty())
        .init_state(None, LogoState::Enabled)
        .init_state(None, Some(CycleColorState::Enabled))
        .add_systems(Startup, setup)
        .add_systems(Update, toggle_logo)
        .add_systems(
            Update,
            (
                bounce_around.run_if(in_state(LogoState::Enabled)),
                cycle_color.run_if(in_state(Some(CycleColorState::Enabled))),
            ),
        )
        .run();
}

#[derive(State, Default, PartialEq, Debug, Clone)]
enum LogoState {
    #[default]
    Enabled,
    Disabled,
}

#[derive(Default, PartialEq, Debug, Clone)]
enum CycleColorState {
    #[default]
    Enabled,
    Disabled,
}

impl State for CycleColorState {
    type Dependencies = LogoState;
    type Update = PersistentUpdate<Self>;
    type Repr = Option<Self>;

    fn update(state: &mut StateData<Self>, logo: StateDependencies<'_, Self>) -> Self::Repr {
        match (logo.current(), &state.target().value) {
            (LogoState::Enabled, value) => Some(value.clone()),
            (LogoState::Disabled, _) => None,
        }
    }
}

#[derive(Default, Debug)]
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

impl<S: State> From<S> for PersistentUpdate<S> {
    fn from(value: S) -> Self {
        PersistentUpdate {
            should_update: true,
            value,
        }
    }
}

/// User controls.
fn toggle_logo(
    mut commands: Commands,
    input: Res<ButtonInput<KeyCode>>,
    state: Global<(&StateData<LogoState>, &StateData<CycleColorState>)>,
) {
    let (logo_state, cycle_color_state) = *state;

    if input.just_pressed(KeyCode::Digit1) {
        // Decide the next state based on current state.
        let next = match logo_state.current() {
            LogoState::Enabled => LogoState::Disabled,
            LogoState::Disabled => LogoState::Enabled,
        };
        // Request a change for the state.
        commands.update_state(None, next);
    }

    if input.just_pressed(KeyCode::Digit2) {
        match cycle_color_state.current() {
            Some(CycleColorState::Enabled) => {
                commands.update_state(None, CycleColorState::Disabled);
            }
            Some(CycleColorState::Disabled) => {
                commands.update_state(None, CycleColorState::Enabled);
            }
            None => {}
        };
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
        SpriteBundle {
            sprite: Sprite {
                color: Color::hsv(rng.gen_range(0.0..=1.0), 1.0, 1.0),
                anchor: Anchor::Center,
                ..default()
            },
            texture,
            transform: Transform::from_xyz(
                rng.gen_range(-200.0..=200.),
                rng.gen_range(-200.0..=200.),
                0.,
            ),
            ..default()
        },
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
