//! This example showcases the usage of state transitions.

use bevy::prelude::*;
use bevy_state_v3::{prelude::*, return_if_not_current, return_if_targeted};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        // Register machinery for the state.
        // We use a custom set of transitions: re-exit & enter.
        // If we used `StateConfig::default()` we'd get: enter & exit.
        // You can register multiple exit/enter transitions.
        .register_state::<MyState>(
            StateConfig::empty()
                .with_on_exit(on_reexit_transition::<MyState>)
                .with_on_enter(on_enter_transition::<MyState>),
        )
        .init_state(None, MyState::Enabled)
        .add_observer(reexit)
        .add_observer(enter_enabled)
        .add_systems(Update, user_input)
        .run();
}

#[derive(State, Default, PartialEq, Debug, Clone)]
enum MyState {
    #[default]
    Enabled,
    Disabled,
}

/// User controls.
fn user_input(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Digit1) {
        commands.update_state(None, MyState::Enabled);
    }
    if input.just_pressed(KeyCode::Digit2) {
        commands.update_state(None, MyState::Disabled);
    }
}

fn reexit(trigger: Trigger<OnReexit<MyState>>) {
    return_if_targeted!(trigger);
    info!("Re-exited {:?}", trigger.current);
}

fn enter_enabled(trigger: Trigger<OnEnter<MyState>>) {
    return_if_targeted!(trigger);
    return_if_not_current!(trigger, MyState::Enabled);
    info!("Entered Enabled");
}
