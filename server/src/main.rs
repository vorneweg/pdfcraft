use anyhow::Context;
use axum::Router;
use axum::extract::Json;
use axum::http::StatusCode;
use axum::routing::post;
use serde::{Deserialize, Serialize};
use std::ffi::OsString;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Router::new().route("/", post(generate_pdf));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .context("bind port for api server")?;
    axum::serve(listener, router)
        .await
        .context("run api server")?;

    Ok(())
}

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

async fn generate_pdf(
    Json(invoice): Json<Invoice>,
) -> Result<axum::response::Response, axum::response::ErrorResponse> {
    println!("{invoice:#?}");
    let mut json_file = tempfile::NamedTempFile::new_in("./")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let path = json_file
        .path()
        .strip_prefix(std::env::current_dir().expect("get working dir"))
        .expect("strip prefix")
        .to_owned();

    serde_json::to_writer(json_file.as_file_mut(), &invoice)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut path_arg = OsString::from("json=");
    path_arg.push(path);
    let output = tokio::process::Command::new("typst")
        .arg("compile")
        .arg("invoice.typ")
        .arg("--input")
        .arg(path_arg)
        .arg("-")
        .output()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if !output.status.success() {
        eprintln!(
            "typst failed: {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        Err((StatusCode::INTERNAL_SERVER_ERROR, "typst failed"))?;
    }

    let response = axum::response::Response::builder()
        .status(200)
        .header("Content-Type", "application/pdf")
        .body(output.stdout.into())
        .expect("build http response");
    Ok(response)
}
