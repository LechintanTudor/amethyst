use amethyst_rendy::palette::Srgba;

pub fn srgba_to_lin_rgba_array(srgba: Srgba) -> [f32; 4] {
    let (r, g, b, a) = srgba.into_linear().into_components();
    [r, g, b, a]
}

pub fn mul_blend_lin_rgba_arrays(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
}