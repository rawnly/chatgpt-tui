use crate::{openai::send_message, utils::trim_spaces};
use crossterm::event::KeyCode;

use crate::components::*;
use crate::models::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Modal {
    NewChat,
    RenameChat,
}

#[derive(Clone, Eq, PartialEq, Copy)]
pub enum Section {
    Chats,
    Messages,
    Input,
    Modal,
}

#[derive(Clone)]
pub struct App {
    pub loading: bool,
    pub active_chat_idx: Option<usize>,
    pub chats: StatefulList<Chat>,
    pub input: Input,
    pub modal_input: Input,
    pub section: Section,
    pub focus: Option<Section>,
    pub modal: Option<Modal>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            modal: None,
            loading: false,
            section: Section::Chats,
            focus: Some(Section::Chats),
            input: Input::default(),
            modal_input: Input::default(),
            active_chat_idx: None,
            chats: StatefulList::with_items(vec![
                Chat::new("Demo"),
                Chat::with_messages(
                    "Christmas",
                    vec![
                        Message::user(
                            "What is christmas?"
                        ),
                        Message::assistant(
                            "Christmas is a religious holiday celebrating the birth of Jesus as well as a cultural and commercial event. Learn about the history of Christmas, Santa Claus, and holiday traditions worldwide."
                        )
                    ]
                )
            ])
        }
    }
}

pub enum Action {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Esc,
    Char(char),
    Key(KeyCode),
    Backspace,
}

impl App {
    #[allow(dead_code)]
    pub fn is_focused(&self, s: Section) -> bool {
        if let Some(focus) = self.focus {
            return focus == s;
        }

        false
    }

    pub fn focus(&mut self, section: Section) {
        self.focus = Some(section);
        self.section = section;
    }

    pub fn open_modal(&mut self, modal: Modal, input_value: Option<String>) {
        self.modal = Some(modal);

        self.focus = Some(Section::Modal);
        self.section = Section::Modal;

        if let Some(value) = input_value {
            self.modal_input.set_value(value);
        }
    }

    pub fn close_modal(&mut self) {
        self.modal = None;
        self.modal_input.clear();
    }

    pub fn select_current_chat(&mut self) {
        if let Some(i) = self.active_chat_idx {
            self.chats.select(i);
        } else {
            self.chats.select_first();
        }
    }

    pub async fn dispatch(&mut self, action: Action) -> color_eyre::Result<()> {
        match &self.focus {
            None => match self.section {
                Section::Chats if matches!(action, Action::Enter) => {
                    self.focus(Section::Chats);
                    self.select_current_chat()
                }
                Section::Messages | Section::Input
                    if matches!(
                        action,
                        Action::Esc | Action::Left | Action::Right | Action::Char('c')
                    ) =>
                {
                    self.focus(Section::Chats)
                }
                Section::Messages => match action {
                    Action::Enter => self.focus(Section::Messages),
                    Action::Up | Action::Down => self.section = Section::Input,
                    Action::Char('i') => self.focus(Section::Input),
                    _ => {}
                },
                Section::Input => match action {
                    Action::Char('m') => self.focus(Section::Messages),
                    Action::Enter => self.focus(Section::Input),
                    Action::Up | Action::Down => self.section = Section::Messages,
                    _ => {}
                },
                _ => {}
            },
            Some(section) => match section {
                Section::Modal => match action {
                    Action::Esc => {
                        match self.modal {
                            Some(Modal::RenameChat) | Some(Modal::NewChat) => {
                                self.section = Section::Chats;
                                self.focus = Some(Section::Chats);

                                self.select_current_chat();
                            }
                            _ => {}
                        };

                        self.close_modal();
                    }
                    Action::Char(to_enter) => self.modal_input.insert(to_enter),
                    Action::Backspace => self.modal_input.delete(),
                    Action::Left => self.modal_input.left(),
                    Action::Right => self.modal_input.right(),
                    Action::Enter => match self.modal {
                        Some(Modal::NewChat) => {
                            let title = &self.modal_input.text.clone();
                            self.new_chat(title)
                        }
                        Some(Modal::RenameChat) => {
                            let title = &self.modal_input.text.clone();
                            self.rename_current_chat(title)
                        }
                        None => {}
                    },
                    _ => {}
                },
                Section::Chats => match action {
                    Action::Up => self.chats.prev(),
                    Action::Down => self.chats.next(),
                    Action::Enter => {
                        self.active_chat_idx = Some(self.chats.state.selected().unwrap());
                        self.focus(Section::Input);
                    }
                    Action::Backspace => self.delete_current_chat(),
                    Action::Char('n') => self.open_modal(Modal::NewChat, None),
                    _ => {}
                },
                Section::Messages => {
                    if let Some(chat) = self.get_active_chat_mut() {
                        match action {
                            Action::Backspace => self.delete_message(),
                            Action::Up => chat.messages.prev(),
                            Action::Down => chat.messages.next(),
                            Action::Char('n') => self.open_modal(Modal::NewChat, None),
                            Action::Char('r') => {
                                let title = chat.title.clone();
                                self.open_modal(Modal::RenameChat, Some(title));
                            }
                            Action::Esc => self.blur(),
                            _ => {}
                        }
                    }
                }
                Section::Input => match action {
                    Action::Enter => self.submit_message().await?,
                    Action::Char(to_enter) => self.input.insert(to_enter),
                    Action::Backspace => self.input.delete(),
                    Action::Left => self.input.left(),
                    Action::Right => self.input.right(),
                    Action::Esc => self.focus = None,
                    _ => {}
                },
            },
        };

        Ok(())
    }

    pub fn get_active_chat_mut(&mut self) -> Option<&mut Chat> {
        match self.active_chat_idx {
            Some(index) => self.chats.items.get_mut(index),
            None => None,
        }
    }

    pub fn blur(&mut self) {
        self.focus = None;
    }

    pub async fn delete_message(&mut self) {
        if let Some(chat) = self.get_active_chat_mut() {
            if let Some(index) = chat.messages.state.selected() {
                chat.messages.prev();
                chat.messages.items.remove(index);
            }
        }
    }

    pub async fn submit_message(&mut self) -> anyhow::Result<()> {
        if self.input.is_empty() {
            self.input.clear();
            return Ok(());
        }

        self.loading = true;

        let message = Message::new(Role::User, trim_spaces(&self.input.text.clone()).as_str());

        self.input.clear();

        if let Some(active_chat_index) = self.active_chat_idx {
            if let Some(chat) = self.chats.items.get_mut(active_chat_index) {
                chat.append_message(message);

                if let Some(m) = send_message(chat.clone()).await? {
                    chat.append_message(m);
                }
            }
        }

        self.input.clear();

        self.loading = false;

        Ok(())
    }

    pub fn new_chat(&mut self, title: &str) {
        self.chats.items.push(Chat::new(title));
        self.close_modal();

        self.section = Section::Chats;
        self.focus = Some(Section::Chats);

        self.chats.select_last();
    }

    pub fn delete_current_chat(&mut self) {
        if let Some(i) = self.chats.state.selected() {
            self.chats.items.remove(i);

            if self.chats.items.is_empty() {
                self.chats.unselect();
                return;
            }

            self.chats.prev();
        }
    }

}
