use futures::prelude::*;
use telegram_bot::{Api, Message, UpdateKind};

use crate::core::Kernel;
use crate::telegram::messages_router::MessagesRouter;

mod messages_router;
mod parser;

#[derive(Clone)]
pub struct Telegram {
    api: Api,
    kernel: Kernel,
}

impl Telegram {
    pub fn new(kernel: Kernel, key: String) -> Self {
        Telegram {
            api: Api::new(key.as_str()),
            kernel,
        }
    }

    pub async fn listen(self: &Self) {
        let mut stream = self.api.stream();

        while let Some(update) = stream.next().await {
            if update.is_err() {
                eprintln!("error: {:?}", update);
                continue;
            }

            let update = update.unwrap();
            let new_self = self.clone();

            tokio::spawn(async move {
                match update.kind {
                    UpdateKind::Message(message) => {
                        new_self.handle_message(message).await;
                    }
                    UpdateKind::EditedMessage(_) => {}
                    UpdateKind::ChannelPost(_) => {}
                    UpdateKind::EditedChannelPost(_) => {}
                    UpdateKind::InlineQuery(_) => {}
                    UpdateKind::CallbackQuery(_) => {}
                    UpdateKind::Error(_) => {}
                    UpdateKind::Unknown => {}
                }
            });
        };
    }

    pub async fn handle_message(self: &Self, msg: Message) {
        MessagesRouter {
            kernel: self.kernel.clone(),
            api: self.api.clone(),
        }
            .handle(msg)
            .await
    }
}
