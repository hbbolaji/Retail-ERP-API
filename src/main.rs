use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::post,
};
use chromiumoxide::{Browser, BrowserConfig};
use chrono::NaiveDate;
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
#[serde(rename_all = "camelCase")]
struct NafdacResponse {
    reg_no: String,
    found: bool,
    product: Option<ProductDetails>,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ProductDetails {
    name: String,
    source: String,
    category: String,
    manufacturer: String,
    nafdac_no: String,
    nafdac_expiration_date: NaiveDate,
    nafdac_approval_date: NaiveDate,
    active_ingredients: String,
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
async fn verify_product(browser: &Browser, reg_no: &str) -> Result<NafdacResponse, String> {
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

async fn parse_result(html: &str, reg_no: &str) -> Result<NafdacResponse, String> {
    let document = Html::parse_document(html);

    let body_text = document
        .select(&Selector::parse("body").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().to_lowercase())
        .unwrap_or_default();

    if body_text.contains("product not found") {
        return Err("Product not found".to_string());
    }

    let mut details = ProductDetails::default();
    let row_selector = Selector::parse("table tr").unwrap();
    for row in document.select(&row_selector) {
        let cells = row
            .select(&Selector::parse("td").unwrap())
            .map(|el| el.text().collect::<String>())
            .collect::<String>();

        let key_value = cells.splitn(2, ":").collect::<Vec<&str>>();
        if key_value.len() >= 2 {
            let label = key_value[0].to_lowercase();
            let value = key_value[1];

            if label.contains("product name") {
                details.name = value.trim().to_string()
            }

            if label.contains("source") {
                details.source = value.trim().to_string()
            }

            if label.contains("manufacturer") {
                details.manufacturer = value.trim().to_string()
            }

            if label.contains("category") {
                details.category = value.trim().to_string()
            }

            if label.contains("nafdac no") {
                details.nafdac_no = value.trim().to_string()
            }

            if label.contains("expiry date") {
                details.nafdac_expiration_date =
                    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").expect("msg")
            }

            if label.contains("date approved") {
                details.nafdac_approval_date =
                    NaiveDate::parse_from_str(value.trim(), "%Y-%m-%d").expect("msg")
            }

            if label.contains("active ingredient") {
                details.active_ingredients = value.trim().to_string()
            }
        }
    }

    let response = NafdacResponse {
        reg_no: reg_no.to_string(),
        found: true,
        product: Some(details),
    };

    Ok(response)
}

// handlers
async fn nafdac_lookup(
    State(state): State<Arc<AppState>>,
    Query(query): Query<NafdacQuery>,
) -> impl IntoResponse {
    let reg_no = query.reg_no;
    if reg_no.is_empty() {
        return Json(NafdacResponse {
            reg_no: reg_no.clone(),
            found: false,
            product: None,
        });
    }
    let browser = state.browser.lock().await;
    Json(verify_product(&browser, &reg_no).await.unwrap())
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
