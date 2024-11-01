//! This example showcases how a custom update data structure can make
//! a state work on a stack with push and pop operations.

use bevy::prelude::*;
use bevy_state_v3::{commands::state_target_entity, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        // We configure re-enter transitions, so we can update text when the state changes.
        .register_state(StateConfig::<MyState>::empty().with_on_reenter(true))
        .init_state(None, None::<MyState>)
        .add_systems(Startup, setup)
        .add_systems(Update, user_input)
        .add_observer(update_text)
        .run();
}

/// The state we use as our example.
#[derive(Default, PartialEq, Debug, Clone)]
enum MyState {
    #[default]
    Alice,
    Had,
    A,
    Little,
    Lamb,
}

impl State for MyState {
    type Dependencies = ();
    type Update = StackUpdate<Self>;
    type Repr = Option<Self>;

    fn update(state: &mut StateData<Self>, _: StateSetData<'_, Self::Dependencies>) -> Self::Repr {
        state.update()
    }
}

/// Helper enum for stack operations.
#[derive(Debug)]
enum StackOp<S> {
    /// Adds a value to top of the stack.
    Push(S),
    /// Removes a value from top of the stack.
    Pop,
}

/// Stack update data structure for states.
#[derive(Debug)]
pub struct StackUpdate<S: State> {
    /// The stack except the top value, which is stored as the `current` state.
    stack: Vec<S>,
    /// Pending operation on the stack.
    op: Option<StackOp<S>>,
}

impl<S: State> Default for StackUpdate<S> {
    fn default() -> Self {
        Self {
            stack: Default::default(),
            op: Default::default(),
        }
    }
}

impl<S: State> StateUpdate for StackUpdate<S> {
    fn should_update(&self) -> bool {
        self.op.is_some()
    }

    fn post_update(&mut self) {
        self.op.take();
    }
}

/// Helper for updating the state data.
pub trait StackUpdateData<S: State<Update = StackUpdate<S>>> {
    /// Updates the stack state.
    fn update(&mut self) -> Option<S>;
}

impl<S: State<Repr = Option<S>, Update = StackUpdate<S>>> StackUpdateData<S> for StateData<S> {
    fn update(&mut self) -> Option<S> {
        // We assume there are no parent states, which means this value being present is the only reason state is being updated.
        let op = self.update_mut().op.take().unwrap();
        match op {
            StackOp::Push(new) => {
                if let Some(current) = self.current().clone() {
                    self.update_mut().stack.push(current);
                }
                Some(new)
            }
            StackOp::Pop => self.update_mut().stack.pop(),
        }
    }
}

/// Command for updating the stack state.
struct StackOpCommand<S> {
    /// Global or local state.
    local: Option<Entity>,
    /// Operation we want to perform.
    op: StackOp<S>,
}

impl<S> Command for StackOpCommand<S>
where
    S: State<Repr = Option<S>, Update = StackUpdate<S>>,
{
    fn apply(self, world: &mut World) {
        let Some(entity) = state_target_entity(world, self.local) else {
            return;
        };
        let mut entity = world.entity_mut(entity);
        let Some(mut state_data) = entity.get_mut::<StateData<S>>() else {
            warn!(
                "Missing state data component for {}.",
                disqualified::ShortName::of::<S>()
            );
            return;
        };
        state_data.update_mut().op = Some(self.op);
    }
}

/// Commands extension for requesting stack operations.
pub trait StackStateExt {
    /// Pushes a new state to the top of the stack.
    fn push_state<S>(&mut self, local: Option<Entity>, value: S)
    where
        S: State<Repr = Option<S>, Update = StackUpdate<S>>;

    /// Pops the top state from the stack.
    /// Repeats the current state if no more states are left on the stack.
    fn pop_state<S>(&mut self, local: Option<Entity>)
    where
        S: State<Repr = Option<S>, Update = StackUpdate<S>>;
}

impl StackStateExt for Commands<'_, '_> {
    fn push_state<S>(&mut self, local: Option<Entity>, value: S)
    where
        S: State<Repr = Option<S>, Update = StackUpdate<S>>,
    {
        self.queue(StackOpCommand {
            local,
            op: StackOp::Push(value),
        });
    }

    fn pop_state<S>(&mut self, local: Option<Entity>)
    where
        S: State<Repr = Option<S>, Update = StackUpdate<S>>,
    {
        self.queue(StackOpCommand {
            local,
            op: StackOp::<S>::Pop,
        });
    }
}

/// Marker component to find the text for updating.
#[derive(Component)]
struct StateLabel;

/// Spawns camera and text UI node.
fn setup(mut commands: Commands) {
    println!();
    println!("Press 1-5 to push new states onto the stack.");
    println!("Press SPACE to pop state from the stack.");
    println!();

    // Spawn camera.
    commands.spawn(Camera2d);

    // Spawn text for displaying state.
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                width: Val::Vw(100.0),
                height: Val::Vh(100.0),
                ..default()
            },
            ..default()
        })
        .with_child((
            Text::new(""),
            TextLayout::new_with_justify(JustifyText::Center),
            StateLabel,
        ));
}

/// User controls.
fn user_input(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Digit1) {
        commands.push_state(None, MyState::Alice);
    }
    if input.just_pressed(KeyCode::Digit2) {
        commands.push_state(None, MyState::Had);
    }
    if input.just_pressed(KeyCode::Digit3) {
        commands.push_state(None, MyState::A);
    }
    if input.just_pressed(KeyCode::Digit4) {
        commands.push_state(None, MyState::Little);
    }
    if input.just_pressed(KeyCode::Digit5) {
        commands.push_state(None, MyState::Lamb);
    }
    if input.just_pressed(KeyCode::Space) {
        commands.pop_state::<MyState>(None);
    }
}

/// Observer that gets called every time the state changes.
/// We use the re-entrant type so we also detect identity transitions.
///
/// Note that only the top of the stack is the current state.
/// We display the rest of the stack for clarity.
fn update_text(
    _: Trigger<OnReenter<MyState>>,
    state: Single<&StateData<MyState>>,
    mut text: Single<&mut Text, With<StateLabel>>,
) {
    let mut content = String::new();
    for state in state.update().stack.iter().chain(state.current().iter()) {
        content.push_str(&format!("{:?}\n", state));
    }
    text.0 = content;
}
