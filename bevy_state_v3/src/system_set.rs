//! System set for scheduling state transitions.

use bevy_ecs::schedule::{
    InternedSystemSet, IntoScheduleConfigs, ScheduleConfigs, ScheduleLabel, SystemSet,
};

use crate::state::State;

/// Schedule where states get updated.
/// This updates the `current`, `previous` and `is_reentrant` state values
/// as well as the `is_updated` flag.
#[derive(ScheduleLabel, Debug, PartialEq, Eq, Hash, Clone)]
pub struct StateUpdates;

/// Updates run from root states to leaf states.
/// Exits run from leaf states to root states.
/// Enters run from root states to leaf states.
#[derive(SystemSet, Clone, Debug, PartialEq, Eq, Hash)]
pub enum StateSystemSet {
    /// All [`Update`]s.
    AllUpdates,
    /// Lower values before higher ones.
    Update(u32),
    /// All [`Exit`]s.
    AllExits,
    /// Higher values then lower ones.
    Exit(u32),
    /// All [`Enter`]s.
    AllEnters,
    /// Same as [`Update`], lower values before higher ones.
    Enter(u32),
}

impl StateSystemSet {
    /// Returns system set used to update this state.
    pub fn update<S: State>() -> Self {
        Self::Update(S::ORDER)
    }

    /// Returns system set used to run exit transitions for this state.
    pub fn exit<S: State>() -> Self {
        Self::Exit(S::ORDER)
    }

    /// Returns system set used to run enter transitions for this state.
    pub fn enter<S: State>() -> Self {
        Self::Enter(S::ORDER)
    }

    /// Returns system set configuration for this set.
    pub fn configuration<S: State>() -> ScheduleConfigs<InternedSystemSet> {
        (
            (Self::AllUpdates, Self::AllExits, Self::AllEnters).chain(),
            (
                Self::update::<S>()
                    .after(Self::Update(S::ORDER - 1))
                    .in_set(Self::AllUpdates),
                Self::exit::<S>()
                    .before(Self::Exit(S::ORDER - 1))
                    .in_set(Self::AllExits),
                Self::enter::<S>()
                    .after(Self::Enter(S::ORDER - 1))
                    .in_set(Self::AllEnters),
            ),
        )
            .into_configs()
    }
}
