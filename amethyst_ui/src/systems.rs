use crate::FontAsset;
use amethyst_core::ecs::prelude::*;

pub use crate::event::build_ui_mouse_system;
pub use crate::glyphs::build_ui_glyphs_system;

pub fn build_font_asset_processor_system(
    world: &mut World, resources: &mut Resources) -> Box<dyn Schedulable>
{
    amethyst_assets::build_asset_processor_system::<FontAsset>(world, resources)
}