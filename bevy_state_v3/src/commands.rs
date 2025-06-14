//! Helper methods for interacting with states.

use bevy_ecs::{
    prelude::{Command, Commands, Entity, Result, With, World},
    query::QuerySingleError,
};
use bevy_log::warn;

use crate::{
    components::StateData,
    config::StateConfig,
    state::{State, StateRepr},
    util::GlobalMarker,
};

struct InitializeStateCommand<S: State> {
    local: Option<Entity>,
    initial: S::Repr,
}

impl<S: State> InitializeStateCommand<S> {
    fn new(local: Option<Entity>, initial: S::Repr) -> Self {
        Self { local, initial }
    }
}

impl<S: State + Send + Sync + 'static> Command<Result> for InitializeStateCommand<S> {
    fn apply(self, world: &mut World) -> Result {
        let entity = match self.local {
            Some(entity) => entity,
            None => {
                let result = world
                    .query_filtered::<Entity, With<GlobalMarker>>()
                    .single(world);
                match result {
                    Ok(entity) => entity,
                    Err(QuerySingleError::NoEntities(_)) => world.spawn(GlobalMarker).id(),
                    Err(QuerySingleError::MultipleEntities(_)) => {
                        warn!(
                            "Insert global state command failed, multiple entities have the `GlobalStateMarker` component."
                        );
                        return Ok(());
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
                world
                    .entity_mut(entity)
                    .insert(StateData::<S>::new(self.initial));
            }
            Some(_) => {
                warn!(
                    "Attempted to initialize state {}, but it was already present.",
                    disqualified::ShortName::of::<S>()
                );
            }
        }
        Ok(())
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

/// Conversion from local/global [`Option<Entity>`] to [`Entity`] for states.
pub fn state_target_entity(world: &mut World, local: Option<Entity>) -> Option<Entity> {
    match local {
        Some(entity) => Some(entity),
        None => {
            match world
                .query_filtered::<Entity, With<GlobalMarker>>()
                .single(world)
            {
                Err(QuerySingleError::NoEntities(_)) => {
                    warn!("No global state entity exists.");
                    None
                }
                Err(QuerySingleError::MultipleEntities(_)) => {
                    warn!("Multiple global state entities exist.");
                    None
                }
                Ok(entity) => Some(entity),
            }
        }
    }
}

impl<S: IntoStateUpdate> Command<Result> for WakeStateTargetCommand<S> {
    fn apply(self, world: &mut World) -> Result {
        let Some(entity) = state_target_entity(world, self.local) else {
            return Ok(());
        };
        let mut entity = world.entity_mut(entity);
        let Some(mut state) = entity.get_mut::<StateData<S>>() else {
            warn!(
                "Set state command failed, entity does not have state {}",
                disqualified::ShortName::of::<S>()
            );
            return Ok(());
        };
        state.update = self.update;
        Ok(())
    }
}

/// Trait for converting
/// States which can be converted to their [`State::Update`].
#[doc(hidden)]
pub trait IntoStateUpdate: State {
    fn into_state_update(self) -> Self::Update;
}

impl<S> IntoStateUpdate for S
where
    S: State,
    S::Update: From<S>,
{
    fn into_state_update(self) -> Self::Update {
        self.into()
    }
}

/// Core methods for interacting with states:
/// - registering state machinery in the world,
/// - initializing states,
/// - updating them.
///
/// Those methods require providing all relevant data.
/// Additional methods can be derived from them by using default values.
///
/// Depending on which medium this is called on, those methods will have:
/// - immediate effect: [`World`], [`SubApp`](bevy_app::SubApp) and [`App`](bevy_app::App),
/// - deferred effect: [`Commands`].
#[doc(hidden)]
pub trait CoreStatesExt {
    fn register_state<S: State>(&mut self, config: StateConfig) -> &mut Self;

    fn init_state<R: StateRepr>(&mut self, local: Option<Entity>, initial: R) -> &mut Self;

    fn update_state<S: IntoStateUpdate>(&mut self, local: Option<Entity>, update: S) -> &mut Self;
}

impl CoreStatesExt for Commands<'_, '_> {
    fn register_state<S: State>(&mut self, config: StateConfig) -> &mut Self {
        self.queue(|world: &mut World| {
            S::register_state(world, config);
        });
        self
    }

    fn init_state<R: StateRepr>(&mut self, local: Option<Entity>, initial: R) -> &mut Self {
        self.queue(InitializeStateCommand::<R::State>::new(local, initial));
        self
    }

    fn update_state<S: IntoStateUpdate>(&mut self, local: Option<Entity>, update: S) -> &mut Self {
        self.queue(WakeStateTargetCommand::<S>::new(local, update));
        self
    }
}

impl CoreStatesExt for World {
    fn register_state<S: State>(&mut self, config: StateConfig) -> &mut Self {
        S::register_state(self, config);
        self
    }

    fn init_state<R: StateRepr>(&mut self, local: Option<Entity>, initial: R) -> &mut Self {
        InitializeStateCommand::<R::State>::new(local, initial)
            .apply(self)
            .unwrap();
        self
    }

    fn update_state<S: IntoStateUpdate>(&mut self, local: Option<Entity>, update: S) -> &mut Self {
        WakeStateTargetCommand::<S>::new(local, update)
            .apply(self)
            .unwrap();
        self
    }
}

#[cfg(feature = "bevy_app")]
impl CoreStatesExt for bevy_app::SubApp {
    fn register_state<S: State>(&mut self, config: StateConfig) -> &mut Self {
        self.world_mut().register_state::<S>(config);
        self
    }

    fn init_state<R: StateRepr>(&mut self, local: Option<Entity>, initial: R) -> &mut Self {
        self.world_mut().init_state(local, initial);
        self
    }

    fn update_state<S: IntoStateUpdate>(&mut self, local: Option<Entity>, update: S) -> &mut Self {
        self.world_mut().update_state::<S>(local, update);
        self
    }
}

#[cfg(feature = "bevy_app")]
impl CoreStatesExt for bevy_app::App {
    fn register_state<S: State>(&mut self, config: StateConfig) -> &mut Self {
        self.main_mut().register_state::<S>(config);
        self
    }

    fn init_state<R: StateRepr>(&mut self, local: Option<Entity>, initial: R) -> &mut Self {
        self.main_mut().init_state(local, initial);
        self
    }

    fn update_state<S: IntoStateUpdate>(&mut self, local: Option<Entity>, update: S) -> &mut Self {
        self.main_mut().update_state::<S>(local, update);
        self
    }
}
