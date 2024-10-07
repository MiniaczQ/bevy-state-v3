//! System set for scheduling state transitions.

use bevy_ecs::schedule::{IntoSystemSetConfigs, ScheduleLabel, SystemSet};

use crate::state::State;

#[derive(ScheduleLabel, Debug, PartialEq, Eq, Hash, Clone)]
pub struct StateUpdates;

#[derive(ScheduleLabel, Debug, PartialEq, Eq, Hash, Clone)]
pub struct StateTransitions;

/// - [`AllUpdates`] - Updates based on `target` and dependency changes from root states to leaf states, sets the `updated` flag.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum UpdateSystemSet {
    /// All [`Update`]s.
    AllUpdates,
    /// Lower values before higher ones.
    Update(u32),
}

#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
/// - [`AllExits`] - Triggers [`StateExit<S>`] observers from leaf states to root states, targeted for local state, untargeted for global state.
/// - [`AllEnters`] - Triggers [`StateEnter<S>`] observers from root states to leaf states, targeted for local state, untargeted for global state.
pub enum TransitionSystemSet {
    /// All [`Exit`]s.
    AllExits,
    /// Higher values then lower ones.
    Exit(u32),
    /// All [`Enter`]s.
    AllEnters,
    /// Same as [`Update`], lower values before higher ones.
    Enter(u32),
}

impl UpdateSystemSet {
    /// Returns system set used to update this state.
    pub fn update<S: State>() -> Self {
        Self::Update(S::ORDER)
    }
    /// Returns system set configuration for this set.
    pub fn configuration<S: State>() -> impl IntoSystemSetConfigs {
        (
            Self::AllUpdates,
            Self::update::<S>()
                .after(Self::Update(S::ORDER - 1))
                .in_set(Self::AllUpdates),
        )
    }
}

impl TransitionSystemSet {
    /// Returns system set used to run exit transitions for this state.
    pub fn exit<S: State>() -> Self {
        Self::Exit(S::ORDER)
    }

    /// Returns system set used to run enter transitions for this state.
    pub fn enter<S: State>() -> Self {
        Self::Enter(S::ORDER)
    }

    /// Returns system set configuration for this set.
    pub fn configuration<S: State>() -> impl IntoSystemSetConfigs {
        (
            (Self::AllExits, Self::AllEnters).chain(),
            Self::exit::<S>()
                .before(Self::Exit(S::ORDER - 1))
                .in_set(Self::AllExits),
            Self::enter::<S>()
                .after(Self::Enter(S::ORDER - 1))
                .in_set(Self::AllEnters),
        )
    }
}
