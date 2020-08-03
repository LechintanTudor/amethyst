use crate::{
    text_editing::*, LineMode, SelectedEntities, TextEditing, UiEvent, UiEventType, UiText,
};
use amethyst_core::{ecs::prelude::*, shrev::EventChannel};
use clipboard::{ClipboardContext, ClipboardProvider};
use log::error;
use std::cmp;
use unicode_normalization::{char::is_combining_mark, UnicodeNormalization};
use unicode_segmentation::UnicodeSegmentation;
use winit::{ElementState, Event, KeyboardInput, ModifiersState, VirtualKeyCode, WindowEvent};

pub fn build_text_editing_input_system(
    _world: &mut World,
    resources: &mut Resources,
) -> Box<dyn Schedulable> {
    let mut winit_reader_id = resources
        .get_mut_or_default::<EventChannel<Event>>()
        .unwrap()
        .register_reader();

    let mut clipboard = ClipboardContext::new().expect("Failed to create clipboard context");

    SystemBuilder::<()>::new("TextEditingInputSystem")
        .with_query(Write::<UiText>::query())
        .read_resource::<EventChannel<Event>>()
        .read_resource::<SelectedEntities>()
        .write_resource::<EventChannel<UiEvent>>()
        .write_component::<UiText>()
        .write_component::<TextEditing>()
        .build(move |_, world, resources, query| {
            let (winit_events, selected, ui_events) = resources;

            for mut ui_text in query.iter_mut(world) {
                if ui_text.text.chars().any(is_combining_mark) {
                    ui_text.text = ui_text.text.nfd().collect();
                }
            }

            for event in winit_events.read(&mut winit_reader_id) {
                if let Some(entity) = selected.last() {
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
                            delete_highlighted_text(&mut text_editing, &mut ui_text);

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
                                if !delete_highlighted_text(&mut text_editing, &mut ui_text)
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
                                if !delete_highlighted_text(&mut text_editing, &mut ui_text) {
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
                            VirtualKeyCode::A => if ctrl_or_cmd(modifiers) {
                                let grapheme_count = ui_text.text.graphemes(true).count() as isize;
                                text_editing.cursor_position = grapheme_count;
                                text_editing.highlight_vector = -grapheme_count;
                            }
                            VirtualKeyCode::X => if ctrl_or_cmd(modifiers) {
                                let cut_text = extract_highlighted_text(&mut text_editing, &mut ui_text);

                                if !cut_text.is_empty() {
                                    match clipboard.set_contents(cut_text) {
                                        Ok(_) => {
                                            ui_events.single_write(UiEvent::new(
                                                UiEventType::ValueChange,
                                                entity,
                                            ));
                                        }
                                        Err(e) => {
                                            error!("Error occured when cutting to clipboard: {:?}", e);
                                        }
                                    }
                                }
                            }
                            VirtualKeyCode::C => if ctrl_or_cmd(modifiers) {
                                let copied_text = highlighted_text(&text_editing, &ui_text);

                                if !copied_text.is_empty() {
                                    match clipboard.set_contents(copied_text.to_string()) {
                                        Ok(_) => {
                                            ui_events.single_write(UiEvent::new(
                                                UiEventType::ValueChange,
                                                entity,
                                            ));
                                        }
                                        Err(e) => {
                                            error!("Error occured when copying to clipboard: {:?}", e);
                                        }
                                    }
                                }
                            }
                            VirtualKeyCode::V => if ctrl_or_cmd(modifiers) {
                                delete_highlighted_text(&mut text_editing, &mut ui_text);

                                match clipboard.get_contents() {
                                    Ok(clipboard_text) => {
                                        let index = cursor_byte_index(&text_editing, &ui_text);

                                        let available_graphemes = (text_editing.max_length as usize
                                            - ui_text.text.graphemes(true).count()).min(clipboard_text.len());

                                        let available_bytes = clipboard_text
                                            .grapheme_indices(true)
                                            .nth(available_graphemes)
                                            .map(|(i, _)| i)
                                            .unwrap_or(clipboard_text.len());

                                        ui_text.text.insert_str(index, &clipboard_text[0..available_bytes]);
                                        text_editing.cursor_position += available_graphemes as isize;

                                        ui_events.single_write(UiEvent::new(
                                            UiEventType::ValueChange,
                                            entity,
                                        ));
                                    }
                                    Err(e) => {
                                        error!("Error occured when pasting contents of clipboard: {:?}", e);
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
    input < '\u{20}'
        || input == '\u{7F}'
        || (input >= '\u{E000}' && input <= '\u{F8FF}')
        || (input >= '\u{F0000}' && input <= '\u{FFFFF}')
        || (input >= '\u{100000}' && input <= '\u{10FFFF}')
}

fn ctrl_or_cmd(modifiers: ModifiersState) -> bool {
    if cfg!(target_os = "macos") {
        modifiers.logo
    } else {
        modifiers.ctrl
    }
}
