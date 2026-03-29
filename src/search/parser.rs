use crate::search::ast::*;
use crate::search::token::Token;

pub fn parse(tokens: &[Token]) -> Result<Filter, String> {
    let (filter, pos) = parse_filter(tokens, 0)?;
    if pos != tokens.len() {
        return Err(format!("unexpected token after expression at position {pos}"));
    }
    Ok(filter)
}

fn parse_filter(tokens: &[Token], pos: usize) -> Result<(Filter, usize), String> {
    let token = tokens.get(pos).ok_or("unexpected end of input")?;
    match token {
        Token::LParen => {
            let pos = pos + 1;
            let sym = match tokens.get(pos) {
                Some(Token::Symbol(s)) => s.as_str(),
                Some(t) => return Err(format!("expected function name, got {t:?}")),
                None => return Err("unexpected end of input after '('".to_string()),
            };
            let pos = pos + 1;
            match sym {
                "and" => parse_variadic(tokens, pos, |children| Filter::And(children)),
                "or" => parse_variadic(tokens, pos, |children| Filter::Or(children)),
                "not" => {
                    let (child, pos) = parse_filter(tokens, pos)?;
                    let pos = expect_rparen(tokens, pos)?;
                    Ok((Filter::Not(Box::new(child)), pos))
                }
                "title" => parse_string_arg(tokens, pos, Filter::Title),
                "category" => parse_string_arg(tokens, pos, Filter::Category),
                "subcategory" => parse_string_arg(tokens, pos, Filter::Subcategory),
                "organic" => {
                    let pos = expect_rparen(tokens, pos)?;
                    Ok((Filter::Organic, pos))
                }
                "heirloom" => {
                    let pos = expect_rparen(tokens, pos)?;
                    Ok((Filter::Heirloom, pos))
                }
                "plan" => {
                    // (plan) or (plan "indoor") / (plan "outdoor") / (plan "skipped")
                    if matches!(tokens.get(pos), Some(Token::RParen)) {
                        let pos = expect_rparen(tokens, pos)?;
                        Ok((Filter::Plan(None), pos))
                    } else {
                        let s = expect_str(tokens, pos)?;
                        let pos = pos + 1;
                        let pos = expect_rparen(tokens, pos)?;
                        Ok((Filter::Plan(Some(s)), pos))
                    }
                }
                "start" => {
                    let (pred, pos) = parse_date_predicate(tokens, pos)?;
                    let pos = expect_rparen(tokens, pos)?;
                    Ok((Filter::Start(pred), pos))
                }
                "sow" => {
                    let (pred, pos) = parse_date_predicate(tokens, pos)?;
                    let pos = expect_rparen(tokens, pos)?;
                    Ok((Filter::Sow(pred), pos))
                }
                "transplant" => {
                    let (pred, pos) = parse_date_predicate(tokens, pos)?;
                    let pos = expect_rparen(tokens, pos)?;
                    Ok((Filter::Transplant(pred), pos))
                }
                "viable" => {
                    let pos = expect_rparen(tokens, pos)?;
                    Ok((Filter::Viable, pos))
                }
                "viability" => {
                    let (cmp, pos) = parse_comparison(tokens, pos)?;
                    let pos = expect_rparen(tokens, pos)?;
                    Ok((Filter::Viability(cmp), pos))
                }
                other => Err(format!("unknown function: {other}")),
            }
        }
        _ => Err(format!("expected '(', got {token:?}")),
    }
}

fn parse_variadic(
    tokens: &[Token],
    mut pos: usize,
    make: impl FnOnce(Vec<Filter>) -> Filter,
) -> Result<(Filter, usize), String> {
    let mut children = Vec::new();
    while pos < tokens.len() && tokens[pos] != Token::RParen {
        let (child, next) = parse_filter(tokens, pos)?;
        children.push(child);
        pos = next;
    }
    let pos = expect_rparen(tokens, pos)?;
    Ok((make(children), pos))
}

fn parse_string_arg(
    tokens: &[Token],
    pos: usize,
    make: impl FnOnce(String) -> Filter,
) -> Result<(Filter, usize), String> {
    let s = expect_str(tokens, pos)?;
    let pos = pos + 1;
    let pos = expect_rparen(tokens, pos)?;
    Ok((make(s), pos))
}

fn parse_date_predicate(tokens: &[Token], pos: usize) -> Result<(DatePredicate, usize), String> {
    match tokens.get(pos) {
        Some(Token::Symbol(s)) if s == "now" => Ok((DatePredicate::Now, pos + 1)),
        Some(Token::LParen) => {
            let pos = pos + 1;
            let sym = match tokens.get(pos) {
                Some(Token::Symbol(s)) => s.as_str(),
                _ => return Err("expected 'before' or 'after'".to_string()),
            };
            let pos = pos + 1;
            let (val, pos) = parse_date_value(tokens, pos)?;
            let pos = expect_rparen(tokens, pos)?;
            match sym {
                "before" => Ok((DatePredicate::Before(val), pos)),
                "after" => Ok((DatePredicate::After(val), pos)),
                other => Err(format!("expected 'before' or 'after', got '{other}'")),
            }
        }
        Some(t) => Err(format!("expected 'now' or '(before ...)' / '(after ...)', got {t:?}")),
        None => Err("unexpected end of input in date predicate".to_string()),
    }
}

fn parse_date_value(tokens: &[Token], pos: usize) -> Result<(DateValue, usize), String> {
    match tokens.get(pos) {
        Some(Token::Symbol(s)) if s == "now" => Ok((DateValue::Now, pos + 1)),
        Some(Token::Str(s)) => {
            let val = parse_date_string(s)?;
            Ok((val, pos + 1))
        }
        Some(t) => Err(format!("expected date value, got {t:?}")),
        None => Err("unexpected end of input in date value".to_string()),
    }
}

fn parse_date_string(s: &str) -> Result<DateValue, String> {
    // Try relative: "N week(s) ago", "N day(s) ago", "N week(s) from now"
    let lower = s.to_lowercase();
    let parts: Vec<&str> = lower.split_whitespace().collect();
    if parts.len() >= 3 {
        if let Ok(amount) = parts[0].parse::<i64>() {
            let unit = if parts[1].starts_with("week") {
                Some(TimeUnit::Weeks)
            } else if parts[1].starts_with("day") {
                Some(TimeUnit::Days)
            } else {
                None
            };
            if let Some(unit) = unit {
                if parts[2] == "ago" {
                    return Ok(DateValue::Relative { amount, unit, direction: Direction::Ago });
                } else if parts.len() >= 4 && parts[2] == "from" && parts[3] == "now" {
                    return Ok(DateValue::Relative { amount, unit, direction: Direction::FromNow });
                }
            }
        }
    }
    // Otherwise treat as absolute date string
    Ok(DateValue::Absolute(s.to_string()))
}

fn parse_comparison(tokens: &[Token], pos: usize) -> Result<(Comparison, usize), String> {
    match tokens.get(pos) {
        Some(Token::LParen) => {
            let pos = pos + 1;
            let sym = match tokens.get(pos) {
                Some(Token::Symbol(s)) => s.as_str(),
                _ => return Err("expected 'above' or 'below'".to_string()),
            };
            let pos = pos + 1;
            let val = match tokens.get(pos) {
                Some(Token::Symbol(s)) => s.parse::<u8>().map_err(|_| format!("expected number, got '{s}'"))?,
                Some(t) => return Err(format!("expected number, got {t:?}")),
                None => return Err("unexpected end of input in comparison".to_string()),
            };
            let pos = pos + 1;
            let pos = expect_rparen(tokens, pos)?;
            match sym {
                "above" => Ok((Comparison::Above(val), pos)),
                "below" => Ok((Comparison::Below(val), pos)),
                other => Err(format!("expected 'above' or 'below', got '{other}'")),
            }
        }
        Some(t) => Err(format!("expected '(above N)' or '(below N)', got {t:?}")),
        None => Err("unexpected end of input in comparison".to_string()),
    }
}

fn expect_rparen(tokens: &[Token], pos: usize) -> Result<usize, String> {
    match tokens.get(pos) {
        Some(Token::RParen) => Ok(pos + 1),
        Some(t) => Err(format!("expected ')', got {t:?}")),
        None => Err("expected ')', got end of input".to_string()),
    }
}

fn expect_str(tokens: &[Token], pos: usize) -> Result<String, String> {
    match tokens.get(pos) {
        Some(Token::Str(s)) => Ok(s.clone()),
        Some(t) => Err(format!("expected string argument, got {t:?}")),
        None => Err("expected string argument, got end of input".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::token::tokenize;

    fn parse_str(input: &str) -> Result<Filter, String> {
        let tokens = tokenize(input)?;
        parse(&tokens)
    }

    #[test]
    fn test_category() {
        assert_eq!(
            parse_str("(category \"Vegetables\")").unwrap(),
            Filter::Category("Vegetables".into()),
        );
    }

    #[test]
    fn test_and() {
        assert_eq!(
            parse_str("(and (organic) (plan))").unwrap(),
            Filter::And(vec![Filter::Organic, Filter::Plan(None)]),
        );
    }

    #[test]
    fn test_or() {
        assert_eq!(
            parse_str("(or (category \"Flower\") (category \"Herb\"))").unwrap(),
            Filter::Or(vec![
                Filter::Category("Flower".into()),
                Filter::Category("Herb".into()),
            ]),
        );
    }

    #[test]
    fn test_not() {
        assert_eq!(
            parse_str("(not (organic))").unwrap(),
            Filter::Not(Box::new(Filter::Organic)),
        );
    }

    #[test]
    fn test_start_now() {
        assert_eq!(
            parse_str("(start now)").unwrap(),
            Filter::Start(DatePredicate::Now),
        );
    }

    #[test]
    fn test_start_before_now() {
        assert_eq!(
            parse_str("(start (before now))").unwrap(),
            Filter::Start(DatePredicate::Before(DateValue::Now)),
        );
    }

    #[test]
    fn test_start_before_date() {
        assert_eq!(
            parse_str("(start (before \"March 30\"))").unwrap(),
            Filter::Start(DatePredicate::Before(DateValue::Absolute("March 30".into()))),
        );
    }

    #[test]
    fn test_relative_date() {
        assert_eq!(
            parse_str("(start (before \"1 week ago\"))").unwrap(),
            Filter::Start(DatePredicate::Before(DateValue::Relative {
                amount: 1,
                unit: TimeUnit::Weeks,
                direction: Direction::Ago,
            })),
        );
    }

    #[test]
    fn test_transplant_after() {
        assert_eq!(
            parse_str("(transplant (after \"April 15\"))").unwrap(),
            Filter::Transplant(DatePredicate::After(DateValue::Absolute("April 15".into()))),
        );
    }

    #[test]
    fn test_plan_no_arg() {
        assert_eq!(
            parse_str("(plan)").unwrap(),
            Filter::Plan(None),
        );
    }

    #[test]
    fn test_plan_method() {
        assert_eq!(
            parse_str("(plan \"indoor\")").unwrap(),
            Filter::Plan(Some("indoor".into())),
        );
    }

    #[test]
    fn test_plan_skipped() {
        assert_eq!(
            parse_str("(plan \"skipped\")").unwrap(),
            Filter::Plan(Some("skipped".into())),
        );
    }

    #[test]
    fn test_viability() {
        assert_eq!(
            parse_str("(viability (above 50))").unwrap(),
            Filter::Viability(Comparison::Above(50)),
        );
    }

    #[test]
    fn test_complex_query() {
        assert_eq!(
            parse_str("(and (category \"Vegetables\") (plan) (start now))").unwrap(),
            Filter::And(vec![
                Filter::Category("Vegetables".into()),
                Filter::Plan(None),
                Filter::Start(DatePredicate::Now),
            ]),
        );
    }

    #[test]
    fn test_unknown_function() {
        assert!(parse_str("(bogus)").is_err());
    }

    #[test]
    fn test_trailing_tokens() {
        assert!(parse_str("(organic) (heirloom)").is_err());
    }
}
