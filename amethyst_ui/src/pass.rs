use amethyst_assets::{Asset, AssetStorage, Handle};
use amethyst_error::Error;
use amethyst_rendy::{
    batch::OrderedOneLevelBatch,
    bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
    palette,
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
use glsl_layout::*;

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

lazy_static::lazy_static! {
    static ref UI_VERTEX: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../compiled/ui.vert.spv"),
        ShaderStageFlags::VERTEX,
        "main",
    ).unwrap();

    static ref UI_FRAGMENT: SpirvShader = SpirvShader::from_bytes(
        include_bytes!("../compiled/ui.frag.spv"),
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

impl<B> RenderPlugin for RenderUi
where B: Backend
{
    fn on_build(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        _builder: &mut DispatcherBuilder<'_>
    ) -> Result<(), Error>
    {
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
            ctx.add(RenderOrder::Overlay, DrawUiDesc::new().builder());
            Ok(())
        });
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
        subpass: gfx_hal::pass::Subpass<'_, B>,
        buffers: Vec<NodeBuffer>,
        images: Vec<NodeImage>
    ) -> Result<Box<dyn RenderGroup<B, T>>, failure::Error>
    {
        #[cfg(feature = "profiler")]
        profile_scope!("build");

        let env = DynamicUniform::new(factory, pso::ShaderStageFlags::VERTEX)?;
        let textures = TextureSub::new(factory)?;
        let vertex = DynamicVertexBuffer::new();

        todo!()
    }
}

#[repr(C, align(4))]
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, AsStd140)]
pub(crate) struct UiArgs {
    pub(crate) coords: vec2,
    pub(crate) dimensions: vec2,
    pub(crate) tex_coord_bounds: vec4,
    pub(crate) color: vec4,
    pub(crate) color_bias: vec4,
}

impl AsVertex for UiArgs {
    fn vertex() -> VertexFormat {
        VertexFormat::new((
            (Format::Rg32Sfloat, "coords"),
            (Format::Rg32Sfloat, "dimensions"),
            (Format::Rgba32Sfloat, "tex_coord_bounds"),
            (Format::Rgba32Sfloat, "color"),
            (Format::Rgba32Sfloat, "color_bias"),
        ))
    }
}

#[derive(Debug)]
pub struct DrawUi<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    env: DynamicUniform<B, UiViewArgs>,
    textures: TextureSub<B>,
    vertex: DynamicVertexBuffer<B, UiArgs>,
    batches: OrderedOneLevelBatch<TextureId, UiArgs>,
    change: ChangeDetection,
    cached_draw_order: CachedDrawOrder,
    white_tex: Handle<Texture>,
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