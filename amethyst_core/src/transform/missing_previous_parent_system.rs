//! System that inserts [PreviousParent] components for entities that have [Transform] and [Parent]

#![allow(missing_docs)]

use super::components::*;
use crate::ecs::*;
use log::*;

/// System that inserts `PreviousParent` components for entities that have `Transform` and `Parent`
#[system(for_each)]
#[filter(component::<Transform>() & component::<Parent>() & !component::<PreviousParent>())]
pub fn missing_previous_parent(commands: &mut CommandBuffer, entity: &Entity) {
    trace!("Adding missing PreviousParent to {:?}", entity);
    commands.add_component(*entity, PreviousParent(None));
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn previous_parent_added() {
        let mut world = World::default();
        let mut resources = Resources::default();
        let mut schedule = Schedule::builder()
            .add_system(missing_previous_parent_system())
            .build();

        let e1 = world.push((Transform::default(),));
        let e2 = world.push((Transform::default(), Parent(e1)));
        schedule.execute(&mut world, &mut resources);

        assert_eq!(
            world
                .entry(e1)
                .unwrap()
                .get_component::<PreviousParent>()
                .is_ok(),
            false
        );

        assert_eq!(
            world
                .entry(e2)
                .unwrap()
                .get_component::<PreviousParent>()
                .is_ok(),
            true
        );
    }
}
