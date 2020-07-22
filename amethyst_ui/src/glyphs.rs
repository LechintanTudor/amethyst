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
    palette::Srgba,
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
    *,
    ab_glyph::{Font, FontArc, PxScale, ScaleFont},
};
use std::{
    collections::HashMap,
    cmp,
    mem,
    ops::Range,
};
use unicode_segmentation::UnicodeSegmentation;

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

#[derive(Copy, Clone, Debug, PartialEq, Hash)]
struct ExtraTextData {
    // Entity to which the text belongs
    entity: Entity,

    // Text color stored as linear RGBA
    color: [u32; 4],
}

impl ExtraTextData {
    fn new(entity: Entity, color: [f32; 4]) -> Self {
        Self {
            entity,
            color: [
                color[0].to_bits(),
                color[1].to_bits(),
                color[2].to_bits(),
                color[3].to_bits(),
            ],
        }
    }

    fn color(&self) -> [f32; 4] {
        [
            f32::from_bits(self.color[0]),
            f32::from_bits(self.color[1]),
            f32::from_bits(self.color[2]),
            f32::from_bits(self.color[3]),
        ]
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
    let mut glyph_brush: GlyphBrush<(u32, UiArgs), ExtraTextData> =
        GlyphBrushBuilder::using_fonts(Vec::<FontArc>::new())
            .initial_cache_size(INITIAL_CACHE_SIZE)
            .build();

    // Maps asset handle ids to `GlyphBrush` `FontId`s
    let mut font_map = HashMap::<u32, FontId>::new();

    SystemBuilder::<()>::new("UiGlyphsSystem")
        .write_resource::<Factory<B>>()
        .read_resource::<QueueId>()
        .write_resource::<AssetStorage<Texture>>()
        .read_resource::<AssetStorage<FontAsset>>()
        .write_resource::<UiGlyphsResource>()
        .with_query(
            <(
                Read<UiTransform>,
                Write<UiText>,
                TryRead<Tint>,
                TryRead<TextEditing>,
            )>::query()
                .filter(!component::<Hidden>() & !component::<HiddenPropagate>())
        )
        .with_query(<(Write<UiGlyphs>,)>::query())
        .with_query(
            <(
                Read<UiTransform>,
                Write<UiText>,
                TryRead<Tint>,
                TryWrite<TextEditing>,
                TryWrite<UiGlyphs>,
            )>::query()
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

            for (entity, (transform, mut ui_text, tint, text_editing)) in text_query.iter_entities_mut(world) {
                ui_text.cached_glyphs.clear();

                let mut cached_glyphs = Vec::new();
                mem::swap(&mut ui_text.cached_glyphs, &mut cached_glyphs);

                let (font, font_id) = match font_storage.get(&ui_text.font) {
                    Some(font) => {
                        let font_id = *font_map
                            .entry(ui_text.font.id())
                            .or_insert_with(|| {
                                glyph_brush.add_font(font.0.clone())
                            });

                        (font, font_id)
                    }
                    None => continue,
                };

                let tint_color = if let Some(tint) = tint {
                    utils::srgba_to_lin_rgba_array(tint.0)
                } else {
                    [1.0, 1.0, 1.0, 1.0]
                };

                let base_color = utils::mul_blend_lin_rgba_arrays(
                    utils::srgba_to_lin_rgba_array(ui_text.color),
                    tint_color,
                );

                let scale = PxScale::from(ui_text.font_size);
                let scaled_font = font.0.as_scaled(scale);

                let text = match (ui_text.password, text_editing) {
                    (false, None) => vec![
                        Text {
                            text: &ui_text.text,
                            scale,
                            font_id,
                            extra: ExtraTextData::new(entity, base_color),
                        }
                    ],
                    (false, Some(text_editing)) => {
                        let selected_color = utils::mul_blend_lin_rgba_arrays(
                            utils::srgba_to_lin_rgba_array(text_editing.selected_text_color),
                            tint_color,
                        );

                        if let Some(range) = selected_bytes(&text_editing, &ui_text.text) {
                            let start = range.start;
                            let end = range.end;

                            vec![
                                Text {
                                    text: &ui_text.text[..start],
                                    scale,
                                    font_id,
                                    extra: ExtraTextData::new(entity, base_color),
                                },
                                Text {
                                    text: &ui_text.text[start..end],
                                    scale,
                                    font_id,
                                    extra: ExtraTextData::new(entity, base_color),
                                },
                                Text {
                                    text: &ui_text.text[end..],
                                    scale,
                                    font_id,
                                    extra: ExtraTextData::new(entity, base_color),
                                },
                            ]
                        } else {
                            vec![
                                Text {
                                    text: &ui_text.text,
                                    scale,
                                    font_id,
                                    extra: ExtraTextData::new(entity, base_color),
                                },
                            ]
                        }
                    }
                    _ => todo!()
                };

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

                let section = Section {
                    screen_position: (
                        transform.pixel_x + transform.pixel_width
                            * ui_text.align.normalized_offset().0,
                        -(transform.pixel_y + transform.pixel_height
                            * ui_text.align.normalized_offset().1),
                    ),
                    bounds: (transform.pixel_width, transform.pixel_height),
                    layout: Layout::default(),
                    text,
                };

                let mut nonempty_cached_glyphs = glyph_brush
                    .glyphs_custom_layout(&section, &layout)
                    .map(|section_glyph| {
                        CachedGlyph {
                            x: section_glyph.glyph.position.x,
                            y: section_glyph.glyph.position.y,
                            advance_width: scaled_font.h_advance(section_glyph.glyph.id),
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
                            advance_width: scaled_font.h_advance(scaled_font.glyph_id(c)),
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
                                    x: rect.min[0] as _,
                                    y: rect.min[1] as _,
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
                            (coords_max_x + coords_min_x) / 2.0,
                            -(coords_max_y + coords_min_y) / 2.0,
                        ];
                        let dimensions = [(coords_max_x - coords_min_x), (coords_max_y - coords_min_y)];
                        let tex_coords_bounds = [uv.min.x, uv.min.y, uv.max.x, uv.max.y];

                        (
                            glyph.extra.entity.index(),
                            UiArgs {
                                position: position.into(),
                                dimensions: dimensions.into(),
                                color: glyph.extra.color().into(),
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

                        for (entity, (transform, ui_text, tint, text_editing, mut glyphs)) in glyph_query2.iter_entities_mut(world) {
                            let entity_id = entity.index();
                            let scale = PxScale::from(ui_text.font_size);

                            let len = vertices[glyph_ctr..]
                                .iter()
                                .take_while(|(id, _)| *id == entity_id)
                                .count();

                            let entity_vertices = vertices[glyph_ctr..glyph_ctr + len]
                                .iter()
                                .map(|(_, v)| *v);
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
                                let scaled_font = font.0.as_scaled(scale);

                                let height = scaled_font.ascent() - scaled_font.descent();
                                let offset = -(scaled_font.ascent() + scaled_font.descent()) / 2.0;
                                let highlight = text_editing.cursor_position + text_editing.highlight_vector;
                                // TODO: Clamp start/end to cached glyph count
                                let start = cmp::min(highlight as usize, text_editing.cursor_position as usize);
                                let end = cmp::max(highlight as usize, text_editing.cursor_position as usize);

                                let color = if let Some(tint) = tint {
                                    utils::mul_blend_srgba_to_lin_rgba_array(
                                        &text_editing.selected_background_color,
                                        &tint.0,
                                    )
                                } else {
                                    utils::srgba_to_lin_rgba_array(text_editing.selected_background_color)
                                };

                                let selection_ui_args_iter = ui_text.cached_glyphs[start..end]
                                    .iter()
                                    .map(|g| UiArgs {
                                        position: [g.x + g.advance_width / 2.0, g.y + offset].into(),
                                        dimensions: [g.advance_width, height].into(),
                                        tex_coords_bounds: [0.0, 0.0, 1.0, 1.0].into(),
                                        color: color.into(),
                                    });

                                if let Some(mut glyph_data) = glyphs {
                                    glyph_data.selection_vertices.extend(selection_ui_args_iter);
                                    glyph_data.height = height;
                                    glyph_data.space_width = scaled_font.h_advance(scaled_font.glyph_id(' '));

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
                        for (entity, (transform, ui_text, tint, text_editing, glyphs)) in glyph_query2.iter_entities_mut(world) {
                            if let (Some(text_editing), Some(mut glyphs)) = (text_editing, glyphs) {
                                let font = font_storage
                                    .get(&ui_text.font)
                                    .expect("Font with rendered glyphs must be loaded");
                                let scale = PxScale::from(ui_text.font_size);
                                let scaled_font = font.0.as_scaled(scale);

                                let height = scaled_font.ascent() - scaled_font.descent();
                                let offset = -(scaled_font.ascent() + scaled_font.descent()) / 2.0;
                                let highlight = text_editing.cursor_position + text_editing.highlight_vector;
                                // TODO: Clamp start/end to cached glyph count
                                let start = cmp::min(highlight as usize, text_editing.cursor_position as usize);
                                let end = cmp::max(highlight as usize, text_editing.cursor_position as usize);

                                update_cursor_position(
                                    &mut glyphs,
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

fn selected_bytes(text_editing: &TextEditing, text: &str) -> Option<Range<usize>> {
    if text_editing.highlight_vector == 0 {
        return None;
    }

    let start = cmp::min(
        text_editing.cursor_position,
        text_editing.cursor_position + text_editing.highlight_vector,
    ) as usize;

    let to_end = cmp::max(
        text_editing.cursor_position,
        text_editing.cursor_position + text_editing.highlight_vector,
    ) as usize - start - 1;

    let mut indexes = text.grapheme_indices(true).map(|(i, _)| i);
    let start_byte = indexes.nth(start).unwrap_or(text.len());
    let end_byte = indexes.nth(to_end).unwrap_or(text.len());

    if start_byte == end_byte {
        None
    } else {
        Some(start_byte..end_byte)
    }
}