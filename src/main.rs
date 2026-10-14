use axum::{Router, routing::post};
use chromiumoxide::{Browser, BrowserConfig};

use futures::StreamExt;

use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};

use crate::{error::Result, handlers::nafdac_lookup, routes::build_router};

mod error;
mod handlers;
mod routes;
mod scrapping;

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

    let app = build_router(state);

    println!("Listening on Port 3000");

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to get port");

    axum::serve(listener, app).await.expect("failed to serve");
    Ok(())
}
