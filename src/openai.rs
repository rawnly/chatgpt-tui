use crate::{
    models::{Chat, Message},
    Role,
};
use openai_rust::{chat, Client as OpenAIClient};

pub async fn send_message(chat: Chat) -> anyhow::Result<Option<Message>> {
    let client = OpenAIClient::new(&std::env::var("OPENAI_API_KEY").unwrap());

    let messages: Vec<chat::Message> = chat.messages.items.into_iter().map(|m| m.into()).collect();
    let args = chat::ChatArguments::new("gpt-3.5-turbo", messages);

    let res = client.create_chat(args).await?;

    match res.choices.first().map(|c| c.clone().message.content) {
        Some(content) => Ok(Some(Message::new(Role::Assistant, content.as_str()))),
        None => Ok(None),
    }
}
