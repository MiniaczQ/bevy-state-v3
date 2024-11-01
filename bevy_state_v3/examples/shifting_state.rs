//! This example shows how to implement a state that operates on a stack.

use std::marker::PhantomData;

use bevy::prelude::*;
use bevy_state_v3::{commands::state_target_entity, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // TODO: remove once lands in `DefaultPlugins`
        .add_plugins(StatePlugin)
        .register_state(StateConfig::<MyState>::empty().with_on_reenter(true))
        .init_state(None, MyState::Alice)
        .add_systems(Startup, setup)
        .add_systems(Update, user_input)
        .add_observer(observer_on_reenter)
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
        &[
            Self::Alice,
            Self::Had,
            Self::A,
            Self::Little,
            Self::Lamb,
            // You can have duplicates
            Self::Little,
            Self::A,
            Self::Had,
        ]
    }
}

impl State for MyState {
    type Dependencies = ();
    type Update = ShiftUpdate<Self>;
    type Repr = Self;

    fn update(state: &mut StateData<Self>, _: StateSetData<'_, Self::Dependencies>) -> Self::Repr {
        state.update_mut().update()
    }
}

/// Helper enum for shift operations.
#[derive(Debug)]
enum ShiftOp {
    Advance,
    Retreat,
}

/// Trait for types that define all their variants in order.
pub trait Variants: Sized {
    /// Returns all variants of a type in order.
    /// Can contain duplicates.
    fn variants() -> &'static [Self];
}

/// Data structure for storing shifting statess.
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
    fn update(&mut self) -> S::Repr {
        let len = S::Repr::variants().len();
        // We assume there are no parent states, which means this value
        // being present is the only reason state is being updated.
        match self.op.take().unwrap() {
            // If we try to shift at the edges, we wrap around.
            ShiftOp::Advance => self.ptr = (self.ptr + 1) % len,
            ShiftOp::Retreat => self.ptr = (self.ptr + len - 1) % len,
        }
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

/// Helper trait for defining shifting state update commands.
pub trait StackStateExt {
    /// Advances the state forward.
    fn advance_state<S>(&mut self, local: Option<Entity>)
    where
        S: State<Update = ShiftUpdate<S>>,
        S::Repr: Variants;

    /// Advances the state backwards.
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

#[derive(Component)]
struct StateLabel;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    commands.spawn((Text::new(""), StateLabel));
}

fn observer_on_reenter(
    trigger: Trigger<OnReenter<MyState>>,
    mut text: Single<&mut Text, With<StateLabel>>,
) {
    text.0 = format!("{:?}", trigger.0);
}
