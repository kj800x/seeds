use std::collections::HashMap;
use std::sync::LazyLock;

/// Maximum viable years by species/subcategory name (lowercase).
/// Sources: High Mowing Seeds viability chart, Finch+Folly seed viability guide.
static VIABILITY_TABLE: LazyLock<HashMap<&str, u8>> = LazyLock::new(|| {
    HashMap::from([
        // Vegetables
        ("artichoke", 5),
        ("arugula", 3),
        ("bean", 3),
        ("beans", 3),
        ("beet", 4),
        ("broccoli", 3),
        ("brussels sprouts", 4),
        ("cabbage", 4),
        ("carrot", 3),
        ("cauliflower", 4),
        ("celery", 5),
        ("celeriac", 5),
        ("chard", 4),
        ("swiss chard", 4),
        ("collards", 5),
        ("corn", 2),
        ("sweet corn", 2),
        ("cucumber", 5),
        ("eggplant", 4),
        ("endive", 5),
        ("fennel", 4),
        ("kale", 4),
        ("kohlrabi", 4),
        ("leek", 1),
        ("lettuce", 5),
        ("melon", 5),
        ("mustard", 4),
        ("okra", 2),
        ("onion", 1),
        ("parsnip", 1),
        ("pea", 3),
        ("peas", 3),
        ("pepper", 2),
        ("peppers", 2),
        ("hot pepper", 2),
        ("sweet pepper", 2),
        ("pumpkin", 4),
        ("radish", 5),
        ("rutabaga", 4),
        ("spinach", 2),
        ("squash", 4),
        ("summer squash", 4),
        ("winter squash", 4),
        ("tomato", 4),
        ("turnip", 5),
        ("watermelon", 4),
        // Herbs
        ("basil", 5),
        ("cilantro", 5),
        ("dill", 3),
        ("parsley", 1),
        ("oregano", 2),
        ("thyme", 3),
        ("sage", 3),
        ("chives", 1),
        ("lavender", 5),
        ("chamomile", 3),
        ("catnip", 5),
        ("savory", 3),
        // Flowers
        ("zinnia", 5),
        ("marigold", 2),
        ("sunflower", 4),
        ("cosmos", 3),
        ("nasturtium", 5),
        ("aster", 1),
        ("snapdragon", 3),
        ("petunia", 3),
        ("pansy", 2),
        ("poppy", 4),
        ("dahlia", 2),
        ("dianthus", 4),
        ("hollyhock", 3),
        ("sweet pea", 3),
        ("lobelia", 3),
        ("alyssum", 4),
        ("celosia", 4),
        ("columbine", 2),
        ("foxglove", 2),
        ("lupine", 2),
        ("larkspur", 1),
        ("impatiens", 2),
        ("geranium", 1),
        ("verbena", 1),
        ("nicotiana", 3),
        ("salvia", 1),
        ("delphinium", 1),
        ("calendula", 5),
        ("bachelor's button", 3),
    ])
});

pub const DEFAULT_MAX_YEARS: u8 = 2;

/// Look up maximum viable years for a seed species.
///
/// Tries subcategory first (more specific), then category as fallback.
/// Normalizes input to lowercase and strips "SubCat - " prefix commonly
/// found in Botanical Interests product tags.
pub fn lookup_max_years(subcategory: Option<&str>, category: Option<&str>) -> u8 {
    // Try subcategory first (more specific)
    if let Some(sub) = subcategory {
        let key = normalize_species(sub);
        if let Some(&years) = VIABILITY_TABLE.get(key.as_str()) {
            return years;
        }
    }
    // Fall back to category
    if let Some(cat) = category {
        let key = normalize_species(cat);
        if let Some(&years) = VIABILITY_TABLE.get(key.as_str()) {
            return years;
        }
    }
    DEFAULT_MAX_YEARS
}

/// Normalize a species string: lowercase, strip "SubCat - " prefix.
fn normalize_species(s: &str) -> String {
    let lower = s.to_lowercase();
    // Strip "subcat - " prefix (Botanical Interests tag format)
    if let Some(stripped) = lower.strip_prefix("subcat - ") {
        stripped.to_string()
    } else {
        lower
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_known_species() {
        assert_eq!(lookup_max_years(Some("tomato"), None), 4);
        assert_eq!(lookup_max_years(Some("Tomato"), None), 4);
        assert_eq!(lookup_max_years(Some("onion"), None), 1);
        assert_eq!(lookup_max_years(Some("basil"), None), 5);
    }

    #[test]
    fn lookup_unknown_returns_default() {
        assert_eq!(lookup_max_years(Some("Unknown Thing"), None), DEFAULT_MAX_YEARS);
    }

    #[test]
    fn lookup_category_fallback() {
        // No subcategory match, falls back to category
        assert_eq!(lookup_max_years(Some("Unknown"), Some("Tomato")), 4);
    }

    #[test]
    fn lookup_none_returns_default() {
        assert_eq!(lookup_max_years(None, None), DEFAULT_MAX_YEARS);
    }

    #[test]
    fn lookup_strips_subcat_prefix() {
        assert_eq!(lookup_max_years(Some("SubCat - Tomato"), None), 4);
        assert_eq!(lookup_max_years(Some("SubCat - Basil"), None), 5);
    }

    #[test]
    fn lookup_category_with_prefix() {
        assert_eq!(lookup_max_years(None, Some("SubCat - Pepper")), 2);
    }

    #[test]
    fn table_has_enough_entries() {
        // Ensure 70+ species are in the table
        assert!(VIABILITY_TABLE.len() >= 70);
    }
}
