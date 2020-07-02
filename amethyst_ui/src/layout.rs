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
    pub fn norm_offset(self) -> (f32, f32) {
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

pub fn build_ui_transform_system() -> Box<dyn Schedulable> {
    let transform_modified = BitSet::new();
    let screen_size = (0_f32, 0_f32);

    SystemBuilder::<()>::new("UiTransformSystem")
        .read_resource::<ScreenDimensions>()
        .build(move |_, _, _, _| {

        });

    todo!("finish UiTransformSystem")
}