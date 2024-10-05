//! State related traits.

use std::{any::type_name, fmt::Debug, u32};

use bevy_ecs::{
    query::{QuerySingleError, With},
    schedule::{IntoSystemConfigs, Schedules},
    system::Populated,
    world::World,
};
use bevy_utils::tracing::warn;

use crate::{
    components::{RegisteredState, StateData},
    scheduling::{StateSystemSet, StateTransition},
    state_set::{StateDependencies, StateSet},
    transitions::StateConfig,
};

/// Types that exhibit state-like behavior.
pub trait State: Sized + Clone + Debug + PartialEq + Send + Sync + 'static {
    /// Dependencies for this state.
    /// Any update in dependencies will result in update of this state.
    type Dependencies: StateSet;

    /// Data structure for updating this state.
    type Update: StateUpdate;

    /// Internal representation of the state.
    type Repr: StateRepr<State = Self>;

    /// State update order in transition graph.
    const ORDER: u32 = Self::Dependencies::HIGHEST_ORDER + 1;

    /// Update function of this state.
    /// Implement manually for custom behavior.
    fn update(state: &mut StateData<Self>, dependencies: StateDependencies<'_, Self>)
        -> Self::Repr;

    /// Registers machinery for this state to work correctly.
    fn register_state(world: &mut World, transitions: StateConfig<Self>, recursive: bool) {
        Self::Dependencies::register_required_states(world);

        match world
            .query_filtered::<(), With<RegisteredState<Self>>>()
            .get_single(world)
        {
            Ok(_) => {
                // Skip warnings from recursive registers.
                if !recursive {
                    warn!(
                        "State {} is already registered, additional configuration will be ignored.",
                        type_name::<Self>()
                    );
                }
                return;
            }
            Err(QuerySingleError::MultipleEntities(_)) => {
                warn!(
                    "Failed to register state {}, edge already registered multiple times.",
                    type_name::<Self>()
                );
                return;
            }
            Err(QuerySingleError::NoEntities(_)) => {}
        }

        world.spawn(RegisteredState::<Self>::default());

        // Register systems for this state.
        let mut schedules = world.resource_mut::<Schedules>();
        let schedule = schedules.entry(StateTransition);
        schedule.configure_sets(StateSystemSet::configuration::<Self>());
        schedule
            .add_systems(Self::update_state_data_system.in_set(StateSystemSet::update::<Self>()));
        for system in transitions.systems {
            schedule.add_systems(system);
        }
    }

    /// System for updating this state.
    fn update_state_data_system(
        mut query: Populated<(&mut StateData<Self>, <Self::Dependencies as StateSet>::Data)>,
    ) {
        for (mut state, dependencies) in query.iter_mut() {
            state.is_updated = false;
            let is_dependency_set_changed = Self::Dependencies::is_changed(&dependencies);
            let is_target_changed = state.waker.should_update();
            if is_dependency_set_changed || is_target_changed {
                let next = Self::update(&mut state, dependencies);
                state.update(next);
                state.waker.post_update();
            }
        }
    }
}

/// Types that store state update data.
/// Implemented by by default for
/// - [`()`] - states with no manual updates,
/// - [`Option<S>`] - states with manual updates,
/// - [`Option<Option<S>>`] - optional states with manual updates.
pub trait StateUpdate: Debug + Default + Send + Sync + 'static {
    /// Whether the state should be updated.
    fn should_update(&self) -> bool;

    /// Reset function for after update.
    fn post_update(&mut self);
}

impl<S: State> StateUpdate for Option<S> {
    fn should_update(&self) -> bool {
        self.is_some()
    }

    fn post_update(&mut self) {
        self.take();
    }
}

impl<S: State> StateUpdate for Option<Option<S>> {
    fn should_update(&self) -> bool {
        self.is_some()
    }

    fn post_update(&mut self) {
        self.take();
    }
}

impl StateUpdate for () {
    fn should_update(&self) -> bool {
        false
    }

    fn post_update(&mut self) {}
}

/// Wrappers that can represent a state.
pub trait StateRepr: Clone + PartialEq + Send + Sync + 'static {
    /// Type of the raw state.
    type State: State<Repr = Self>;
}

/// Raw state, good for root states that always exist.
impl<S: State<Repr = S>> StateRepr for S {
    type State = S;
}

/// Optional state, good for computed/sub states which exist conditionally.
impl<S: State<Repr = Option<S>>> StateRepr for Option<S> {
    type State = S;
}
