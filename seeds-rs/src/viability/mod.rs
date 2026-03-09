pub mod lookup;

use chrono::Datelike;
use lookup::lookup_max_years;

/// Estimated seed viability based on species and age.
#[derive(Debug, Clone)]
pub struct ViabilityEstimate {
    /// Viability percentage (0-100)
    pub percentage: u8,
    /// Maximum viable years for this species
    pub max_years: u8,
    /// Age of the seeds in years
    pub age_years: u8,
    /// The species key that was matched in the lookup table
    pub species_key: String,
}

/// Estimate seed viability based on species, category, and purchase year.
///
/// Returns None if purchase_year is None or if both subcategory and category are None
/// (cannot determine species).
///
/// Uses a linear decline model: 100% at age 0, 0% at max_years.
pub fn estimate_viability(
    subcategory: Option<&str>,
    category: Option<&str>,
    purchase_year: Option<i64>,
) -> Option<ViabilityEstimate> {
    let purchase_year = purchase_year?;

    // Need at least one of subcategory or category to determine species
    if subcategory.is_none() && category.is_none() {
        return None;
    }

    let current_year = chrono::Local::now().year() as i64;
    let age = (current_year - purchase_year).max(0) as u8;
    let max_years = lookup_max_years(subcategory, category);

    // Determine the species key used for display
    let species_key = subcategory
        .or(category)
        .unwrap_or("unknown")
        .to_string();

    let percentage = if age == 0 {
        100
    } else if age >= max_years {
        0
    } else {
        ((max_years - age) as f32 / max_years as f32 * 100.0) as u8
    };

    Some(ViabilityEstimate {
        percentage,
        max_years,
        age_years: age,
        species_key,
    })
}

/// Internal helper for testing with a specific current year.
fn estimate_viability_with_year(
    subcategory: Option<&str>,
    category: Option<&str>,
    purchase_year: Option<i64>,
    current_year: i64,
) -> Option<ViabilityEstimate> {
    let purchase_year = purchase_year?;

    if subcategory.is_none() && category.is_none() {
        return None;
    }

    let age = (current_year - purchase_year).max(0) as u8;
    let max_years = lookup_max_years(subcategory, category);

    let species_key = subcategory
        .or(category)
        .unwrap_or("unknown")
        .to_string();

    let percentage = if age == 0 {
        100
    } else if age >= max_years {
        0
    } else {
        ((max_years - age) as f32 / max_years as f32 * 100.0) as u8
    };

    Some(ViabilityEstimate {
        percentage,
        max_years,
        age_years: age,
        species_key,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tomato_age_2_about_50_percent() {
        // Tomato max_years = 4, age = 2 -> (4-2)/4 = 50%
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2024), 2026).unwrap();
        assert_eq!(est.percentage, 50);
        assert_eq!(est.max_years, 4);
        assert_eq!(est.age_years, 2);
    }

    #[test]
    fn tomato_age_0_is_100_percent() {
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2026), 2026).unwrap();
        assert_eq!(est.percentage, 100);
        assert_eq!(est.age_years, 0);
    }

    #[test]
    fn tomato_exceeds_max_is_0_percent() {
        // Tomato max 4 years, age = 6
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2020), 2026).unwrap();
        assert_eq!(est.percentage, 0);
    }

    #[test]
    fn no_species_returns_none() {
        let est = estimate_viability_with_year(None, None, Some(2024), 2026);
        assert!(est.is_none());
    }

    #[test]
    fn no_purchase_year_returns_none() {
        let est = estimate_viability_with_year(Some("Tomato"), None, None, 2026);
        assert!(est.is_none());
    }

    #[test]
    fn category_fallback_works() {
        // subcategory unknown, category = "Basil" (max 5)
        let est =
            estimate_viability_with_year(Some("Unknown"), Some("Basil"), Some(2024), 2026).unwrap();
        assert_eq!(est.max_years, 5);
        // age 2, max 5 -> (5-2)/5 = 60%
        assert_eq!(est.percentage, 60);
    }

    #[test]
    fn subcat_prefix_normalization() {
        let est =
            estimate_viability_with_year(Some("SubCat - Tomato"), None, Some(2025), 2026).unwrap();
        assert_eq!(est.max_years, 4);
        assert_eq!(est.age_years, 1);
        // (4-1)/4 = 75%
        assert_eq!(est.percentage, 75);
    }

    #[test]
    fn onion_short_viability() {
        // Onion max 1 year, age 1 -> 0%
        let est = estimate_viability_with_year(Some("Onion"), None, Some(2025), 2026).unwrap();
        assert_eq!(est.max_years, 1);
        assert_eq!(est.percentage, 0);
    }

    // --- color_tier tests ---

    #[test]
    fn color_tier_100_is_green() {
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2026), 2026).unwrap();
        assert_eq!(est.percentage, 100);
        assert_eq!(est.color_tier(), "viability-green");
    }

    #[test]
    fn color_tier_75_is_green() {
        // Tomato max 4, age 1 -> 75%
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2025), 2026).unwrap();
        assert_eq!(est.percentage, 75);
        assert_eq!(est.color_tier(), "viability-green");
    }

    #[test]
    fn color_tier_74_is_yellow() {
        // Lettuce max 5, age 2 -> 60% (yellow range)
        let est = estimate_viability_with_year(Some("Lettuce"), None, Some(2024), 2026).unwrap();
        assert!(est.percentage >= 50 && est.percentage <= 74);
        assert_eq!(est.color_tier(), "viability-yellow");
    }

    #[test]
    fn color_tier_50_is_yellow() {
        // Tomato max 4, age 2 -> 50%
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2024), 2026).unwrap();
        assert_eq!(est.percentage, 50);
        assert_eq!(est.color_tier(), "viability-yellow");
    }

    #[test]
    fn color_tier_49_is_orange() {
        // Basil max 5, age 3 -> 40% (orange range)
        let est = estimate_viability_with_year(Some("Basil"), None, Some(2023), 2026).unwrap();
        assert!(est.percentage >= 25 && est.percentage <= 49);
        assert_eq!(est.color_tier(), "viability-orange");
    }

    #[test]
    fn color_tier_25_is_orange() {
        // Tomato max 4, age 3 -> 25%
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2023), 2026).unwrap();
        assert_eq!(est.percentage, 25);
        assert_eq!(est.color_tier(), "viability-orange");
    }

    #[test]
    fn color_tier_24_is_red() {
        // Corn max 5, age 4 -> 20% (red range)
        let est = estimate_viability_with_year(Some("Corn"), None, Some(2022), 2026).unwrap();
        assert!(est.percentage <= 24);
        assert_eq!(est.color_tier(), "viability-red");
    }

    #[test]
    fn color_tier_0_is_red() {
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2020), 2026).unwrap();
        assert_eq!(est.percentage, 0);
        assert_eq!(est.color_tier(), "viability-red");
    }

    // --- warning_message tests ---

    #[test]
    fn warning_message_at_0_percent() {
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2020), 2026).unwrap();
        assert_eq!(est.percentage, 0);
        let msg = est.warning_message().unwrap();
        assert!(msg.contains("exceeded"), "Expected 'exceeded' in: {}", msg);
        assert!(msg.contains("replacing") || msg.contains("replace"), "Expected 'replace' in: {}", msg);
    }

    #[test]
    fn warning_message_last_year() {
        // Tomato max 4, age 3 -> 25%, age_years+1 >= max_years (3+1=4 >= 4)
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2023), 2026).unwrap();
        assert!(est.percentage > 0);
        assert!(est.age_years + 1 >= est.max_years);
        let msg = est.warning_message().unwrap();
        assert!(msg.contains("last year") || msg.contains("Last year"), "Expected 'last year' in: {}", msg);
    }

    #[test]
    fn warning_message_healthy_seed() {
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2026), 2026).unwrap();
        assert_eq!(est.percentage, 100);
        assert!(est.warning_message().is_none());
    }

    // --- sow_multiplier tests ---

    #[test]
    fn sow_multiplier_high_viability_none() {
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2026), 2026).unwrap();
        assert!(est.percentage >= 90);
        assert!(est.sow_multiplier().is_none());
    }

    #[test]
    fn sow_multiplier_zero_viability_none() {
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2020), 2026).unwrap();
        assert_eq!(est.percentage, 0);
        assert!(est.sow_multiplier().is_none());
    }

    #[test]
    fn sow_multiplier_50_percent_is_2x() {
        let est = estimate_viability_with_year(Some("Tomato"), None, Some(2024), 2026).unwrap();
        assert_eq!(est.percentage, 50);
        let mult = est.sow_multiplier().unwrap();
        assert!((mult - 2.0).abs() < 0.01, "Expected ~2.0, got {}", mult);
    }
}
