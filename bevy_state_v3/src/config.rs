//! State configuration during registration.

use std::marker::PhantomData;

use bevy_ecs::schedule::{IntoSystemConfigs, SystemConfigs};

use crate::{
    prelude::{on_enter_transition, on_exit_transition},
    state::State,
    system_set::StateSystemSet,
};

/// State registration configuration.
/// Currently only transitions can be configured.
/// Configuration is only applied when registering state for the first time.
pub struct StateConfig<S: State> {
    pub(crate) systems: Vec<SystemConfigs>,
    pub(crate) state_scoped: bool,
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
            state_scoped: true,
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
            state_scoped: false,
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

    /// Enabled state scoped entities for this state.
    pub fn with_state_scoped(mut self) -> Self {
        self.state_scoped = true;
        self
    }
}
