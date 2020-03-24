use crate::models::transactions::{Category, NewTransaction};
use crate::telegram::parser::ParseState::FindShopOrAmount;
use chrono::Utc;
use diesel::pg::data_types::Cents;
use std::ops::Add;

pub struct Parser {}

enum ParseState {
    Start,
    FindShopOrAmount,
    FindNote,
    End,
}

#[derive(Debug)]
pub enum ResultCommand {
    AddTransaction(NewTransaction),
    ListTransaction,
    SummaryCurrentMonth,
    Help,
}

impl Parser {
    // TODO: allow input not only usage but also income
    pub fn parse_message(&self, msg: &str) -> Option<ResultCommand> {
        let tokens = &mut msg.split_ascii_whitespace();
        let mut token = tokens.peekable();
        let token = token.next()?;

        // Predict command with first token
        if Category::from_string(token).is_some() {
            self.parse_new_transaction(msg)
                .map(ResultCommand::AddTransaction)
        } else if token.starts_with("list") {
            Some(ResultCommand::ListTransaction)
        } else if token.starts_with("help") {
            Some(ResultCommand::Help)
        } else if token.starts_with("current") {
            Some(ResultCommand::SummaryCurrentMonth)
        } else {
            None
        }
    }

    pub fn parse_new_transaction(&self, msg: &str) -> Option<NewTransaction> {
        let mut state = ParseState::Start;
        let mut result = NewTransaction {
            amount: Cents(0),
            category: Category::Food,
            date: Utc::now().naive_utc(),
            note: None,
            shop_name: None,
        };
        let tokens = &mut msg.split_ascii_whitespace();
        let tokens = &mut tokens.peekable();

        loop {
            let token = tokens.next();
            let ahead_token = tokens.peek();

            match state {
                ParseState::Start => {
                    result.category = Category::from_string(token?)?;
                    state = FindShopOrAmount;
                }
                // TODO: Simplify this state
                ParseState::FindShopOrAmount => {
                    //                     parse ahead if it is decimal then treat current token as shop
                    if ahead_token.is_some() {
                        let amount = ahead_token?.parse::<f64>();

                        if let Ok(amount) = amount {
                            state = ParseState::FindNote;
                            let builder = if result.shop_name.is_some() {
                                let builder = result.shop_name.unwrap().clone();
                                builder.add(" ")
                            } else {
                                String::new()
                            };
                            let builder = builder.add(token?);
                            result.shop_name = Some(builder);
                            result.amount = Cents((amount * 100_f64).round() as i64);
                            tokens.next();
                            continue;
                        }
                    }

                    let amount = token?.parse::<f64>();
                    if let Ok(amount) = amount {
                        state = ParseState::FindNote;
                        result.shop_name = None;
                        result.amount = Cents((amount * 100_f64).round() as i64);
                        continue;
                    }

                    state = ParseState::FindShopOrAmount;
                    let builder = if result.shop_name.is_some() {
                        let builder = result.shop_name.unwrap().clone();
                        builder.add(" ")
                    } else {
                        String::new()
                    };
                    let builder = builder.add(token?);
                    result.shop_name = Some(builder);
                }
                ParseState::FindNote => {
                    if token.is_some() {
                        let note = tokens.fold(String::from(token?), move |builder, str| {
                            let builder = builder.add(" ");
                            let builder = builder.add(str);
                            builder
                        });
                        result.note = Some(note)
                    }
                    state = ParseState::End
                }
                ParseState::End => break,
            }
        }

        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: improve macro output consistency
    macro_rules! assert_parse_transaction {
        ($text:expr, None) => {{
            {
                let parser = Parser {};
                let left = parser.parse_new_transaction($text);
                if left.is_some() {
                    panic!(
                        r#"assertion failed: `(text == result)`
    text:  `{:?}`,
  result:  `{:?}`
 expected: `None`"#,
                        $text, left
                    )
                }
                assert_eq!(left, None,)
            }
        }};

        ($text:expr, $result:expr) => {{
            {
                let parser = Parser {};
                let left = parser.parse_new_transaction($text).unwrap();
                let right = $result;
                if left != right {
                    panic!(
                        r#"assertion failed: `(text == result)`
   text: `{:?}`,
 result: `{:?}`"#,
                        right, left
                    )
                }
            }
        }};
    }

    #[test]
    fn test_parse_simple_transaction_input() {
        assert_parse_transaction!(
            "food boon tong kee 300.00",
            NewTransaction {
                amount: Cents(300_00),
                category: Category::Food,
                date: Utc::now().naive_utc(),
                note: None,
                shop_name: Some("boon tong kee".to_string()),
            }
        );
        assert_parse_transaction!(
            "food yayoi 300.00",
            NewTransaction {
                amount: Cents(300_00),
                category: Category::Food,
                date: Utc::now().naive_utc(),
                note: None,
                shop_name: Some("yayoi".to_string()),
            }
        );

        assert_parse_transaction!(
            " food mk 425 noted ",
            NewTransaction {
                amount: Cents(425_00),
                category: Category::Food,
                date: Utc::now().naive_utc(),
                note: Some("noted".to_string()),
                shop_name: Some("mk".to_string()),
            }
        );

        assert_parse_transaction!(
            "food 425  Note Note     \n  ",
            NewTransaction {
                amount: Cents(425_00),
                category: Category::Food,
                date: Utc::now().naive_utc(),
                note: Some("Note Note".to_string()),
                shop_name: None,
            }
        );

        assert_parse_transaction!(
            " FooD 425.0 Noted ",
            NewTransaction {
                amount: Cents(425_00),
                category: Category::Food,
                date: Utc::now().naive_utc(),
                note: Some("Noted".to_string()),
                shop_name: None,
            }
        );
    }

    #[test]
    fn test_parse_invalid_message() {
        let invalids = vec![
            "dude yayoi",
            "1337 hello",
            "yayoi food 300.00",
            "x",
            "300.00 yayoi food",
            "300.00 yayoi food jhaaa",
        ];

        invalids
            .iter()
            .for_each(|invalid| assert_parse_transaction!(invalid, None))
    }
}
