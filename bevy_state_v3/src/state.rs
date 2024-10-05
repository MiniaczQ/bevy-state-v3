use std::{any::type_name, fmt::Debug, u32};

use bevy_ecs::{
    query::{QuerySingleError, With},
    schedule::{IntoSystemConfigs, Schedules},
    system::Populated,
    world::World,
};
use bevy_utils::tracing::warn;

use crate::{
    data::{RegisteredState, StateData},
    scheduling::{StateSystemSet, StateTransition},
    state_set::{StateDependencies, StateSet},
    transitions::StateConfig,
};

pub trait State: Sized + Clone + Debug + PartialEq + Send + Sync + 'static {
    type Dependencies: StateSet;
    type Update: StateUpdate;
    type Repr: StateRepr<State = Self>;

    const ORDER: u32 = Self::Dependencies::HIGHEST_ORDER + 1;

    fn update(state: &mut StateData<Self>, dependencies: StateDependencies<'_, Self>)
        -> Self::Repr;

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

    fn update_state_data_system(
        mut query: Populated<(
            &mut StateData<Self>,
            <Self::Dependencies as StateSet>::Query,
        )>,
    ) {
        for (mut state, dependencies) in query.iter_mut() {
            state.is_updated = false;
            let is_dependency_set_changed = Self::Dependencies::is_changed(&dependencies);
            let is_target_changed = state.waker.should_update();
            if is_dependency_set_changed || is_target_changed {
                let next = Self::update(&mut state, dependencies);
                state.update(next);
                state.waker.reset();
            }
        }
    }
}

pub trait StateUpdate: Default + Send + Sync + 'static {
    fn should_update(&self) -> bool;

    fn reset(&mut self);
}

impl<S: State> StateUpdate for Option<S> {
    fn should_update(&self) -> bool {
        self.is_some()
    }

    fn reset(&mut self) {
        self.take();
    }
}

impl<S: State> StateUpdate for Option<Option<S>> {
    fn should_update(&self) -> bool {
        self.is_some()
    }

    fn reset(&mut self) {
        self.take();
    }
}

impl StateUpdate for () {
    fn should_update(&self) -> bool {
        false
    }

    fn reset(&mut self) {}
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
