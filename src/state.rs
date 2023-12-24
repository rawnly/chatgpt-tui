use crate::{openai::send_message, utils::trim_spaces};
use crossterm::event::KeyCode;

use crate::components::*;
use crate::models::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Modal {
    NewChat,
    RenameChat,
}

#[derive(Clone, Eq, PartialEq)]
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
    pub fn open_modal(&mut self, modal: Modal, input_value: Option<String>) {
        self.modal = Some(modal);

        self.focus = Some(Section::Modal);
        self.section = Section::Modal;

        if let Some(value) = input_value {
            self.modal_input.text = value;
        }
    }

    pub fn close_modal(&mut self) {
        self.focus = None;
        self.section = Section::Chats;
        self.modal = None;
        self.modal_input.clear();
    }

    pub async fn dispatch(&mut self, action: Action) -> anyhow::Result<()> {
        match &self.focus {
            None => match self.section {
                Section::Chats => match action {
                    Action::Enter => {
                        self.focus = Some(Section::Chats);
                        self.chats.next()
                    }
                    Action::Left | Action::Right => {
                        self.section = Section::Messages;
                    }
                    _ => {}
                },
                Section::Messages => match action {
                    Action::Enter => {
                        self.focus = Some(Section::Messages);
                    }
                    Action::Left | Action::Right => {
                        self.section = Section::Chats;
                    }
                    Action::Up | Action::Down => {
                        self.section = Section::Input;
                    }
                    _ => {}
                },
                Section::Input => match action {
                    Action::Enter => {
                        self.focus = Some(Section::Input);
                    }
                    Action::Left | Action::Right => {
                        self.section = Section::Chats;
                    }
                    Action::Up | Action::Down => {
                        self.section = Section::Messages;
                    }
                    _ => {}
                },
                _ => {}
            },
            Some(section) => match section {
                Section::Modal => match action {
                    Action::Esc => self.close_modal(),
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
                    Action::Char(to_enter) => self.modal_input.insert(to_enter),
                    Action::Backspace => self.modal_input.delete(),
                    Action::Left => self.modal_input.left(),
                    Action::Right => self.modal_input.right(),
                    _ => {}
                },
                Section::Chats => match action {
                    Action::Up => self.chats.prev(),
                    Action::Down => self.chats.next(),
                    Action::Esc => {
                        self.focus = None;
                        self.chats.unselect();
                        self.active_chat_idx = None;
                    }
                    Action::Enter => {
                        self.active_chat_idx = Some(self.chats.state.selected().unwrap());
                        self.section = Section::Input;
                        self.focus = Some(Section::Input);
                    }
                    _ => {}
                },
                Section::Messages => {
                    if let Some(chat) = self.get_active_chat_mut() {
                        match action {
                            Action::Backspace => self.delete_message(),
                            Action::Esc => self.focus = None,
                            Action::Up => chat.messages.prev(),
                            Action::Down => chat.messages.next(),
                            Action::Char('n') => self.open_modal(Modal::NewChat, None),
                            Action::Char('r') => {
                                let title = chat.title.clone();
                                self.open_modal(Modal::RenameChat, Some(title));
                            }
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

    pub fn delete_message(&mut self) {
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
    }

    pub fn rename_current_chat(&mut self, title: &str) {
        if let Some(chat) = self.get_active_chat_mut() {
            chat.title = title.to_string();
            self.close_modal();
        }
    }
}
