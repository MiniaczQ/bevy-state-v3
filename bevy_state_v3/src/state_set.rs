//! Sets of states.
//! This feature is only used for specifying dependencies.

use bevy_ecs::{
    component::{ComponentId, ComponentsRegistrator, RequiredComponents},
    query::QueryData,
};
use variadics_please::all_tuples;

use crate::{components::StateData, state::State};

/// Shorthand for converting state set into a set of state data.
pub type StateSetData<'a, S> = <<S as StateSet>::Query as QueryData>::Item<'a>;

/// Set of states which can be used as dependencies.
pub trait StateSet {
    /// Query for state data of all states in this set.
    type Query: QueryData + 'static;

    /// Highest update order in the set.
    /// This is 0 for empty sets.
    const HIGHEST_ORDER: u32;

    /// Registers all states in the set as required components.
    /// Missing dependency state data components will result in a panic.
    fn register_required_components(
        component_id: ComponentId,
        components: &mut ComponentsRegistrator,
        required_components: &mut RequiredComponents,
        inheritance_depth: u16,
    );

    /// Returns whether any of the dependencies updated in last update schedule.
    fn is_updated(set: &<Self::Query as QueryData>::Item<'_>) -> bool;
}

/// Helper function for panicking if parent state data component is missing.
fn panic_missing_state<S: State>() -> StateData<S> {
    let name = disqualified::ShortName::of::<S>();
    panic!("Missing required dependency state {name}");
}

impl<S1: State> StateSet for S1 {
    type Query = &'static StateData<S1>;

    const HIGHEST_ORDER: u32 = S1::ORDER;

    fn register_required_components(
        _component_id: ComponentId,
        components: &mut ComponentsRegistrator,
        required_components: &mut RequiredComponents,
        inheritance_depth: u16,
    ) {
        required_components.register(components, panic_missing_state::<S1>, inheritance_depth);
    }

    fn is_updated(s1: &<Self::Query as QueryData>::Item<'_>) -> bool {
        s1.is_updated
    }
}

/// Helper function for compile time max.
const fn const_max(a: u32, b: u32) -> u32 {
    if a > b { a } else { b }
}

/// Helper macro for variable argument max function.
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
            type Query = ($(&'static StateData<$type>, )*);

            const HIGHEST_ORDER: u32 = max!($($type::ORDER,)* 0);

            fn register_required_components(
                _component_id: ComponentId,
                _components: &mut ComponentsRegistrator,
                _required_components: &mut RequiredComponents,
                _inheritance_depth: u16,
            ) {
                $(_required_components.register(_components, panic_missing_state::<$type>, _inheritance_depth);)*
            }

            fn is_updated(($($var,)*): &<Self::Query as QueryData>::Item<'_>) -> bool {
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
