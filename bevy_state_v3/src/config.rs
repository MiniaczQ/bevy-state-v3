//! State configuration during registration.

use std::marker::PhantomData;

use bevy_ecs::{
    schedule::{IntoSystemConfigs, Schedules, SystemConfigs},
    world::World,
};

use crate::{
    prelude::{on_enter_transition, on_exit_transition},
    state::State,
    state_scoped::despawn_state_scoped,
    system_set::{StateTransitions, TransitionSystemSet},
    transitions::on_state_init,
};

/// State registration configuration.
/// Allows for configuration of enter/exit state systems like transitions and state scoped entities.
/// Configuration is only applied when registering state for the first time.
pub struct StateConfig<S: State> {
    defaults: bool,
    systems: Vec<SystemConfigs>,
    _state: PhantomData<S>,
}

impl<S: State> Default for StateConfig<S> {
    fn default() -> Self {
        Self {
            defaults: true,
            systems: vec![],
            _state: Default::default(),
        }
    }
}

impl<S: State> StateConfig<S> {
    /// Applies the configuration to the world.
    pub(crate) fn apply(self, world: &mut World) {
        let mut schedules = world.resource_mut::<Schedules>();
        let transition = schedules.entry(StateTransitions);

        if self.defaults {
            transition.add_systems((
                on_exit_transition::<S>.in_set(TransitionSystemSet::exit::<S>()),
                on_state_init::<S>
                    .in_set(TransitionSystemSet::enter::<S>())
                    .before(on_enter_transition::<S>),
                on_enter_transition::<S>.in_set(TransitionSystemSet::enter::<S>()),
                despawn_state_scoped::<S>.in_set(TransitionSystemSet::exit::<S>()),
            ));
        }

        for system in self.systems {
            transition.add_systems(system);
        }
    }

    /// Config that creates no transitions.
    /// For standard [`OnExit`] and [`OnEnter`] use the [`StateTransitionsConfig::default`].
    pub fn empty() -> Self {
        Self {
            defaults: false,
            systems: vec![],
            _state: PhantomData,
        }
    }

    /// Adds a system to run when state is exited.
    /// Example systems:
    /// - [`on_exit_transition<S>`](crate::transitions::on_exit_transition),
    /// - [`on_reexit_transition<S>`](crate::transitions::on_reexit_transition),
    /// - [`despawn_state_scoped<S>`](crate::state_scoped::despawn_state_scoped).
    pub fn with_on_exit<M>(mut self, system: impl IntoSystemConfigs<M>) -> Self {
        self.systems
            .push(system.in_set(TransitionSystemSet::exit::<S>()));
        self
    }

    /// Adds a system to run when state is entered.
    /// Example systems:
    /// - [`on_enter_transition<S>`](crate::transitions::on_enter_transition),
    /// - [`on_reenter_transition<S>`](crate::transitions::on_reenter_transition).
    pub fn with_on_enter<M>(mut self, system: impl IntoSystemConfigs<M>) -> Self {
        self.systems
            .push(system.in_set(TransitionSystemSet::enter::<S>()));
        self
    }
}
