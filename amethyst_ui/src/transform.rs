use crate::{Anchor, ScaleMode, Stretch};
use serde::{Deserialize, Serialize};

#[non_exhaustive]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UiTransform {
    pub id: String,
    pub anchor: Anchor,
    pub pivot: Anchor,
    pub stretch: Stretch,
    pub local_x: f32,
    pub local_y: f32,
    pub local_z: f32,
    pub width: f32,
    pub height: f32,
    pub(crate) pixel_x: f32,
    pub(crate) pixel_y: f32,
    pub(crate) global_z: f32,
    pub(crate) pixel_width: f32,
    pub(crate) pixel_height: f32,
    pub scale_mode: ScaleMode,
    pub opaque: bool,
    pub transparent_target: bool,
}

impl UiTransform {
    pub fn new(id: String, anchor: Anchor, pivot: Anchor, x: f32, y: f32, z: f32, width: f32, height: f32) -> Self {
        Self {
            id,
            anchor,
            pivot,
            stretch: Stretch::NoStretch,
            local_x: x,
            local_y: y,
            local_z: z,
            width,
            height,
            pixel_x: x,
            pixel_y: y,
            global_z: z,
            pixel_width: width,
            pixel_height: height,
            scale_mode: ScaleMode::Pixel,
            opaque: true,
            transparent_target: false,
        }
    }

    pub fn position_inside_local(&self, x: f32, y: f32) -> bool {
        x > self.local_x - self.width / 2.0 &&
        y > self.local_y - self.height / 2.0 &&
        x < self.local_x + self.width / 2.0 &&
        y < self.local_y + self.height / 2.0
    }

    pub fn position_inside(&self, x: f32, y: f32) -> bool {
        x > self.pixel_x - self.pixel_width / 2.0 &&
        y > self.pixel_y - self.pixel_height / 2.0 &&
        x < self.pixel_x + self.pixel_width / 2.0 &&
        y < self.pixel_y + self.pixel_height / 2.0
    }

    pub fn into_percent(mut self) -> Self {
        self.scale_mode = ScaleMode::Percent;
        self
    }

    pub fn into_transparent(mut self) -> Self {
        self.opaque = false;
        self
    }

    pub fn with_stretch(mut self, stretch: Stretch) -> Self {
        self.stretch = stretch;
        self
    }

    pub fn pixel_x(&self) -> f32 {
        self.pixel_x
    }

    pub fn pixel_y(&self) -> f32 {
        self.pixel_y
    }

    pub fn global_z(&self) -> f32 {
        self.global_z
    }

    pub fn pixel_width(&self) -> f32 {
        self.pixel_width
    }

    pub fn pixel_height(&self) -> f32 {
        self.pixel_height
    }
}