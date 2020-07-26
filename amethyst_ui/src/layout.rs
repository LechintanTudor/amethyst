use crate::{Parent, UiTransform};
use amethyst_core::ecs::prelude::*;
use amethyst_window::ScreenDimensions;
use glyph_brush::{HorizontalAlign, VerticalAlign};
use serde::{Deserialize, Serialize};

#[cfg(feature = "profiler")]
use thread_profiler::profile_scope;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ScaleMode {
    Pixel,
    Percent,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Anchor {
    TopLeft,
    TopMiddle,
    TopRight,
    MiddleLeft,
    Middle,
    MiddleRight,
    BottomLeft,
    BottomMiddle,
    BottomRight,
}

impl Anchor {
    pub fn normalized_offset(self) -> (f32, f32) {
        match self {
            Anchor::TopLeft => (-0.5, 0.5),
            Anchor::TopMiddle => (0.0, 0.5),
            Anchor::TopRight => (0.5, 0.5),
            Anchor::MiddleLeft => (-0.5, 0.0),
            Anchor::Middle => (0.0, 0.0),
            Anchor::MiddleRight => (0.5, 0.0),
            Anchor::BottomLeft => (-0.5, -0.5),
            Anchor::BottomMiddle => (0.0, -0.5),
            Anchor::BottomRight => (0.5, -0.5),
        }
    }

    pub(crate) fn vertical_align(self) -> VerticalAlign {
        match self {
            Anchor::TopLeft => VerticalAlign::Top,
            Anchor::TopMiddle => VerticalAlign::Top,
            Anchor::TopRight => VerticalAlign::Top,
            Anchor::MiddleLeft => VerticalAlign::Center,
            Anchor::Middle => VerticalAlign::Center,
            Anchor::MiddleRight => VerticalAlign::Center,
            Anchor::BottomLeft => VerticalAlign::Bottom,
            Anchor::BottomMiddle => VerticalAlign::Bottom,
            Anchor::BottomRight => VerticalAlign::Bottom,
        }
    }

    pub(crate) fn horizontal_align(self) -> HorizontalAlign {
        match self {
            Anchor::TopLeft => HorizontalAlign::Left,
            Anchor::TopMiddle => HorizontalAlign::Center,
            Anchor::TopRight => HorizontalAlign::Right,
            Anchor::MiddleLeft => HorizontalAlign::Left,
            Anchor::Middle => HorizontalAlign::Center,
            Anchor::MiddleRight => HorizontalAlign::Right,
            Anchor::BottomLeft => HorizontalAlign::Left,
            Anchor::BottomMiddle => HorizontalAlign::Center,
            Anchor::BottomRight => HorizontalAlign::Right,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Stretch {
    NoStretch,
    X {
        x_margin: f32,
    },
    Y {
        y_margin: f32,
    },
    XY {
        x_margin: f32,
        y_margin: f32,
        keep_aspect_ratio: bool,
    },
}

pub fn build_ui_transform_system(_world: &mut World, _resources: &mut Resources) -> Box<dyn Schedulable> {
    let mut entities = Vec::<Entity>::new();
    let mut solved_transforms = BitSet::new();

    SystemBuilder::<()>::new("UiTransformSystem")
        .read_resource::<ScreenDimensions>()
        .with_query(
            <(TryRead<Parent>,)>::query().filter(component::<UiTransform>()),
        )
        .read_component::<Parent>()
        .write_component::<UiTransform>()
        .build(move |_, world, resources, query| {
            let screen_dimensions = resources;
            let screen_width = screen_dimensions.width();
            let screen_height = screen_dimensions.height();

            entities.clear();
            entities.extend(query.iter_entities(world).map(|(e, _)| e));

            solved_transforms.clear();

            for entity in entities.iter() {
                solve_transform(*entity, screen_width, screen_height, world, &mut solved_transforms);
            }
        })
}

fn solve_transform<E>(
    entity: Entity,
    screen_width: f32,
    screen_height: f32,
    world: &mut E,
    solved_transforms: &mut BitSet
)
where
    E: EntityStore
{
    // Mark transform as solved and skip solved transforms
    if !solved_transforms.insert(entity.index() as usize) {
        return;
    }

    let (
        parent_pixel_x,
        parent_pixel_y,
        parent_global_z,
        parent_pixel_width,
        parent_pixel_height,
    ) = match world.get_component::<Parent>(entity).map(|p| *p) {
        Some(Parent(parent)) => {
            solve_transform(parent, screen_width, screen_height, world, solved_transforms);

            match world.get_component::<UiTransform>(parent) {
                Some(transform) => (
                    transform.pixel_x,
                    transform.pixel_y,
                    transform.global_z,
                    transform.pixel_width,
                    transform.pixel_height,
                ),
                None => return,
            }
        }
        None => (
            0.0,
            0.0,
            0.0,
            screen_width,
            screen_height,
        )
    };

    if let Some(mut transform) = world.get_component_mut::<UiTransform>(entity) {
        modify_transform_bounds(
            &mut transform,
            parent_pixel_x,
            parent_pixel_y,
            parent_global_z,
            parent_pixel_width,
            parent_pixel_height,
        );
    }
}

fn modify_transform_bounds(
    transform: &mut UiTransform,
    parent_pixel_x: f32,
    parent_pixel_y: f32,
    parent_global_z: f32,
    parent_pixel_width: f32,
    parent_pixel_height: f32,
)
{
    let (offset_x, offset_y) = transform.anchor.normalized_offset();
    transform.pixel_x = parent_pixel_x + offset_x * parent_pixel_width;
    transform.pixel_y = parent_pixel_y + offset_y * parent_pixel_height;

    transform.global_z = parent_global_z + transform.local_z;

    let (new_width, new_height) = match transform.stretch {
        Stretch::NoStretch => (transform.width, transform.height),
        Stretch::X { x_margin } => (
            parent_pixel_width - x_margin * 2.0,
            transform.height,
        ),
        Stretch::Y { y_margin } => (
            transform.width,
            parent_pixel_height - y_margin * 2.0,
        ),
        Stretch::XY { x_margin, y_margin, keep_aspect_ratio: false } => (
            parent_pixel_width - x_margin * 2.0,
            parent_pixel_height - y_margin * 2.0,
        ),
        Stretch::XY { x_margin, y_margin, keep_aspect_ratio: true } => {
            let scale = f32::min(
                (parent_pixel_width - x_margin * 2.0) / transform.width,
                (parent_pixel_height - y_margin * 2.0) / transform.height,
            );

            (transform.width * scale, transform.height * scale)
        },
    };

    transform.width = new_width;
    transform.height = new_height;

    match transform.scale_mode {
        ScaleMode::Pixel => {
            transform.pixel_x += transform.local_x;
            transform.pixel_y += transform.local_y;
            transform.pixel_width = transform.width;
            transform.pixel_height = transform.height;
        },
        ScaleMode::Percent => {
            transform.pixel_x += transform.local_x * parent_pixel_width;
            transform.pixel_y += transform.local_y * parent_pixel_height;
            transform.pixel_width = transform.width * parent_pixel_width;
            transform.pixel_height = transform.height * parent_pixel_height;
        },
    }

    let (offset_x, offset_y) = transform.pivot.normalized_offset();
    transform.pixel_x += transform.pixel_width * -offset_x;
    transform.pixel_y += transform.pixel_height * -offset_y;
}