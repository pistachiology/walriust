use std::io::Write;

use chrono::NaiveDateTime;
use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::pg::types::money::PgMoney;
use diesel::pg::Pg;
use diesel::result::Error;
use diesel::serialize::{Output, ToSql};
use diesel::sql_types::Text;
use diesel::{deserialize, serialize, Insertable, Queryable};
use diesel::{PgConnection, RunQueryDsl};

use crate::schema::transactions;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression)]
#[sql_type = "Text"]
pub enum Category {
    Food,
    Travel,
    Miscellaneous,
}

impl Category {
    pub fn from_string(str: &str) -> Option<Self> {
        match str.to_lowercase().trim() {
            "food" => Some(Category::Food),
            "travel" => Some(Category::Travel),
            "misc" => Some(Category::Miscellaneous),
            _ => None,
        }
    }

    pub fn is_category(token: &str) -> bool {
        return Self::from_string(token).is_some();
    }
}

impl ToSql<Text, Pg> for Category {
    fn to_sql<'a, W: Write>(&self, out: &mut Output<'a, W, Pg>) -> serialize::Result {
        let category = match self {
            Category::Food => "food",
            Category::Travel => "travel",
            Category::Miscellaneous => "misc",
        };
        <&str as ToSql<Text, Pg>>::to_sql(&category, out)
    }
}

impl FromSql<Text, Pg> for Category {
    fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> deserialize::Result<Self> {
        let str = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        match Self::from_string(&str) {
            Some(cat) => Ok(cat),
            None => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Insertable, Debug)]
#[table_name = "transactions"]
pub struct NewTransaction {
    pub amount: PgMoney,
    pub category: Category,
    pub date: NaiveDateTime,
    pub note: Option<String>,
    pub shop_name: Option<String>,
}

impl NewTransaction {
    pub fn create(&self, conn: &PgConnection) -> Result<Transaction, Error> {
        diesel::insert_into(transactions::table)
            .values(self)
            .get_result(conn)
    }
}

impl PartialEq<Self> for NewTransaction {
    // TODO: Compare time should not rely on delta time would be better if we can freeze time
    fn eq(&self, other: &NewTransaction) -> bool {
        let timedelta = self
            .date
            .signed_duration_since(other.date)
            .num_seconds()
            .abs();
        let same_date = timedelta < 2;

        (same_date
            && self.category == other.category
            && self.amount == other.amount
            && self.note == other.note
            && self.shop_name == other.shop_name)
    }

    fn ne(&self, other: &NewTransaction) -> bool {
        !self.eq(other)
    }
}

#[derive(Queryable, Debug)]
pub struct Transaction {
    pub id: i32,
    pub amount: PgMoney,
    pub category: Category,
    pub date: NaiveDateTime,
    pub note: Option<String>,
    pub shop_name: Option<String>,
}

impl Transaction {
    pub fn list(conn: &PgConnection) -> Result<Vec<Transaction>, Error> {
        use crate::schema::transactions::dsl::*;

        transactions.load::<Transaction>(conn)
    }
}
