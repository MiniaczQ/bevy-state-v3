use std::marker::PhantomData;

use bevy_ecs::{
    component::{Component, ComponentId, Components, RequiredComponents, StorageType},
    storage::Storages,
};

use crate::{state::State, state_set::StateSet};

/// State data component.
#[derive(Debug)]
pub struct StateData<S: State> {
    /// Whether this state was reentered.
    pub(crate) is_reentrant: bool,
    /// Last different state value.
    pub(crate) previous: S::Repr,
    /// Current value of the state.
    pub(crate) current: S::Repr,
    /// Proposed state value to be considered during next [`StateTransition`](crate::state::StateTransition).
    /// How this value actually impacts the state depends on the [`State::update`] function.
    pub(crate) waker: S::Update,
    /// Whether this state was updated in the last [`StateTransition`] schedule.
    /// For a standard use case, this happens once per frame.
    pub(crate) is_updated: bool,
}

impl<S: State> Component for StateData<S> {
    const STORAGE_TYPE: StorageType = StorageType::Table;

    fn register_required_components(
        component_id: ComponentId,
        components: &mut Components,
        storages: &mut Storages,
        required_components: &mut RequiredComponents,
        inheritance_depth: u16,
    ) {
        <S::Dependencies as StateSet>::register_required_components(
            component_id,
            components,
            storages,
            required_components,
            inheritance_depth,
        );
    }
}

impl<S: State> StateData<S> {
    pub(crate) fn update(&mut self, next: S::Repr) {
        if next == self.current {
            self.is_reentrant = true;
        } else {
            self.is_reentrant = false;
            self.previous = core::mem::replace(&mut self.current, next);
        }
        self.is_updated = true;
    }
}

impl<S: State> Default for StateData<S> {
    fn default() -> Self {
        Self::new(S::Repr::default(), false)
    }
}

impl<S: State> StateData<S> {
    /// Creates a new instance with initial value.
    pub fn new(initial: S::Repr, suppress_initial_update: bool) -> Self {
        Self {
            current: initial.clone(),
            previous: initial,
            is_updated: !suppress_initial_update,
            is_reentrant: true,
            waker: S::Update::default(),
        }
    }

    /// Returns the current state.
    pub fn current(&self) -> &S::Repr {
        &self.current
    }

    /// Returns the last different state.
    /// If the current state was reentered, this value will remain unchanged,
    /// instead the [`Self::is_reentrant()`] flag will be raised.
    pub fn previous(&self) -> &S::Repr {
        &self.previous
    }

    /// Returns the previous state with reentries included.
    pub fn reentrant_previous(&self) -> &S::Repr {
        if self.is_reentrant {
            self.current()
        } else {
            self.previous()
        }
    }

    /// Returns whether the current state was reentered.
    pub fn is_reentrant(&self) -> bool {
        self.is_reentrant
    }

    /// Returns whether the current state was updated last state transition.
    pub fn is_updated(&self) -> bool {
        self.is_updated
    }

    /// Reference to the target.
    pub fn target(&self) -> &S::Update {
        &self.waker
    }

    /// Mutable reference to the target.
    pub fn target_mut(&mut self) -> &mut S::Update {
        &mut self.waker
    }
}

/// Used to keep track of which states are registered and which aren't.
#[derive(Component)]
pub struct RegisteredState<S: State>(PhantomData<S>);

impl<S: State> Default for RegisteredState<S> {
    fn default() -> Self {
        Self(Default::default())
    }
}
