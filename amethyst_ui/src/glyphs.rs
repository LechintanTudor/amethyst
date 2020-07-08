use crate::{
    FontAsset, UiTransform,
    pass::UiArgs,
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::ecs::prelude::*;
use amethyst_rendy::{
    Texture,
    rendy::{
        command::QueueId,
        factory::{Factory, ImageState},
        hal,
        texture::{
            TextureBuilder,
            pixel::R8Unorm,
        },
    },
    resources::Tint,
    types::Backend,
};
use glyph_brush::{BuiltInLineBreaker, FontId, GlyphBrush, GlyphBrushBuilder, LineBreak, LineBreaker};
use std::collections::HashMap;

const INITIAL_CACHE_SIZE: (u32, u32) = (512, 512);

#[derive(Default, Debug)]
pub struct UiGlyphsResource {
    glyph_texture: Option<Handle<Texture>>,
}

impl UiGlyphsResource {
    pub fn glyph_texture(&self) -> Option<&Handle<Texture>> {
        self.glyph_texture.as_ref()
    }
}

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
        .write_resource::<UiGlyphsResource>()
        .build(move |_, world, resources, _| {
            let (factory, queue, texture_storage, font_storage, glyphs_res) = resources;

            let glyph_texture = glyphs_res.glyph_texture.get_or_insert_with(|| {
                let (width, height) = glyph_brush.texture_dimensions();
                texture_storage.insert(create_glyph_texture(factory, **queue, width, height))
            });

            let glyph_texture = texture_storage
                .get(glyph_texture)
                .and_then(B::unwrap_texture)
                .expect("Glyph texture is created synchronously");
        })
}

fn create_glyph_texture<B>(factory: &mut Factory<B>, queue: QueueId, width: u32, height: u32) -> Texture
where B: Backend
{
    use hal::format::{Component as C, Swizzle};

    log::trace!("Creating new glyph texture with size ({}, {})",
        width, height);

    TextureBuilder::new()
        .with_kind(hal::image::Kind::D2(width, height, 1, 1))
        .with_view_kind(hal::image::ViewKind::D2)
        .with_data_width(width)
        .with_data_height(height)
        .with_data(vec![R8Unorm { repr: [0] }; (width * height) as _])
        // This swizzle is required when working with `R8Unorm` on metal.
        // Glyph texture is biased towards 1.0 using "color_bias" attribute instead.
        .with_swizzle(Swizzle(C::Zero, C::Zero, C::Zero, C::R))
        .build(
            ImageState {
                queue,
                stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                access: hal::image::Access::SHADER_READ,
                layout: hal::image::Layout::General,
            },
            factory,
        )
        .map(B::wrap_texture)
        .expect("Failed to create glyph texture")
}
