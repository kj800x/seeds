#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    And(Vec<Filter>),
    Or(Vec<Filter>),
    Not(Box<Filter>),

    // Field substring match (case-insensitive)
    Title(String),
    Category(String),
    Subcategory(String),

    // Boolean flags
    Organic,
    Heirloom,

    // Plan status: None = any active, Some("indoor"/"outdoor"/"skipped")
    Plan(Option<String>),

    // Timing predicates
    Start(DatePredicate),
    Sow(DatePredicate),
    Transplant(DatePredicate),

    // Viability
    Viable,
    Viability(Comparison),
}

impl Filter {
    /// Check if this filter (or any nested child) references skipped plan status.
    pub fn references_skipped(&self) -> bool {
        match self {
            Filter::Plan(Some(s)) if s == "skipped" => true,
            Filter::And(children) | Filter::Or(children) => children.iter().any(|c| c.references_skipped()),
            Filter::Not(child) => child.references_skipped(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DatePredicate {
    /// Currently in the action window
    Now,
    Before(DateValue),
    After(DateValue),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DateValue {
    Now,
    Absolute(String),
    Relative { amount: i64, unit: TimeUnit, direction: Direction },
}

#[derive(Debug, Clone, PartialEq)]
pub enum TimeUnit {
    Days,
    Weeks,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Ago,
    FromNow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Comparison {
    Above(u8),
    Below(u8),
}
