use scraper::{Element, Html, Selector};

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

/// All growing/planting details extracted from the HTML product page.
#[derive(Debug, Default)]
pub struct GrowingDetails {
    pub days_to_maturity: Option<String>,
    pub sow_depth: Option<String>,
    pub plant_spacing: Option<String>,
    pub germination_info: Option<String>,
    pub planting_instructions: Option<String>,
    pub growing_instructions: Option<String>,
    pub harvest_instructions: Option<String>,
    // New expanded fields
    pub plant_type: Option<String>,
    pub botanical_name: Option<String>,
    pub family: Option<String>,
    pub native_region: Option<String>,
    pub hardiness: Option<String>,
    pub exposure: Option<String>,
    pub bloom_period: Option<String>,
    pub plant_dimensions: Option<String>,
    pub variety_info: Option<String>,
    pub attributes: Option<String>,
    pub when_to_sow_outside: Option<String>,
    pub when_to_start_inside: Option<String>,
    pub days_to_emerge: Option<String>,
    pub row_spacing: Option<String>,
    pub thinning: Option<String>,
    pub special_care: Option<String>,
}

/// Parse growing details from a Botanical Interests HTML product page.
///
/// Botanical Interests consistently structures their product details as:
///   `<p><b>Label:</b> Value text</p>`
/// within the product description (body_html). We also extract the botanical
/// name from `<em>` tags in the description.
///
/// The raw HTML is stored in the database so parsing can be refined later.
pub fn parse_growing_details(html: &str) -> GrowingDetails {
    let document = Html::parse_document(html);
    let mut details = GrowingDetails::default();

    // Extract all <p><b>Label:</b> Value</p> pairs from the page.
    let pairs = extract_bold_label_pairs(&document);

    for (label, value) in &pairs {
        let label_lower = label.to_lowercase();
        let value = value.trim().to_string();
        if value.is_empty() {
            continue;
        }

        match label_lower.as_str() {
            "days to maturity" => details.days_to_maturity = Some(value),
            "seed depth" | "sow depth" | "planting depth" => details.sow_depth = Some(value),
            "seed spacing" | "plant spacing" => details.plant_spacing = Some(value),
            "days to emerge" | "days to germinate" | "germination" => {
                details.days_to_emerge = Some(value.clone());
                // Also populate germination_info for backward compat
                details.germination_info = Some(value);
            }
            "type" => details.plant_type = Some(value.replace(" (Learn more)", "")),
            "family" => details.family = Some(value),
            "native" | "native region" => details.native_region = Some(value),
            "hardiness" => details.hardiness = Some(value),
            "exposure" => details.exposure = Some(value),
            "bloom period" | "bloom season" => details.bloom_period = Some(value),
            "plant dimensions" | "plant size" => details.plant_dimensions = Some(value),
            "variety info" => details.variety_info = Some(value),
            "attributes" => details.attributes = Some(value),
            "when to sow outside" | "when to sow outdoors" => {
                details.when_to_sow_outside = Some(value.clone());
                // Include in planting_instructions as well
                if details.planting_instructions.is_none() {
                    details.planting_instructions = Some(format!("Sow Outside: {}", value));
                } else if let Some(ref mut pi) = details.planting_instructions {
                    pi.push_str(&format!(" Sow Outside: {}", value));
                }
            }
            "when to start inside" | "when to start indoors" => {
                details.when_to_start_inside = Some(value.clone());
                if details.planting_instructions.is_none() {
                    details.planting_instructions = Some(format!("Start Inside: {}", value));
                } else if let Some(ref mut pi) = details.planting_instructions {
                    pi.push_str(&format!(" Start Inside: {}", value));
                }
            }
            "row spacing" => details.row_spacing = Some(value),
            "thinning" => details.thinning = Some(value),
            "harvesting" | "harvest" => details.harvest_instructions = Some(value),
            "special care" => details.special_care = Some(value),
            _ => {
                // Capture any growing/cultivation info we haven't explicitly mapped
                if label_lower.contains("growing") || label_lower.contains("cultivation")
                    || label_lower.contains("care")
                {
                    details.growing_instructions = Some(value);
                }
            }
        }
    }

    // Extract botanical name from <em> tags (e.g. <em>Allium schoenoprasum</em>)
    if details.botanical_name.is_none() {
        details.botanical_name = extract_botanical_name(&document);
    }

    if details.is_empty() {
        tracing::warn!("Could not extract growing details from HTML -- selectors may need updating");
    }

    details
}

/// Extract all `<p><b>Label:</b> Value</p>` pairs from the document.
///
/// This is the primary structured data pattern used by Botanical Interests.
/// The bold tag contains the label followed by a colon, and the rest of the
/// paragraph is the value.
///
/// We first try the more specific `div.tab-content p b` selector (matching
/// the known Botanical Interests theme structure), then fall back to scanning
/// all `<p>` tags if that yields no results.
fn extract_bold_label_pairs(document: &Html) -> Vec<(String, String)> {
    // Try the specific tab-content selector first (avoids review text)
    let pairs = extract_bold_pairs_with_b_selector(document, "div.tab-content p b");
    if !pairs.is_empty() {
        return pairs;
    }

    // Fallback: scan all <p><b> pairs in the document
    extract_bold_pairs_with_b_selector(document, "p b")
}

fn extract_bold_pairs_with_b_selector(document: &Html, selector_str: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();

    let b_selector = Selector::parse(selector_str).expect("valid selector");

    for b_elem in document.select(&b_selector) {
        let bold_text: String = b_elem.text().collect::<Vec<_>>().join("");
        // The label should end with ':'
        if let Some(label) = bold_text.strip_suffix(':') {
            let label = label.trim().to_string();
            if label.is_empty() {
                continue;
            }

            // Get the parent <p> element's full text, then strip the bold label prefix
            if let Some(parent) = b_elem.parent_element() {
                let full_text: String = parent.text().collect::<Vec<_>>().join("");
                let value = full_text
                    .replace(&bold_text, "")
                    .trim()
                    .to_string();

                if !value.is_empty() {
                    pairs.push((label, value));
                }
            }
        }
    }

    pairs
}

/// Extract botanical name from <em> tags in the product description.
///
/// Botanical Interests renders the botanical/Latin name in italics, e.g.:
///   `<em>Allium schoenoprasum</em>`
///
/// We look for <em> text that looks like a binomial name (two+ capitalized
/// Latin words, possibly with variety/subspecies).
fn extract_botanical_name(document: &Html) -> Option<String> {
    let em_selector = Selector::parse("em").expect("valid selector");

    for em_elem in document.select(&em_selector) {
        let text: String = em_elem.text().collect::<Vec<_>>().join("");
        let text = text.trim();

        // Simple heuristic: looks like a Latin binomial name if it has 2+ words,
        // starts with a capital letter, and the words are mostly alphabetic.
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() >= 2 {
            let first_char = words[0].chars().next().unwrap_or(' ');
            if first_char.is_uppercase()
                && words.iter().all(|w| {
                    w.chars()
                        .all(|c| c.is_alphabetic() || c == '.' || c == '-' || c == '\'')
                })
            {
                return Some(text.to_string());
            }
        }
    }

    None
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
            && self.plant_type.is_none()
            && self.botanical_name.is_none()
            && self.family.is_none()
            && self.native_region.is_none()
            && self.hardiness.is_none()
            && self.exposure.is_none()
            && self.bloom_period.is_none()
            && self.plant_dimensions.is_none()
            && self.variety_info.is_none()
            && self.attributes.is_none()
            && self.when_to_sow_outside.is_none()
            && self.when_to_start_inside.is_none()
            && self.days_to_emerge.is_none()
            && self.row_spacing.is_none()
            && self.thinning.is_none()
            && self.special_care.is_none()
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
    fn test_parse_bold_label_pairs() {
        let html = r#"<html><body>
            <p><b>Family:</b> Solanaceae</p>
            <p><b>Days to Maturity:</b> 75-80 days from transplanting</p>
            <p><b>Seed Depth:</b> 1/4"</p>
            <p><b>Harvesting:</b> Pick when ripe.</p>
            <p>This is a normal paragraph without bold labels.</p>
        </body></html>"#;

        let details = parse_growing_details(html);
        assert_eq!(details.family, Some("Solanaceae".to_string()));
        assert_eq!(
            details.days_to_maturity,
            Some("75-80 days from transplanting".to_string())
        );
        assert_eq!(details.sow_depth, Some("1/4\"".to_string()));
        assert_eq!(details.harvest_instructions, Some("Pick when ripe.".to_string()));
    }

    #[test]
    fn test_parse_days_to_emerge() {
        let html = r#"<html><body>
            <p><b>Days to Emerge:</b> 10-15 days</p>
        </body></html>"#;

        let details = parse_growing_details(html);
        assert_eq!(details.days_to_emerge, Some("10-15 days".to_string()));
        assert_eq!(details.germination_info, Some("10-15 days".to_string()));
    }

    #[test]
    fn test_parse_botanical_name() {
        let html = r#"<html><body>
            <em>Allium schoenoprasum</em>
            <p><b>Family:</b> Alliaceae</p>
        </body></html>"#;

        let details = parse_growing_details(html);
        assert_eq!(
            details.botanical_name,
            Some("Allium schoenoprasum".to_string())
        );
    }

    #[test]
    fn test_ignores_non_botanical_em() {
        let html = r#"<html><body>
            <em>recommended</em>
            <p><b>Family:</b> Test</p>
        </body></html>"#;

        let details = parse_growing_details(html);
        // Single word "recommended" should not match as botanical name
        assert!(details.botanical_name.is_none());
    }

    #[test]
    fn test_parse_planting_instructions_combined() {
        let html = r#"<html><body>
            <p><b>When to Sow Outside:</b> 4 to 6 weeks before last frost</p>
            <p><b>When to Start Inside:</b> 6 to 8 weeks before last frost</p>
        </body></html>"#;

        let details = parse_growing_details(html);
        assert_eq!(
            details.when_to_sow_outside,
            Some("4 to 6 weeks before last frost".to_string())
        );
        assert_eq!(
            details.when_to_start_inside,
            Some("6 to 8 weeks before last frost".to_string())
        );
        // planting_instructions should combine both
        let pi = details.planting_instructions.unwrap();
        assert!(pi.contains("Sow Outside:"));
        assert!(pi.contains("Start Inside:"));
    }
}
