//! Integration with Bevy App.

use bevy_app::{MainScheduleOrder, Plugin, PreStartup, PreUpdate};

use crate::system_set::{StateTransitions, StateUpdates};

/// Plugin state registers:
/// - [`StateUpdates`] schedule, which uses state's update data and dependencies to set the new value of a state,
/// - [`StateTransitions`] schedule, which uses buffered update data to run exit/enter systems and transition events.
///
/// State updates and transitions run in the main schedule "inbetween" frames, meanwhile
/// in startup only the transition schedule is executed to trigger initial transition events.
#[derive(Default)]
pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut bevy_app::App) {
        let mut schedule = app.world_mut().resource_mut::<MainScheduleOrder>();
        schedule.insert_startup_before(PreStartup, StateTransitions);
        schedule.insert_after(PreUpdate, StateUpdates);
        schedule.insert_after(PreUpdate, StateTransitions);
    }
}
