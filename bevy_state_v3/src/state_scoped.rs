//! Components and systems for state scoped entities.

use bevy_ecs::{
    component::Component,
    entity::Entity,
    system::{Commands, Populated},
};
use bevy_hierarchy::DespawnRecursiveExt;

use crate::{prelude::StateData, state::State, util::Global};

/// Entities marked with this component will be deleted when provided state is exited.
#[derive(Component)]
pub struct StateScoped<S: State>(pub S::Repr);

/// System for despawning scoped entities when exiting a state.
pub fn despawn_state_scoped<S: State>(
    mut commands: Commands,
    state: Global<&StateData<S>>,
    query: Populated<(Entity, &StateScoped<S>)>,
) {
    let Some(exited) = state.previous() else {
        return;
    };
    for (entity, scope) in query.iter() {
        if &scope.0 == exited {
            commands.entity(entity).try_despawn_recursive();
        }
    }
}
