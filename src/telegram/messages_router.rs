use telegram_bot::{Api, Message, MessageKind};
use telegram_bot::prelude::*;

use crate::core::Kernel;
use crate::telegram::parser::{Parser, ResultCommand};
use crate::models::transactions::Transaction;

pub struct MessagesRouter {
    pub kernel: Kernel,
    pub api: Api,
}

impl MessagesRouter {
    pub async fn handle(&self, message: Message) {
        let message = &message;

        if let MessageKind::Text {
            data: text,
            entities: _entity_type,
        } = &message.kind
        {
            self.process(message, &text).await
        }
    }

    async fn process<'a>(&self, message: &'a Message, text: &'a str) {
        let text = String::from(text);
        let chat = message.chat.clone();
        let result = Parser {}.parse_message(&text);

        match result {
            None => {
                let _res = self.api
                    .send(chat.text("Ehhh, what do you mean dude?"))
                    .await;
            }
            Some(cmd) => {
                self.execute(message, cmd).await
            }
        };
    }

    async fn execute(&self, message: &Message, cmd: ResultCommand) {
        let chat = message.chat.clone();
        let db = self.kernel.conn().unwrap();

        match cmd {
            ResultCommand::AddTransaction(tran) => {
                let tran = tran.create(&db);

                let text = format!("Dude, nice job! '{:?}'", tran);
                let text = chat.text(text);

                let _res = self.api
                    .send(text)
                    .await;
            }
            ResultCommand::ListTransaction => {
                let trans = Transaction::list(&db);

                let text = format!("Your list! '{:?}'", trans);
                let text = chat.text(text);

                let _res = self.api
                    .send(text)
                    .await;
            }
        }
    }
}
