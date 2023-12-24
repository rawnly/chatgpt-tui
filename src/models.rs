use std::fmt::Display;
use std::str::FromStr;

use crate::components::stateful_list::StatefulList;
use openai_rust::chat::Message as OpenAIMessage;
use rand::distributions::Alphanumeric;
use rand::Rng;

pub type ID = String;

/// Generates a random ID
fn random_id(length: usize) -> ID {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

// ---- Role

#[derive(Clone, Copy, Debug)]
pub enum Role {
    User,
    Assistant,
}

impl FromStr for Role {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(Role::User),
            "assistant" => Ok(Role::Assistant),
            _ => Err(anyhow::anyhow!("Invalid role")),
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Role::User => "user".to_string(),
            Role::Assistant => "assistant".to_string(),
        };

        write!(f, "{}", str)
    }
}

// ---- Chat

#[derive(Debug, Clone)]
pub struct Chat {
    pub id: ID,
    pub title: String,
    pub messages: StatefulList<Message>,
}

impl Chat {
    pub fn new(title: &str) -> Self {
        Self {
            id: random_id(7),
            title: title.to_string(),
            messages: StatefulList::with_items(vec![]),
        }
    }

    pub fn append_message(&mut self, message: Message) {
        self.messages.items.push(message);
    }

    pub fn with_messages(title: &str, messages: Vec<Message>) -> Self {
        Self {
            id: random_id(7),
            title: title.to_string(),
            messages: StatefulList::with_items(messages),
        }
    }
}

// ------ Message

#[derive(Clone, Debug)]
pub struct Message {
    pub id: ID,
    pub content: String,
    pub role: Role,
}

impl From<Message> for OpenAIMessage {
    fn from(value: Message) -> Self {
        Self {
            content: value.content,
            role: value.role.to_string(),
        }
    }
}

impl Message {
    pub fn new(role: Role, content: &str) -> Self {
        Self {
            id: random_id(7),
            content: content.to_string(),
            role,
        }
    }

    pub fn assistant(content: &str) -> Self {
        Self::new(Role::Assistant, content)
    }

    pub fn user(content: &str) -> Self {
        Self::new(Role::User, content)
    }
}
