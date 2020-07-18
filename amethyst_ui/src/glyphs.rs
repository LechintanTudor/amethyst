use crate::{
    FontAsset, LineMode, TextEditing, UiText, UiTransform,
    pass::UiArgs,
    text::CachedGlyph,
    utils,
};
use amethyst_assets::{AssetStorage, Handle};
use amethyst_core::{
    Hidden, HiddenPropagate,
    ecs::prelude::*,
};
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
use glyph_brush::{
    BrushAction, BrushError, BuiltInLineBreaker, FontId, GlyphBrush, GlyphBrushBuilder, GlyphCruncher,
    Layout, LineBreak, LineBreaker, SectionText, VariedSection,
    rusttype::Scale,
};
use std::{
    collections::HashMap,
    cmp,
    mem,
};

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

#[derive(Clone, Default, Debug)]
pub struct UiGlyphs {
    pub(crate) vertices: Vec<UiArgs>,
    pub(crate) selection_vertices: Vec<UiArgs>,
    pub(crate) cursor_position: (f32, f32),
    pub(crate) height: f32,
    pub(crate) space_width: f32,
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
    let mut glyph_brush: GlyphBrush<'static, (u32, UiArgs)> = GlyphBrushBuilder::without_fonts()
        .initial_cache_size(INITIAL_CACHE_SIZE)
        .build();

    let mut font_map = HashMap::<u32, FontState>::new();

    SystemBuilder::<()>::new("UiGlyphsSystem")
        .write_resource::<Factory<B>>()
        .read_resource::<QueueId>()
        .write_resource::<AssetStorage<Texture>>()
        .read_resource::<AssetStorage<FontAsset>>()
        .write_resource::<UiGlyphsResource>()
        .with_query(
            <(Read<UiTransform>, Write<UiText>)>::query()
                .filter(!component::<Hidden>() & !component::<HiddenPropagate>())
        )
        .with_query(<(Write<UiGlyphs>,)>::query())
        .with_query(
            <(Read<UiTransform>, Write<UiText>, TryWrite<TextEditing>, TryWrite<UiGlyphs>)>::query()
                .filter(!component::<Hidden>() & !component::<HiddenPropagate>())
        )
        .write_component::<UiGlyphs>()
        .build(move |commands, world, resources, queries| {
            let (factory, queue, texture_storage, font_storage, glyphs_res) = resources;
            let (text_query, glyph_query, glyph_query2) = queries;

            let glyph_texture_handle = glyphs_res.glyph_texture.get_or_insert_with(|| {
                let (width, height) = glyph_brush.texture_dimensions();
                texture_storage.insert(create_glyph_texture(factory, **queue, width, height))
            });

            let mut glyph_texture = texture_storage
                .get(glyph_texture_handle)
                .and_then(B::unwrap_texture)
                .expect("Glyph texture is created synchronously");

            for (entity, (transform, mut ui_text)) in text_query.iter_entities_mut(world) {
                let mut cached_glyphs = Vec::new();
                mem::swap(&mut ui_text.cached_glyphs, &mut cached_glyphs);

                cached_glyphs.clear();

                let font_asset = font_storage.get(&ui_text.font);

                let font_lookup = font_map
                    .entry(ui_text.font.id())
                    .or_insert(FontState::NotFound);

                if font_lookup.font_id().is_none() {
                    if let Some(font) = font_storage.get(&ui_text.font) {
                        *font_lookup = FontState::Ready(glyph_brush.add_font(font.0.clone()));
                    }
                }

                if let (Some(font_id), Some(font_asset)) = (font_lookup.font_id(), font_asset) {
                    let scale = Scale::uniform(ui_text.font_size);
                    let text = vec![
                        SectionText {
                            text: &ui_text.text,
                            scale,
                            color: utils::srgba_to_lin_rgba_array(ui_text.color),
                            font_id,
                        },
                    ];
                    let layout = match ui_text.line_mode {
                        LineMode::Single => Layout::SingleLine {
                            line_breaker: CustomLineBreaker::None,
                            h_align: ui_text.align.horizontal_align(),
                            v_align: ui_text.align.vertical_align(),
                        },
                        LineMode::Wrap => Layout::Wrap {
                            line_breaker: CustomLineBreaker::BuiltIn(
                                BuiltInLineBreaker::UnicodeLineBreaker,
                            ),
                            h_align: ui_text.align.horizontal_align(),
                            v_align: ui_text.align.vertical_align(),
                        },
                    };

                    let section = VariedSection {
                        screen_position: (
                            transform.pixel_x + transform.pixel_width
                                * ui_text.align.normalized_offset().0,
                            -(transform.pixel_y + transform.pixel_height
                                * ui_text.align.normalized_offset().1),
                        ),
                        bounds: (transform.pixel_width, transform.pixel_height),
                        z: f32::from_bits(entity.index()),
                        layout: Layout::default(),
                        text,
                    };

                    let mut nonempty_cached_glyphs = glyph_brush
                        .glyphs_custom_layout(&section, &layout)
                        .map(|g| {
                            CachedGlyph {
                                x: g.position().x,
                                y: -g.position().y,
                                advance_width: g.unpositioned().h_metrics().advance_width,
                            }
                        });

                    let mut last_cached_glyph = Option::<CachedGlyph>::None;
                    let all_glyphs = ui_text.text.chars().filter_map(move |c| {
                        if c.is_whitespace() {
                            let (x, y) = if let Some(last_cached_glyph) = last_cached_glyph {
                                (
                                    last_cached_glyph.x + last_cached_glyph.advance_width,
                                    last_cached_glyph.y,
                                )
                            } else {
                                (0.0, 0.0)
                            };

                            last_cached_glyph = Some(CachedGlyph {
                                x,
                                y,
                                advance_width: font_asset.0.glyph(c).scaled(scale).h_metrics().advance_width,
                            });
                            last_cached_glyph
                        } else {
                            last_cached_glyph = nonempty_cached_glyphs.next();
                            last_cached_glyph
                        }
                    });

                    cached_glyphs.extend(all_glyphs);
                    glyph_brush.queue_custom_layout(section, &layout);
                    mem::swap(&mut ui_text.cached_glyphs, &mut cached_glyphs);
                }
            }

            loop {
                let action = glyph_brush.process_queued(
                    |rect, data| unsafe {
                        factory
                            .upload_image(
                                glyph_texture.image().clone(),
                                rect.width(),
                                rect.height(),
                                hal::image::SubresourceLayers {
                                    aspects: hal::format::Aspects::COLOR,
                                    level: 0,
                                    layers: 0..1,
                                },
                                hal::image::Offset {
                                    x: rect.min.x as _,
                                    y: rect.min.y as _,
                                    z: 0,
                                },
                                hal::image::Extent {
                                    width: rect.width(),
                                    height: rect.height(),
                                    depth: 1,
                                },
                                data,
                                ImageState {
                                    queue: **queue,
                                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                                    access: hal::image::Access::SHADER_READ,
                                    layout: hal::image::Layout::General,
                                },
                                ImageState {
                                    queue: **queue,
                                    stage: hal::pso::PipelineStage::FRAGMENT_SHADER,
                                    access: hal::image::Access::SHADER_READ,
                                    layout: hal::image::Layout::General,
                                },
                            )
                            .unwrap();
                    },
                    move |glyph| {
                        let entity: u32 = glyph.z.to_bits();

                        let bounds_min_x = glyph.bounds.min.x as f32;
                        let bounds_min_y = glyph.bounds.min.y as f32;
                        let bounds_max_x = glyph.bounds.max.x as f32;
                        let bounds_max_y = glyph.bounds.max.y as f32;

                        let mut uv = glyph.tex_coords;
                        let mut coords_min_x = glyph.pixel_coords.min.x as f32;
                        let mut coords_min_y = glyph.pixel_coords.min.y as f32;
                        let mut coords_max_x = glyph.pixel_coords.max.x as f32;
                        let mut coords_max_y = glyph.pixel_coords.max.y as f32;

                        if coords_max_x > bounds_max_x {
                            let old_width = coords_max_x - coords_min_x;
                            coords_max_x = bounds_max_x;
                            uv.max.x = uv.min.x
                                + (uv.max.x - uv.min.x) * (coords_max_x - coords_min_x) / old_width;
                        }
                        if coords_min_x < bounds_min_x {
                            let old_width = coords_max_x - coords_min_x;
                            coords_min_x = bounds_min_x;
                            uv.min.x = uv.max.x
                                - (uv.max.x - uv.min.x) * (coords_max_x - coords_min_x) / old_width;
                        }
                        if coords_max_y > bounds_max_y {
                            let old_height = coords_max_y - coords_min_y;
                            coords_max_y = bounds_max_y;
                            uv.max.y = uv.min.y
                                + (uv.max.y - uv.min.y) * (coords_max_y - coords_min_y) / old_height;
                        }
                        if coords_min_y < bounds_min_y {
                            let old_height = coords_max_y - coords_min_y;
                            coords_min_y = bounds_min_y;
                            uv.min.y = uv.max.y
                                - (uv.max.y - uv.min.y) * (coords_max_y - coords_min_y) / old_height;
                        }

                        let position = [
                            (coords_max_x + coords_min_x) * 0.5,
                            -(coords_max_y + coords_min_y) * 0.5,
                        ];
                        let dimensions = [(coords_max_x - coords_min_x), (coords_max_y - coords_min_y)];
                        let tex_coords_bounds = [uv.min.x, uv.min.y, uv.max.x, uv.max.y];

                        (
                            entity,
                            UiArgs {
                                position: position.into(),
                                dimensions: dimensions.into(),
                                color: glyph.color.into(),
                                tex_coords_bounds: tex_coords_bounds.into(),
                            },
                        )
                    },
                );

                match action {
                    Ok(BrushAction::Draw(vertices)) => {
                        let mut glyph_ctr = 0;

                        for (mut glyph_data,) in glyph_query.iter_mut(world) {
                            glyph_data.selection_vertices.clear();
                            glyph_data.vertices.clear();
                        }

                        for (entity, (transform, ui_text, text_editing, mut glyphs)) in glyph_query2.iter_entities_mut(world) {
                            let entity_id = entity.index();

                            let len = vertices[glyph_ctr..]
                                .iter()
                                .take_while(|(id, _)| *id == entity_id)
                                .count();

                            let entity_vertices = vertices[glyph_ctr..glyph_ctr + len]
                                .iter()
                                .map(|v| v.1);
                            glyph_ctr += len;

                            if let Some(mut glyph_data) = glyphs.as_mut() {
                                glyph_data.vertices.extend(entity_vertices);
                            } else {
                                commands.add_component(entity, UiGlyphs {
                                    vertices: entity_vertices.collect(),
                                    ..UiGlyphs::default()
                                });
                            }

                            if let Some(text_editing) = text_editing {
                                let font = font_storage
                                    .get(&ui_text.font)
                                    .expect("Font with rendered glyphs must be loaded");
                                let scale = Scale::uniform(ui_text.font_size);
                                let v_metrics = font.0.v_metrics(scale);
                                let height = v_metrics.ascent - v_metrics.descent;
                                let offset = (v_metrics.ascent + v_metrics.descent) / 2.0;
                                let highlight = text_editing.cursor_position + text_editing.highlight_vector;
                                let glyph_count = ui_text.cached_glyphs.len();
                                let start = cmp::min(highlight as usize, glyph_count);
                                let end = cmp::max(highlight as usize, glyph_count);

                                let selection_ui_args_iter = ui_text.cached_glyphs[start..end]
                                    .iter()
                                    .map(|g| UiArgs {
                                        position: [g.x + g.advance_width / 2.0, g.y + offset].into(),
                                        dimensions: [g.advance_width, height].into(),
                                        tex_coords_bounds: [0.0, 0.0, 1.0, 1.0].into(),
                                        color: [1.0, 1.0, 1.0, 1.0].into(), // TODO: Tint
                                    });

                                if let Some(mut glyph_data) = glyphs {
                                    glyph_data.selection_vertices.extend(selection_ui_args_iter);
                                    glyph_data.height = height;
                                    glyph_data.space_width =
                                        font.0.glyph(' ').scaled(scale).h_metrics().advance_width;

                                    update_cursor_position(
                                        &mut glyph_data,
                                        &ui_text,
                                        &transform,
                                        text_editing.cursor_position as usize,
                                        offset,
                                    );
                                }
                            }
                        }

                        break;
                    },
                    Ok(BrushAction::ReDraw) => {
                        for (entity, (transform, ui_text, text_editing, glyphs)) in glyph_query2.iter_entities_mut(world) {
                            let font = font_storage
                                .get(&ui_text.font)
                                .expect("Font with rendered glyphs must be loaded");
                            let scale = Scale::uniform(ui_text.font_size);
                            let v_metrics = font.0.v_metrics(scale);
                            let offset = (v_metrics.ascent + v_metrics.descent) / 2.0;

                            if let (Some(text_editing), Some(mut glyph_data)) = (text_editing, glyphs) {
                                update_cursor_position(
                                    &mut glyph_data,
                                    &ui_text,
                                    &transform,
                                    text_editing.cursor_position as usize,
                                    offset,
                                );
                            }
                        }
                        break;
                    }
                    Err(BrushError::TextureTooSmall { suggested: (width, height) }) => {
                        texture_storage.replace(
                            glyph_texture_handle,
                            create_glyph_texture(
                                factory,
                                **queue,
                                width,
                                height,
                            ),
                        );

                        glyph_texture = texture_storage
                            .get(glyph_texture_handle)
                            .and_then(B::unwrap_texture)
                            .unwrap();

                        glyph_brush.resize_texture(width, height);
                    }
                }
            }
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
        .with_swizzle(Swizzle(C::One, C::One, C::One, C::R))
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

fn update_cursor_position(
    glyph_data: &mut UiGlyphs,
    ui_text: &UiText,
    transform: &UiTransform,
    cursor_position: usize,
    offset: f32,
)
{
    glyph_data.cursor_position =
        if let Some(glyph) = ui_text.cached_glyphs.get(cursor_position) {
            (glyph.x, glyph.y + offset)
        } else if let Some(glyph) = ui_text.cached_glyphs.last() {
            (glyph.x + glyph.advance_width, glyph.y + offset)
        } else {
            (
                transform.pixel_x + transform.pixel_width * ui_text.align.normalized_offset().0,
                transform.pixel_y + transform.pixel_height * ui_text.align.normalized_offset().1,
            )
        }
}