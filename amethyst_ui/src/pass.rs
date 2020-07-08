use crate::{
    UiImage, UiTransform,
    systems,
};
use amethyst_assets::{AssetStorage, Handle, Loader};
use amethyst_core::{
    Hidden, HiddenPropagate,
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
use specs::hibitset::BitSet;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

lazy_static::lazy_static! {
    static ref UI_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../compiled/ui2.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref UI_FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../compiled/ui2.frag.spv"),
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
where B: Backend
{
    fn on_build(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        builder: &mut DispatcherBuilder<'_>
    ) -> Result<(), Error>
    {
        // builder.add_system(Stage::Logic, systems::build_ui_glyphs_system::<B>);
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        factory: &mut Factory<B>,
        world: &World,
        resources: &Resources
    ) -> Result<(), Error>
    {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Overlay, DrawUiDesc::new().builder())?;
            Ok(())
        });
        Ok(())
    }
}

#[derive(Default, Debug)]
pub struct DrawUiDesc;

impl DrawUiDesc {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<B> RenderGroupDesc<B, GraphAuxData> for DrawUiDesc
where B: Backend
{
    fn build<'a>(
        self,
        ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        aux: &GraphAuxData,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>
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
            cached_draw_order: CachedDrawOrder::default(),
            batches: OrderedOneLevelBatch::default(),
            white_texture,
        }))
    }
}

#[repr(C)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, AsStd140)]
pub(crate) struct UiArgs {
    pub(crate) position: vec2,
    pub(crate) dimensions: vec2,
    pub(crate) color: vec4,
    pub(crate) tex_coords_bounds: vec4,
}

impl AsVertex for UiArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rg32Sfloat, "position"),
            (Format::Rg32Sfloat, "dimensions"),
            (Format::Rgba32Sfloat, "color"),
            (Format::Rgba32Sfloat, "tex_coords_bounds"),
        ))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, AsStd140)]
struct UiViewArgs {
    inverse_window_size: vec2,
}

#[derive(Debug)]
pub struct DrawUi<B>
where B: Backend
{
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: DynamicUniform<B, UiViewArgs>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, UiArgs>,
    batches: OrderedOneLevelBatch<TextureId, UiArgs>,
    change: ChangeDetection,
    cached_draw_order: CachedDrawOrder,
    white_texture: Handle<Texture>,
}

impl<B> RenderGroup<B, GraphAuxData> for DrawUi<B>
where B: Backend
{
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        queue: QueueId,
        index: usize,
        subpass: hal::pass::Subpass<'_, B>,
        aux: &GraphAuxData
    ) -> PrepareResult
    {
        let mut changed = false;

        let white_texture_id = {
            if let Some((white_texture_id, white_texture_changed)) = self.textures.insert(
                factory,
                aux.resources,
                &self.white_texture,
                hal::image::Layout::ShaderReadOnlyOptimal,
            ) {
                changed = changed || white_texture_changed;
                white_texture_id
            } else {
                self.textures.maintain(factory, aux.resources);
                return PrepareResult::DrawReuse;
            }
        };

        // Batches
        self.batches.swap_clear();

        let widget_query = <(Read<UiTransform>,)>::query()
            .filter(!component::<Hidden>() & !component::<HiddenPropagate>());

        for (entity, (transform,)) in widget_query.iter_entities(aux.world) {
            let tint = aux.world.get_component::<Tint>(entity).map(|t| t.as_ref().clone());

            if let Some(image) = aux.world.get_component::<UiImage>(entity) {
                let image_changed = render_image(
                    factory,
                    &transform,
                    &image,
                    tint,
                    white_texture_id,
                    &mut self.textures,
                    &mut self.batches,
                    aux.resources,
                );

                changed = changed || image_changed;
            }
        }

        self.textures.maintain(factory, aux.resources);

        self.vertex.write(
            factory,
            index,
            self.batches.count() as u64,
            Some(self.batches.data()),
        );

        // View args
        let screen_dimensions = aux.resources.get::<ScreenDimensions>().unwrap();

        let view_args = UiViewArgs {
            inverse_window_size: [
                1.0 / screen_dimensions.width() as f32,
                1.0 / screen_dimensions.height() as f32,
            ].into(),
        };

        let env_changed = self.env.write(factory, index, view_args.std140());
        changed = changed || env_changed;

        self.change.prepare_result(index, changed)
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize, subpass:
        hal::pass::Subpass<'_, B>,
        aux: &GraphAuxData)
    {
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

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, aux: &GraphAuxData) {
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
where B: Backend
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
where B: Backend
{
    let color = mul_blend(image_color(image), tint_color(tint));

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
                    color: color.into(),
                    tex_coords_bounds: [0.0_f32, 0.0, 1.0, 1.0].into(),
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
                    color: color.into(),
                    tex_coords_bounds: [*left, *top, *right, *bottom].into(),
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
                        color: color.into(),
                        tex_coords_bounds: [
                            tex_coords.left,
                            tex_coords.top,
                            tex_coords.right,
                            tex_coords.bottom,
                        ].into(),
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
                color: color.into(),
                tex_coords_bounds: [0.0_f32, 0.0, 1.0, 1.0].into(),
            };

            batches.insert(white_texture_id, Some(args));
            false
        }
        _ => false,
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

fn mul_blend(color1: [f32; 4], color2: [f32; 4]) -> [f32; 4] {
    [color1[0] * color2[0], color1[1] * color2[1], color1[2] * color2[2], color1[3] * color2[3]]
}

#[derive(Clone, Default, Debug)]
struct CachedDrawOrder {
    cached: BitSet,
    cache: Vec<(f32, Entity)>,
}