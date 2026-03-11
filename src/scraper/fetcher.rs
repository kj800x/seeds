use serde::Deserialize;

use crate::error::AppError;

#[derive(Debug, Deserialize)]
pub struct ShopifyProductResponse {
    pub product: ShopifyProduct,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ShopifyProduct {
    pub id: i64,
    pub title: String,
    pub body_html: Option<String>,
    pub handle: String,
    pub tags: String,
    pub images: Vec<ShopifyImage>,
    pub image: Option<ShopifyImage>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ShopifyImage {
    pub id: i64,
    pub src: String,
    pub position: i32,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt: Option<String>,
}

/// Extract the product handle from a Botanical Interests product URL.
///
/// Handles variations like:
/// - https://www.botanicalinterests.com/products/sun-gold-cherry-tomato-seeds
/// - https://www.botanicalinterests.com/products/sun-gold-cherry-tomato-seeds?variant=123
/// - https://botanicalinterests.com/products/sun-gold-cherry-tomato-seeds/
fn extract_handle(url: &str) -> Result<String, AppError> {
    // Find the "/products/" segment
    let products_marker = "/products/";
    let start = url
        .find(products_marker)
        .ok_or_else(|| AppError::ScraperError("URL does not contain /products/ path".into()))?;

    let after_products = &url[start + products_marker.len()..];

    // Strip query params and fragment
    let handle = after_products
        .split('?')
        .next()
        .unwrap_or(after_products)
        .split('#')
        .next()
        .unwrap_or(after_products)
        .trim_end_matches('/');

    if handle.is_empty() {
        return Err(AppError::ScraperError(
            "Could not extract product handle from URL".into(),
        ));
    }

    // Lowercase and strip .json suffix if accidentally included
    let handle = handle.to_lowercase().trim_end_matches(".json").to_string();

    Ok(handle)
}

/// Fetch a Botanical Interests product via both the Shopify JSON API and the HTML page.
///
/// Returns the deserialized product data and the raw HTML of the product page.
pub async fn fetch_product(
    client: &reqwest::Client,
    url: &str,
) -> Result<(ShopifyProduct, String, String), AppError> {
    let handle = extract_handle(url)?;

    // Construct the base URL from the handle
    let base_url = format!(
        "https://www.botanicalinterests.com/products/{}",
        handle
    );
    let json_url = format!("{}.json", base_url);

    // Fetch JSON API endpoint
    let json_resp = client
        .get(&json_url)
        .send()
        .await
        .map_err(|e| AppError::ScraperError(format!("Failed to fetch JSON API: {}", e)))?;

    if json_resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(AppError::ScraperError(
            "Product not found — check the URL and try again".into(),
        ));
    }

    if !json_resp.status().is_success() {
        return Err(AppError::ScraperError(format!(
            "JSON API returned status {}",
            json_resp.status()
        )));
    }

    let product_resp: ShopifyProductResponse = json_resp.json().await.map_err(|e| {
        AppError::ScraperError(format!("Failed to parse JSON response: {}", e))
    })?;

    // Fetch the HTML page for growing details
    let html_resp = client
        .get(&base_url)
        .send()
        .await
        .map_err(|e| AppError::ScraperError(format!("Failed to fetch HTML page: {}", e)))?;

    let raw_html = html_resp.text().await.map_err(|e| {
        AppError::ScraperError(format!("Failed to read HTML response body: {}", e))
    })?;

    Ok((product_resp.product, raw_html, handle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_handle_basic() {
        let url = "https://www.botanicalinterests.com/products/sun-gold-cherry-tomato-seeds";
        assert_eq!(
            extract_handle(url).unwrap(),
            "sun-gold-cherry-tomato-seeds"
        );
    }

    #[test]
    fn test_extract_handle_with_query_params() {
        let url = "https://www.botanicalinterests.com/products/basil-seeds?variant=123";
        assert_eq!(extract_handle(url).unwrap(), "basil-seeds");
    }

    #[test]
    fn test_extract_handle_with_trailing_slash() {
        let url = "https://www.botanicalinterests.com/products/basil-seeds/";
        assert_eq!(extract_handle(url).unwrap(), "basil-seeds");
    }

    #[test]
    fn test_extract_handle_uppercase() {
        let url = "https://www.botanicalinterests.com/products/Basil-Seeds";
        assert_eq!(extract_handle(url).unwrap(), "basil-seeds");
    }

    #[test]
    fn test_extract_handle_invalid_url() {
        let url = "https://www.example.com/not-products/something";
        assert!(extract_handle(url).is_err());
    }
}
