//! ECS transform bundle

use super::systems::*;
use crate::ecs::*;
use amethyst_error::Error;

/// Transform bundle
#[derive(Default, Debug)]
pub struct TransformBundle;

impl SystemBundle for TransformBundle {
    fn load(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder,
    ) -> Result<(), Error> {
        builder
            .add_system(missing_previous_parent_system())
            .add_system(parent_update_system())
            .add_system(transform_system());

        Ok(())
    }
}
