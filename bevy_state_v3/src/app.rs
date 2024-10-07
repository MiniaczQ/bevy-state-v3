//! Integration with app-level Bevy.

use bevy_app::{MainScheduleOrder, Plugin, PreStartup, PreUpdate};

use crate::system_set::{StateTransitions, StateUpdates};

/// Plugin that registers state transition schedule.
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
