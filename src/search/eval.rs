use chrono::{Duration, NaiveDate};

use crate::db::models::Seed;
use crate::schedule::{PlantingTiming, SowingStatus, parse_planting_timing_from_fields};
use crate::schedule::calculator::last_frost_date;
use crate::viability::estimate_viability;

use super::ast::*;

pub struct SeedContext<'a> {
    pub seed: &'a Seed,
    pub in_plan: bool,
    pub plan_method: Option<&'a str>,
    pub sowing_status: Option<&'a SowingStatus>,
    pub newest_purchase_year: Option<i64>,
    pub today: NaiveDate,
    pub current_year: i32,
}

pub fn matches(filter: &Filter, ctx: &SeedContext) -> bool {
    match filter {
        Filter::And(children) => children.iter().all(|c| matches(c, ctx)),
        Filter::Or(children) => children.iter().any(|c| matches(c, ctx)),
        Filter::Not(child) => !matches(child, ctx),

        Filter::Title(s) => contains_ci(&ctx.seed.title, s),
        Filter::Category(s) => ctx.seed.category.as_deref().map_or(false, |v| contains_ci(v, s)),
        Filter::Subcategory(s) => ctx.seed.subcategory.as_deref().map_or(false, |v| contains_ci(v, s)),

        Filter::Organic => ctx.seed.is_organic,
        Filter::Heirloom => ctx.seed.is_heirloom,

        Filter::InPlan => ctx.in_plan,
        Filter::Plan(method) => {
            ctx.in_plan && ctx.plan_method.map_or(false, |m| contains_ci(m, method))
        }

        Filter::Start(pred) => eval_timing_predicate(pred, TimingField::Start, ctx),
        Filter::Sow(pred) => eval_timing_predicate(pred, TimingField::Sow, ctx),
        Filter::Transplant(pred) => eval_timing_predicate(pred, TimingField::Transplant, ctx),

        Filter::Viable => {
            estimate_viability(
                ctx.seed.subcategory.as_deref(),
                ctx.seed.category.as_deref(),
                ctx.newest_purchase_year,
            ).map_or(false, |v| v.percentage > 0)
        }
        Filter::Viability(cmp) => {
            estimate_viability(
                ctx.seed.subcategory.as_deref(),
                ctx.seed.category.as_deref(),
                ctx.newest_purchase_year,
            ).map_or(false, |v| match cmp {
                Comparison::Above(threshold) => v.percentage > *threshold,
                Comparison::Below(threshold) => v.percentage < *threshold,
            })
        }
    }
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

enum TimingField {
    Start,
    Sow,
    Transplant,
}

fn eval_timing_predicate(pred: &DatePredicate, field: TimingField, ctx: &SeedContext) -> bool {
    match pred {
        DatePredicate::Now => {
            // Check if currently in the relevant window
            match field {
                TimingField::Start => {
                    ctx.sowing_status.map_or(false, |s| {
                        s.method == "Start Indoors" && s.days_relative == 0
                    })
                }
                TimingField::Sow => {
                    ctx.sowing_status.map_or(false, |s| {
                        s.method == "Direct Sow" && s.days_relative == 0
                    })
                }
                TimingField::Transplant => {
                    // Transplant "now" means transplant date is within ~1 week of today
                    let timing = parse_seed_timing(ctx.seed);
                    timing.transplant_weeks_relative.map_or(false, |weeks_rel| {
                        let frost = last_frost_date(ctx.current_year);
                        let transplant_date = frost + Duration::weeks(weeks_rel as i64);
                        let diff = (ctx.today - transplant_date).num_days().abs();
                        diff <= 7
                    })
                }
            }
        }
        DatePredicate::Before(date_val) | DatePredicate::After(date_val) => {
            let seed_date = compute_field_date(field, ctx);
            let seed_date = match seed_date {
                Some(d) => d,
                None => return false,
            };
            let target = resolve_date_value(date_val, ctx.today, ctx.current_year);
            let target = match target {
                Some(d) => d,
                None => return false,
            };
            match pred {
                DatePredicate::Before(_) => seed_date < target,
                DatePredicate::After(_) => seed_date > target,
                _ => unreachable!(),
            }
        }
    }
}

fn parse_seed_timing(seed: &Seed) -> PlantingTiming {
    parse_planting_timing_from_fields(
        seed.when_to_sow_outside.as_deref(),
        seed.when_to_start_inside.as_deref(),
    )
}

fn compute_field_date(field: TimingField, ctx: &SeedContext) -> Option<NaiveDate> {
    let timing = parse_seed_timing(ctx.seed);
    let frost = last_frost_date(ctx.current_year);
    match field {
        TimingField::Start => {
            let weeks_before = timing.start_indoors_weeks_before?;
            let base = if let Some(weeks_rel) = timing.transplant_weeks_relative {
                frost + Duration::weeks(weeks_rel as i64)
            } else {
                frost
            };
            Some(base - Duration::weeks(weeks_before as i64))
        }
        TimingField::Sow => {
            let weeks_rel = timing.direct_sow_weeks_relative?;
            Some(frost + Duration::weeks(weeks_rel as i64))
        }
        TimingField::Transplant => {
            let weeks_rel = timing.transplant_weeks_relative?;
            Some(frost + Duration::weeks(weeks_rel as i64))
        }
    }
}

fn resolve_date_value(val: &DateValue, today: NaiveDate, current_year: i32) -> Option<NaiveDate> {
    match val {
        DateValue::Now => Some(today),
        DateValue::Absolute(s) => parse_absolute_date(s, current_year),
        DateValue::Relative { amount, unit, direction } => {
            let days = match unit {
                TimeUnit::Days => *amount,
                TimeUnit::Weeks => *amount * 7,
            };
            match direction {
                Direction::Ago => Some(today - Duration::days(days)),
                Direction::FromNow => Some(today + Duration::days(days)),
            }
        }
    }
}

fn parse_absolute_date(s: &str, year: i32) -> Option<NaiveDate> {
    // Try "Month Day" formats: "March 30", "Mar 30"
    let formats = ["%B %d", "%b %d"];
    for fmt in &formats {
        // Chrono needs a year for parsing, so we prepend it
        let with_year = format!("{year} {s}");
        let year_fmt = format!("%Y {fmt}");
        if let Ok(d) = NaiveDate::parse_from_str(&with_year, &year_fmt) {
            return Some(d);
        }
    }
    // Try full date "2026-03-30" or "March 30, 2026"
    if let Ok(d) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(s, "%B %d, %Y") {
        return Some(d);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_seed() -> Seed {
        Seed {
            id: 1,
            product_handle: "test".into(),
            source_url: "http://test.com".into(),
            title: "Cherry Tomato".into(),
            description: None,
            category: Some("Vegetables".into()),
            subcategory: Some("Tomato".into()),
            light_requirement: None,
            frost_tolerance: None,
            is_organic: true,
            is_heirloom: false,
            days_to_maturity: Some("75 days".into()),
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
            plant_type: Some("Annual".into()),
            botanical_name: None,
            family: None,
            native_region: None,
            hardiness: None,
            exposure: None,
            bloom_period: None,
            plant_dimensions: None,
            variety_info: None,
            attributes: None,
            when_to_sow_outside: Some("1 to 2 weeks after your average last frost date".into()),
            when_to_start_inside: Some("RECOMMENDED. 4 to 6 weeks before transplanting. Transplant outdoors 1 to 2 weeks after your average last frost date.".into()),
            days_to_emerge: Some("7-14 days".into()),
            row_spacing: None,
            thinning: None,
            special_care: None,
        }
    }

    fn make_ctx(seed: &Seed) -> SeedContext<'_> {
        SeedContext {
            seed,
            in_plan: true,
            plan_method: Some("indoor"),
            sowing_status: None,
            newest_purchase_year: Some(2025),
            today: NaiveDate::from_ymd_opt(2026, 3, 29).unwrap(),
            current_year: 2026,
        }
    }

    #[test]
    fn test_category_match() {
        let seed = make_seed();
        let ctx = make_ctx(&seed);
        assert!(matches(&Filter::Category("Vegetables".into()), &ctx));
        assert!(matches(&Filter::Category("veget".into()), &ctx));
        assert!(!matches(&Filter::Category("Herb".into()), &ctx));
    }

    #[test]
    fn test_organic() {
        let seed = make_seed();
        let ctx = make_ctx(&seed);
        assert!(matches(&Filter::Organic, &ctx));
        assert!(!matches(&Filter::Heirloom, &ctx));
    }

    #[test]
    fn test_in_plan() {
        let seed = make_seed();
        let ctx = make_ctx(&seed);
        assert!(matches(&Filter::InPlan, &ctx));
    }

    #[test]
    fn test_plan_method() {
        let seed = make_seed();
        let ctx = make_ctx(&seed);
        assert!(matches(&Filter::Plan("indoor".into()), &ctx));
        assert!(!matches(&Filter::Plan("outdoor".into()), &ctx));
    }

    #[test]
    fn test_and_or_not() {
        let seed = make_seed();
        let ctx = make_ctx(&seed);
        assert!(matches(&Filter::And(vec![Filter::Organic, Filter::InPlan]), &ctx));
        assert!(!matches(&Filter::And(vec![Filter::Organic, Filter::Heirloom]), &ctx));
        assert!(matches(&Filter::Or(vec![Filter::Heirloom, Filter::Organic]), &ctx));
        assert!(matches(&Filter::Not(Box::new(Filter::Heirloom)), &ctx));
    }

    #[test]
    fn test_start_before_date() {
        let seed = make_seed();
        let ctx = make_ctx(&seed);
        // Tomato: start indoors 6 weeks before transplant (May 17) = April 5
        // (start (before "April 15")) -> April 5 < April 15 = true
        assert!(matches(
            &Filter::Start(DatePredicate::Before(DateValue::Absolute("April 15".into()))),
            &ctx,
        ));
        // (start (before "March 15")) -> April 5 < March 15 = false
        assert!(!matches(
            &Filter::Start(DatePredicate::Before(DateValue::Absolute("March 15".into()))),
            &ctx,
        ));
    }

    #[test]
    fn test_transplant_after() {
        let seed = make_seed();
        let ctx = make_ctx(&seed);
        // Tomato transplant: 1 week after frost (May 10) = May 17
        // (transplant (after "May 1")) -> May 17 > May 1 = true
        assert!(matches(
            &Filter::Transplant(DatePredicate::After(DateValue::Absolute("May 1".into()))),
            &ctx,
        ));
    }

    #[test]
    fn test_absolute_date_parsing() {
        assert_eq!(
            parse_absolute_date("March 30", 2026),
            Some(NaiveDate::from_ymd_opt(2026, 3, 30).unwrap()),
        );
        assert_eq!(
            parse_absolute_date("Apr 15", 2026),
            Some(NaiveDate::from_ymd_opt(2026, 4, 15).unwrap()),
        );
    }

    #[test]
    fn test_viable() {
        let seed = make_seed();
        let ctx = make_ctx(&seed);
        assert!(matches(&Filter::Viable, &ctx));
    }
}
