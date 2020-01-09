table! {
    transactions (id) {
        id -> Int4,
        amount -> Money,
        category -> Text,
        date -> Timestamptz,
        note -> Nullable<Text>,
        shop_name -> Nullable<Text>,
    }
}
