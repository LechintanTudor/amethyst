use crate::{TextEditing, UiText};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

pub fn cursor_byte_index(text_editing: &TextEditing, ui_text: &UiText) -> usize {
    cursor_byte_index_str(text_editing, &ui_text.text)
}

pub fn cursor_byte_index_str(text_editing: &TextEditing, text: &str) -> usize {
    text
        .grapheme_indices(true)
        .nth(text_editing.cursor_position.max(0) as usize)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
}

pub fn highlighted_bytes_range(text_editing: &TextEditing, ui_text: &UiText) -> Range<usize> {
    highlighted_bytes_range_str(text_editing, &ui_text.text)
}

pub fn highlighted_bytes_range_str(text_editing: &TextEditing, text: &str) -> Range<usize> {
    let start = text_editing.cursor_position.min(
        text_editing.cursor_position + text_editing.highlight_vector,
    ).max(0);

    let to_end = (text_editing.cursor_position.max(
        text_editing.cursor_position + text_editing.highlight_vector,
    ) - start - 1).max(0);

    let mut indexes = text.grapheme_indices(true).map(|(i, _)| i);
    let start_byte = indexes.nth(start as usize).unwrap_or(text.len());
    let end_byte = indexes.nth(to_end as usize).unwrap_or(text.len());

    start_byte..end_byte
}

pub fn highlighted_text<'a>(text_editing: &TextEditing, ui_text: &'a UiText) -> &'a str {
    highlighted_text_str(text_editing, &ui_text.text)
}

pub fn highlighted_text_str<'a>(text_editing: &TextEditing, text: &'a str) -> &'a str {
    &text[highlighted_bytes_range_str(text_editing, text)]
}

pub fn delete_highlighted_text(text_editing: &mut TextEditing, ui_text: &mut UiText) -> bool {
    if text_editing.highlight_vector == 0 {
        return false;
    }

    let range = highlighted_bytes_range(text_editing, ui_text);
    text_editing.cursor_position = range.start as isize;
    text_editing.highlight_vector = 0;

    ui_text.text.replace_range(range, "");
    true
}

pub fn extract_highlighted_text(text_editing: &mut TextEditing, ui_text: &mut UiText) -> String {
    let range = highlighted_bytes_range(text_editing, ui_text);
    text_editing.cursor_position = range.start as isize;
    text_editing.highlight_vector = 0;

    ui_text.text.drain(range).collect()
}