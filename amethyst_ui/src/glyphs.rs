use crate::{
    FontAsset,
    pass::UiArgs,
};
use amethyst_assets::AssetStorage;
use amethyst_core::ecs::prelude::*;
use amethyst_rendy::{
    Texture,
    rendy::{
        command::QueueId,
        factory::Factory,
    },
    types::Backend,
};
use glyph_brush::{BuiltInLineBreaker, FontId, GlyphBrush, GlyphBrushBuilder, LineBreak, LineBreaker};
use std::collections::HashMap;

const INITIAL_CACHE_SIZE: (u32, u32) = (512, 512);

#[derive(Clone, Debug)]
pub struct UiGlyphs {
    pub(crate) sel_vertices: Vec<UiArgs>,
    pub(crate) vertices: Vec<UiArgs>,
}

#[derive(Copy, Clone, Debug)]
enum FontState {
    Ready(FontId),
    NotFound,
}

impl FontState {
    fn font_id(&self) -> Option<FontId> {
        match self {
            Self::Ready(font_id) => Some(*font_id),
            Self::NotFound => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash)]
enum CustomLineBreaker {
    BuiltIn(BuiltInLineBreaker),
    None,
}

impl LineBreaker for CustomLineBreaker {
    fn line_breaks<'a>(&self, glyph_info: &'a str) -> Box<dyn Iterator<Item = LineBreak> + 'a> {
        match self {
            Self::BuiltIn(inner) => inner.line_breaks(glyph_info),
            Self::None => Box::new(None.into_iter()),
        }
    }
}

pub fn build_ui_glyphs_system<B>(world: &mut World, resources: &mut Resources) -> Box<dyn Schedulable>
where B: Backend
{
    let glyph_brush: GlyphBrush<'static, (u32, UiArgs)> = GlyphBrushBuilder::without_fonts()
        .initial_cache_size(INITIAL_CACHE_SIZE)
        .build();

    SystemBuilder::<()>::new("UiGlyphsSystem")
        .write_resource::<Factory<B>>()
        .read_resource::<QueueId>()
        .write_resource::<AssetStorage<Texture>>()
        .read_resource::<AssetStorage<FontAsset>>()
        .build(move |_, world, resources, _| {
            let (factory, queue, texture_storage, font_storage) = resources;
        })
}