use crate::{
    Selected, TextEditing, UiEvent, UiEventType, UiText,
};
use amethyst_core::{
    ecs::prelude::*,
    shrev::EventChannel,
};
use std::{
    cmp,
    ops::Range,
};
use unicode_normalization::{
    UnicodeNormalization,
    char::is_combining_mark,
};
use unicode_segmentation::UnicodeSegmentation;
use winit::{Event, WindowEvent};

pub fn build_text_editing_input_system(_: &mut World, resources: &mut Resources) -> Box<dyn Schedulable> {
    let mut winit_reader_id = resources
        .get_mut::<EventChannel<Event>>()
        .expect("`EventChannel<Event>` was not found in resources")
        .register_reader();

    SystemBuilder::<()>::new("TextEditingInputSystem")
        .read_resource::<EventChannel<Event>>()
        .read_resource::<Selected>()
        .write_resource::<EventChannel<UiEvent>>()
        .write_component::<UiText>()
        .write_component::<TextEditing>()
        .build(move |_, world, resources, queries| {
            let (winit_events, selected, ui_events) = resources;

            for event in winit_events.read(&mut winit_reader_id) {
                if let Some(entity) = selected.entity {
                    // Safe because system locks `UiText`
                    let mut ui_text = unsafe {
                        match world.get_component_mut_unchecked::<UiText>(entity) {
                            Some(ui_text) => ui_text,
                            None => continue,
                        }
                    };

                    // Safe because system locks `TextEditing`
                    let mut text_editing = unsafe {
                        match world.get_component_mut_unchecked::<TextEditing>(entity) {
                            Some(text_editing) => text_editing,
                            None => continue,
                        }
                    };

                    match *event {
                        Event::WindowEvent { event: WindowEvent::ReceivedCharacter(input), .. } => {
                            if should_skip_char(input) {
                                continue;
                            }

                            text_editing.cursor_blink_timer = 0.0;
                            delete_highlighted(&mut text_editing, &mut ui_text);

                            let start_byte = ui_text
                                .text
                                .grapheme_indices(true)
                                .nth(text_editing.cursor_position as usize)
                                .map(|(i, _)| i)
                                .unwrap_or_else(|| ui_text.text.len());

                            if ui_text.text.graphemes(true).count() < text_editing.max_length {
                                ui_text.text.insert(start_byte, input);
                                text_editing.cursor_position += 1;
                                ui_events.single_write(UiEvent::new(UiEventType::ValueChange, entity));
                            }
                        }
                        _ => (),
                    }
                }
            }
        })
}

fn should_skip_char(input: char) -> bool {
    input < '\u{20}' ||
    input == '\u{7F}'||
    (input >= '\u{E000}' && input <= '\u{F8FF}') ||
    (input >= '\u{F0000}' && input <= '\u{FFFFF}') ||
    (input >= '\u{100000}' && input <= '\u{10FFFF}')
}

fn delete_highlighted(text_editing: &mut TextEditing, ui_text: &mut UiText) -> bool {
    if text_editing.highlight_vector != 0 {
        let range = highlighted_bytes(text_editing, ui_text);
        text_editing.cursor_position = range.start as isize;
        text_editing.highlight_vector = 0;
        ui_text.text.drain(range);
        true
    } else {
        false
    }
}

fn highlighted_bytes(text_editing: &TextEditing, ui_text: &UiText) -> Range<usize> {
    let start = cmp::min(
        text_editing.cursor_position,
        text_editing.cursor_position + text_editing.highlight_vector,
    ) as usize;

    let end = cmp::max(
        text_editing.cursor_position,
        text_editing.cursor_position + text_editing.highlight_vector,
    ) as usize;

    let start_byte = ui_text
        .text
        .grapheme_indices(true)
        .nth(start)
        .map(|(i, _)| i)
        .unwrap_or_else(|| ui_text.text.len());

    let end_byte = ui_text
        .text.grapheme_indices(true)
        .nth(end)
        .map(|(i, _)| i)
        .unwrap_or_else(|| ui_text.text.len());

    start_byte..end_byte
}