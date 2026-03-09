/// Parsed planting timing extracted from free-text planting instructions.
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

/// Parse planting timing from the planting_instructions text field.
///
/// Handles these patterns from Botanical Interests data:
/// - "X to Y weeks before your average last frost date" (cool-season)
/// - "X to Y weeks after your average last frost date" (warm-season)
/// - "X to Y weeks before transplanting" (warm-season indoor start relative to transplant date)
/// - "RECOMMENDED" near "Start Inside" -> indoor_start_recommended = true
///
/// For "before" ranges, uses the LARGER number (earlier = safer).
/// For "after" ranges, uses the SMALLER number (earliest safe date).
///
/// Returns all-None PlantingTiming if text has no parseable frost-relative timing.
pub fn parse_planting_timing(text: &str) -> PlantingTiming {
    let lower = text.to_lowercase();
    let mut timing = PlantingTiming::default();

    // Check if indoor start is recommended
    if lower.contains("recommended") && lower.contains("start inside") {
        timing.indoor_start_recommended = true;
    }
    // Also match "start indoors" variant
    if lower.contains("recommended") && lower.contains("start indoors") {
        timing.indoor_start_recommended = true;
    }
    // Some products just say "Start Inside" as a section header with "RECOMMENDED"
    if lower.contains("start inside") && lower.contains("recommended") {
        timing.indoor_start_recommended = true;
    }

    // Parse "X to Y weeks before transplanting" (warm-season indoor start)
    // This must be checked BEFORE the "before your average last frost" pattern
    let has_before_transplanting = extract_weeks_range(&lower, "before transplanting");

    // Parse "X to Y weeks before your average last frost date"
    let has_before_frost = extract_weeks_range(&lower, "before your average last frost");
    // Also match variant "before last frost"
    let has_before_frost = has_before_frost.or_else(|| extract_weeks_range(&lower, "before last frost"));

    // Parse "X to Y weeks after your average last frost date"
    let has_after_frost = extract_weeks_range(&lower, "after your average last frost");
    let has_after_frost = has_after_frost.or_else(|| extract_weeks_range(&lower, "after last frost"));

    // Determine warm-season two-phase pattern:
    // "X to Y weeks before transplanting" + "A to B weeks after last frost"
    // = indoor start relative to transplant date, transplant relative to frost date
    if let (Some((_, larger_before_transplant)), Some((smaller_after, _))) =
        (has_before_transplanting, has_after_frost)
    {
        // Transplant is `smaller_after` weeks after frost
        timing.transplant_weeks_relative = Some(smaller_after as i8);
        // Indoor start: `larger_before_transplant` weeks before transplant date
        // We store this as weeks-before-frost by computing:
        // total = larger_before_transplant - smaller_after (weeks before frost)
        // But actually, the plan says: store start_indoors_weeks_before and let calculator
        // figure it out using transplant date. We need to store the raw "before transplanting"
        // value, and the calculator will compute: transplant_date - N weeks.
        // So start_indoors_weeks_before here means "weeks before transplant date".
        // Actually, looking at the plan more carefully -- the calculator handles this:
        // "For warm-season two-phase crops: transplant date = frost + after_weeks,
        //  indoor start = transplant - indoor_weeks"
        // So we store the raw weeks values and let calculator combine them.
        timing.start_indoors_weeks_before = Some(larger_before_transplant);
        timing.indoor_start_recommended = true; // Two-phase pattern implies indoor start
        return timing;
    }

    // Cool-season or single-phase: "X to Y weeks before your average last frost date"
    if let Some((_, larger)) = has_before_frost {
        // Determine if this is indoor start or direct sow based on context
        if timing.indoor_start_recommended
            || lower.contains("start inside")
            || lower.contains("start indoors")
        {
            timing.start_indoors_weeks_before = Some(larger);
        } else {
            // Could be direct sow for cool-season crops
            // Use negative value (before frost)
            timing.direct_sow_weeks_relative = Some(-(larger as i8));
        }
    }

    // "X to Y weeks after your average last frost date" (warm-season direct sow or transplant)
    if let Some((smaller, _)) = has_after_frost {
        if timing.start_indoors_weeks_before.is_some() {
            // If we already have indoor start, this is transplant timing
            timing.transplant_weeks_relative = Some(smaller as i8);
        } else {
            // Direct sow after frost
            timing.direct_sow_weeks_relative = Some(smaller as i8);
        }
    }

    timing
}

/// Extract a "X to Y weeks {phrase}" pattern, returning (smaller, larger) values.
/// For example: "4 to 6 weeks before your average last frost" -> Some((4, 6))
fn extract_weeks_range(text: &str, phrase: &str) -> Option<(u8, u8)> {
    // Find the phrase in text
    let phrase_pos = text.find(phrase)?;

    // Look backwards from the phrase to find "X to Y weeks"
    let before_phrase = &text[..phrase_pos];

    // Find "weeks" just before the phrase
    let weeks_pos = before_phrase.rfind("weeks")?;

    // Get the text before "weeks" to extract the numbers
    let before_weeks = before_phrase[..weeks_pos].trim();

    // Try to find "X to Y" pattern
    if let Some(to_pos) = before_weeks.rfind(" to ") {
        let y_str = before_weeks[to_pos + 4..].trim();
        // Find X -- it's the last number before " to "
        let x_part = before_weeks[..to_pos].trim();
        let x_str = x_part.rsplit_once(' ').map(|(_, x)| x).unwrap_or(x_part);

        if let (Ok(x), Ok(y)) = (x_str.parse::<u8>(), y_str.parse::<u8>()) {
            let smaller = x.min(y);
            let larger = x.max(y);
            return Some((smaller, larger));
        }
    }

    // Try single number: "Y weeks {phrase}"
    let num_str = before_weeks.rsplit_once(' ').map(|(_, n)| n).unwrap_or(before_weeks);
    if let Ok(n) = num_str.parse::<u8>() {
        return Some((n, n));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_warm_season_two_phase_tomato() {
        // Tomato: "4 to 6 weeks before transplanting" + "1 to 2 weeks after last frost"
        let text = "Start Inside: RECOMMENDED. Sow seeds 4 to 6 weeks before transplanting. \
                     Transplant outdoors 1 to 2 weeks after your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.start_indoors_weeks_before, Some(6)); // larger = safer
        assert_eq!(timing.transplant_weeks_relative, Some(1)); // smaller = earliest safe
        assert!(timing.indoor_start_recommended);
    }

    #[test]
    fn test_warm_season_direct_sow_bean() {
        // Bean: "1 to 2 weeks after your average last frost date" (outdoor only)
        let text = "Sow Outside: Sow seeds 1 to 2 weeks after your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.start_indoors_weeks_before, None);
        assert_eq!(timing.direct_sow_weeks_relative, Some(1)); // smaller = earliest safe
        assert!(!timing.indoor_start_recommended);
    }

    #[test]
    fn test_cool_season_lettuce() {
        // Lettuce: "2 to 4 weeks before your average last frost date" (direct sow)
        let text = "Sow Outside: Sow seeds 2 to 4 weeks before your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.direct_sow_weeks_relative, Some(-4)); // negative = before frost, larger = safer
        assert_eq!(timing.start_indoors_weeks_before, None);
    }

    #[test]
    fn test_cool_season_start_indoors() {
        // Herb with indoor start recommended before frost
        let text = "Start Inside: RECOMMENDED. Start seeds 4 to 6 weeks before your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.start_indoors_weeks_before, Some(6)); // larger = safer
        assert!(timing.indoor_start_recommended);
    }

    #[test]
    fn test_long_start_rosemary() {
        // Rosemary: "10 to 12 weeks before your average last frost date"
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
        // "4 to 6 weeks before" should pick 6 (more conservative / earlier)
        let text = "Start Inside: RECOMMENDED. Start seeds 4 to 6 weeks before your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.start_indoors_weeks_before, Some(6));
    }

    #[test]
    fn test_after_frost_uses_smaller_number() {
        // "1 to 2 weeks after" should pick 1 (earliest safe date)
        let text = "Sow Outside: 1 to 2 weeks after your average last frost date.";
        let timing = parse_planting_timing(text);
        assert_eq!(timing.direct_sow_weeks_relative, Some(1));
    }
}
