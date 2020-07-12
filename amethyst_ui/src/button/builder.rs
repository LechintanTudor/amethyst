const DEFAULT_Z: f32 = 1.0;
const DEFAULT_WIDTH: f32 = 128.0;
const DEFAULT_HEIGHT: f32 = 64.0;

#[derive(Clone, Debug)]
pub struct UiButtonBuilder {
    x: f32,
    y: f32,
    z: f32,
    width: f32,
    height: f32,
}

impl Default for UiButtonBuilder {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: DEFAULT_Z,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
        }
    }
}