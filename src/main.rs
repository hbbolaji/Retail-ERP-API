use chromiumoxide::{Browser, BrowserConfig};
use futures::StreamExt;
use sqlx::{PgConnection, PgPool, postgres::PgPoolOptions};
use std::{env, sync::Arc};
use tokio::{net::TcpListener, sync::Mutex};

use crate::{error::Result, routes::build_router};

mod error;
mod handlers;
mod routes;
mod scrapping;

#[derive(Debug)]
struct AppState {
    browser: Mutex<Browser>,
    handler_task: tokio::task::JoinHandle<()>,
    db: PgPool,
}

impl Drop for AppState {
    fn drop(&mut self) {
        self.handler_task.abort();
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().unwrap();
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .no_sandbox()
            .arg("--profile.managed_default_content_settings.images=2")
            .build()
            .unwrap(),
    )
    .await
    .unwrap();

    let handler_task = tokio::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = PgPoolOptions::new()
        .min_connections(3)
        .max_connections(10)
        .connect(&db_url)
        .await
        .expect("database connection failed");

    let state = Arc::new(AppState {
        browser: Mutex::new(browser),
        handler_task,
        db,
    });

    let app = build_router(state);

    println!("Listening on Port 3000");

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to get port");

    axum::serve(listener, app).await.expect("failed to serve");
    Ok(())
}
