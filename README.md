# About

This repository is a work in progress of state V3 proposal for [Bevy Engine](https://github.com/bevyengine/bevy).

Features:
- Unified naming.  
  Currently names fluctuate between singular (state) and plural (states).
- Single, unified `State` trait.  
  The current crate uses 3 different traits: `States`, `FreelyMutableState`, `SubState`.
- Unified state data struct.  
  No longer separating current state from buffered next state.  
  This allows for buffering more useful information too.
- Strongly typed optional and non-optional states.  
  Currently, states are always optional.
- Flexible state update backends.  
  While built-ins replace the current crate's features,  
  new backends can be created for custom mechanics like:  
  retained substate, state stack, ring-shifting state, etc.
- Cleaner edge cases.  
  Mainly, separation between state updates and transitions,  
  which makes initial transitions trivial.
- State hierarchy (DAG).  
  Like current crate, update order from root states to leaf states.
- State transitions through observers.  
  As opposed to existing crate which uses schedules.  
  Update order is still the same; exit from leaf to root, then enter from root to leaf.  
  This fits well with states as entities, allowing observation of global and local events.  
  The drawback is the minor additional boilerplate to filter event data.
- State scoped entities.  
  Same behavior as current crate, tweaked configuration.
- Global (one per world) and local (one per entity) state machines.  
  Current crate supports only global states.
- Extensible state configuration (transitions, state scoped, etc.),  
  Current crate is missing some configuration (opt-out transitions),  
  some is also fragmented away from state registration (state scoped).
- Command based state updates for basic state types.  
  Similar to the existing one.
- Derive macro for simple root and sub states.  
  Much like current macro, but uses the state optionality.
- Examples to cover both old and new features.
- Feature gated serialization support.  
  Current crate does not provide it.
- Feature gated reflection.  
  Much like current crate.

Out of scope:
- Removing state machinery.  
  Requires systems as entities.
- Dynamic dependencies and update function.  
  Probably through an additional generic component, to overwrite original behavior.  
  Needs design.
- Single state machine split between many entities.  
  Needs use cases and design.

# Justification

Some reasons why this is the direction we should take, in reference to the current crate:

- Debt, removing it gradually will cause a lot of breaking changes,
- Bad ergonomics with non-optional states,
- Cleanup of edge cases, like initial transitions,
- Resource migration to entities,
- Transition migration to observers,
- Inflexible next state, which prevents mechanics like retained substate or stack-based state.

# Major implementation changes

State traits merged into `State` which contains additional:
- `type Update` - data structure for updating this state, basic impls include `()`, `Option<S>` and `Option<Option<S>>`.
- `type Repr` - optionality of this state, impl for `S` and `Option<S>`.

New `StateData<S>` component replaces `State<S>` and `NextState<S>` resources.
Being a component is required for local state machines.
Additional data is stored:
- buffered "is_updated" flag,
- buffered last state and reentry flag.

Component `GlobalMarker` for entity that stores global state.
This change may not belong here, but is required in some form.
Technically this belongs to `bevy_ecs` for when resources are stored as entities?

Transition schedules `OnEnter`, `OnExit` have been replaced with analogous observable events.
This means filtering whether the correct state was entered requires a check in the observer.

# Migration

TODO

# Questions

1. Reducing boilerplate in transition observers.
  - Filtering un-/targeted.
  - Filtering current state.

2. Initial transitions.
  - State added during startup vs state added at runtime.
  - When to emit it?
  - Implementation details.

3. Filtering global state.
  - Move the component to `bevy_ecs` or keep it here for now?
