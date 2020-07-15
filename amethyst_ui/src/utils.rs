use amethyst_rendy::palette::Srgba;
use amethyst_window::ScreenDimensions;

pub fn srgba_to_lin_rgba_array(srgba: Srgba) -> [f32; 4] {
    let (r, g, b, a) = srgba.into_linear().into_components();
    [r, g, b, a]
}

pub fn mul_blend_lin_rgba_arrays(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
}

pub fn world_position(
    (mouse_x, mouse_y): (f32, f32),
    screen_dimensions: &ScreenDimensions
) -> (f32, f32)
{
    (
        mouse_x - screen_dimensions.width() / 2.0,
        screen_dimensions.height() - mouse_y - screen_dimensions.height() / 2.0,
    )
}