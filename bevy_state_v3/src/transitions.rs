//! Built-in state transitions.

use bevy_derive::Deref;
use bevy_ecs::{
    entity::Entity,
    event::Event,
    observer::Trigger,
    query::Has,
    system::{Commands, Populated, Query},
    world::{OnAdd, OnRemove},
};

use crate::{components::StateData, state::State, util::GlobalMarker};

/// Event triggered when state is added.
#[derive(Event)]
pub struct OnInit<S: State>(pub S::Repr);

/// Observer that emits state [`OnInit`] event.
pub fn on_init_transition<S: State>(
    trigger: Trigger<OnAdd, StateData<S>>,
    mut commands: Commands,
    query: Query<(&StateData<S>, Has<GlobalMarker>)>,
) {
    let entity = trigger.target();
    let (state, is_global) = query.get(entity).unwrap();
    let event = OnInit::<S>(state.current().clone());
    if is_global {
        commands.trigger(event);
    } else {
        commands.trigger_targets(event, entity);
    };
}

/// Event triggered when state is removed.
#[derive(Event)]
pub struct OnDeinit<S: State>(pub S::Repr);

/// Observer that emits state [`OnDeinit`] event.
pub fn on_deinit_transition<S: State>(
    trigger: Trigger<OnRemove, StateData<S>>,
    mut commands: Commands,
    query: Query<(&StateData<S>, Has<GlobalMarker>)>,
) {
    let entity = trigger.target();
    let (state, is_global) = query.get(entity).unwrap();
    let event = OnDeinit::<S>(state.current().clone());
    if is_global {
        commands.trigger(event);
    } else {
        commands.trigger_targets(event, entity);
    };
}

/// Event triggered when a state is exited.
/// Reentrant transitions are ignored.
#[derive(Event, Deref)]
pub struct OnExit<S: State>(pub S::Repr);

/// System for triggering exit transition events.
pub fn on_exit_transition<S: State>(
    mut commands: Commands,
    query: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in query.iter() {
        if !state.is_updated || state.is_reentrant() {
            continue;
        }
        let event = OnExit::<S>(state.previous().cloned().unwrap());
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
pub struct OnEnter<S: State>(pub S::Repr);

/// System for triggering enter transition events.
pub fn on_enter_transition<S: State>(
    mut commands: Commands,
    states: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in states.iter() {
        if !state.is_updated || state.is_reentrant() {
            continue;
        }
        let event = OnEnter::<S>(state.current().clone());
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
pub struct OnReexit<S: State>(pub S::Repr);

/// System for triggering re-exit transition events.
pub fn on_reexit_transition<S: State>(
    mut commands: Commands,
    query: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in query.iter() {
        if !state.is_updated {
            continue;
        }
        let event = OnReexit::<S>(state.reentrant_previous().cloned().unwrap());
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
pub struct OnReenter<S: State>(pub S::Repr);

/// System for triggering re-enter transition events.
pub fn on_reenter_transition<S: State>(
    mut commands: Commands,
    states: Populated<(Entity, &StateData<S>, Has<GlobalMarker>)>,
) {
    for (entity, state, is_global) in states.iter() {
        if !state.is_updated {
            continue;
        }
        let event = OnReenter::<S>(state.current().clone());
        if is_global {
            commands.trigger(event);
        } else {
            commands.trigger_targets(event, entity);
        };
    }
}
