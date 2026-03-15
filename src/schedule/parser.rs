/// Parsed planting timing extracted from when_to_sow_outside and when_to_start_inside fields.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlantingTiming {
    /// Weeks before last frost to start indoors (None = not recommended indoors)
    pub start_indoors_weeks_before: Option<u8>,
    /// Weeks relative to last frost for transplanting outdoors (positive = after frost)
    pub transplant_weeks_relative: Option<i8>,
    /// Weeks relative to last frost for direct sowing (negative = before frost, positive = after)
    pub direct_sow_weeks_relative: Option<i8>,
    /// Whether starting indoors is explicitly recommended
    pub indoor_start_recommended: bool,
}

/// Parse planting timing from the structured when_to_sow_outside and when_to_start_inside fields.
///
/// This replaces the old approach of parsing the combined planting_instructions text.
/// The new approach uses the dedicated columns which are cleaner and more reliable.
pub fn parse_planting_timing_from_fields(
    when_to_sow_outside: Option<&str>,
    when_to_start_inside: Option<&str>,
) -> PlantingTiming {
    let mut timing = PlantingTiming::default();

    // Parse indoor start timing
    if let Some(inside_text) = when_to_start_inside {
        let lower = inside_text.to_lowercase();

        // Check if indoor start is recommended (must start with "RECOMMENDED")
        if lower.starts_with("recommended.") || lower.starts_with("recommended:") {
            timing.indoor_start_recommended = true;
        }

        // Check for "not recommended"
        if lower.contains("not recommended") {
            // Explicitly not recommended, skip indoor parsing
        } else if lower.contains("before transplanting") || lower.contains("before transplant") {
            // Two-phase warm season: "4 to 6 weeks before transplanting"
            if let Some((_, larger)) = extract_weeks_range(&lower, "before transplanting")
                .or_else(|| extract_weeks_range(&lower, "before transplant"))
            {
                timing.start_indoors_weeks_before = Some(larger);
            }

            // Look for transplant timing in the same text: "usually X to Y weeks after your average last frost"
            if let Some((smaller, _)) = extract_weeks_range(&lower, "after your average last frost")
                .or_else(|| extract_weeks_range(&lower, "after last frost"))
            {
                timing.transplant_weeks_relative = Some(smaller as i8);
            }
        } else if let Some((_, larger)) = extract_weeks_range(&lower, "before your average last frost")
            .or_else(|| extract_weeks_range(&lower, "before last frost"))
        {
            // Single-phase indoor start: "6 to 8 weeks before your average last frost date"
            timing.start_indoors_weeks_before = Some(larger);
        }
    }

    // Parse outdoor sow timing
    if let Some(outside_text) = when_to_sow_outside {
        let lower = outside_text.to_lowercase();

        if let Some((smaller, _)) = extract_weeks_range(&lower, "after your average last frost")
            .or_else(|| extract_weeks_range(&lower, "after last frost"))
        {
            if timing.start_indoors_weeks_before.is_some() && timing.transplant_weeks_relative.is_none() {
                // If we have indoor start but no transplant yet from the inside text,
                // and the outside text says "after frost", this might be the transplant timing
                // for a two-phase crop, OR it could be a direct sow option.
                // If indoor is recommended, treat this as direct sow alternative.
                timing.direct_sow_weeks_relative = Some(smaller as i8);
            } else if timing.transplant_weeks_relative.is_none() && timing.start_indoors_weeks_before.is_some() {
                timing.transplant_weeks_relative = Some(smaller as i8);
            } else {
                timing.direct_sow_weeks_relative = Some(smaller as i8);
            }
        } else if let Some((_, larger)) = extract_weeks_range(&lower, "before your average last frost")
            .or_else(|| extract_weeks_range(&lower, "before last frost"))
        {
            // Cool-season: sow before frost
            timing.direct_sow_weeks_relative = Some(-(larger as i8));
        }
    }

    timing
}

/// Legacy: Parse planting timing from the combined planting_instructions text field.
/// Kept for backward compatibility; prefer parse_planting_timing_from_fields.
#[allow(dead_code)]
pub fn parse_planting_timing(text: &str) -> PlantingTiming {
    let lower = text.to_lowercase();
    let mut timing = PlantingTiming::default();

    if lower.contains("recommended") && (lower.contains("start inside") || lower.contains("start indoors")) {
        timing.indoor_start_recommended = true;
    }

    let has_before_transplanting = extract_weeks_range(&lower, "before transplanting");
    let has_before_frost = extract_weeks_range(&lower, "before your average last frost")
        .or_else(|| extract_weeks_range(&lower, "before last frost"));
    let has_after_frost = extract_weeks_range(&lower, "after your average last frost")
        .or_else(|| extract_weeks_range(&lower, "after last frost"));

    if let (Some((_, larger_before_transplant)), Some((smaller_after, _))) =
        (has_before_transplanting, has_after_frost)
    {
        timing.transplant_weeks_relative = Some(smaller_after as i8);
        timing.start_indoors_weeks_before = Some(larger_before_transplant);
        timing.indoor_start_recommended = true;
        return timing;
    }

    if let Some((_, larger)) = has_before_frost {
        if timing.indoor_start_recommended
            || lower.contains("start inside")
            || lower.contains("start indoors")
        {
            timing.start_indoors_weeks_before = Some(larger);
        } else {
            timing.direct_sow_weeks_relative = Some(-(larger as i8));
        }
    }

    if let Some((smaller, _)) = has_after_frost {
        if timing.start_indoors_weeks_before.is_some() {
            timing.transplant_weeks_relative = Some(smaller as i8);
        } else {
            timing.direct_sow_weeks_relative = Some(smaller as i8);
        }
    }

    timing
}

/// Extract a "X to Y weeks {phrase}" pattern, returning (smaller, larger) values.
fn extract_weeks_range(text: &str, phrase: &str) -> Option<(u8, u8)> {
    let phrase_pos = text.find(phrase)?;
    let before_phrase = &text[..phrase_pos];
    let weeks_pos = before_phrase.rfind("weeks")?;
    let before_weeks = before_phrase[..weeks_pos].trim();

    if let Some(to_pos) = before_weeks.rfind(" to ") {
        let y_str = before_weeks[to_pos + 4..].trim();
        let x_part = before_weeks[..to_pos].trim();
        let x_str = x_part.rsplit_once(' ').map(|(_, x)| x).unwrap_or(x_part);

        if let (Ok(x), Ok(y)) = (x_str.parse::<u8>(), y_str.parse::<u8>()) {
            let smaller = x.min(y);
            let larger = x.max(y);
            return Some((smaller, larger));
        }
    }

    let num_str = before_weeks.rsplit_once(' ').map(|(_, n)| n).unwrap_or(before_weeks);
    if let Ok(n) = num_str.parse::<u8>() {
        return Some((n, n));
    }

    None
}

/// Parse a "days to maturity" string into an approximate number of days.
/// Handles patterns like "58 days", "75-80 days from transplanting", etc.
pub fn parse_days_to_maturity(text: &str) -> Option<u16> {
    let lower = text.to_lowercase();
    // Find the first number or range
    let mut nums: Vec<u16> = Vec::new();
    for word in lower.split(|c: char| !c.is_ascii_digit() && c != '-') {
        if word.contains('-') {
            // Range like "75-80"
            for part in word.split('-') {
                if let Ok(n) = part.parse::<u16>() {
                    nums.push(n);
                }
            }
        } else if let Ok(n) = word.parse::<u16>() {
            nums.push(n);
        }
    }

    if nums.is_empty() {
        return None;
    }

    // Average the first two numbers if it's a range
    if nums.len() >= 2 {
        Some((nums[0] + nums[1]) / 2)
    } else {
        Some(nums[0])
    }
}

/// Parse "days to emerge" into a number of days.
pub fn parse_days_to_emerge(text: &str) -> Option<u16> {
    // Same format as days to maturity
    parse_days_to_maturity(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for the new field-based parser

    #[test]
    fn test_fields_tomato_two_phase() {
        let timing = parse_planting_timing_from_fields(
            Some("For mild climates only: 1 to 2 weeks after your average last frost date"),
            Some("RECOMMENDED. 4 to 6 weeks before transplanting. Transplant when air temperature is 45°F or warmer, usually 1 to 2 weeks after your average last frost date."),
        );
        assert_eq!(timing.start_indoors_weeks_before, Some(6));
        assert_eq!(timing.transplant_weeks_relative, Some(1));
        assert!(timing.indoor_start_recommended);
    }

    #[test]
    fn test_fields_bean_direct_sow() {
        let timing = parse_planting_timing_from_fields(
            Some("RECOMMENDED. 1 to 2 weeks after your average last frost date"),
            Some("Not recommended."),
        );
        assert_eq!(timing.direct_sow_weeks_relative, Some(1));
        assert_eq!(timing.start_indoors_weeks_before, None);
    }

    #[test]
    fn test_fields_chives_cool_season() {
        let timing = parse_planting_timing_from_fields(
            Some("4 to 6 weeks before your average last frost date, when soil temperature is at least 45°F"),
            Some("6 to 8 weeks before your average last frost date."),
        );
        assert_eq!(timing.start_indoors_weeks_before, Some(8));
        assert_eq!(timing.direct_sow_weeks_relative, Some(-6));
        // Neither text explicitly says "recommended", so no recommendation
        assert!(!timing.indoor_start_recommended);
    }

    #[test]
    fn test_fields_rosemary_long_indoor() {
        let timing = parse_planting_timing_from_fields(
            Some("1 to 2 weeks after your average last frost date in mild climates"),
            Some("RECOMMENDED. 10 to 12 weeks before your average last frost date."),
        );
        assert_eq!(timing.start_indoors_weeks_before, Some(12));
        assert!(timing.indoor_start_recommended);
        assert_eq!(timing.direct_sow_weeks_relative, Some(1));
    }

    #[test]
    fn test_fields_lettuce_outdoor_recommended() {
        let timing = parse_planting_timing_from_fields(
            Some("RECOMMENDED. 2 to 4 weeks before your average last frost date, and when soil temperature is at least 40°F, ideally 60°–70°F."),
            Some("4 to 6 weeks before your average last frost date, and in summer when soil temperatures are too warm (above 80°F) to germinate lettuce seed."),
        );
        assert_eq!(timing.start_indoors_weeks_before, Some(6));
        assert_eq!(timing.direct_sow_weeks_relative, Some(-4));
        // Outdoor text says "RECOMMENDED", indoor does not
        assert!(!timing.indoor_start_recommended);
    }

    #[test]
    fn test_fields_none() {
        let timing = parse_planting_timing_from_fields(None, None);
        assert_eq!(timing, PlantingTiming::default());
    }

    // Legacy parser tests

    #[test]
    fn test_warm_season_two_phase_tomato() {
        let text = "Start Inside: RECOMMENDED. Sow seeds 4 to 6 weeks before transplanting. \
                     Transplant outdoors 1 to 2 weeks after your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.start_indoors_weeks_before, Some(6));
        assert_eq!(timing.transplant_weeks_relative, Some(1));
        assert!(timing.indoor_start_recommended);
    }

    #[test]
    fn test_warm_season_direct_sow_bean() {
        let text = "Sow Outside: Sow seeds 1 to 2 weeks after your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.start_indoors_weeks_before, None);
        assert_eq!(timing.direct_sow_weeks_relative, Some(1));
        assert!(!timing.indoor_start_recommended);
    }

    #[test]
    fn test_cool_season_lettuce() {
        let text = "Sow Outside: Sow seeds 2 to 4 weeks before your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.direct_sow_weeks_relative, Some(-4));
        assert_eq!(timing.start_indoors_weeks_before, None);
    }

    #[test]
    fn test_cool_season_start_indoors() {
        let text = "Start Inside: RECOMMENDED. Start seeds 4 to 6 weeks before your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.start_indoors_weeks_before, Some(6));
        assert!(timing.indoor_start_recommended);
    }

    #[test]
    fn test_long_start_rosemary() {
        let text = "Start Inside: RECOMMENDED. Start seeds indoors 10 to 12 weeks before your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.start_indoors_weeks_before, Some(12));
        assert!(timing.indoor_start_recommended);
    }

    #[test]
    fn test_unparseable_returns_default() {
        let text = "Plant in spring when soil is warm.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing, PlantingTiming::default());
    }

    #[test]
    fn test_empty_string() {
        let timing = parse_planting_timing("");
        assert_eq!(timing, PlantingTiming::default());
    }

    #[test]
    fn test_before_frost_uses_larger_number() {
        let text = "Start Inside: RECOMMENDED. Start seeds 4 to 6 weeks before your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.start_indoors_weeks_before, Some(6));
    }

    #[test]
    fn test_after_frost_uses_smaller_number() {
        let text = "Sow Outside: 1 to 2 weeks after your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.direct_sow_weeks_relative, Some(1));
    }

    #[test]
    fn test_parse_days_to_maturity_simple() {
        assert_eq!(parse_days_to_maturity("58 days"), Some(58));
    }

    #[test]
    fn test_parse_days_to_maturity_range() {
        assert_eq!(parse_days_to_maturity("75-80 days from transplanting"), Some(77));
    }

    #[test]
    fn test_parse_days_to_maturity_empty() {
        assert_eq!(parse_days_to_maturity(""), None);
    }

    #[test]
    fn test_parse_days_to_emerge() {
        assert_eq!(parse_days_to_emerge("10-15 days"), Some(12));
        assert_eq!(parse_days_to_emerge("6–12 days"), Some(9)); // en-dash splits on non-digit chars too
    }
}
