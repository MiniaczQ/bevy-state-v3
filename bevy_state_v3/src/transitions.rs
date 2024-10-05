//! Built-in state transitions.

use bevy_ecs::{
    entity::Entity,
    event::Event,
    query::Has,
    system::{Commands, Populated},
};

use crate::{components::StateData, state::State, util::GlobalMarker};

/// Event triggered when a state is exited.
/// Reentrant transitions are ignored.
#[derive(Event)]
pub struct OnExit<S: State> {
    /// Previous state.
    pub previous: S::Repr,
    /// Current state.
    pub current: S::Repr,
}

impl<S: State> OnExit<S> {
    /// Creates a new exit transition event.
    pub fn new(previous: S::Repr, current: S::Repr) -> Self {
        Self { previous, current }
    }
}

/// System for triggering exit transition events.
pub fn on_exit_transition<S: State>(
    mut commands: Commands,
    query: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in query.iter() {
        if !state.is_updated || state.is_reentrant() {
            continue;
        }
        let target = is_global.then_some(Entity::PLACEHOLDER).unwrap_or(entity);
        // Guaranteed to exist.
        let previous = state.previous().unwrap().clone();
        let current = state.current().clone();
        commands.trigger_targets(OnExit::<S>::new(previous, current), target);
    }
}

/// Event triggered when a state is entered.
/// Reentrant transitions are ignored.
#[derive(Event)]
pub struct OnEnter<S: State> {
    /// Previous state.
    pub previous: S::Repr,
    /// Current state.
    pub current: S::Repr,
}

impl<S: State> OnEnter<S> {
    /// Creates a new enter transition event.
    pub fn new(previous: S::Repr, current: S::Repr) -> Self {
        Self { previous, current }
    }
}

/// System for triggering enter transition events.
pub fn on_enter_transition<S: State>(
    mut commands: Commands,
    states: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in states.iter() {
        if !state.is_updated || state.is_reentrant() {
            continue;
        }
        let target = is_global.then_some(Entity::PLACEHOLDER).unwrap_or(entity);
        // Guaranteed to exist.
        let previous = state.previous().unwrap().clone();
        let current = state.current().clone();
        commands.trigger_targets(OnEnter::<S>::new(previous, current), target);
    }
}

/// Event triggered when a state is exited.
/// Reentrant transitions are included.
#[derive(Event)]
pub struct OnReexit<S: State> {
    /// Previous state.
    pub previous: S::Repr,
    /// Current state.
    pub current: S::Repr,
}

impl<S: State> OnReexit<S> {
    /// Creates a new re-exit transition event.
    pub fn new(previous: S::Repr, current: S::Repr) -> Self {
        Self { previous, current }
    }
}

/// System for triggering re-exit transition events.
pub fn on_reexit_transition<S: State>(
    mut commands: Commands,
    query: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in query.iter() {
        if !state.is_updated {
            continue;
        }
        // Guaranteed to be at least reentrant.
        let target = is_global.then_some(Entity::PLACEHOLDER).unwrap_or(entity);
        let previous = state.reentrant_previous().unwrap().clone();
        let current = state.current().clone();
        commands.trigger_targets(OnReexit::<S>::new(previous, current), target);
    }
}

/// Event triggered when a state is exited.
/// Reentrant transitions are included.
#[derive(Event)]
pub struct OnReenter<S: State> {
    /// Previous state.
    pub previous: S::Repr,
    /// Current state.
    pub current: S::Repr,
}

impl<S: State> OnReenter<S> {
    /// Creates a new re-enter transition event.
    pub fn new(previous: S::Repr, current: S::Repr) -> Self {
        Self { previous, current }
    }
}

/// System for triggering re-enter transition events.
pub fn on_reenter_transition<S: State>(
    mut commands: Commands,
    states: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in states.iter() {
        if !state.is_updated {
            continue;
        }
        let target = is_global.then_some(Entity::PLACEHOLDER).unwrap_or(entity);
        // Guaranteed to be at least reentrant.
        let previous = state.reentrant_previous().unwrap().clone();
        let current = state.current().clone();
        commands.trigger_targets(OnReenter::<S>::new(previous, current), target);
    }
}
