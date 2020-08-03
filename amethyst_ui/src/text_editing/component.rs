use amethyst_rendy::palette::Srgba;

// Enables text editing when attached to an entity with
// a `UiText` component
#[derive(Copy, Clone, Debug)]
pub struct TextEditing {
    /// Cursor position in graphemes
    pub cursor_position: isize,
    /// Max number of graphemes in `UiText`'s string
    pub max_length: usize,
    /// The highlight position in graphemes relative to `cursor_position`
    pub highlight_vector: isize,
    /// The color of text when highlighted
    pub selected_text_color: Srgba,
    /// The background color of text when highlighted
    pub selected_background_color: Srgba,
    /// Whether to use a block cursor for editing. Not recommended in font
    /// is not monospace
    pub use_block_cursor: bool,
    pub(crate) cursor_blink_timer: f32,
}

impl TextEditing {
    /// Creates a new `TextEditing`.
    pub fn new(
        max_length: usize,
        selected_text_color: Srgba,
        selected_background_color: Srgba,
        use_block_cursor: bool,
    ) -> Self {
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
