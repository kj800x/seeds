use chrono::{Duration, NaiveDate};

use crate::db::models::Seed;
use super::parser::PlantingTiming;

/// Halifax MA last frost date: May 10
pub const HALIFAX_MA_LAST_FROST: (u32, u32) = (5, 10);

/// Types of planting actions in the schedule.
#[derive(Debug, Clone, PartialEq)]
pub enum ActionType {
    StartIndoors,
    TransplantOutdoors,
    DirectSow,
}

impl std::fmt::Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionType::StartIndoors => write!(f, "Start Indoors"),
            ActionType::TransplantOutdoors => write!(f, "Transplant Outdoors"),
            ActionType::DirectSow => write!(f, "Direct Sow"),
        }
    }
}

/// A single planting action with a computed calendar date.
#[derive(Debug, Clone)]
pub struct PlantingAction {
    pub seed_id: i64,
    pub seed_title: String,
    pub action_type: ActionType,
    pub date: NaiveDate,
    pub notes: String,
}

/// Generate a schedule of planting actions for seeds with parsed timing data.
///
/// Computes calendar dates relative to Halifax MA last frost date (May 10).
/// For warm-season two-phase crops (have both start_indoors and transplant):
///   transplant_date = frost_date + transplant_weeks_relative
///   indoor_start_date = transplant_date - start_indoors_weeks_before
///
/// Seeds with no timing data produce zero PlantingActions.
/// Results are sorted by date.
pub fn generate_schedule(seeds_with_timing: &[(Seed, PlantingTiming)], year: i32) -> Vec<PlantingAction> {
    let last_frost = NaiveDate::from_ymd_opt(year, HALIFAX_MA_LAST_FROST.0, HALIFAX_MA_LAST_FROST.1)
        .expect("Invalid last frost date");

    let mut actions: Vec<PlantingAction> = Vec::new();

    for (seed, timing) in seeds_with_timing {
        // Handle transplant timing
        if let Some(weeks_rel) = timing.transplant_weeks_relative {
            let transplant_date = last_frost + Duration::weeks(weeks_rel as i64);

            actions.push(PlantingAction {
                seed_id: seed.id,
                seed_title: seed.title.clone(),
                action_type: ActionType::TransplantOutdoors,
                date: transplant_date,
                notes: format!("{} weeks after last frost", weeks_rel),
            });

            // For two-phase warm-season crops: indoor start is relative to transplant date
            if let Some(weeks_before) = timing.start_indoors_weeks_before {
                let indoor_date = transplant_date - Duration::weeks(weeks_before as i64);
                actions.push(PlantingAction {
                    seed_id: seed.id,
                    seed_title: seed.title.clone(),
                    action_type: ActionType::StartIndoors,
                    date: indoor_date,
                    notes: format!("{} weeks before transplant", weeks_before),
                });
            }
        } else if let Some(weeks_before) = timing.start_indoors_weeks_before {
            // Single-phase indoor start: relative to frost date directly
            let indoor_date = last_frost - Duration::weeks(weeks_before as i64);
            actions.push(PlantingAction {
                seed_id: seed.id,
                seed_title: seed.title.clone(),
                action_type: ActionType::StartIndoors,
                date: indoor_date,
                notes: format!("{} weeks before last frost", weeks_before),
            });
        }

        // Handle direct sow timing
        if let Some(weeks_rel) = timing.direct_sow_weeks_relative {
            let sow_date = last_frost + Duration::weeks(weeks_rel as i64);
            let direction = if weeks_rel < 0 { "before" } else { "after" };
            actions.push(PlantingAction {
                seed_id: seed.id,
                seed_title: seed.title.clone(),
                action_type: ActionType::DirectSow,
                date: sow_date,
                notes: format!("{} weeks {} last frost", weeks_rel.abs(), direction),
            });
        }
    }

    actions.sort_by_key(|a| a.date);
    actions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_seed(id: i64, title: &str) -> Seed {
        Seed {
            id,
            product_handle: String::new(),
            source_url: String::new(),
            title: title.to_string(),
            description: None,
            category: None,
            subcategory: None,
            light_requirement: None,
            frost_tolerance: None,
            is_organic: false,
            is_heirloom: false,
            days_to_maturity: None,
            sow_depth: None,
            plant_spacing: None,
            germination_info: None,
            planting_instructions: None,
            growing_instructions: None,
            harvest_instructions: None,
            raw_html: None,
            shopify_product_id: None,
            tags_raw: None,
            purchase_year: None,
            notes: None,
            created_at: None,
        }
    }

    #[test]
    fn test_start_indoors_6_weeks_before_frost() {
        // 6 weeks before May 10, 2026 -> March 29, 2026
        let seed = make_seed(1, "Rosemary");
        let timing = PlantingTiming {
            start_indoors_weeks_before: Some(6),
            transplant_weeks_relative: None,
            direct_sow_weeks_relative: None,
            indoor_start_recommended: true,
        };

        let actions = generate_schedule(&[(seed, timing)], 2026);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_type, ActionType::StartIndoors);
        assert_eq!(actions[0].date, NaiveDate::from_ymd_opt(2026, 3, 29).unwrap());
    }

    #[test]
    fn test_transplant_1_week_after_frost() {
        // 1 week after May 10, 2026 -> May 17, 2026
        let seed = make_seed(2, "Tomato");
        let timing = PlantingTiming {
            start_indoors_weeks_before: None,
            transplant_weeks_relative: Some(1),
            direct_sow_weeks_relative: None,
            indoor_start_recommended: false,
        };

        let actions = generate_schedule(&[(seed, timing)], 2026);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_type, ActionType::TransplantOutdoors);
        assert_eq!(actions[0].date, NaiveDate::from_ymd_opt(2026, 5, 17).unwrap());
    }

    #[test]
    fn test_warm_season_two_phase() {
        // Tomato: indoor start 6 weeks before transplant, transplant 1 week after frost
        // Transplant: May 10 + 1 week = May 17
        // Indoor start: May 17 - 6 weeks = April 5
        let seed = make_seed(3, "Tomato");
        let timing = PlantingTiming {
            start_indoors_weeks_before: Some(6),
            transplant_weeks_relative: Some(1),
            direct_sow_weeks_relative: None,
            indoor_start_recommended: true,
        };

        let actions = generate_schedule(&[(seed, timing)], 2026);
        assert_eq!(actions.len(), 2);

        // Sorted by date: indoor start first
        assert_eq!(actions[0].action_type, ActionType::StartIndoors);
        assert_eq!(actions[0].date, NaiveDate::from_ymd_opt(2026, 4, 5).unwrap());

        assert_eq!(actions[1].action_type, ActionType::TransplantOutdoors);
        assert_eq!(actions[1].date, NaiveDate::from_ymd_opt(2026, 5, 17).unwrap());
    }

    #[test]
    fn test_cool_season_direct_sow_before_frost() {
        // 4 weeks before May 10 -> April 12
        let seed = make_seed(4, "Lettuce");
        let timing = PlantingTiming {
            start_indoors_weeks_before: None,
            transplant_weeks_relative: None,
            direct_sow_weeks_relative: Some(-4),
            indoor_start_recommended: false,
        };

        let actions = generate_schedule(&[(seed, timing)], 2026);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_type, ActionType::DirectSow);
        assert_eq!(actions[0].date, NaiveDate::from_ymd_opt(2026, 4, 12).unwrap());
    }

    #[test]
    fn test_no_timing_produces_no_actions() {
        let seed = make_seed(5, "Mystery Plant");
        let timing = PlantingTiming::default();

        let actions = generate_schedule(&[(seed, timing)], 2026);
        assert!(actions.is_empty());
    }

    #[test]
    fn test_schedule_sorted_by_date() {
        // Multiple seeds, different dates -- should be sorted
        let seeds_timing = vec![
            (make_seed(1, "Bean"), PlantingTiming {
                start_indoors_weeks_before: None,
                transplant_weeks_relative: None,
                direct_sow_weeks_relative: Some(2), // May 24
                indoor_start_recommended: false,
            }),
            (make_seed(2, "Lettuce"), PlantingTiming {
                start_indoors_weeks_before: None,
                transplant_weeks_relative: None,
                direct_sow_weeks_relative: Some(-4), // April 12
                indoor_start_recommended: false,
            }),
        ];

        let actions = generate_schedule(&seeds_timing, 2026);
        assert_eq!(actions.len(), 2);
        assert!(actions[0].date < actions[1].date);
        assert_eq!(actions[0].seed_title, "Lettuce");
        assert_eq!(actions[1].seed_title, "Bean");
    }
}
