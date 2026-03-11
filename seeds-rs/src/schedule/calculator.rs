use chrono::{Duration, NaiveDate};

use crate::db::models::Seed;
use super::parser::{PlantingTiming, parse_days_to_maturity, parse_days_to_emerge};

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

/// A timeline phase represents a contiguous period in a seed's lifecycle.
#[derive(Debug, Clone)]
pub struct TimelinePhase {
    pub phase_type: PhaseType,
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PhaseType {
    /// Planting window (when to sow outside)
    PlantingWindow,
    /// Indoor sowing / seeding period
    IndoorSowing,
    /// Indoor growing period (seedling growth to transplant)
    IndoorGrowing,
    /// Transplant window
    TransplantWindow,
    /// Outdoor growing (from sow/transplant to harvest)
    OutdoorGrowing,
    /// Harvest period
    Harvest,
}

/// Which start method to compute a timeline for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StartMethod {
    Indoor,
    Outdoor,
}

impl StartMethod {
    pub fn from_str_opt(s: Option<&str>) -> Option<Self> {
        match s {
            Some("indoor") => Some(Self::Indoor),
            Some("outdoor") => Some(Self::Outdoor),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Indoor => "indoor",
            Self::Outdoor => "outdoor",
        }
    }
}

/// Complete timeline for a seed with all phases.
#[derive(Debug, Clone)]
pub struct SeedTimeline {
    pub seed_id: i64,
    pub seed_title: String,
    pub phases: Vec<TimelinePhase>,
    pub actions: Vec<PlantingAction>,
}

pub fn last_frost_date(year: i32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, HALIFAX_MA_LAST_FROST.0, HALIFAX_MA_LAST_FROST.1)
        .expect("Invalid last frost date")
}

/// Compute a full timeline for a single seed including all lifecycle phases.
pub fn compute_seed_timeline(seed: &Seed, timing: &PlantingTiming, year: i32) -> SeedTimeline {
    let frost = last_frost_date(year);
    let mut phases = Vec::new();
    let mut actions = Vec::new();

    let days_to_maturity = seed.days_to_maturity.as_deref()
        .and_then(parse_days_to_maturity)
        .unwrap_or(60) as i64;

    let days_to_emerge = seed.days_to_emerge.as_deref()
        .and_then(parse_days_to_emerge)
        .unwrap_or(10) as i64;

    // Determine the key dates based on timing pattern
    if let Some(weeks_rel) = timing.transplant_weeks_relative {
        // Two-phase: indoor sow -> indoor growth -> transplant -> outdoor growing -> harvest
        let transplant_date = frost + Duration::weeks(weeks_rel as i64);

        // Indoor start
        if let Some(weeks_before) = timing.start_indoors_weeks_before {
            let indoor_start = transplant_date - Duration::weeks(weeks_before as i64);
            let sow_end = indoor_start + Duration::days(days_to_emerge);

            actions.push(PlantingAction {
                seed_id: seed.id,
                seed_title: seed.title.clone(),
                action_type: ActionType::StartIndoors,
                date: indoor_start,
                notes: format!("{} weeks before transplant", weeks_before),
            });

            // Indoor sowing phase (seeding through emergence)
            phases.push(TimelinePhase {
                phase_type: PhaseType::IndoorSowing,
                start: indoor_start,
                end: sow_end,
            });

            // Indoor growing phase (emergence to transplant)
            phases.push(TimelinePhase {
                phase_type: PhaseType::IndoorGrowing,
                start: sow_end,
                end: transplant_date,
            });
        }

        // Transplant window (give it a ~1 week window)
        phases.push(TimelinePhase {
            phase_type: PhaseType::TransplantWindow,
            start: transplant_date,
            end: transplant_date + Duration::weeks(1),
        });

        actions.push(PlantingAction {
            seed_id: seed.id,
            seed_title: seed.title.clone(),
            action_type: ActionType::TransplantOutdoors,
            date: transplant_date,
            notes: format!("{} weeks after last frost", weeks_rel),
        });

        // Outdoor growing: from transplant to harvest
        let harvest_start = transplant_date + Duration::days(days_to_maturity);
        phases.push(TimelinePhase {
            phase_type: PhaseType::OutdoorGrowing,
            start: transplant_date + Duration::weeks(1),
            end: harvest_start,
        });

        // Harvest phase (~3 weeks)
        phases.push(TimelinePhase {
            phase_type: PhaseType::Harvest,
            start: harvest_start,
            end: harvest_start + Duration::weeks(3),
        });
    } else if let Some(weeks_before) = timing.start_indoors_weeks_before {
        // Indoor start without explicit transplant (e.g., herbs like rosemary)
        let indoor_start = frost - Duration::weeks(weeks_before as i64);
        let sow_end = indoor_start + Duration::days(days_to_emerge);
        // Assume transplant around last frost
        let transplant_date = frost;

        actions.push(PlantingAction {
            seed_id: seed.id,
            seed_title: seed.title.clone(),
            action_type: ActionType::StartIndoors,
            date: indoor_start,
            notes: format!("{} weeks before last frost", weeks_before),
        });

        phases.push(TimelinePhase {
            phase_type: PhaseType::IndoorSowing,
            start: indoor_start,
            end: sow_end,
        });

        phases.push(TimelinePhase {
            phase_type: PhaseType::IndoorGrowing,
            start: sow_end,
            end: transplant_date,
        });

        phases.push(TimelinePhase {
            phase_type: PhaseType::TransplantWindow,
            start: transplant_date,
            end: transplant_date + Duration::weeks(1),
        });

        // Outdoor growing
        let harvest_start = transplant_date + Duration::days(days_to_maturity);
        phases.push(TimelinePhase {
            phase_type: PhaseType::OutdoorGrowing,
            start: transplant_date + Duration::weeks(1),
            end: harvest_start,
        });

        phases.push(TimelinePhase {
            phase_type: PhaseType::Harvest,
            start: harvest_start,
            end: harvest_start + Duration::weeks(3),
        });
    }

    // Direct sow path
    if let Some(weeks_rel) = timing.direct_sow_weeks_relative {
        let sow_date = frost + Duration::weeks(weeks_rel as i64);
        let direction = if weeks_rel < 0 { "before" } else { "after" };

        actions.push(PlantingAction {
            seed_id: seed.id,
            seed_title: seed.title.clone(),
            action_type: ActionType::DirectSow,
            date: sow_date,
            notes: format!("{} weeks {} last frost", weeks_rel.abs(), direction),
        });

        // Sowing window covers sowing through emergence (at least 2 weeks)
        let sow_window_end = sow_date + Duration::days(days_to_emerge.max(14));
        let harvest_start = sow_date + Duration::days(days_to_maturity);

        phases.push(TimelinePhase {
            phase_type: PhaseType::PlantingWindow,
            start: sow_date,
            end: sow_window_end,
        });

        // Only add outdoor growing if we don't already have it (from indoor path)
        if timing.transplant_weeks_relative.is_none() && timing.start_indoors_weeks_before.is_none() {
            phases.push(TimelinePhase {
                phase_type: PhaseType::OutdoorGrowing,
                start: sow_window_end,
                end: harvest_start,
            });

            phases.push(TimelinePhase {
                phase_type: PhaseType::Harvest,
                start: harvest_start,
                end: harvest_start + Duration::weeks(3),
            });
        }
    }

    actions.sort_by_key(|a| a.date);

    SeedTimeline {
        seed_id: seed.id,
        seed_title: seed.title.clone(),
        phases,
        actions,
    }
}

/// Compute the indoor-only timeline: sow → indoor growth → transplant → outdoor growing → harvest.
/// Returns None if the seed has no indoor start option.
pub fn compute_indoor_timeline(seed: &Seed, timing: &PlantingTiming, year: i32) -> Option<SeedTimeline> {
    if timing.start_indoors_weeks_before.is_none() {
        return None;
    }

    let frost = last_frost_date(year);
    let mut phases = Vec::new();
    let mut actions = Vec::new();

    let days_to_maturity = seed.days_to_maturity.as_deref()
        .and_then(parse_days_to_maturity)
        .unwrap_or(60) as i64;

    let days_to_emerge = seed.days_to_emerge.as_deref()
        .and_then(parse_days_to_emerge)
        .unwrap_or(10) as i64;

    if let Some(weeks_rel) = timing.transplant_weeks_relative {
        let transplant_date = frost + Duration::weeks(weeks_rel as i64);
        let weeks_before = timing.start_indoors_weeks_before.unwrap();
        let indoor_start = transplant_date - Duration::weeks(weeks_before as i64);
        let sow_end = indoor_start + Duration::days(days_to_emerge);

        actions.push(PlantingAction {
            seed_id: seed.id,
            seed_title: seed.title.clone(),
            action_type: ActionType::StartIndoors,
            date: indoor_start,
            notes: format!("{} weeks before transplant", weeks_before),
        });

        phases.push(TimelinePhase {
            phase_type: PhaseType::IndoorSowing,
            start: indoor_start,
            end: sow_end,
        });

        phases.push(TimelinePhase {
            phase_type: PhaseType::IndoorGrowing,
            start: sow_end,
            end: transplant_date,
        });

        phases.push(TimelinePhase {
            phase_type: PhaseType::TransplantWindow,
            start: transplant_date,
            end: transplant_date + Duration::weeks(1),
        });

        actions.push(PlantingAction {
            seed_id: seed.id,
            seed_title: seed.title.clone(),
            action_type: ActionType::TransplantOutdoors,
            date: transplant_date,
            notes: format!("{} weeks after last frost", weeks_rel),
        });

        let harvest_start = transplant_date + Duration::days(days_to_maturity);
        phases.push(TimelinePhase {
            phase_type: PhaseType::OutdoorGrowing,
            start: transplant_date + Duration::weeks(1),
            end: harvest_start,
        });

        phases.push(TimelinePhase {
            phase_type: PhaseType::Harvest,
            start: harvest_start,
            end: harvest_start + Duration::weeks(3),
        });
    } else {
        let weeks_before = timing.start_indoors_weeks_before.unwrap();
        let indoor_start = frost - Duration::weeks(weeks_before as i64);
        let sow_end = indoor_start + Duration::days(days_to_emerge);
        let transplant_date = frost;

        actions.push(PlantingAction {
            seed_id: seed.id,
            seed_title: seed.title.clone(),
            action_type: ActionType::StartIndoors,
            date: indoor_start,
            notes: format!("{} weeks before last frost", weeks_before),
        });

        phases.push(TimelinePhase {
            phase_type: PhaseType::IndoorSowing,
            start: indoor_start,
            end: sow_end,
        });

        phases.push(TimelinePhase {
            phase_type: PhaseType::IndoorGrowing,
            start: sow_end,
            end: transplant_date,
        });

        phases.push(TimelinePhase {
            phase_type: PhaseType::TransplantWindow,
            start: transplant_date,
            end: transplant_date + Duration::weeks(1),
        });

        let harvest_start = transplant_date + Duration::days(days_to_maturity);
        phases.push(TimelinePhase {
            phase_type: PhaseType::OutdoorGrowing,
            start: transplant_date + Duration::weeks(1),
            end: harvest_start,
        });

        phases.push(TimelinePhase {
            phase_type: PhaseType::Harvest,
            start: harvest_start,
            end: harvest_start + Duration::weeks(3),
        });
    }

    actions.sort_by_key(|a| a.date);

    Some(SeedTimeline {
        seed_id: seed.id,
        seed_title: seed.title.clone(),
        phases,
        actions,
    })
}

/// Compute the outdoor-only timeline: direct sow → outdoor growing → harvest.
/// Returns None if the seed has no direct sow option.
pub fn compute_outdoor_timeline(seed: &Seed, timing: &PlantingTiming, year: i32) -> Option<SeedTimeline> {
    if timing.direct_sow_weeks_relative.is_none() {
        return None;
    }

    let frost = last_frost_date(year);
    let mut phases = Vec::new();
    let mut actions = Vec::new();

    let days_to_maturity = seed.days_to_maturity.as_deref()
        .and_then(parse_days_to_maturity)
        .unwrap_or(60) as i64;

    let days_to_emerge = seed.days_to_emerge.as_deref()
        .and_then(parse_days_to_emerge)
        .unwrap_or(10) as i64;

    let weeks_rel = timing.direct_sow_weeks_relative.unwrap();
    let sow_date = frost + Duration::weeks(weeks_rel as i64);
    let direction = if weeks_rel < 0 { "before" } else { "after" };

    actions.push(PlantingAction {
        seed_id: seed.id,
        seed_title: seed.title.clone(),
        action_type: ActionType::DirectSow,
        date: sow_date,
        notes: format!("{} weeks {} last frost", weeks_rel.abs(), direction),
    });

    // Sowing window covers sowing through emergence (at least 2 weeks)
    let sow_window_end = sow_date + Duration::days(days_to_emerge.max(14));
    let harvest_start = sow_date + Duration::days(days_to_maturity);

    phases.push(TimelinePhase {
        phase_type: PhaseType::PlantingWindow,
        start: sow_date,
        end: sow_window_end,
    });

    phases.push(TimelinePhase {
        phase_type: PhaseType::OutdoorGrowing,
        start: sow_window_end,
        end: harvest_start,
    });

    phases.push(TimelinePhase {
        phase_type: PhaseType::Harvest,
        start: harvest_start,
        end: harvest_start + Duration::weeks(3),
    });

    actions.sort_by_key(|a| a.date);

    Some(SeedTimeline {
        seed_id: seed.id,
        seed_title: seed.title.clone(),
        phases,
        actions,
    })
}

/// Compute timeline for a specific start method, falling back to the combined timeline.
pub fn compute_timeline_for_method(seed: &Seed, timing: &PlantingTiming, year: i32, method: StartMethod) -> SeedTimeline {
    match method {
        StartMethod::Indoor => compute_indoor_timeline(seed, timing, year)
            .unwrap_or_else(|| compute_seed_timeline(seed, timing, year)),
        StartMethod::Outdoor => compute_outdoor_timeline(seed, timing, year)
            .unwrap_or_else(|| compute_seed_timeline(seed, timing, year)),
    }
}

/// Generate a schedule of planting actions for seeds with parsed timing data.
pub fn generate_schedule(seeds_with_timing: &[(Seed, PlantingTiming)], year: i32) -> Vec<PlantingAction> {
    let mut actions: Vec<PlantingAction> = Vec::new();

    for (seed, timing) in seeds_with_timing {
        let timeline = compute_seed_timeline(seed, timing, year);
        actions.extend(timeline.actions);
    }

    actions.sort_by_key(|a| a.date);
    actions
}

/// Generate a schedule respecting each seed's chosen start method.
pub fn generate_schedule_with_methods(
    seeds_with_timing: &[(Seed, PlantingTiming, Option<StartMethod>)],
    year: i32,
) -> Vec<PlantingAction> {
    let mut actions: Vec<PlantingAction> = Vec::new();

    for (seed, timing, method) in seeds_with_timing {
        let timeline = if let Some(m) = method {
            compute_timeline_for_method(seed, timing, year, *m)
        } else {
            compute_seed_timeline(seed, timing, year)
        };
        actions.extend(timeline.actions);
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
            plant_type: None,
            botanical_name: None,
            family: None,
            native_region: None,
            hardiness: None,
            exposure: None,
            bloom_period: None,
            plant_dimensions: None,
            variety_info: None,
            attributes: None,
            when_to_sow_outside: None,
            when_to_start_inside: None,
            days_to_emerge: None,
            row_spacing: None,
            thinning: None,
            special_care: None,
        }
    }

    #[test]
    fn test_start_indoors_6_weeks_before_frost() {
        let seed = make_seed(1, "Rosemary");
        let timing = PlantingTiming {
            start_indoors_weeks_before: Some(6),
            transplant_weeks_relative: None,
            direct_sow_weeks_relative: None,
            indoor_start_recommended: true,
        };

        let actions = generate_schedule(&[(seed, timing)], 2026);
        assert!(actions.len() >= 1);
        assert_eq!(actions[0].action_type, ActionType::StartIndoors);
        assert_eq!(actions[0].date, NaiveDate::from_ymd_opt(2026, 3, 29).unwrap());
    }

    #[test]
    fn test_transplant_1_week_after_frost() {
        let seed = make_seed(2, "Tomato");
        let timing = PlantingTiming {
            start_indoors_weeks_before: None,
            transplant_weeks_relative: Some(1),
            direct_sow_weeks_relative: None,
            indoor_start_recommended: false,
        };

        let actions = generate_schedule(&[(seed, timing)], 2026);
        let transplant = actions.iter().find(|a| a.action_type == ActionType::TransplantOutdoors).unwrap();
        assert_eq!(transplant.date, NaiveDate::from_ymd_opt(2026, 5, 17).unwrap());
    }

    #[test]
    fn test_warm_season_two_phase() {
        let seed = make_seed(3, "Tomato");
        let timing = PlantingTiming {
            start_indoors_weeks_before: Some(6),
            transplant_weeks_relative: Some(1),
            direct_sow_weeks_relative: None,
            indoor_start_recommended: true,
        };

        let actions = generate_schedule(&[(seed, timing)], 2026);
        let indoor = actions.iter().find(|a| a.action_type == ActionType::StartIndoors).unwrap();
        let transplant = actions.iter().find(|a| a.action_type == ActionType::TransplantOutdoors).unwrap();

        assert_eq!(indoor.date, NaiveDate::from_ymd_opt(2026, 4, 5).unwrap());
        assert_eq!(transplant.date, NaiveDate::from_ymd_opt(2026, 5, 17).unwrap());
    }

    #[test]
    fn test_cool_season_direct_sow_before_frost() {
        let seed = make_seed(4, "Lettuce");
        let timing = PlantingTiming {
            start_indoors_weeks_before: None,
            transplant_weeks_relative: None,
            direct_sow_weeks_relative: Some(-4),
            indoor_start_recommended: false,
        };

        let actions = generate_schedule(&[(seed, timing)], 2026);
        let sow = actions.iter().find(|a| a.action_type == ActionType::DirectSow).unwrap();
        assert_eq!(sow.date, NaiveDate::from_ymd_opt(2026, 4, 12).unwrap());
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
        let seeds_timing = vec![
            (make_seed(1, "Bean"), PlantingTiming {
                start_indoors_weeks_before: None,
                transplant_weeks_relative: None,
                direct_sow_weeks_relative: Some(2),
                indoor_start_recommended: false,
            }),
            (make_seed(2, "Lettuce"), PlantingTiming {
                start_indoors_weeks_before: None,
                transplant_weeks_relative: None,
                direct_sow_weeks_relative: Some(-4),
                indoor_start_recommended: false,
            }),
        ];

        let actions = generate_schedule(&seeds_timing, 2026);
        assert!(actions.len() >= 2);
        let bean_sow = actions.iter().find(|a| a.seed_title == "Bean" && a.action_type == ActionType::DirectSow).unwrap();
        let lettuce_sow = actions.iter().find(|a| a.seed_title == "Lettuce" && a.action_type == ActionType::DirectSow).unwrap();
        assert!(lettuce_sow.date < bean_sow.date);
    }

    #[test]
    fn test_timeline_phases_two_phase_crop() {
        let mut seed = make_seed(1, "Tomato");
        seed.days_to_maturity = Some("75 days".to_string());

        let timing = PlantingTiming {
            start_indoors_weeks_before: Some(6),
            transplant_weeks_relative: Some(1),
            direct_sow_weeks_relative: None,
            indoor_start_recommended: true,
        };

        let timeline = compute_seed_timeline(&seed, &timing, 2026);
        assert!(timeline.phases.iter().any(|p| p.phase_type == PhaseType::IndoorGrowing));
        assert!(timeline.phases.iter().any(|p| p.phase_type == PhaseType::TransplantWindow));
        assert!(timeline.phases.iter().any(|p| p.phase_type == PhaseType::OutdoorGrowing));
        assert!(timeline.phases.iter().any(|p| p.phase_type == PhaseType::Harvest));
    }

    #[test]
    fn test_timeline_phases_direct_sow() {
        let mut seed = make_seed(1, "Bean");
        seed.days_to_maturity = Some("58 days".to_string());

        let timing = PlantingTiming {
            start_indoors_weeks_before: None,
            transplant_weeks_relative: None,
            direct_sow_weeks_relative: Some(1),
            indoor_start_recommended: false,
        };

        let timeline = compute_seed_timeline(&seed, &timing, 2026);
        assert!(timeline.phases.iter().any(|p| p.phase_type == PhaseType::PlantingWindow));
        assert!(timeline.phases.iter().any(|p| p.phase_type == PhaseType::OutdoorGrowing));
        assert!(timeline.phases.iter().any(|p| p.phase_type == PhaseType::Harvest));
        assert!(!timeline.phases.iter().any(|p| p.phase_type == PhaseType::IndoorGrowing));
    }
}
