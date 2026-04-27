use iced::{
    Subscription,
    keyboard::{self, Key, key::Named},
};

use crate::{AppState, Message, Mode};

pub fn keyboard_input(state: &AppState) -> Subscription<Message> {
    keyboard::listen().with((state.page, state.mode)).map(
        |((page, mode), event): ((u32, Mode), keyboard::Event)| match event {
            keyboard::Event::KeyPressed {
                key,
                modified_key: _,
                physical_key: _,
                location: _,
                modifiers: _,
                text: _,
                repeat: _,
            } => {
                if let Key::Named(k) = key {
                    match k {
                        Named::ArrowLeft => match mode {
                            Mode::Explorer => {
                                if page > 0 {
                                    Message::Page(page - 1)
                                } else {
                                    Message::None
                                }
                            }
                            Mode::Viewer(index) => {
                                if index > 0 {
                                    Message::Mode(Mode::Viewer(index - 1))
                                } else {
                                    Message::None
                                }
                            }
                            _ => Message::None,
                        },
                        Named::ArrowRight => match mode {
                            Mode::Explorer => Message::Page(page + 1),
                            Mode::Viewer(index) => Message::Mode(Mode::Viewer(index + 1)),
                            _ => Message::None,
                        },
                        Named::Escape => {
                            if let Mode::Viewer(_) = mode {
                                Message::Mode(Mode::Explorer)
                            } else {
                                Message::None
                            }
                        }
                        _ => Message::None,
                    }
                } else {
                    Message::None
                }
            }
            _ => Message::None,
        },
    )
}
