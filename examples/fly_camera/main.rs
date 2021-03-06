//! Demonstrates how to use the fly camera

use amethyst::{
    assets::{AssetStorage, Loader},
    controls::{FlyControl, FlyControlBundle, HideCursor},
    core::{
        frame_limiter::FrameRateLimitStrategy,
        transform::{Transform, TransformBundle},
    },
    input::{is_key_down, is_mouse_button_down, InputBundle, StringBindings},
    prelude::*,
    renderer::{
        camera::Camera,
        light::{Light, PointLight},
        mtl::{Material, MaterialDefaults},
        palette::{LinSrgba, Srgb},
        plugins::{RenderShaded3D, RenderToWindow},
        rendy::{
            mesh::{Normal, Position, Tangent, TexCoord},
            texture::palette::load_from_linear_rgba,
        },
        shape::Shape,
        types::DefaultBackend,
        Mesh, RenderingBundle, Texture,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
    winit::{MouseButton, VirtualKeyCode},
    Error,
};

//type MyPrefabData = BasicScenePrefab<(Vec<Position>, Vec<Normal>, Vec<TexCoord>)>;

struct ExampleState;

impl SimpleState for ExampleState {
    fn on_start(&mut self, data: StateData<'_, GameData>) {
        let StateData {
            world, resources, ..
        } = data;

        let mat_defaults = resources.get::<MaterialDefaults>().unwrap().0.clone();
        let loader = resources.get::<Loader>().unwrap();
        let mesh_storage = resources.get::<AssetStorage<Mesh>>().unwrap();
        let tex_storage = resources.get::<AssetStorage<Texture>>().unwrap();
        let mtl_storage = resources.get::<AssetStorage<Material>>().unwrap();

        println!("Load mesh");
        let (mesh, albedo) = {
            let mesh = loader.load_from_data(
                Shape::Sphere(32, 32)
                    .generate::<(Vec<Position>, Vec<Normal>, Vec<Tangent>, Vec<TexCoord>)>(None)
                    .into(),
                (),
                &mesh_storage,
            );

            let albedo = loader.load_from_data(
                load_from_linear_rgba(LinSrgba::new(1.0, 1.0, 1.0, 0.5)).into(),
                (),
                &tex_storage,
            );

            (mesh, albedo)
        };

        println!("Create spheres");
        let spheres = (0..25).map(|n| {
            let i = n / 5;
            let j = n % 5;

            let roughness = 1.0f32 * (i as f32 / 4.0f32);
            let metallic = 1.0f32 * (j as f32 / 4.0f32);

            let mut pos = Transform::default();
            pos.set_translation_xyz(2.0f32 * (i - 2) as f32, 2.0f32 * (j - 2) as f32, 0.0);

            let mtl = {
                let metallic_roughness = loader.load_from_data(
                    load_from_linear_rgba(LinSrgba::new(0.0, roughness, metallic, 0.0)).into(),
                    (),
                    &tex_storage,
                );

                loader.load_from_data(
                    Material {
                        albedo: albedo.clone(),
                        metallic_roughness,
                        ..mat_defaults.clone()
                    },
                    (),
                    &mtl_storage,
                )
            };

            (pos, mesh.clone(), mtl)
        });

        world.extend(spheres);

        println!("Create lights");
        let light1: Light = PointLight {
            intensity: 6.0,
            color: Srgb::new(0.8, 0.0, 0.0),
            ..PointLight::default()
        }
        .into();

        let mut light1_transform = Transform::default();
        light1_transform.set_translation_xyz(6.0, 6.0, -6.0);

        let light2: Light = PointLight {
            intensity: 5.0,
            color: Srgb::new(0.0, 0.3, 0.7),
            ..PointLight::default()
        }
        .into();

        let mut light2_transform = Transform::default();
        light2_transform.set_translation_xyz(6.0, -6.0, -6.0);

        world.extend(vec![(light1, light1_transform), (light2, light2_transform)]);

        println!("Put camera");

        let mut transform = Transform::default();
        transform.set_translation_xyz(0.0, 0.0, -12.0);
        transform.prepend_rotation_y_axis(std::f32::consts::PI);

        let (width, height) = {
            let dim = resources.get::<ScreenDimensions>().unwrap();
            (dim.width(), dim.height())
        };

        world.extend(vec![(
            Camera::standard_3d(width, height),
            transform,
            FlyControl,
        )]);
    }

    fn handle_event(&mut self, data: StateData<'_, GameData>, event: StateEvent) -> SimpleTrans {
        let StateData { resources, .. } = data;
        if let StateEvent::Window(event) = &event {
            if is_key_down(&event, VirtualKeyCode::Escape) {
                let mut hide_cursor = resources.get_mut::<HideCursor>().unwrap();
                hide_cursor.hide = false;
            } else if is_mouse_button_down(&event, MouseButton::Left) {
                let mut hide_cursor = resources.get_mut::<HideCursor>().unwrap();
                hide_cursor.hide = true;
            }
        }
        Trans::None
    }
}

fn main() -> Result<(), Error> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let assets_dir = app_root.join("examples/fly_camera/assets");
    let display_config_path = app_root.join("examples/fly_camera/config/display.ron");
    let key_bindings_path = app_root.join("examples/fly_camera/config/input.ron");

    let mut builder = DispatcherBuilder::default();
    builder
        .add_bundle(
            InputBundle::<StringBindings>::new().with_bindings_from_file(&key_bindings_path)?,
        )
        .add_bundle(
            FlyControlBundle::<StringBindings>::new(
                Some(String::from("move_x")),
                Some(String::from("move_y")),
                Some(String::from("move_z")),
            )
            .with_sensitivity(0.1, 0.1),
        )
        .add_bundle(TransformBundle)
        .add_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config_path)?
                        .with_clear([0.0, 0.0, 0.0, 1.0]),
                )
                .with_plugin(RenderShaded3D::default()),
        );

    let mut game = Application::build(assets_dir, ExampleState)?
        .with_frame_limit(FrameRateLimitStrategy::Sleep, 60)
        .build(builder)?;

    game.run();
    Ok(())
}
