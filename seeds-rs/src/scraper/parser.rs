use scraper::{Html, Selector};

/// Structured data parsed from the Shopify product tags string.
#[derive(Debug, Default)]
pub struct ParsedTags {
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub light_requirement: Option<String>,
    pub frost_tolerance: Option<String>,
    pub is_organic: bool,
    pub is_heirloom: bool,
}

/// Parse the comma-separated tags string from the Shopify JSON API.
///
/// Known tag conventions from Botanical Interests:
/// - "Cat - Vegetables" -> category
/// - "SubCat - Tomato" -> subcategory
/// - "Full Sun", "Full Sun to Part Shade" -> light_requirement
/// - "Frost Tolerant" -> frost_tolerance
/// - "organic" -> is_organic
/// - "heirloom-*" -> is_heirloom
pub fn parse_tags(tags_str: &str) -> ParsedTags {
    let tags: Vec<&str> = tags_str.split(", ").collect();
    let mut result = ParsedTags::default();

    for tag in &tags {
        let tag_trimmed = tag.trim();
        let tag_lower = tag_trimmed.to_lowercase();

        if let Some(cat) = tag_trimmed.strip_prefix("Cat - ") {
            result.category = Some(cat.to_string());
        } else if let Some(sub) = tag_trimmed.strip_prefix("SubCat - ") {
            result.subcategory = Some(sub.to_string());
        } else if tag_lower.contains("sun") || tag_lower.contains("shade") {
            // Light requirement patterns: "Full Sun", "Full Sun to Part Shade",
            // "Part Shade", "Full Shade"
            if tag_lower.contains("sun") || tag_lower.contains("shade") {
                result.light_requirement = Some(tag_trimmed.to_string());
            }
        } else if tag_lower == "frost tolerant" {
            result.frost_tolerance = Some("Frost Tolerant".to_string());
        } else if tag_lower == "organic" {
            result.is_organic = true;
        } else if tag_lower.starts_with("heirloom") {
            result.is_heirloom = true;
        }
    }

    result
}

/// Growing details extracted from the HTML product page.
#[derive(Debug, Default)]
pub struct GrowingDetails {
    pub days_to_maturity: Option<String>,
    pub sow_depth: Option<String>,
    pub plant_spacing: Option<String>,
    pub germination_info: Option<String>,
    pub planting_instructions: Option<String>,
    pub growing_instructions: Option<String>,
    pub harvest_instructions: Option<String>,
}

/// Parse growing details from a Botanical Interests HTML product page.
///
/// This uses a best-effort approach since the exact HTML structure is not fully known
/// (Shopify metafields rendered via Liquid theme). The function attempts several
/// strategies to extract growing information and gracefully returns None for any
/// fields it cannot find.
///
/// The raw HTML is stored in the database (SCRP-05) so parsing can be refined later.
pub fn parse_growing_details(html: &str) -> GrowingDetails {
    let document = Html::parse_document(html);
    let mut details = GrowingDetails::default();

    // Strategy 1: Look for common Shopify metafield patterns.
    // Many Shopify themes render metafields inside elements with class names
    // containing "product-details", "product__description", "metafield", or
    // specific data attributes.

    // Try to find days to maturity from text patterns in the page
    details.days_to_maturity = extract_text_pattern(&document, &[
        "days to maturity",
        "days to harvest",
        "maturity",
    ]);

    // Try to find sow depth
    details.sow_depth = extract_text_pattern(&document, &[
        "sow depth",
        "planting depth",
        "seed depth",
    ]);

    // Try to find plant spacing
    details.plant_spacing = extract_text_pattern(&document, &[
        "plant spacing",
        "spacing",
        "thin to",
    ]);

    // Try to find germination info
    details.germination_info = extract_text_pattern(&document, &[
        "germination",
        "germinate",
    ]);

    // Strategy 2: Look for product description sections that contain planting/growing/harvest info.
    // These are often in div.product-description, div.product__content, or the body_html rendered
    // section of the Shopify theme.
    let section_selectors = [
        "div.product-single__description",
        "div.product__description",
        "div.product-description",
        "div.rte",
        "div[data-product-description]",
        "#product-description",
    ];

    let mut description_text = String::new();
    for sel_str in &section_selectors {
        if let Ok(selector) = Selector::parse(sel_str) {
            for element in document.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                if text.len() > description_text.len() {
                    description_text = text;
                }
            }
        }
    }

    // If we found a description block, try to extract sections from it
    if !description_text.is_empty() {
        details.planting_instructions =
            extract_section_from_text(&description_text, &["planting", "sowing", "when to sow"]);
        details.growing_instructions =
            extract_section_from_text(&description_text, &["growing", "cultivation", "care"]);
        details.harvest_instructions =
            extract_section_from_text(&description_text, &["harvest", "picking", "when to pick"]);
    }

    // Strategy 3: Scan all text nodes in tab panels or accordion sections
    let tab_selectors = [
        "div.tabs__content",
        "div.tab-content",
        "div.accordion__content",
        "div[role='tabpanel']",
        "div.collapsible-content",
    ];

    for sel_str in &tab_selectors {
        if let Ok(selector) = Selector::parse(sel_str) {
            for element in document.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                let text_lower = text.to_lowercase();

                if details.planting_instructions.is_none()
                    && (text_lower.contains("planting") || text_lower.contains("sowing"))
                {
                    details.planting_instructions = Some(clean_text(&text));
                }
                if details.growing_instructions.is_none()
                    && (text_lower.contains("growing") || text_lower.contains("care"))
                {
                    details.growing_instructions = Some(clean_text(&text));
                }
                if details.harvest_instructions.is_none()
                    && text_lower.contains("harvest")
                {
                    details.harvest_instructions = Some(clean_text(&text));
                }
            }
        }
    }

    if details.is_empty() {
        tracing::warn!("Could not extract growing details from HTML -- selectors may need updating");
    }

    details
}

/// Search the parsed HTML document for text containing any of the given keywords,
/// and extract the surrounding context as a value.
fn extract_text_pattern(document: &Html, keywords: &[&str]) -> Option<String> {
    // Look for elements that might contain the target info
    let selectors_to_try = [
        "li",
        "p",
        "td",
        "span",
        "div.metafield",
        "div.product-detail",
    ];

    for sel_str in &selectors_to_try {
        if let Ok(selector) = Selector::parse(sel_str) {
            for element in document.select(&selector) {
                let text = element.text().collect::<Vec<_>>().join(" ");
                let text_lower = text.to_lowercase();

                for keyword in keywords {
                    if text_lower.contains(keyword) {
                        let cleaned = clean_text(&text);
                        if !cleaned.is_empty() && cleaned.len() < 500 {
                            return Some(cleaned);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Extract a section of text from a larger block based on keyword proximity.
fn extract_section_from_text(text: &str, keywords: &[&str]) -> Option<String> {
    let text_lower = text.to_lowercase();

    for keyword in keywords {
        if let Some(pos) = text_lower.find(keyword) {
            // Extract a window around the keyword (up to ~500 chars after it)
            let start = text[..pos]
                .rfind(|c: char| c == '.' || c == '\n')
                .map(|p| p + 1)
                .unwrap_or(pos);

            let end_offset = (pos + 500).min(text.len());
            let end = text[pos..end_offset]
                .rfind(|c: char| c == '.' || c == '\n')
                .map(|p| pos + p + 1)
                .unwrap_or(end_offset);

            let section = clean_text(&text[start..end]);
            if !section.is_empty() {
                return Some(section);
            }
        }
    }

    None
}

/// Clean up extracted text: normalize whitespace, trim.
fn clean_text(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

impl GrowingDetails {
    /// Returns true if no growing details were extracted.
    pub fn is_empty(&self) -> bool {
        self.days_to_maturity.is_none()
            && self.sow_depth.is_none()
            && self.plant_spacing.is_none()
            && self.germination_info.is_none()
            && self.planting_instructions.is_none()
            && self.growing_instructions.is_none()
            && self.harvest_instructions.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tags_category() {
        let tags = "Cat - Vegetables, SubCat - Tomato, Full Sun to Part Shade, organic";
        let parsed = parse_tags(tags);
        assert_eq!(parsed.category, Some("Vegetables".to_string()));
        assert_eq!(parsed.subcategory, Some("Tomato".to_string()));
        assert_eq!(
            parsed.light_requirement,
            Some("Full Sun to Part Shade".to_string())
        );
        assert!(parsed.is_organic);
        assert!(!parsed.is_heirloom);
    }

    #[test]
    fn test_parse_tags_heirloom() {
        let tags = "Cat - Herbs, heirloom-herbs, Full Sun";
        let parsed = parse_tags(tags);
        assert!(parsed.is_heirloom);
        assert_eq!(parsed.category, Some("Herbs".to_string()));
    }

    #[test]
    fn test_parse_tags_frost_tolerant() {
        let tags = "Cat - Vegetables, Frost Tolerant";
        let parsed = parse_tags(tags);
        assert_eq!(
            parsed.frost_tolerance,
            Some("Frost Tolerant".to_string())
        );
    }

    #[test]
    fn test_parse_tags_empty() {
        let parsed = parse_tags("");
        assert!(parsed.category.is_none());
        assert!(!parsed.is_organic);
    }

    #[test]
    fn test_growing_details_is_empty() {
        let details = GrowingDetails::default();
        assert!(details.is_empty());
    }

    #[test]
    fn test_clean_text() {
        assert_eq!(clean_text("  hello   world  "), "hello world");
    }
}
