use anyhow::{Context, anyhow};
use axum::Router;
use axum::extract::{FromRequestParts, Json, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use std::ops::Deref;
use std::sync::Arc;
use std::{iter, time};
use tokio::net::TcpListener;
use typst::foundations::Bytes;
use typst::layout::PagedDocument;
use typst::syntax::VirtualPath;
use typst_pdf::{PdfOptions, pdf};

use crate::typstadapt::{World, WorldWithDataOverlay};

mod model;
mod typstadapt;

const INVOICE_TEMPLATE: &str = include_str!("template.typ");

pub struct App {
    world: World,
}

#[derive(Clone, FromRequestParts)]
#[from_request(via(State))]
pub struct AppState(pub Arc<App>);

// deref so you can still access the inner fields easily
impl Deref for AppState {
    type Target = App;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let typst_world = typstadapt::World::init(INVOICE_TEMPLATE)?;

    eprintln!("starting API server");
    let router = Router::new()
        .route("/", post(generate_pdf))
        .with_state(AppState(Arc::new(App { world: typst_world })));
    let listener = TcpListener::bind("0.0.0.0:8080")
        .await
        .context("bind port for api server")?;
    axum::serve(listener, router)
        .await
        .context("run api server")?;

    Ok(())
}

struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn generate_pdf(
    state: AppState,
    Json(invoice): Json<model::Invoice>,
) -> Result<Response, AppError> {
    let start = time::Instant::now();

    let payload = serde_json::to_vec(&invoice)?;

    // typst compile & export are not async, run them on a special thread
    let out = tokio::task::spawn_blocking(move || {
        let data_world = WorldWithDataOverlay::new(
            &state.world,
            [(VirtualPath::new("inputs"), Bytes::new(payload))],
        );

        // todo: don't swallow warnings
        let doc = typst::compile::<PagedDocument>(&data_world)
            .output
            .map_err(|diags| anyhow!("failed to compile: {diags:?}"))?;
        eprintln!(
            "compile took {}ms, exporting to pdf",
            start.elapsed().as_millis()
        );

        pdf(&doc, &PdfOptions::default())
            .map_err(|diags| anyhow!("failed to export pdf: {diags:?}"))
    })
    .await??;

    let response = Response::builder()
        .status(200)
        .header("Content-Type", "application/pdf")
        .body(out.into())
        .expect("build http response");

    eprintln!("Responded in {}ms", start.elapsed().as_millis());

    Ok(response)
}
