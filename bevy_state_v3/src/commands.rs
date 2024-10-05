//! Helper methods for interacting with states.

use std::any::type_name;

use bevy_ecs::{
    entity::Entity,
    query::{QuerySingleError, With},
    system::Commands,
    world::{Command, World},
};
use bevy_utils::tracing::warn;

use crate::{
    components::StateData,
    state::{State, StateRepr},
    transitions::StateConfig,
    util::GlobalMarker,
};

struct InitializeStateCommand<S: State> {
    local: Option<Entity>,
    initial: S::Repr,
    suppress_initial_update: bool,
}

impl<S: State> InitializeStateCommand<S> {
    fn new(local: Option<Entity>, initial: S::Repr, suppress_initial_update: bool) -> Self {
        Self {
            local,
            initial,
            suppress_initial_update,
        }
    }
}

impl<S: State + Send + Sync + 'static> Command for InitializeStateCommand<S> {
    fn apply(self, world: &mut World) {
        let entity = match self.local {
            Some(entity) => entity,
            None => {
                let result = world
                    .query_filtered::<Entity, With<GlobalMarker>>()
                    .get_single(world);
                match result {
                    Ok(entity) => entity,
                    Err(QuerySingleError::NoEntities(_)) => world.spawn(GlobalMarker).id(),
                    Err(QuerySingleError::MultipleEntities(_)) => {
                        warn!("Insert global state command failed, multiple entities have the `GlobalStateMarker` component.");
                        return;
                    }
                }
            }
        };

        // Register storage for state `S`.
        let state_data = world
            .query::<&mut StateData<S>>()
            .get_mut(world, entity)
            .ok();
        match state_data {
            None => {
                world.entity_mut(entity).insert(StateData::<S>::new(
                    self.initial,
                    self.suppress_initial_update,
                ));
            }
            Some(_) => {
                warn!(
                    "Attempted to initialize state {}, but it was already present.",
                    type_name::<S>()
                );
            }
        }
    }
}

struct WakeStateTargetCommand<S: IntoStateUpdate> {
    local: Option<Entity>,
    update: S::Update,
}

impl<S: IntoStateUpdate> WakeStateTargetCommand<S> {
    fn new(local: Option<Entity>, update: S) -> Self {
        Self {
            local,
            update: update.into_state_update(),
        }
    }
}

fn target_entity(world: &mut World, local: Option<Entity>) -> Option<Entity> {
    match local {
        Some(entity) => Some(entity),
        None => {
            match world
                .query_filtered::<Entity, With<GlobalMarker>>()
                .get_single(world)
            {
                Err(QuerySingleError::NoEntities(_)) => {
                    warn!("Set global state command failed, no global state entity exists.");
                    return None;
                }
                Err(QuerySingleError::MultipleEntities(_)) => {
                    warn!("Set global state command failed, multiple global state entities exist.");
                    return None;
                }
                Ok(entity) => Some(entity),
            }
        }
    }
}

impl<S: IntoStateUpdate> Command for WakeStateTargetCommand<S> {
    fn apply(self, world: &mut World) {
        let Some(entity) = target_entity(world, self.local) else {
            return;
        };
        let mut entity = world.entity_mut(entity);
        let Some(mut state) = entity.get_mut::<StateData<S>>() else {
            warn!(
                "Set state command failed, entity does not have state {}",
                type_name::<S>()
            );
            return;
        };
        state.waker = self.update;
    }
}

/// States which can be converted to their [`State::Update`].
#[doc(hidden)]
pub trait IntoStateUpdate: State {
    fn into_state_update(self) -> Self::Update;
}

impl<S: State<Update = Option<S>>> IntoStateUpdate for S {
    fn into_state_update(self) -> Self::Update {
        Some(self)
    }
}

/// State related methods for [`Commands`], [`World`], [`SubApp`](bevy_app::SubApp) and [`App`](bevy_app::App).
#[doc(hidden)]
pub trait StatesExt {
    fn register_state<S: State>(&mut self, config: StateConfig<S>) -> &mut Self;

    fn init_state<R: StateRepr>(
        &mut self,
        local: Option<Entity>,
        initial: R,
        suppress_initial_update: bool,
    ) -> &mut Self;

    fn update_state<S: IntoStateUpdate>(&mut self, local: Option<Entity>, update: S) -> &mut Self;
}

impl StatesExt for Commands<'_, '_> {
    fn register_state<S: State>(&mut self, config: StateConfig<S>) -> &mut Self {
        self.queue(|world: &mut World| {
            S::register_state(world, config, false);
        });
        self
    }

    fn init_state<R: StateRepr>(
        &mut self,
        local: Option<Entity>,
        initial: R,
        suppress_initial_update: bool,
    ) -> &mut Self {
        self.queue(InitializeStateCommand::<R::State>::new(
            local,
            initial,
            suppress_initial_update,
        ));
        self
    }

    fn update_state<S: IntoStateUpdate>(&mut self, local: Option<Entity>, update: S) -> &mut Self {
        self.queue(WakeStateTargetCommand::<S>::new(local, update));
        self
    }
}

impl StatesExt for World {
    fn register_state<S: State>(&mut self, config: StateConfig<S>) -> &mut Self {
        S::register_state(self, config, false);
        self
    }

    fn init_state<R: StateRepr>(
        &mut self,
        local: Option<Entity>,
        initial: R,
        suppress_initial_update: bool,
    ) -> &mut Self {
        InitializeStateCommand::<R::State>::new(local, initial, suppress_initial_update)
            .apply(self);
        self
    }

    fn update_state<S: IntoStateUpdate>(&mut self, local: Option<Entity>, update: S) -> &mut Self {
        WakeStateTargetCommand::<S>::new(local, update).apply(self);
        self
    }
}

#[cfg(feature = "bevy_app")]
impl StatesExt for bevy_app::SubApp {
    fn register_state<S: State>(&mut self, config: StateConfig<S>) -> &mut Self {
        self.world_mut().register_state::<S>(config);
        self
    }

    fn init_state<R: StateRepr>(
        &mut self,
        local: Option<Entity>,
        initial: R,
        suppress_initial_update: bool,
    ) -> &mut Self {
        self.world_mut()
            .init_state(local, initial, suppress_initial_update);
        self
    }

    fn update_state<S: IntoStateUpdate>(&mut self, local: Option<Entity>, update: S) -> &mut Self {
        self.world_mut().update_state::<S>(local, update);
        self
    }
}

#[cfg(feature = "bevy_app")]
impl StatesExt for bevy_app::App {
    fn register_state<S: State>(&mut self, config: StateConfig<S>) -> &mut Self {
        self.main_mut().register_state::<S>(config);
        self
    }

    fn init_state<R: StateRepr>(
        &mut self,
        local: Option<Entity>,
        initial: R,
        suppress_initial_update: bool,
    ) -> &mut Self {
        self.main_mut()
            .init_state(local, initial, suppress_initial_update);
        self
    }

    fn update_state<S: IntoStateUpdate>(&mut self, local: Option<Entity>, update: S) -> &mut Self {
        self.main_mut().update_state::<S>(local, update);
        self
    }
}
