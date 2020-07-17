use amethyst_core::ecs::prelude::*;

#[derive(Copy, Clone, Default, Debug)]
pub struct Selected {
    pub entity: Option<Entity>,
}
