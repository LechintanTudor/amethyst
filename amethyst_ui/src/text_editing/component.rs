use amethyst_rendy::palette::Srgba;

#[derive(Copy, Clone, Debug)]
pub struct TextEditing {
    pub cursor_position: isize,
    pub max_length: usize,
    pub highlight_vector: isize,
    pub selected_text_color: Srgba,
    pub selected_background_color: Srgba,
    pub use_block_cursor: bool,
    pub(crate) cursor_blink_timer: f32,
}

impl TextEditing {
    pub fn new(
        max_length: usize,
        selected_text_color: Srgba,
        selected_background_color: Srgba,
        use_block_cursor: bool,
    ) -> Self
    {
        Self {
            cursor_position: 0,
            max_length,
            highlight_vector: 0,
            selected_text_color,
            selected_background_color,
            use_block_cursor,
            cursor_blink_timer: 0.0,
        }
    }
}