use crate::{
    LineMode, SelectedEntities, TextEditing, UiEvent, UiEventType, UiText,
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
use winit::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent};

pub fn build_text_editing_input_system(_: &mut World, resources: &mut Resources) -> Box<dyn Schedulable> {
    let mut winit_reader_id = resources
        .get_mut::<EventChannel<Event>>()
        .expect("`EventChannel<Event>` was not found in resources")
        .register_reader();

    SystemBuilder::<()>::new("TextEditingInputSystem")
        .read_resource::<EventChannel<Event>>()
        .read_resource::<SelectedEntities>()
        .write_resource::<EventChannel<UiEvent>>()
        .write_component::<UiText>()
        .write_component::<TextEditing>()
        .build(move |_, world, resources, queries| {
            let (winit_events, selected, ui_events) = resources;

            for event in winit_events.read(&mut winit_reader_id) {
                if let Some(entity) = selected.last_entity() {
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
                        Event::WindowEvent {
                            event: WindowEvent::KeyboardInput {
                                input: KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(keycode),
                                    modifiers,
                                    ..
                                },
                                ..
                            },
                            ..
                        } => match keycode {
                            VirtualKeyCode::Home | VirtualKeyCode::Up => {
                                text_editing.highlight_vector = if modifiers.shift {
                                    text_editing.cursor_position
                                } else {
                                    0
                                };
                                text_editing.cursor_position = 0;
                                text_editing.cursor_blink_timer = 0.0;
                            }
                            VirtualKeyCode::End | VirtualKeyCode::Down => {
                                let glyph_count = ui_text.text.graphemes(true).count() as isize;

                                text_editing.highlight_vector = if modifiers.shift {
                                    text_editing.cursor_position - glyph_count
                                } else {
                                    0
                                };

                                text_editing.cursor_position = glyph_count;
                                text_editing.cursor_blink_timer = 0.0;
                            }
                            VirtualKeyCode::Back => {
                                if !delete_highlighted(&mut text_editing, &mut ui_text)
                                    && text_editing.cursor_position > 0
                                {
                                    if let Some((byte, len)) = ui_text
                                        .text
                                        .grapheme_indices(true)
                                        .nth(text_editing.cursor_position as usize - 1)
                                        .map(|(byte, grapheme)| (byte, grapheme.len()))
                                    {
                                        ui_text.text.drain(byte..byte + len);
                                        text_editing.cursor_position -= 1;
                                    }
                                }
                            },
                            VirtualKeyCode::Delete => {
                                if !delete_highlighted(&mut text_editing, &mut ui_text) {
                                    if let Some((byte, len)) = ui_text
                                        .text
                                        .grapheme_indices(true)
                                        .nth(text_editing.cursor_position as usize)
                                        .map(|(byte, grapheme)| (byte, grapheme.len()))
                                    {
                                        ui_text.text.drain(byte..byte + len);
                                        text_editing.cursor_blink_timer = 0.0;
                                    }
                                }
                            }
                            VirtualKeyCode::Left => {
                                if text_editing.highlight_vector == 0 || modifiers.shift {
                                    if text_editing.cursor_position > 0 {
                                        let delta = if ctrl_or_cmd(modifiers) {
                                            let mut grapheme_count = 0;

                                            for word in ui_text.text.split_word_bounds() {
                                                let word_grapheme_count =
                                                    word.graphemes(true).count() as isize;

                                                if grapheme_count + word_grapheme_count
                                                    >= text_editing.cursor_position
                                                {
                                                    break;
                                                }

                                                grapheme_count += word_grapheme_count;
                                            }

                                            text_editing.cursor_position - grapheme_count
                                        } else {
                                            1
                                        };

                                        text_editing.cursor_position -= delta;

                                        if modifiers.shift {
                                            text_editing.highlight_vector += delta;
                                        }

                                        text_editing.cursor_blink_timer = 0.0;
                                    }
                                } else {
                                    text_editing.cursor_position = cmp::min(
                                        text_editing.cursor_position,
                                        text_editing.cursor_position + text_editing.highlight_vector,
                                    );
                                    text_editing.highlight_vector = 0;
                                }
                            },
                            VirtualKeyCode::Right => {
                                if text_editing.highlight_vector == 0 || modifiers.shift {
                                    let glyph_count = ui_text.text.graphemes(true).count();

                                    if (text_editing.cursor_position as usize) < glyph_count {
                                        let delta = if ctrl_or_cmd(modifiers) {
                                            let mut grapheme_count = 0_isize;

                                            for word in ui_text.text.split_word_bounds() {
                                                grapheme_count += word.graphemes(true).count() as isize;

                                                if grapheme_count > text_editing.cursor_position {
                                                    break;
                                                }
                                            }

                                            grapheme_count - text_editing.cursor_position
                                        } else {
                                            1
                                        };

                                        text_editing.cursor_position += delta;

                                        if modifiers.shift {
                                            text_editing.highlight_vector -= delta;
                                        }

                                        text_editing.cursor_blink_timer = 0.0;
                                    }
                                } else {
                                    text_editing.cursor_position = cmp::max(
                                        text_editing.cursor_position,
                                        text_editing.cursor_position + text_editing.highlight_vector,
                                    );
                                    text_editing.highlight_vector = 0;
                                }
                            }
                            VirtualKeyCode::Return | VirtualKeyCode::NumpadEnter => {
                                match ui_text.line_mode {
                                    LineMode::Single => {
                                        ui_events.single_write(UiEvent::new(
                                            UiEventType::ValueCommit,
                                            entity,
                                        ));
                                    }
                                    LineMode::Wrap => {
                                        if modifiers.shift {
                                            if ui_text.text.graphemes(true).count()
                                                < text_editing.max_length
                                            {
                                                let start_byte = ui_text
                                                    .text
                                                    .grapheme_indices(true)
                                                    .nth(text_editing.cursor_position as usize)
                                                    .map(|(i, _)| i)
                                                    .unwrap_or_else(|| ui_text.text.len());

                                                ui_text.text.insert_str(start_byte, "\n");
                                                text_editing.cursor_position += 1;

                                                ui_events.single_write(UiEvent::new(
                                                    UiEventType::ValueChange,
                                                    entity,
                                                ));
                                            }
                                        } else {
                                            ui_events.single_write(UiEvent::new(
                                                UiEventType::ValueCommit,
                                                entity,
                                            ));
                                        }
                                    }
                                }
                            }
                            _ => (),
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

fn ctrl_or_cmd(modifiers: ModifiersState) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.logo
    } else {
        modifiers.ctrl
    }
}

fn cursor_byte_index(text_editing: &TextEditing, ui_text: &UiText) -> usize {
    ui_text
        .text
        .grapheme_indices(true)
        .nth(text_editing.cursor_position as usize)
        .map(|(i, _)| i)
        .unwrap_or_else(|| ui_text.text.len())
}