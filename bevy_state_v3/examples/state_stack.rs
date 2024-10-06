//! This example shows how to implement a state that operates on a stack.

use bevy::prelude::*;
use bevy_state_v3::{commands::state_target_entity, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        .register_state::<MyState>(
            StateConfig::empty().with_on_enter(on_reenter_transition::<MyState>),
        )
        .init_state(None, MyState::A)
        .add_systems(Update, user_input)
        .observe(observer_on_reenter)
        .run();
}

#[derive(Default, PartialEq, Debug, Clone)]
enum MyState {
    #[default]
    A,
    B,
    C,
}

impl State for MyState {
    type Dependencies = ();
    type Update = StackUpdate<Self>;
    type Repr = Self;

    fn update(state: &mut StateData<Self>, _: StateDependencies<'_, Self>) -> Self::Repr {
        let op = state.update_mut().op.take().unwrap();
        match op {
            StackOp::Push(new) => {
                let current = state.current().clone();
                state.update_mut().stack.push(current);
                new
            }
            StackOp::Pop => {
                let maybe_new = state.update_mut().stack.pop();
                let new = maybe_new.unwrap_or_else(|| state.current().clone());
                new
            }
        }
    }
}

/// Helper enum for stack operations.
#[derive(Debug)]
enum StackOp<S> {
    Push(S),
    Pop,
}

/// Stack based backend for states.
/// Note that the "top" value of the stack is stored in the `current` field as opposed to here.
/// Operation field stores the next operation to be executed on the stack.
#[derive(Debug)]
pub struct StackUpdate<S: State> {
    stack: Vec<S::Repr>,
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

impl<S: State + Default> StateUpdate for StackUpdate<S> {
    fn should_update(&self) -> bool {
        self.op.is_some()
    }

    fn post_update(&mut self) {
        self.op.take();
    }
}

/// Helper command for pushing and popping the stack.
struct StackOpCommand<S> {
    local: Option<Entity>,
    op: StackOp<S>,
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

/// Command shorthands for operating on the stack.
pub trait StackStateExt {
    /// Pushes a new state to the top of the stack.
    fn push_state<R>(&mut self, local: Option<Entity>, value: R)
    where
        R: StateRepr,
        R::State: State<Update = StackUpdate<R::State>>;

    /// Pops the top state from the stack.
    /// Does nothing if only one state is left.
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

/// User controls.
fn user_input(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Digit1) {
        commands.push_state(None, MyState::A);
    }
    if input.just_pressed(KeyCode::Digit2) {
        commands.push_state(None, MyState::B);
    }
    if input.just_pressed(KeyCode::Digit3) {
        commands.push_state(None, MyState::C);
    }
    if input.just_pressed(KeyCode::Digit4) {
        commands.pop_state::<MyState>(None);
    }
}

fn observer_on_reenter(trigger: Trigger<OnReenter<MyState>>) {
    // We ignore the target entity since we only have global states here.
    let event = trigger.event();
    info!("Re-entered state {:?}", event.current);
}
