//! System that generates [Children] components for entities that are targeted by [Parent] component.

#![allow(missing_docs)]

use super::components::*;
use crate::ecs::*;
use log::*;
use smallvec::SmallVec;
use std::collections::HashMap;

#[system]
#[write_component(Children)]
pub fn parent_update(commands: &mut CommandBuffer, world: &mut SubWorld<'_>) {
    // Entities with a removed `Parent`
    let mut removed_parent_query =
        <(Entity, &PreviousParent)>::query().filter(!component::<Parent>());

    // Entities with a changed `Parent`
    let mut changed_parent_query = <(Entity, &Parent, &mut PreviousParent)>::query()
        .filter(component::<Transform>() & maybe_changed::<Parent>());

    // Deleted Parents (i.e. Entities with `Children` and without a `Transform`)
    let mut deleted_parent_query = <(Entity, &Children)>::query().filter(!component::<Transform>());

    // Iterate the entities with a missing `Parent` (i.e. the ones that have a `PreviousParent`)
    // and remove them from the `Children` of the `PreviousParent`.
    let (mut left, mut right) = world.split::<&Children>();

    for (entity, previous_parent) in removed_parent_query.iter(&right) {
        trace!("Parent was removed from {:?}.", entity);

        if let Some(previous_parent_entity) = previous_parent.0 {
            if let Some(previous_parent_children) = left
                .entry_mut(previous_parent_entity)
                .ok()
                .and_then(|entry| entry.into_component_mut::<Children>().ok())
            {
                trace!(
                    " > Removing {:?} from its previous parent's children.",
                    entity
                );
                previous_parent_children.0.retain(|e| e != entity);
            }
        }
    }

    // Tracks all newly created `Children` components this frame.
    let mut children_additions = HashMap::<Entity, SmallVec<[Entity; 8]>>::with_capacity(16);

    // Entities with a changed `Parent` (that also have a `PreviousParent`, even if `None`)
    for (entity, parent, previous_parent) in changed_parent_query.iter_mut(&mut right) {
        trace!("Parent changed for {:?}.", entity);

        // If the `PreviousParent` is not None
        if let Some(previous_parent_entity) = previous_parent.0 {
            // New and previous point to the same Entity, carry on, nothing to see here.
            if previous_parent_entity == parent.0 {
                trace!(" > But the previous parent is the same, ignoring...");
                continue;
            }

            // Remove from `PreviousParent`'s `Children`.
            if let Some(previous_parent_children) = left
                .entry_mut(previous_parent_entity)
                .ok()
                .and_then(|entry| entry.into_component_mut::<Children>().ok())
            {
                trace!(
                    " > Removing {:?} from its previous parent's children.",
                    entity
                );
                previous_parent_children.0.retain(|e| e != entity);
            }
        }

        // Set `PreviousParent = Parent`.
        *previous_parent = PreviousParent(Some(parent.0));

        // Add to the parent's `Children` (either the real component, or
        // `children_additions`).
        trace!("Adding {:?} to it's new parent {:?}.", entity, parent.0);

        if let Some(new_parent_children) = left
            .entry_mut(parent.0)
            .ok()
            .and_then(|entry| entry.into_component_mut::<Children>().ok())
        {
            // This is the parent.
            trace!(
                " > The new parent {:?} already has a `Children`, adding to it...",
                parent.0
            );
            new_parent_children.0.push(*entity);
        } else {
            // The parent doesn't have a `Children` component, let's add it.
            trace!(
                "The new parent {:?} doesn't have a `Children` component yet.",
                parent.0
            );
            children_additions
                .entry(parent.0)
                .or_insert_with(Default::default)
                .push(*entity);
        }
    }

    // Iterate the entities with a deleted `Parent` (i.e. entities with a `Children` but no `Transform`)
    // and remove their `Parent`, `PreviousParent` and `Children` components
    for (entity, children) in deleted_parent_query.iter(world) {
        trace!("The entity {:?} doesn't have a `Transform`.", entity);

        if children_additions.remove(&entity).is_none() {
            trace!(" > It needs to be removed from `World`.");

            for &child_entity in children.0.iter() {
                commands.remove_component::<Parent>(child_entity);
                commands.remove_component::<PreviousParent>(child_entity);
            }

            commands.remove_component::<Children>(*entity);
        } else {
            trace!(" > It was a new addition, removing it from additions map...");
        }
    }

    // Flush the `children_additions` to the command buffer. It is stored separate to
    // collect multiple new children that point to the same parent into the same
    // SmallVec, and to prevent redundant add+remove operations.
    children_additions.iter().for_each(|(k, v)| {
        trace!(
            "Flushing: Entity {:?} adding `Children` component {:?}.",
            k,
            v
        );
        commands.add_component(*k, Children::with(v));
    });
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::transform::systems::missing_previous_parent_system;

    #[test]
    fn correct_children() {
        let mut world = World::default();
        let mut resources = Resources::default();

        let mut schedule = Schedule::builder()
            .add_system(missing_previous_parent_system())
            .flush()
            .add_system(parent_update_system())
            .build();

        // Add parent entities
        let parent = world.push((Transform::default(),));
        let children = world.extend(vec![(Transform::default(),), (Transform::default(),)]);
        let (e1, e2) = (children[0], children[1]);

        // Parent `e1` and `e2` to `parent`.
        world.entry(e1).unwrap().add_component(Parent(parent));
        world.entry(e2).unwrap().add_component(Parent(parent));

        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            world
                .entry(parent)
                .unwrap()
                .get_component::<Children>()
                .unwrap()
                .0
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![e1, e2]
        );

        // Parent `e1` to `e2`.
        world
            .entry_mut(e1)
            .unwrap()
            .get_component_mut::<Parent>()
            .unwrap()
            .0 = e2;

        // Run the systems
        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            world
                .entry(parent)
                .unwrap()
                .get_component::<Children>()
                .unwrap()
                .0
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![e2]
        );

        assert_eq!(
            world
                .entry(e2)
                .unwrap()
                .get_component::<Children>()
                .unwrap()
                .0
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![e1]
        );

        world.remove(e1);

        // Run the systems
        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            world
                .entry(parent)
                .unwrap()
                .get_component::<Children>()
                .unwrap()
                .0
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            vec![e2]
        );
    }
}
