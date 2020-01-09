-- Your SQL goes here

CREATE TABLE transactions (
    id serial PRIMARY KEY,
    amount money NOT NULL,
    category text NOT NULL,
    date timestamptz NOT NULL DEFAULT now(),
    note text DEFAULT '',
    shop_name text DEFAULT ''
);