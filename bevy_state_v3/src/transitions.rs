//! Built-in state transitions.

use bevy_derive::Deref;
use bevy_ecs::{
    entity::Entity,
    event::Event,
    query::Has,
    system::{Commands, Populated},
};

use crate::{components::StateData, state::State, util::GlobalMarker};

/// Helper struct for previous/current state pairing.
pub struct StateChange<S: State> {
    /// Previous state.
    pub previous: Option<S::Repr>,
    /// Current state.
    pub current: S::Repr,
}

impl<S: State> StateChange<S> {
    /// Creates a new instance with custom data.
    pub fn new(previous: Option<S::Repr>, current: S::Repr) -> Self {
        Self { previous, current }
    }

    /// Creates a new instance using a reference to state data.
    pub fn from_data(data: &StateData<S>) -> Self {
        let previous = data.previous().cloned();
        let current = data.current().clone();
        Self::new(previous, current)
    }
}

/// Event triggered when a state is exited.
/// Reentrant transitions are ignored.
#[derive(Event, Deref)]
pub struct OnExit<S: State>(pub StateChange<S>);

/// System for triggering exit transition events.
pub fn on_exit_transition<S: State>(
    mut commands: Commands,
    query: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in query.iter() {
        if !state.is_updated || state.is_reentrant() {
            continue;
        }
        let event = OnExit::<S>(StateChange::from_data(state));
        if is_global {
            commands.trigger(event);
        } else {
            commands.trigger_targets(event, entity);
        };
    }
}

/// Event triggered when a state is entered.
/// Reentrant transitions are ignored.
#[derive(Event, Deref)]
pub struct OnEnter<S: State>(pub StateChange<S>);

/// System for triggering enter transition events.
pub fn on_enter_transition<S: State>(
    mut commands: Commands,
    states: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in states.iter() {
        if !state.is_updated || state.is_reentrant() {
            continue;
        }
        let event = OnEnter::<S>(StateChange::from_data(state));
        if is_global {
            commands.trigger(event);
        } else {
            commands.trigger_targets(event, entity);
        };
    }
}

/// Event triggered when a state is exited.
/// Reentrant transitions are included.
#[derive(Event, Deref)]
pub struct OnReexit<S: State>(pub StateChange<S>);

/// System for triggering re-exit transition events.
pub fn on_reexit_transition<S: State>(
    mut commands: Commands,
    query: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in query.iter() {
        if !state.is_updated {
            continue;
        }
        let event = OnReexit::<S>(StateChange::from_data(state));
        if is_global {
            commands.trigger(event);
        } else {
            commands.trigger_targets(event, entity);
        };
    }
}

/// Event triggered when a state is exited.
/// Reentrant transitions are included.
#[derive(Event, Deref)]
pub struct OnReenter<S: State>(pub StateChange<S>);

/// System for triggering re-enter transition events.
pub fn on_reenter_transition<S: State>(
    mut commands: Commands,
    states: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in states.iter() {
        if !state.is_updated {
            continue;
        }
        let event = OnReenter::<S>(StateChange::from_data(state));
        if is_global {
            commands.trigger(event);
        } else {
            commands.trigger_targets(event, entity);
        };
    }
}
