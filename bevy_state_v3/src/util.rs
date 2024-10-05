use bevy_ecs::{component::Component, query::With, system::Single};

use crate::{
    data::StateData,
    state::{State, StateRepr},
};

/// Marker for global entity.
#[derive(Component)]
pub struct GlobalMarker;

pub type Global<'w, D> = Single<'w, D, With<GlobalMarker>>;

/// Run condition.
/// Returns true if global state is set to the specified target.
pub fn in_state<R: StateRepr>(target: R) -> impl Fn(Global<&StateData<R::State>>) -> bool {
    move |state: Global<&StateData<R::State>>| state.current() == &target
}

/// Run condition.
/// Returns true if global state changed.
pub fn state_changed<S: State>(state: Global<&StateData<S>>) -> bool {
    state.is_updated()
}

/// Run condition.
/// Returns true if global state changed to the specified target.
pub fn state_changed_to<R: StateRepr>(target: R) -> impl Fn(Global<&StateData<R::State>>) -> bool {
    move |state: Global<&StateData<R::State>>| state.is_updated() && state.current() == &target
}
