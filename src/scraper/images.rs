use std::path::Path;

use crate::scraper::fetcher::ShopifyImage;

/// Represents a successfully downloaded image.
#[derive(Debug)]
pub struct DownloadedImage {
    pub shopify_image_id: i64,
    pub position: i32,
    pub filename: String,
    pub original_url: String,
    pub width: Option<i32>,
    pub height: Option<i32>,
}

/// Download all product images to the local filesystem.
///
/// Creates the directory `data_dir/images/{seed_id}/` and downloads each image.
/// Partial success is acceptable -- failed downloads are logged as warnings
/// but do not prevent the rest from completing (Pitfall 4).
pub async fn download_images(
    client: &reqwest::Client,
    images: &[ShopifyImage],
    seed_id: i64,
    data_dir: &Path,
) -> Vec<DownloadedImage> {
    let dir = data_dir.join("images").join(seed_id.to_string());

    if let Err(e) = tokio::fs::create_dir_all(&dir).await {
        tracing::error!("Failed to create image directory {:?}: {}", dir, e);
        return Vec::new();
    }

    let mut downloaded = Vec::new();

    for image in images {
        let extension = detect_extension_from_url(&image.src);
        let filename = format!("{}{}", image.position, extension);

        match client.get(&image.src).send().await {
            Ok(resp) => {
                if !resp.status().is_success() {
                    tracing::warn!(
                        "Image download returned status {} for {}",
                        resp.status(),
                        image.src
                    );
                    continue;
                }

                // Check content-type header for extension override
                let content_ext = resp
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|v| v.to_str().ok())
                    .and_then(extension_from_content_type);

                let final_filename = if let Some(ext) = content_ext {
                    format!("{}{}", image.position, ext)
                } else {
                    filename.clone()
                };

                let final_path = dir.join(&final_filename);

                match resp.bytes().await {
                    Ok(bytes) => {
                        if let Err(e) = tokio::fs::write(&final_path, &bytes).await {
                            tracing::warn!(
                                "Failed to write image file {:?}: {}",
                                final_path,
                                e
                            );
                            continue;
                        }

                        downloaded.push(DownloadedImage {
                            shopify_image_id: image.id,
                            position: image.position,
                            filename: final_filename,
                            original_url: image.src.clone(),
                            width: image.width,
                            height: image.height,
                        });
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to read image bytes from {}: {}",
                            image.src,
                            e
                        );
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to download image {}: {}", image.src, e);
                // Continue -- don't fail the whole scrape for one image
            }
        }
    }

    tracing::info!(
        "Downloaded {}/{} images for seed {}",
        downloaded.len(),
        images.len(),
        seed_id
    );

    downloaded
}

/// Detect file extension from the URL path.
fn detect_extension_from_url(url: &str) -> &str {
    let path = url.split('?').next().unwrap_or(url);
    if path.ends_with(".png") {
        ".png"
    } else if path.ends_with(".webp") {
        ".webp"
    } else if path.ends_with(".gif") {
        ".gif"
    } else {
        ".jpg"
    }
}

/// Map content-type header to file extension.
fn extension_from_content_type(ct: &str) -> Option<&'static str> {
    if ct.contains("image/png") {
        Some(".png")
    } else if ct.contains("image/webp") {
        Some(".webp")
    } else if ct.contains("image/gif") {
        Some(".gif")
    } else if ct.contains("image/jpeg") || ct.contains("image/jpg") {
        Some(".jpg")
    } else {
        None
    }
}
