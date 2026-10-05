use axum::{
    Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::post,
};
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;

#[derive(Debug, Deserialize)]
struct NafdacQuery {
    reg_no: String,
}

#[derive(Debug, Serialize)]
struct NafdacResponse {
    reg_no: String,
    found: bool,
    product: Option<ProductDetails>,
}

#[derive(Debug, Serialize)]
struct ProductDetails {
    name: String,
    category: String,
    manufacturer: String,
}

struct AppState {
    browser: Mutex<Browser>,
    handler_task: tokio::task::JoinHandle<()>,
}

const NAFDAC_URL: &str = "https://registration.nafdac.gov.ng/";

// scrapping
async fn verify_product(browser: &Browser, reg_no: &str) -> Result<(), String> {
    Ok(())
}

fn parse_result(html: &str, reg_no: &str) -> Result<(), String> {
    Ok(())
}

// handlers
async fn nafdac_lookup(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NafdacQuery>,
) -> impl IntoResponse {
    println!("{:?}", query);
    "success"
}

#[tokio::main]
async fn main() {
    let (browser, mut handler) =
        Browser::launch(BrowserConfig::builder().no_sandbox().build().unwrap())
            .await
            .unwrap();

    let handler_task = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    let state = Arc::new(AppState {
        browser: Mutex::new(browser),
        handler_task,
    });

    let app = Router::new()
        .route("/nafdac_lookup", post(nafdac_lookup))
        .with_state(state);

    println!("Listening on Port 3000");

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to get port");

    axum::serve(listener, app).await.expect("failed to serve")
}
