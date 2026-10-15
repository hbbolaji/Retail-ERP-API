use std::sync::Arc;

use axum::{Router, routing::post};

use crate::{AppState, handlers::nafdac_lookup};

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new().nest("/api/v1", nafdac_lookup_route(state))
}

fn nafdac_lookup_route(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/nafdac_lookup", post(nafdac_lookup))
        .with_state(state)
}
