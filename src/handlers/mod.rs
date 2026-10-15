use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    AppState,
    scrapping::{NafdacResponse, verify_product},
};

#[derive(Debug, Deserialize)]
pub struct NafdacQuery {
    reg_no: String,
}

pub async fn nafdac_lookup(
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
