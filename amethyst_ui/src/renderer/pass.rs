use crate::{
    SelectedEntities, TextEditing, UiImage, UiTransform,
    renderer::{UiGlyphs, UiGlyphsResource},
    sorted::SortedWidgets,
    systems,
    utils,
};
use amethyst_assets::{AssetStorage, Handle, Loader};
use amethyst_core::{
    dispatcher::{DispatcherBuilder, Stage},
    ecs::prelude::*,
};
use amethyst_error::Error;
use amethyst_rendy::{
    batch::OrderedOneLevelBatch,
    bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
    palette::Srgba,
    pipeline::{PipelineDescBuilder, PipelinesBuilder},
    rendy::{
        command::{QueueId, RenderPassEncoder},
        factory::Factory,
        graph::{
            render::{PrepareResult, RenderGroup, RenderGroupDesc},
            GraphContext, NodeBuffer, NodeImage,
        },
        hal::{
            self,
            device::Device,
            format::Format,
            pso::{self, ShaderStageFlags},
        },
        mesh::{AsVertex, VertexFormat},
        shader::{Shader, SpirvShader},
        texture::palette::load_from_srgba,
    },
    resources::Tint,
    simple_shader_set,
    submodules::{DynamicUniform, DynamicVertexBuffer, TextureId, TextureSub},
    system::GraphAuxData,
    types::{Backend, Texture},
    ChangeDetection, SpriteSheet,
};
use amethyst_window::ScreenDimensions;
use glsl_layout::*;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

lazy_static::lazy_static! {
    static ref UI_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/ui.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref UI_FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../../compiled/ui.frag.spv"),
        ShaderStageFlags::FRAGMENT,
        "main",
    ).unwrap();
}

#[derive(Default, Debug)]
pub struct RenderUi {
    target: Target,
}

impl RenderUi {
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}

impl<B> RenderPlugin<B> for RenderUi
where
    B: Backend
{
    fn on_build(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>,
    ) -> Result<(), Error>
    {
        builder.add_system(Stage::Render, systems::build_ui_glyphs_system::<B>);
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _world: &World,
        _resources: &Resources
    ) -> Result<(), Error>
    {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Overlay, DrawUiDesc::new().builder())?;
            Ok(())
        });
        Ok(())
    }
}

#[derive(Clone, Default, Debug)]
pub struct DrawUiDesc;

impl DrawUiDesc {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<B> RenderGroupDesc<B, GraphAuxData> for DrawUiDesc
where
    B: Backend
{
    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        _queue: QueueId,
        aux: &GraphAuxData,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>
    ) -> Result<Box<dyn RenderGroup<B, GraphAuxData>>, failure::Error>
    {
        #[cfg(feature = "profiler")]
        profile_scope!("build");

        let env = DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        let (pipeline, pipeline_layout) = build_ui_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            vec![env.raw_layout(), textures.raw_layout()],
        )?;

        let loader = aux.resources.get::<Loader>().unwrap();
        let texture_storage = aux.resources.get::<AssetStorage<Texture>>().unwrap();
        let white_texture = loader.load_from_data(
            load_from_srgba(Srgba::new(1.0, 1.0, 1.0, 1.0)).into(),
            (),
            &texture_storage
        );

        Ok(Box::new(DrawUi::<B> {
            pipeline,
            pipeline_layout,
            env,
            textures,
            vertex,
            change: ChangeDetection::default(),
            batches: OrderedOneLevelBatch::default(),
            white_texture,
        }))
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, AsStd140)]
pub(crate) struct UiArgs {
    // [center_x, center_y]
    pub position: vec2,
    // [width, height]
    pub dimensions: vec2,
    // [left, top, right, bottom] texture coordinates
    pub tex_coords_bounds: vec4,
    // Linear rgba color
    pub color: vec4,
    // Used for Metal support. Must be `[1.0, 1.0, 1.0, 0.0]` when sampling
    // from glyph texture and `[0.0, 0.0, 0.0, 0.0]` otherwise.
    pub color_bias: vec4,
}

impl AsVertex for UiArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rg32Sfloat, "position"),
            (Format::Rg32Sfloat, "dimensions"),
            (Format::Rgba32Sfloat, "tex_coords_bounds"),
            (Format::Rgba32Sfloat, "color"),
            (Format::Rgba32Sfloat, "color_bias"),
        ))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, AsStd140)]
struct UiViewArgs {
    inverse_window_half_size: vec2,
}

#[derive(Debug)]
pub struct DrawUi<B>
where
    B: Backend
{
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: DynamicUniform<B, UiViewArgs>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, UiArgs>,
    batches: OrderedOneLevelBatch<TextureId, UiArgs>,
    change: ChangeDetection,
    white_texture: Handle<Texture>,
}

impl<B> RenderGroup<B, GraphAuxData> for DrawUi<B>
where
    B: Backend
{
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        aux: &GraphAuxData,
    ) -> PrepareResult
    {
        #[cfg(feature = "profiler")]
        profile_scope!("prepare");

        let mut changed = false;
        let glyph_texture = aux.resources.get::<UiGlyphsResource>()
            .unwrap()
            .glyph_texture()
            .cloned();

        let (white_texture_id, glyph_texture_id) = if let (
            Some((white_texture_id, white_texture_changed)),
            Some((glyph_texture_id, glyph_texture_changed)),
        ) = (
            self.textures.insert(
                factory,
                aux.resources,
                &self.white_texture,
                hal::image::Layout::ShaderReadOnlyOptimal,
            ),
            glyph_texture.and_then(|texture| {
                self.textures.insert(
                    factory,
                    aux.resources,
                    &texture,
                    hal::image::Layout::General,
                )
            }),
        ) {
            changed = changed || white_texture_changed || glyph_texture_changed;
            (white_texture_id, glyph_texture_id)
        } else {
            self.textures.maintain(factory, aux.resources);
            return PrepareResult::DrawReuse;
        };

        // Batches
        self.batches.swap_clear();
        let selected = aux.resources.get::<SelectedEntities>().map(|s| s.last_entity()).flatten();

        for &(entity, _) in aux.resources.get::<SortedWidgets>().unwrap().widgets() {
            let transform = aux.world.get_component::<UiTransform>(entity).unwrap();
            let tint = aux.world.get_component::<Tint>(entity).map(|t| t.as_ref().clone());

            if let Some(image) = aux.world.get_component::<UiImage>(entity) {
                changed |= render_image(
                    factory,
                    &transform,
                    &image,
                    tint,
                    white_texture_id,
                    &mut self.textures,
                    &mut self.batches,
                    aux.resources,
                );
            }

            if let Some(glyph_data) = aux.world.get_component::<UiGlyphs>(entity) {
                if !glyph_data.selection_vertices.is_empty() {
                    self.batches.insert(white_texture_id, glyph_data.selection_vertices.iter().cloned());
                }

                if selected == Some(entity) {
                    if let Some(text_editing) = aux.world.get_component::<TextEditing>(entity) {
                        let blink = text_editing.cursor_blink_timer < 0.25;

                        let (w, h) = match (blink, text_editing.use_block_cursor) {
                            (false, false) => (0.0, 0.0),
                            (true, false) => (2.0, glyph_data.height),
                            (false, true) => (
                                glyph_data.space_width,
                                f32::max(1.0, glyph_data.height * 0.1),
                            ),
                            (true, true) => (glyph_data.space_width, glyph_data.height),
                        };

                        let base_x = glyph_data.cursor_position.0 + w / 2.0;
                        let base_y = glyph_data.cursor_position.1 - (glyph_data.height - h) / 2.0;

                        let min_x = transform.pixel_x - transform.pixel_width / 2.0;
                        let max_x = transform.pixel_x + transform.pixel_width / 2.0;
                        let min_y = transform.pixel_y - transform.pixel_height / 2.0;
                        let max_y = transform.pixel_y + transform.pixel_height / 2.0;

                        let left = (base_x - w / 2.0).max(min_x).min(max_x);
                        let right = (base_x + w / 2.0).max(min_x).min(max_x);
                        let top = (base_y - h / 2.0).max(min_y).min(max_y);
                        let bottom = (base_y + h / 2.0).max(min_y).min(max_y);

                        let x = (left + right) / 2.0;
                        let y = (top + bottom) / 2.0;
                        let w = right - left;
                        let h = bottom - top;

                        self.batches.insert(
                            white_texture_id,
                            Some(UiArgs {
                                position: [x, y].into(),
                                dimensions: [w, h].into(),
                                tex_coords_bounds: [0.0, 0.0, 1.0, 1.0].into(),
                                color: [1.0, 1.0, 1.0, 1.0].into(),
                                color_bias: [0.0, 0.0, 0.0, 0.0].into(),
                            }),
                        );
                    }
                }

                if !glyph_data.vertices.is_empty() {
                    self.batches.insert(glyph_texture_id, glyph_data.vertices.iter().cloned());
                }
            }
        }

        self.textures.maintain(factory, aux.resources);
        changed |= self.batches.changed();

        {
            #[cfg(feature = "profiler")]
            profile_scope!("write");

            changed |= self.vertex.write(
                factory,
                index,
                self.batches.count() as u64,
                Some(self.batches.data()),
            );

            let screen_dimensions = aux.resources.get::<ScreenDimensions>().unwrap();

            let view_args = UiViewArgs {
                inverse_window_half_size: [
                    1.0 / (screen_dimensions.width() as f32 / 2.0),
                    1.0 / (screen_dimensions.height() as f32 / 2.0),
                ].into(),
            };

            changed |= self.env.write(factory, index, view_args.std140());
        }

        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _aux: &GraphAuxData,
    )
    {
        #[cfg(feature = "profiler")]
        profile_scope!("draw");

        if self.batches.count() > 0 {
            encoder.bind_graphics_pipeline(&self.pipeline);
            self.env.bind(index, &self.pipeline_layout, 0, &mut encoder);
            self.vertex.bind(index, 0, 0, &mut encoder);

            for (&texture, range) in self.batches.iter() {
                self.textures.bind(&self.pipeline_layout, 1, texture, &mut encoder);
                unsafe {
                    encoder.draw(0..4, range);
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &GraphAuxData) {
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory.device().destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

fn build_ui_pipeline<B>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(B::GraphicsPipeline, B::PipelineLayout), failure::Error>
where
    B: Backend
{
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let shader_vertex = unsafe { UI_VERTEX.module(factory).unwrap() };
    let shader_fragment = unsafe { UI_FRAGMENT.module(factory).unwrap() };

    let pipes = PipelinesBuilder::new()
        .with_pipeline(
            PipelineDescBuilder::new()
                .with_vertex_desc(&[(UiArgs::vertex(), pso::VertexInputRate::Instance(1))])
                .with_input_assembler(pso::InputAssemblerDesc::new(hal::Primitive::TriangleStrip))
                .with_shaders(simple_shader_set(&shader_vertex, Some(&shader_fragment)))
                .with_layout(&pipeline_layout)
                .with_subpass(subpass)
                .with_framebuffer_size(framebuffer_width, framebuffer_height)
                .with_blend_targets(vec![pso::ColorBlendDesc {
                    mask: pso::ColorMask::ALL,
                    blend: Some(pso::BlendState::ALPHA),
                }]),
        )
        .build(factory, None);

    unsafe {
        factory.destroy_shader_module(shader_vertex);
        factory.destroy_shader_module(shader_fragment);
    }

    match pipes {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(mut pipes) => Ok((pipes.remove(0), pipeline_layout)),
    }
}

fn render_image<B>(
    factory: &Factory<B>,
    transform: &UiTransform,
    image: &UiImage,
    tint: Option<Tint>,
    white_texture_id: TextureId,
    textures: &mut TextureSub<B>,
    batches: &mut OrderedOneLevelBatch<TextureId, UiArgs>,
    resources: &Resources,
) -> bool
where
    B: Backend
{
    let color = utils::mul_blend_lin_rgba_arrays(image_color(image), tint_color(tint));

    match image {
        UiImage::Texture(texture) => {
            if let Some((texture_id, changed)) = textures.insert(
                factory,
                resources,
                texture,
                hal::image::Layout::ShaderReadOnlyOptimal,
            ) {
                let args = UiArgs {
                    position: [transform.pixel_x, transform.pixel_y].into(),
                    dimensions: [transform.pixel_width, transform.pixel_height].into(),
                    tex_coords_bounds: [0.0, 0.0, 1.0, 1.0].into(),
                    color: color.into(),
                    color_bias: [0.0, 0.0, 0.0, 0.0].into(),
                };

                batches.insert(texture_id, Some(args));
                changed
            } else {
                false
            }
        }
        UiImage::PartialTexture { texture, left, top, right, bottom } => {
            if let Some((texture_id, changed)) = textures.insert(
                factory,
                resources,
                texture,
                hal::image::Layout::ShaderReadOnlyOptimal,
            ) {
                let args = UiArgs {
                    position: [transform.pixel_x, transform.pixel_y].into(),
                    dimensions: [transform.pixel_width, transform.pixel_height].into(),
                    tex_coords_bounds: [*left, *top, *right, *bottom].into(),
                    color: color.into(),
                    color_bias: [0.0, 0.0, 0.0, 0.0].into(),
                };

                batches.insert(texture_id, Some(args));
                changed
            } else {
                false
            }
        }
        UiImage::Sprite(sprite_render) => {
            let sprite_sheet_storage = resources.get::<AssetStorage<SpriteSheet>>().unwrap();

            if let Some(sprite_sheet) = sprite_sheet_storage.get(&sprite_render.sprite_sheet) {
                if let Some((texture_id, changed)) = textures.insert(
                    factory,
                    resources,
                    &sprite_sheet.texture,
                    hal::image::Layout::ShaderReadOnlyOptimal,
                ) {
                    let tex_coords = &sprite_sheet.sprites[sprite_render.sprite_number].tex_coords;

                    let args = UiArgs {
                        position: [transform.pixel_x, transform.pixel_y].into(),
                        dimensions: [transform.pixel_width, transform.pixel_height].into(),
                        tex_coords_bounds: [
                            tex_coords.left,
                            tex_coords.top,
                            tex_coords.right,
                            tex_coords.bottom,
                        ].into(),
                        color: color.into(),
                        color_bias: [0.0, 0.0, 0.0, 0.0].into(),
                    };

                    batches.insert(texture_id, Some(args));
                    changed
                } else {
                    false
                }
            } else {
                false
            }
        }
        UiImage::SolidColor(_) => {
            let args = UiArgs {
                position: [transform.pixel_x, transform.pixel_y].into(),
                dimensions: [transform.pixel_width, transform.pixel_height].into(),
                tex_coords_bounds: [0.0, 0.0, 1.0, 1.0].into(),
                color: color.into(),
                color_bias: [0.0, 0.0, 0.0, 0.0].into(),
            };

            batches.insert(white_texture_id, Some(args));
            false
        }
    }
}

// Returns the `UiImage` color as linear RGBA array
fn image_color(image: &UiImage) -> [f32; 4] {
    match image {
        UiImage::SolidColor(color) => {
            let (r, g, b, a) = color.into_linear().into_components();
            [r, g, b, a]
        }
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

// Returns the `Tint` color as linear RGBA array
fn tint_color(tint: Option<Tint>) -> [f32; 4] {
    match tint {
        Some(Tint(color)) => {
            let (r, g, b, a) = color.into_linear().into_components();
            [r, g, b, a]
        }
        None => [1.0, 1.0, 1.0, 1.0]
    }
}