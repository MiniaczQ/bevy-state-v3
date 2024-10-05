use bevy_ecs::{component::Component, query::With, system::Single};

use crate::{data::StateData, state::State};

/// Marker for global entity.
#[derive(Component)]
pub struct GlobalMarker;

type Global<'w, D> = Single<'w, D, With<GlobalMarker>>;

/// Run condition.
/// Returns true if global state is set to the specified target.
pub fn in_state<S: State>(target: S::Repr) -> impl Fn(Global<&StateData<S>>) -> bool {
    move |state: Global<&StateData<S>>| state.current() == &target
}

/// Run condition.
/// Returns true if global state changed.
pub fn state_changed<S: State>(state: Global<&StateData<S>>) -> bool {
    state.is_updated()
}

/// Run condition.
/// Returns true if global state changed to the specified target.
pub fn state_changed_to<S: State>(target: S::Repr) -> impl Fn(Global<&StateData<S>>) -> bool {
    move |state: Global<&StateData<S>>| state.is_updated() && state.current() == &target
}
