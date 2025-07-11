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
        .register_state::<MyState>(
            StateConfig::empty()
                .with_on_init(true)
                .with_on_reexit(true)
                .with_on_enter(true),
        )
        // Register all observers which react to our state transitions.
        .add_observer(setup)
        .add_observer(on_enter)
        .add_observer(on_reexit)
        .add_systems(Update, user_input)
        // Initialize state last, after all systems and observers are registered.
        .init_state(None, MyState::Eeny)
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
fn setup(_: On<OnInit<MyState>>, mut commands: Commands) {
    println!();
    println!("Press 1-4 to change state.");
    println!();

    // Spawn camera.
    commands.spawn(Camera2d);

    // Spawn text for displaying state.
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            overflow: Overflow::scroll_y(),
            width: Val::Vw(100.0),
            height: Val::Vh(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_child((
            Text::new(""),
            TextLayout::new_with_justify(Justify::Center),
            TransitionLog(vec![]),
        ));
}

/// Observer that gets triggered on every non-reentrant [`MyState`] transition in the enter order.
fn on_enter(trigger: On<OnEnter<MyState>>, mut label: Single<(&mut Text, &mut TransitionLog)>) {
    let (text, log) = &mut *label;
    let transition = format!("Entered {:?}", trigger.0);
    update_log(log, text, transition);
}

/// Observer that gets triggered on every [`MyState`] transition in the exit order.
fn on_reexit(
    trigger: On<OnReexit<MyState>>,
    state: Single<&StateData<MyState>>,
    mut label: Single<(&mut Text, &mut TransitionLog)>,
) {
    let (text, log) = &mut *label;
    // We can check whether we re-entered the same state or not.
    let transition = if state.is_reentrant() {
        format!("Re-exited {:?}", trigger.0)
    } else {
        format!("Exited {:?}", trigger.0)
    };
    update_log(log, text, transition);
}

fn update_log(log: &mut TransitionLog, text: &mut Text, transition: String) {
    log.0.insert(0, transition);
    log.0.truncate(10);
    text.0 = log.0.join("\n");
}
