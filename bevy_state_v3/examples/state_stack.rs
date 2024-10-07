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
        .register_state::<MyState>(
            StateConfig::empty().with_on_enter(on_reenter_transition::<MyState>),
        )
        .init_state(None, MyState::Alice)
        .add_systems(Startup, setup)
        .add_systems(Update, user_input)
        .observe(update_text_node)
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
    type Repr = Self;

    fn update(state: &mut StateData<Self>, _: StateDependencies<'_, Self>) -> Self::Repr {
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
    /// The stack except the top value.
    /// Top value is stored as the `current` state.
    stack: Vec<S::Repr>,
    /// Pending operation on the stack.
    op: Option<StackOp<S::Repr>>,
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
    fn update(&mut self) -> S::Repr;
}

impl<S: State<Update = StackUpdate<S>>> StackUpdateData<S> for StateData<S> {
    fn update(&mut self) -> S::Repr {
        // We assume there are no parent states, which means this value being present is the only reason state is being updated.
        let op = self.update_mut().op.take().unwrap();
        match op {
            StackOp::Push(new) => {
                let current = self.current().clone();
                self.update_mut().stack.push(current);
                new
            }
            StackOp::Pop => {
                let maybe_new = self.update_mut().stack.pop();
                // If there are no values on the stack, repeat the current state.
                let new = maybe_new.unwrap_or_else(|| self.current().clone());
                new
            }
        }
    }
}

/// Command for updating the stack state.
struct StackOpCommand<R> {
    /// Global or local state.
    local: Option<Entity>,
    /// Operation we want to perform.
    op: StackOp<R>,
}

impl<R> Command for StackOpCommand<R>
where
    R: StateRepr,
    R::State: State<Update = StackUpdate<R::State>>,
{
    fn apply(self, world: &mut World) {
        let Some(entity) = state_target_entity(world, self.local) else {
            return;
        };
        let mut entity = world.entity_mut(entity);
        let Some(mut state_data) = entity.get_mut::<StateData<R::State>>() else {
            warn!(
                "Missing state data component for {}.",
                disqualified::ShortName::of::<R::State>()
            );
            return;
        };
        state_data.update_mut().op = Some(self.op);
    }
}

/// Commands extension for requesting stack operations.
pub trait StackStateExt {
    /// Pushes a new state to the top of the stack.
    fn push_state<R>(&mut self, local: Option<Entity>, value: R)
    where
        R: StateRepr,
        R::State: State<Update = StackUpdate<R::State>>;

    /// Pops the top state from the stack.
    /// Repeats the current state if no more states are left on the stack.
    fn pop_state<S>(&mut self, local: Option<Entity>)
    where
        S: State<Update = StackUpdate<S>>;
}

impl StackStateExt for Commands<'_, '_> {
    fn push_state<R>(&mut self, local: Option<Entity>, value: R)
    where
        R: StateRepr,
        R::State: State<Update = StackUpdate<R::State>>,
    {
        self.queue(StackOpCommand {
            local,
            op: StackOp::Push(value),
        });
    }

    fn pop_state<S>(&mut self, local: Option<Entity>)
    where
        S: State<Update = StackUpdate<S>>,
    {
        self.queue(StackOpCommand {
            local,
            op: StackOp::<S::Repr>::Pop,
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
    commands.spawn((
        TextBundle::from_section("Alice", TextStyle::default()),
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
fn update_text_node(
    _: Trigger<OnReenter<MyState>>,
    state: Single<&StateData<MyState>>,
    mut label: Single<&mut Text, With<StateLabel>>,
) {
    let mut sections = vec![];
    for state in state.update().stack.iter().chain([state.current()]) {
        sections.push(TextSection::from(format!("{:?}", state)));
    }
    label.sections = sections;
}
