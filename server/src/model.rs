use std::cell::LazyCell;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Invoice {
    pub number: String,
    pub date: Date,
    pub items: Vec<Item>,
    pub author: Person,
    pub recipient: Address,
    pub bank_account: BankAccount,
    pub vat: f64,
    pub kleinunternehmer: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Date {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    pub description: String,
    pub price: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
    pub street: String,
    pub zip: String,
    pub city: String,
    pub tax_nr: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub name: String,
    pub street: String,
    pub zip: String,
    pub city: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BankAccount {
    pub name: String,
    pub bank: String,
    pub iban: String,
    pub bic: String,
    pub gender: Gender,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gender {
    pub account_holder: String,
}

pub const EXAMPLE_DATA: LazyCell<Invoice> = LazyCell::new(|| Invoice {
    number: "2026-01".into(),
    date: Date {
        year: 2026,
        month: 7,
        day: 2,
    },
    items: vec![Item {
        description: "first item".into(),
        price: 30.0,
    }],
    author: Person {
        name: "Christina Musterfrau".into(),
        street: "Rostweg 1".into(),
        zip: "45678".into(),
        city: "Rust".into(),
        tax_nr: "".into(),
    },
    recipient: Address {
        name: "Rolf Mustermann".into(),
        street: "Distelweg 2".into(),
        zip: "67890".into(),
        city: "Eisenhausen".into(),
    },
    bank_account: BankAccount {
        name: "Christina Musterfrau".into(),
        bank: "Bank of Crabs".into(),
        iban: "DE89 3704 0044 0532 0130 00".into(),
        bic: "FRSPDE66".into(),
        gender: Gender {
            account_holder: "Kontoinhaberin".into(),
        },
    },
    vat: 0.19,
    kleinunternehmer: false,
});
