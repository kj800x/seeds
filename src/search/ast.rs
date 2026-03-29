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

    // Plan status
    InPlan,
    Plan(String), // start method: "indoor" or "outdoor"

    // Timing predicates
    Start(DatePredicate),
    Sow(DatePredicate),
    Transplant(DatePredicate),

    // Viability
    Viable,
    Viability(Comparison),
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
