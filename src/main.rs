use axum::{
    Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
};
use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

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

#[derive(Debug)]
struct AppState {
    browser: Mutex<Browser>,
    handler_task: tokio::task::JoinHandle<()>,
}

impl Drop for AppState {
    fn drop(&mut self) {
        self.handler_task.abort();
    }
}

const NAFDAC_URL: &str = "https://registration.nafdac.gov.ng/";

// scrapping
async fn verify_product(browser: &Browser, reg_no: &str) -> Result<(), String> {
    let page = browser
        .new_page(NAFDAC_URL)
        .await
        .map_err(|e| format!("failed to open page: {}", e))?;

    // input box
    let input_selector =
        "input[type='text'], input#CertificateNumber, input[name='CertificateNumber']";
    page.find_element(input_selector)
        .await
        .map_err(|e| format!("input box not found: {}", e))?
        .click()
        .await
        .map_err(|e| format!("click input: {}", e))?
        .type_str(reg_no)
        .await
        .map_err(|e| format!("type reg no: {}", e))?;

    // button submission
    let button_selector = "button[data-action='submit']";
    page.find_element(button_selector)
        .await
        .map_err(|e| format!("verify button not found: {}", e))?
        .click()
        .await
        .map_err(|e| format!("click button: {}", e))?;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let html = page
        .wait_for_navigation()
        .await
        .unwrap()
        .content()
        .await
        .unwrap();

    let _ = page.close().await;

    parse_result(&html, reg_no).await
}

async fn parse_result(html: &str, reg_no: &str) -> Result<(), String> {
    let document = Html::parse_document(html);
    println!("{:?}", document);
    // let body_text = document.select(&Selector::parse("body").unwrap()).next().map(|el| el.text().collect()::<String>()).unwrap_or_default();
    Ok(())
}

// handlers
async fn nafdac_lookup(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NafdacQuery>,
) -> impl IntoResponse {
    let reg_no = query.reg_no;
    if reg_no.is_empty() {
        return StatusCode::NOT_FOUND;
    }
    let browser = state.browser.lock().await;
    verify_product(&browser, &reg_no).await.unwrap();

    StatusCode::OK
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    axum::serve(listener, app).await.expect("failed to serve");
    Ok(())
}
