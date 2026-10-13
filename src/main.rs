use axum::{
    Json, Router,
    extract::{Query, State},
    response::IntoResponse,
    routing::post,
};
use chromiumoxide::{Browser, BrowserConfig};

use futures::StreamExt;

use serde::Deserialize;
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

use crate::error::Result;

use scrapping::{NafdacResponse, verify_product};

mod error;
mod scrapping;

#[derive(Debug, Deserialize)]
struct NafdacQuery {
    reg_no: String,
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
async fn main() -> Result<()> {
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
