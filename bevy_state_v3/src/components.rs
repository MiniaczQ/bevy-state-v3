//! State related components.

use std::marker::PhantomData;

use bevy_ecs::{
    component::{Component, ComponentId, Components, RequiredComponents, StorageType},
    storage::Storages,
};
#[cfg(feature = "bevy_reflect")]
use bevy_reflect::prelude::*;

use crate::{state::State, state_set::StateSet};

/// Component that stores state data.
#[derive(Debug)]
#[cfg_attr(feature = "bevy_reflect", derive(Reflect))]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    all(feature = "serialize", feature = "bevy_reflect"),
    reflect(Serialize, Deserialize)
)]
pub struct StateData<S: State> {
    /// Whether this state was reentered.
    pub(crate) is_reentrant: bool,

    /// Last different state value.
    pub(crate) previous: Option<S::Repr>,

    /// Current value of the state.
    pub(crate) current: S::Repr,

    /// Proposed state value to be considered during next [`StateTransition`](crate::state::StateTransition).
    /// How this value actually impacts the state depends on the [`State::update`] function.
    pub(crate) update: S::Update,

    /// Whether this state was updated in the last [`StateTransition`] schedule.
    /// For a standard use case, this happens once per frame.
    pub(crate) is_updated: bool,
}

impl<S> Default for StateData<S>
where
    S: State,
    S::Repr: Default,
{
    fn default() -> Self {
        Self {
            is_reentrant: false,
            previous: None,
            current: Default::default(),
            update: Default::default(),
            is_updated: false,
        }
    }
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
    /// Update current state.
    pub(crate) fn inner_update(&mut self, next: S::Repr) {
        if next == self.current {
            self.is_reentrant = true;
        } else {
            self.is_reentrant = false;
            self.previous = Some(core::mem::replace(&mut self.current, next));
        }
    }

    /// Creates a new instance with initial value.
    pub fn new(initial: S::Repr) -> Self {
        Self {
            current: initial.clone(),
            previous: None,
            is_reentrant: false,
            update: S::Update::default(),
            is_updated: false,
        }
    }

    /// Returns the current state.
    pub fn current(&self) -> &S::Repr {
        &self.current
    }

    /// Returns the last different state.
    /// If the current state was reentered, this value will remain unchanged,
    /// instead the [`Self::is_reentrant()`] flag will be raised.
    pub fn previous(&self) -> Option<&S::Repr> {
        self.previous.as_ref()
    }

    /// Returns the previous state with reentries included.
    pub fn reentrant_previous(&self) -> Option<&S::Repr> {
        if self.is_reentrant {
            Some(self.current())
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

    /// Reference to the state update.
    pub fn update(&self) -> &S::Update {
        &self.update
    }

    /// Mutable reference to the state update.
    pub fn update_mut(&mut self) -> &mut S::Update {
        &mut self.update
    }
}

/// Component for tracking registered states.
#[derive(Component)]
pub struct RegisteredState<S: State>(PhantomData<S>);

impl<S: State> Default for RegisteredState<S> {
    fn default() -> Self {
        Self(Default::default())
    }
}
