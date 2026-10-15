use chromiumoxide::Browser;
use chrono::NaiveDate;
use scraper::{Html, Selector};
use serde::Serialize;

use crate::error::{AppError, Result};

const NAFDAC_URL: &str = "https://registration.nafdac.gov.ng/";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NafdacResponse {
    pub reg_no: String,
    pub found: bool,
    pub product: Option<ProductDetails>,
}

#[derive(Debug, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProductDetails {
    name: String,
    source: String,
    category: String,
    manufacturer: String,
    nafdac_no: String,
    nafdac_expiration_date: NaiveDate,
    nafdac_approval_date: NaiveDate,
    active_ingredients: String,
}

pub async fn verify_product(browser: &Browser, reg_no: &str) -> Result<NafdacResponse> {
    let page = browser.new_page(NAFDAC_URL).await?;

    let input_selector =
        "input[type='text'], input#CertificateNumber, input[name='CertificateNumber']";
    page.find_element(input_selector)
        .await?
        .click()
        .await?
        .type_str(reg_no)
        .await?;

    let button_selector = "button[data-action='submit']";
    page.find_element(button_selector).await?.click().await?;

    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    let html = page.wait_for_navigation().await?.content().await?;

    let _ = page.close().await;

    parse_result(&html, reg_no).await
}

async fn parse_result(html: &str, reg_no: &str) -> Result<NafdacResponse> {
    let document = Html::parse_document(html);

    let body_text = document
        .select(&Selector::parse("body").unwrap())
        .next()
        .map(|el| el.text().collect::<String>().to_lowercase())
        .unwrap_or_default();

    if body_text.contains("product not found") {
        return Err(AppError::InternalError("product not found".to_string()));
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
