#[macro_use]
extern crate diesel;

use crate::core::build_kernel;
use crate::models::transactions::Transaction;
use crate::telegram::Telegram;
use futures::io::Error;
use tokio;

pub mod core;
pub mod models;
pub mod schema;
pub mod telegram;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let kernel = build_kernel();

    if let Ok(transactions) = Transaction::list(&kernel.conn().unwrap()) {
        println!("Transactions");

        for transaction in transactions {
            println!("{:?}", transaction)
        }
    }

    let telegram = Telegram::new(kernel.clone(), kernel.config().telegram_token.clone());
    let _ = telegram.listen().await;

    Ok(())
}
