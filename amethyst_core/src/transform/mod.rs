//! `amethyst` transform ecs module

pub use self::{bundle::TransformBundle, components::*};

pub mod bundle;
pub mod components;

/// Re-exports all transform-related systems.
pub mod systems {
    pub use super::{
        missing_previous_parent_system::*, parent_update_system::*, transform_system::*,
    };
}

mod missing_previous_parent_system;
mod parent_update_system;
mod transform_system;
