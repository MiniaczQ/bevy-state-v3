//! Sets of states for specifying dependencies.

use std::u32;

use bevy_ecs::{
    component::{ComponentId, Components, RequiredComponents},
    query::{QueryData, WorldQuery},
    storage::Storages,
    world::World,
};
use bevy_utils::all_tuples;

use crate::{components::StateData, state::State, transitions::StateConfig};

/// Shorthand for arguments provided to state `update` function.
pub type StateDependencies<'a, S> =
    <<<S as State>::Dependencies as StateSet>::Data as WorldQuery>::Item<'a>;

/// Set of states which can be used as dependencies.
pub trait StateSet {
    /// Data of dependency states.
    type Data: QueryData + 'static;

    /// Highest update order in the set.
    /// This is 0 for empty sets.
    const HIGHEST_ORDER: u32;

    /// Registers all states in the set as required components.
    /// Missing dependency state components will result in a panic.
    fn register_required_components(
        component_id: ComponentId,
        components: &mut Components,
        storages: &mut Storages,
        required_components: &mut RequiredComponents,
        inheritance_depth: u16,
    );

    /// Registers all states in the set in the world.
    /// Default state configuration is used.
    fn register_required_states(world: &mut World);

    /// Returns whether any of the dependencies changed.
    fn is_changed(set: &<Self::Data as WorldQuery>::Item<'_>) -> bool;
}

fn missing_state<S: State>() -> StateData<S> {
    let name = disqualified::ShortName::of::<S>();
    panic!("Missing required dependency state {name}");
}

impl<S1: State> StateSet for S1 {
    type Data = &'static StateData<S1>;

    const HIGHEST_ORDER: u32 = S1::ORDER;

    fn register_required_components(
        _component_id: ComponentId,
        components: &mut Components,
        storages: &mut Storages,
        required_components: &mut RequiredComponents,
        inheritance_depth: u16,
    ) {
        required_components.register(components, storages, missing_state::<S1>, inheritance_depth);
    }

    fn register_required_states(world: &mut World) {
        S1::register_state(world, StateConfig::default(), true);
    }

    fn is_changed(s1: &<Self::Data as WorldQuery>::Item<'_>) -> bool {
        s1.is_updated
    }
}

const fn const_max(a: u32, b: u32) -> u32 {
    if a > b {
        a
    } else {
        b
    }
}

macro_rules! max {
    ($a:expr) => ( $a );
    ($a:expr, $b:expr) => {
        const_max($a, $b)
    };
    ($a:expr, $b:expr, $($other:expr), *) => {
        max!(const_max($a, $b), $($other), +)
    };
}

macro_rules! impl_state_set {
    ($(#[$meta:meta])* $(($type:ident, $var:ident)), *) => {
        $(#[$meta])*
        impl<$($type: State), *> StateSet for ($($type, )*) {
            type Data = ($(&'static StateData<$type>, )*);

            const HIGHEST_ORDER: u32 = max!($($type::ORDER,)* 0);

            fn register_required_components(
                _component_id: ComponentId,
                _components: &mut Components,
                _storages: &mut Storages,
                _required_components: &mut RequiredComponents,
                _inheritance_depth: u16,
            ) {
                $(_required_components.register(_components, _storages, missing_state::<$type>, _inheritance_depth);)*
            }

            fn register_required_states(_world: &mut World) {
                $($type::register_state(_world, StateConfig::default(), true);)*
            }

            fn is_changed(($($var,)*): &<Self::Data as WorldQuery>::Item<'_>) -> bool {
                $($var.is_updated ||)* false
            }
        }
    };
}

all_tuples!(
    #[doc(fake_variadic)]
    impl_state_set,
    0,
    15,
    S,
    s
);
