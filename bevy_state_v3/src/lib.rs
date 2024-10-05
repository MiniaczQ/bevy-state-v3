//! State machines.

#![allow(unsafe_code)]

#[cfg(feature = "bevy_app")]
pub mod app;
pub mod commands;
pub mod components;
pub mod config;
pub mod state;
pub mod state_scoped;
pub mod state_set;
pub mod system_set;
pub mod transitions;
pub mod util;

/// Re-export of common state types and functions.
pub mod prelude {
    #[cfg(feature = "bevy_app")]
    pub use crate::app::StatePlugin;
    pub use crate::commands::{IntoStateUpdate, StatesExt};
    pub use crate::components::StateData;
    pub use crate::config::StateConfig;
    pub use crate::state::{State, StateRepr, StateUpdate};
    pub use crate::state_scoped::StateScoped;
    pub use crate::state_set::{StateDependencies, StateSet};
    pub use crate::transitions::{
        on_enter_transition, on_exit_transition, on_reenter_transition, on_reexit_transition,
        OnEnter, OnExit, OnReenter, OnReexit,
    };
    pub use crate::util::{in_state, state_changed, state_changed_to, Global};

    pub use bevy_state_macros::State;
}

#[cfg(test)]
mod tests {
    use std::{any::type_name, fmt::Debug};

    use bevy_ecs::{
        entity::Entity,
        event::Event,
        observer::Trigger,
        schedule::Schedules,
        system::{ResMut, Resource},
        world::World,
    };
    use bevy_state_macros::State;

    use crate::{
        self as bevy_state_v3,
        config::StateConfig,
        state_set::StateDependencies,
        system_set::StateTransition,
        transitions::{OnEnter, OnExit},
    };
    use crate::{commands::StatesExt, components::StateData, state::State};

    #[derive(State, Default, Clone, Debug, PartialEq)]
    enum ManualState {
        #[default]
        A,
        B,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct ComputedState;

    impl State for ComputedState {
        type Dependencies = ManualState;
        type Update = ();
        type Repr = Option<Self>;

        fn update<'a>(
            _state: &mut StateData<Self>,
            dependencies: StateDependencies<'_, Self>,
        ) -> Self::Repr {
            let manual = dependencies;
            match manual.current() {
                ManualState::A => Some(ComputedState),
                _ => None,
            }
        }
    }

    #[derive(State, Clone, Debug, Default, PartialEq)]
    #[dependency(ManualState = ManualState::B)]
    enum SubState {
        #[default]
        X,
        Y,
    }

    macro_rules! assert_states {
        ($world:expr, $(($ty:ident, $state:expr)),* $(,)*) => {
            $(assert_eq!($world.query::<&StateData<$ty>>().single($world).current, $state));*
        };
    }

    fn test_all_states(world: &mut World, local: Option<Entity>) {
        world.init_resource::<Schedules>();
        world.register_state(StateConfig::<ManualState>::empty());
        world.register_state(StateConfig::<ComputedState>::empty());
        world.register_state(StateConfig::<SubState>::empty());
        world.init_state(local, ManualState::A);
        world.init_state(local, None::<ComputedState>);
        world.init_state(local, None::<SubState>);
        world.update_state(local, ManualState::A);
        world.run_schedule(StateTransition);
        assert_states!(
            world,
            (ManualState, ManualState::A),
            (ComputedState, Some(ComputedState)),
            (SubState, None),
        );

        world.update_state(local, ManualState::B);
        world.run_schedule(StateTransition);
        assert_states!(
            world,
            (ManualState, ManualState::B),
            (ComputedState, None),
            (SubState, Some(SubState::X)),
        );

        world.update_state(local, SubState::Y);
        world.run_schedule(StateTransition);
        assert_states!(
            world,
            (ManualState, ManualState::B),
            (ComputedState, None),
            (SubState, Some(SubState::Y)),
        );
    }

    #[test]
    fn global_state() {
        let mut world = World::new();
        let local = None;
        test_all_states(&mut world, local);
    }

    #[test]
    fn local_state() {
        let mut world = World::new();
        let local = Some(world.spawn_empty().id());
        test_all_states(&mut world, local);
    }

    #[derive(Default, Resource)]
    struct StateTransitionTracker(Vec<&'static str>);

    fn track<E: Event>() -> impl Fn(Trigger<E>, ResMut<StateTransitionTracker>) {
        move |_: Trigger<E>, mut reg: ResMut<StateTransitionTracker>| {
            reg.0.push(type_name::<E>());
        }
    }

    #[derive(State, Default, Clone, Debug, PartialEq)]
    enum ManualState2 {
        #[default]
        C,
        D,
    }

    #[derive(Clone, Debug, Default, PartialEq)]
    enum SubState2 {
        #[default]
        X,
        Y,
    }

    impl State for SubState2 {
        type Dependencies = (ManualState, ManualState2);
        type Update = Option<Self>;
        type Repr = Option<Self>;

        fn update<'a>(
            state: &mut StateData<Self>,
            dependencies: StateDependencies<'_, Self>,
        ) -> Option<Self> {
            let (manual1, manual2) = dependencies;
            match (
                manual1.current(),
                manual2.current(),
                state.target_mut().take(),
            ) {
                (ManualState::B, ManualState2::D, Some(next)) => Some(next),
                (ManualState::B, ManualState2::D, None) => Some(SubState2::X),
                _ => None,
            }
        }
    }

    #[test]
    fn transition_order() {
        let mut world = World::new();
        world.init_resource::<Schedules>();
        world.register_state(StateConfig::<ManualState>::default());
        world.register_state(StateConfig::<ManualState2>::default());
        world.register_state(StateConfig::<SubState2>::default());
        world.register_state(StateConfig::<ComputedState>::default());
        world.init_state(None, ManualState::A);
        world.init_state(None, ManualState2::C);
        world.init_state(None, None::<SubState2>);
        world.init_state(None, None::<ComputedState>);
        world.update_state(None, ManualState::A);
        world.update_state(None, ManualState2::C);
        world.update_state(None, SubState2::Y);
        world.run_schedule(StateTransition);

        world.init_resource::<StateTransitionTracker>();
        world.observe(track::<OnExit<ManualState>>());
        world.observe(track::<OnEnter<ManualState>>());
        world.observe(track::<OnExit<ManualState2>>());
        world.observe(track::<OnEnter<ManualState2>>());
        world.observe(track::<OnExit<SubState2>>());
        world.observe(track::<OnEnter<SubState2>>());
        world.observe(track::<OnExit<ComputedState>>());
        world.observe(track::<OnEnter<ComputedState>>());
        world.update_state(None, ManualState::B);
        world.update_state(None, ManualState2::D);
        world.run_schedule(StateTransition);

        let transitions = &world.resource::<StateTransitionTracker>().0;
        // Test in groups, because order of directly unrelated states is non-deterministic.
        assert!(transitions[0..=1].contains(&type_name::<OnExit<SubState2>>()));
        assert!(transitions[0..=1].contains(&type_name::<OnExit<ComputedState>>()));
        assert!(transitions[2..=3].contains(&type_name::<OnExit<ManualState>>()));
        assert!(transitions[2..=3].contains(&type_name::<OnExit<ManualState2>>()));
        assert!(transitions[4..=5].contains(&type_name::<OnEnter<ManualState>>()));
        assert!(transitions[4..=5].contains(&type_name::<OnEnter<ManualState2>>()));
        assert!(transitions[6..=7].contains(&type_name::<OnEnter<SubState2>>()));
        assert!(transitions[6..=7].contains(&type_name::<OnEnter<ComputedState>>()));
    }

    // Debug stuff

    #[allow(unused_macros)]
    macro_rules! print_states {
        ($world:expr, $($state:ty),+) => {
            $(println!("{:?}", $world.query::<&StateData<$state>>().single($world)));+
        };
    }
}
