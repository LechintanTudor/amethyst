use crate::{Anchor, Stretch};

pub struct UiTransformData {
    pub id: String,
    pub anchor: Anchor,
    pub pivot: Anchor,
    pub stretch: Stretch,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for UiTransformData {
    fn default() -> Self {
        Self {
            id: String::new(),
            anchor: Anchor::Middle,
            pivot: Anchor::Middle,
            stretch: Stretch::NoStretch,
            x: 0.0,
            y: 0.0,
            z: 1.0,
            width: 0.0,
            height: 0.0,
        }
    }
}
