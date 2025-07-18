//! State configuration during registration.

use bevy_ecs::{
    schedule::{IntoScheduleConfigs, Schedules},
    world::World,
};

use crate::{
    prelude::{
        on_enter_transition, on_exit_transition, on_reenter_transition, on_reexit_transition,
    },
    state::State,
    state_scoped::despawn_state_scoped,
    system_set::{StateSystemSet, StateUpdates},
    transitions::{on_deinit_transition, on_init_transition},
};

/// State registration configuration.
/// Allows for configuration of enter/exit state systems like transitions and state scoped entities.
/// Configuration is only applied when registering state for the first time.
pub struct StateConfig {
    state_scoped: bool,
    on_enter: bool,
    on_exit: bool,
    on_reenter: bool,
    on_reexit: bool,
    on_init: bool,
    on_deinit: bool,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            state_scoped: true,
            on_enter: true,
            on_exit: true,
            on_reenter: false,
            on_reexit: false,
            on_init: true,
            on_deinit: true,
        }
    }
}

impl StateConfig {
    /// Applies the configuration to the world.
    pub(crate) fn apply<S: State>(self, world: &mut World) {
        let mut schedules = world.resource_mut::<Schedules>();
        let schedule = schedules.entry(StateUpdates);
        if self.state_scoped {
            schedule.add_systems(despawn_state_scoped::<S>.in_set(StateSystemSet::exit::<S>()));
        }
        if self.on_enter {
            schedule.add_systems(on_enter_transition::<S>.in_set(StateSystemSet::enter::<S>()));
        }
        if self.on_exit {
            schedule.add_systems(on_exit_transition::<S>.in_set(StateSystemSet::exit::<S>()));
        }
        if self.on_reenter {
            schedule.add_systems(on_reenter_transition::<S>.in_set(StateSystemSet::enter::<S>()));
        }
        if self.on_reexit {
            schedule.add_systems(on_reexit_transition::<S>.in_set(StateSystemSet::exit::<S>()));
        }

        if self.on_init {
            world.add_observer(on_init_transition::<S>);
        }
        if self.on_deinit {
            world.add_observer(on_deinit_transition::<S>);
        }
    }

    /// Config that creates no transitions.
    /// For standard [`OnExit`] and [`OnEnter`] use the [`StateTransitionsConfig::default`].
    pub fn empty() -> Self {
        Self {
            state_scoped: false,
            on_enter: false,
            on_exit: false,
            on_reenter: false,
            on_reexit: false,
            on_init: false,
            on_deinit: false,
        }
    }

    /// Sets whether state scoped entity despawning will be enabled.
    pub fn with_state_scoped(mut self, enabled: bool) -> Self {
        self.state_scoped = enabled;
        self
    }

    /// Sets whether state on enter transition will be enabled.
    pub fn with_on_enter(mut self, enabled: bool) -> Self {
        self.on_enter = enabled;
        self
    }

    /// Sets whether state on exit transition will be enabled.
    pub fn with_on_exit(mut self, enabled: bool) -> Self {
        self.on_exit = enabled;
        self
    }

    /// Sets whether state on re-enter transition will be enabled.
    pub fn with_on_reenter(mut self, enabled: bool) -> Self {
        self.on_reenter = enabled;
        self
    }

    /// Sets whether state on re-exit transition will be enabled.
    pub fn with_on_reexit(mut self, enabled: bool) -> Self {
        self.on_reexit = enabled;
        self
    }

    /// Sets whether state init transition will be enabled.
    pub fn with_on_init(mut self, enabled: bool) -> Self {
        self.on_init = enabled;
        self
    }

    /// Sets whether state de-init transition will be enabled.
    pub fn with_on_deinit(mut self, enabled: bool) -> Self {
        self.on_deinit = enabled;
        self
    }
}
