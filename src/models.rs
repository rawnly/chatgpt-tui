use crate::stateful_list::StatefulList;
use crate::Role;
use openai_rust::chat::Message as OpenAIMessage;
use rand::distributions::Alphanumeric;
use rand::Rng;

pub type ID = String;

fn random_id(length: usize) -> ID {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

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

#[derive(Clone, Debug)]
pub struct Message {
    pub id: ID,
    pub content: String,
    pub role: Role,
}

impl Into<OpenAIMessage> for Message {
    fn into(self) -> OpenAIMessage {
        OpenAIMessage {
            role: self.role.to_string().to_owned(),
            content: self.content.to_owned(),
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
}
