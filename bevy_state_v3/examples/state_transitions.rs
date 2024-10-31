//! This example shows how to interact with state transition events.
//! State transition events are generated for both global and local states,
//! only difference being whether the transition event is targeted (local) or not (global).
//!
//! As mentioned in `global_state` example, global states are entities too.
//! Despite that, their state transition events are untargeted for better
//! user interface.

use bevy::prelude::*;
use bevy_state_v3::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        // We use a custom set of transitions for re-exit & enter, instead of
        // `StateConfig::default()` which contains exit & enter transitions.
        // You can register any amount of exit/enter transitions as well as implement custom ones.
        .register_state(
            StateConfig::<MyState>::empty()
                // Exit transitions always run first in sub state to root state order.
                .with_on_exit(on_reexit_transition::<MyState>)
                // Enter transitions run after exit transitions in root state to sub state order.
                .with_on_enter(on_enter_transition::<MyState>),
        )
        .init_state(None, MyState::Eeny)
        // Register all observers which react to our state transitions.
        .add_observer(setup)
        .add_observer(meeny_entered)
        .add_observer(any_reexited)
        .add_systems(Update, user_input)
        .run();
}

#[derive(State, Default, PartialEq, Debug, Clone)]
enum MyState {
    #[default]
    Eeny,
    Meeny,
    Miny,
    Moe,
}

/// User controls.
fn user_input(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Digit1) {
        commands.update_state(None, MyState::Eeny);
    }
    if input.just_pressed(KeyCode::Digit2) {
        commands.update_state(None, MyState::Meeny);
    }
    if input.just_pressed(KeyCode::Digit3) {
        commands.update_state(None, MyState::Miny);
    }
    if input.just_pressed(KeyCode::Digit4) {
        commands.update_state(None, MyState::Moe);
    }
}

#[derive(Component)]
struct TransitionLog(Vec<String>);

/// The setup here is an observer instead of a system.
/// This is to ensure that everything is spawned for initial
/// system transitions, which run before [`Startup`] schedule.
fn setup(trigger: Trigger<OnEnter<MyState>>, mut commands: Commands) {
    // Return if this isn't an initial transition.
    if trigger.previous.is_some() {
        return;
    }

    println!();
    println!("");
    println!();

    // Spawn camera.
    commands.spawn(Camera2d);

    // Spawn text for displaying state.
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                overflow: Overflow::scroll_y(),
                width: Val::Vw(100.0),
                height: Val::Vh(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            ..default()
        })
        .with_child((
            Text::new(""),
            TextLayout::new_with_justify(JustifyText::Center),
            TransitionLog(vec![]),
        ));
}

/// Observer that gets triggered on every non-reentrant [`MyState`] transition in the enter order.
fn meeny_entered(
    trigger: Trigger<OnEnter<MyState>>,
    mut label: Single<(&mut Text, &mut TransitionLog)>,
) {
    // We can skip checking for un-/targeted events if we don't use our state as both local and global.
    // Ignore transitions into states that aren't Meeny.
    if trigger.current != MyState::Meeny {
        return;
    }

    let (text, log) = &mut *label;
    log.0.insert(0, format!("Entered {:?}", trigger.current));
    log.0.truncate(10);
    text.0 = log.0.join("\n");
}

/// Observer that gets triggered on every [`MyState`] transition in the exit order.
fn any_reexited(
    trigger: Trigger<OnReexit<MyState>>,
    mut label: Single<(&mut Text, &mut TransitionLog)>,
) {
    // Ignore targeted observers which are for local states.
    if trigger.entity() != Entity::PLACEHOLDER {
        return;
    }

    let (text, log) = &mut *label;
    // We can check whether we re-entered the same state or not.
    let transition = if trigger.previous.as_ref() == Some(&trigger.current) {
        format!("Re-exited {:?}", trigger.current)
    } else {
        format!("Exited {:?}", trigger.current)
    };
    log.0.insert(0, transition);
    log.0.truncate(10);
    text.0 = log.0.join("\n");
}
