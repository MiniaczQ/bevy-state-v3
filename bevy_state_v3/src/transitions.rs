use std::marker::PhantomData;

use bevy_ecs::{
    entity::Entity,
    event::Event,
    query::Has,
    schedule::{IntoSystemConfigs, SystemConfigs},
    system::{Commands, Populated},
};

use crate::{data::StateData, scheduling::StateSystemSet, state::State, util::GlobalMarker};

pub struct StateConfig<S: State> {
    pub(crate) systems: Vec<SystemConfigs>,
    _state: PhantomData<S>,
}

impl<S: State> Default for StateConfig<S> {
    fn default() -> Self {
        Self {
            systems: vec![
                on_exit_transition::<S>
                    .in_set(StateSystemSet::exit::<S>())
                    .into(),
                on_enter_transition::<S>
                    .in_set(StateSystemSet::enter::<S>())
                    .into(),
            ],
            _state: Default::default(),
        }
    }
}

impl<S: State> StateConfig<S> {
    /// Config that creates no transitions.
    /// For standard [`OnExit`] and [`OnEnter`] use the [`StateTransitionsConfig::default`].
    pub fn empty() -> Self {
        Self {
            systems: vec![],
            _state: PhantomData,
        }
    }

    /// Adds a system to run when state is exited.
    /// An example system that runs [`OnExit`] is [`on_exit_transition`].
    pub fn with_on_exit<M>(mut self, system: impl IntoSystemConfigs<M>) -> Self {
        self.systems
            .push(system.in_set(StateSystemSet::exit::<S>()));
        self
    }

    /// Adds a system to run when state is entered.
    /// An example system that runs [`OnEnter`] is [`on_enter_transition`].
    pub fn with_on_enter<M>(mut self, system: impl IntoSystemConfigs<M>) -> Self {
        self.systems
            .push(system.in_set(StateSystemSet::enter::<S>()));
        self
    }
}

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
    pub fn new(previous: S::Repr, current: S::Repr) -> Self {
        Self { previous, current }
    }
}

pub fn on_exit_transition<S: State>(
    mut commands: Commands,
    query: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in query.iter() {
        if !state.is_updated || state.is_reentrant() {
            continue;
        }
        let target = is_global.then_some(Entity::PLACEHOLDER).unwrap_or(entity);
        commands.trigger_targets(
            OnExit::<S>::new(state.previous().clone(), state.current().clone()),
            target,
        );
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
    pub fn new(previous: S::Repr, current: S::Repr) -> Self {
        Self { previous, current }
    }
}

pub fn on_enter_transition<S: State>(
    mut commands: Commands,
    states: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in states.iter() {
        if !state.is_updated || state.is_reentrant() {
            continue;
        }
        let target = is_global.then_some(Entity::PLACEHOLDER).unwrap_or(entity);
        commands.trigger_targets(
            OnEnter::<S>::new(state.previous().clone(), state.current().clone()),
            target,
        );
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
    pub fn new(previous: S::Repr, current: S::Repr) -> Self {
        Self { previous, current }
    }
}

pub fn on_reexit_transition<S: State>(
    mut commands: Commands,
    query: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in query.iter() {
        if !state.is_updated || state.is_reentrant() {
            continue;
        }
        let target = is_global.then_some(Entity::PLACEHOLDER).unwrap_or(entity);
        commands.trigger_targets(
            OnReexit::<S>::new(state.previous().clone(), state.current().clone()),
            target,
        );
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
    pub fn new(previous: S::Repr, current: S::Repr) -> Self {
        Self { previous, current }
    }
}

pub fn on_reenter_transition<S: State>(
    mut commands: Commands,
    states: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in states.iter() {
        if !state.is_updated || state.is_reentrant() {
            continue;
        }
        let target = is_global.then_some(Entity::PLACEHOLDER).unwrap_or(entity);
        commands.trigger_targets(
            OnReenter::<S>::new(state.previous().clone(), state.current().clone()),
            target,
        );
    }
}
