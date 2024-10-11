//! State related traits.

use core::fmt::Debug;

use bevy_ecs::{
    query::{QuerySingleError, With},
    schedule::{IntoSystemConfigs, Schedules},
    system::Populated,
    world::World,
};
use bevy_utils::tracing::warn;

use crate::{
    components::{RegisteredState, StateData},
    config::StateConfig,
    state_scoped::despawn_state_scoped,
    state_set::{StateSet, StateSetData},
    system_set::{StateTransitions, StateUpdates, TransitionSystemSet, UpdateSystemSet},
};

/// Trait for states in a hierarchy.
/// The implementation detail impact how state value will change dependning on user-requested data as well as parent states.
/// It will also impact what kind of data is stored inside the [`StateData`] component.
///
/// # Derive macro
///
/// Simplest implementation of this trait can be done through the [`State`](bevy_state_macros::State) derive macro:
/// ```rs
/// #[derive(State, Debug, Clone, PartialEq)]
/// struct MyState {
///     Alice,
///     Bob,
/// }
/// ```
/// in which case the state will be non-optional and mutation will be done by providing a new state, which will overwrite the previous one.
/// The [`Debug`], [`Clone`], [`PartialEq`] traits are required by the [`States`] trait.
///
/// The macro also provides a simple substate:
/// ```rs
/// #[derive(State, Default, Debug, Clone, PartialEq)]
/// #[dependency(MyState = MyState::Alice)]
/// struct MySubstate {
///     #[default]
///     Foo,
///     Bar,
/// }
/// ```
/// In this case the state will be optional, only existing if parent state has the correct value.
/// Mutation is done in the exact same way, by providing a new value.
/// If the state is currently disabled, the update value will be lost.
/// Additionally the [`Default`] trait is required to select a value if update was not set.
///
/// # Manual implementation
///
/// Manual implementation is very helpful for non-basic use cases and heavily encouraged.
/// For example, the substate macro can be implemented as:
/// ```rs
/// #[derive(Default, Debug, Clone, PartialEq)]
/// struct MySubstate {
///     #[default]
///     Foo,
///     Bar,
/// }
///
/// impl State for MySubstate {
///     // One parent state.
///     type Dependencies = MyState;
///     // Update data structure, if the value is `Some` it will be taken as the next value (as long as parent state is correct).
///     type Update = Option<Self>;
///     // State is represented as an optional value, it can be `None` if parent is not in the correct state.
///     type Repr = Option<Self>;
///
///     // Function that calculates the next value of this state if:
///     // - update was requested through the data structure,
///     // - any parent state was updated.
///     fn update(state: &mut StateData<Self>, dependencies: StateDependencies<'_, Self>) -> Self::Repr {
///         // `StateDependencies` is a helper type which returns tuples of state data `(&StateData<S1>, &StateData<S2>)`, rather than raw state dependencies `(S1, S2)`.
///         // A good practice is to destructure it into our specific dependencies first.
///         let my_state = dependencies;
///         match my_state {
///             // If parent state is correct, this state will be whatever the update value is or default.
///             MyState::Alice => Some(state.update_mut().take().unwrap_or_default()),
///             // If parent state is incorrect, this state will always be `None`.
///             _ => None,
///         }
///     }
///
///     // All other trait members have default implementations and it's not recommended to modify them.
/// }
/// ```

pub trait State: Sized + Clone + Debug + PartialEq + Send + Sync + 'static {
    /// Dependencies for this state.
    /// Any update in dependencies will result in update of this state.
    /// Dependency can be either:
    /// - [`()`] - empty tuple for no dependencies,
    /// - [`S`] - single parent state,
    /// - [`(S1, S2, ...)`] - tuple of up to 15 states.
    type Dependencies: StateSet;

    /// Data structure for updating this state.
    /// By default this is implemented for:
    /// - [`()`] - no updates, mutations only through parent states,
    /// - [`Option<Self>`]/[`Option<Option<Self>>`] - manual mutation, triggers update if the outer `Option` is `Some`.
    ///
    /// but custom implementations are possible, few representative ones can be found in the examples.
    type Update: StateUpdate;

    /// Internal representation of the state.
    /// This can be either:
    /// - [`Self`] - if state is non-optional,
    /// - [`Option<Self>`] - if state is optional.
    type Repr: StateRepr<State = Self>;

    /// State update order in transition graph.
    /// Never manually overwrite, always use the derived value.
    const ORDER: u32 = Self::Dependencies::HIGHEST_ORDER + 1;

    /// Update function of this state.
    /// Implement manually for custom behavior.
    fn update(
        state: &mut StateData<Self>,
        dependencies: StateSetData<'_, Self::Dependencies>,
    ) -> Self::Repr;

    /// Registers machinery for this state type to work correctly.
    fn register_state(world: &mut World, config: StateConfig<Self>) {
        // TODO: check states plugin

        match world
            .query_filtered::<(), With<RegisteredState<Self>>>()
            .get_single(world)
        {
            Ok(_) => {
                warn!(
                    "State {} is already registered.",
                    disqualified::ShortName::of::<Self>()
                );
                return;
            }
            Err(QuerySingleError::MultipleEntities(_)) => {
                panic!(
                    "Found multiple {} state registrations which is invalid.",
                    disqualified::ShortName::of::<Self>()
                );
            }
            Err(QuerySingleError::NoEntities(_)) => {}
        }

        world.spawn(RegisteredState::<Self>::default());

        // Register systems for this state.
        let mut schedules = world.resource_mut::<Schedules>();

        let update = schedules.entry(StateUpdates);
        update.configure_sets(UpdateSystemSet::configuration::<Self>());
        update
            .add_systems(Self::update_state_data_system.in_set(UpdateSystemSet::update::<Self>()));

        let transition = schedules.entry(StateTransitions);
        transition.configure_sets(TransitionSystemSet::configuration::<Self>());
        for system in config.systems {
            transition.add_systems(system);
        }
        if config.state_scoped {
            transition.add_systems(
                despawn_state_scoped::<Self>.in_set(TransitionSystemSet::exit::<Self>()),
            );
        }
    }

    /// System that updates the value of this state.
    fn update_state_data_system(
        mut query: Populated<(
            &mut StateData<Self>,
            <Self::Dependencies as StateSet>::Query,
        )>,
    ) {
        for (mut state, dependencies) in query.iter_mut() {
            state.is_updated = false;
            let dependency_updated = Self::Dependencies::is_updated(&dependencies);
            let state_should_update = state.update.should_update();
            if dependency_updated || state_should_update {
                let next = Self::update(&mut state, dependencies);
                state.inner_update(next);
                state.update.post_update();
            }
        }
    }
}

/// Types that store state update data.
/// Implemented by by default for:
/// - [`()`] - states with no manual updates,
/// - [`Option<S>`] - states with manual updates,
/// - [`Option<Option<S>>`] - optional states with manual updates.
pub trait StateUpdate: Debug + Default + Send + Sync + 'static {
    /// Whether the state should be updated this frame.
    fn should_update(&self) -> bool;

    /// Reset function for after update happened.
    /// This is a good place for reseting update flags.
    /// It's best to not rely on [`State::update`] to reset flags.
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

/// Possible state representations.
/// Currently this only allows for non-optional or optional states.
pub trait StateRepr: Debug + Clone + PartialEq + Send + Sync + 'static {
    /// Mapping back to the state type.
    type State: State<Repr = Self>;

    /// Helper for converting state representation into state data.
    fn into_data(self) -> StateData<Self::State> {
        StateData::new(self)
    }
}

impl<S: State<Repr = S>> StateRepr for S {
    type State = S;
}

impl<S: State<Repr = Option<S>>> StateRepr for Option<S> {
    type State = S;
}
