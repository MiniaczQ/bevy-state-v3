//! This example shows how to implement a state that operates on a stack.

use std::marker::PhantomData;

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
        .init_state(None, MyState::Alice)
        .add_systems(Update, user_input)
        .observe(observer_on_reenter)
        .run();
}

#[derive(PartialEq, Debug, Clone)]
enum MyState {
    Alice,
    Had,
    A,
    Little,
    Lamb,
}

impl Variants for MyState {
    fn variants() -> &'static [Self] {
        &[Self::Alice, Self::Had, Self::A, Self::Little, Self::Lamb]
    }
}

impl State for MyState {
    type Dependencies = ();
    type Update = ShiftUpdate<Self>;
    type Repr = Self;

    fn update(state: &mut StateData<Self>, _: StateDependencies<'_, Self>) -> Self::Repr {
        let op = state.update_mut().op.take().unwrap();
        match op {
            ShiftOp::Advance => state.update_mut().advance(),
            ShiftOp::Retreat => state.update_mut().retreat(),
        }
    }
}

/// Helper enum for stack operations.
#[derive(Debug)]
enum ShiftOp {
    Advance,
    Retreat,
}

/// Trait for types that define all their variants in order.
pub trait Variants: Sized {
    fn variants() -> &'static [Self];
}

#[derive(Debug)]
pub struct ShiftUpdate<S>
where
    S: State,
    S::Repr: Variants,
{
    _data: PhantomData<S>,
    ptr: usize,
    op: Option<ShiftOp>,
}

impl<S> Default for ShiftUpdate<S>
where
    S: State,
    S::Repr: Variants,
{
    fn default() -> Self {
        Self {
            _data: PhantomData,
            ptr: 0,
            op: Default::default(),
        }
    }
}

impl<S> StateUpdate for ShiftUpdate<S>
where
    S: State,
    S::Repr: Variants,
{
    fn should_update(&self) -> bool {
        self.op.is_some()
    }

    fn post_update(&mut self) {
        self.op.take();
    }
}

impl<S> ShiftUpdate<S>
where
    S: State,
    S::Repr: Variants,
{
    fn advance(&mut self) -> S::Repr {
        let len = S::Repr::variants().len();
        self.ptr = (self.ptr + 1) % len;
        S::Repr::variants()[self.ptr].clone()
    }

    fn retreat(&mut self) -> S::Repr {
        let len = S::Repr::variants().len();
        self.ptr = (self.ptr + len - 1) % len;
        S::Repr::variants()[self.ptr].clone()
    }
}

struct ShiftOpCommand<S> {
    _data: PhantomData<S>,
    local: Option<Entity>,
    op: ShiftOp,
}

impl<S> Command for ShiftOpCommand<S>
where
    S: State<Update = ShiftUpdate<S>>,
    S::Repr: Variants,
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

pub trait StackStateExt {
    fn advance_state<S>(&mut self, local: Option<Entity>)
    where
        S: State<Update = ShiftUpdate<S>>,
        S::Repr: Variants;

    fn retreat_state<S>(&mut self, local: Option<Entity>)
    where
        S: State<Update = ShiftUpdate<S>>,
        S::Repr: Variants;
}

impl StackStateExt for Commands<'_, '_> {
    fn advance_state<S>(&mut self, local: Option<Entity>)
    where
        S: State<Update = ShiftUpdate<S>>,
        S::Repr: Variants,
    {
        self.queue(ShiftOpCommand {
            _data: PhantomData::<S>,
            local,
            op: ShiftOp::Advance,
        });
    }

    fn retreat_state<S>(&mut self, local: Option<Entity>)
    where
        S: State<Update = ShiftUpdate<S>>,
        S::Repr: Variants,
    {
        self.queue(ShiftOpCommand {
            _data: PhantomData::<S>,
            local,
            op: ShiftOp::Retreat,
        });
    }
}

/// User controls.
fn user_input(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    if input.just_pressed(KeyCode::Digit1) {
        commands.advance_state::<MyState>(None);
    }
    if input.just_pressed(KeyCode::Digit2) {
        commands.retreat_state::<MyState>(None);
    }
}

fn observer_on_reenter(trigger: Trigger<OnReenter<MyState>>) {
    // We ignore the target entity since we only have global states here.
    let event = trigger.event();
    info!("Entered state {:?}", event.current);
}
