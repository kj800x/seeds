#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LParen,
    RParen,
    Symbol(String),
    Str(String),
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => { chars.next(); }
            '(' => { chars.next(); tokens.push(Token::LParen); }
            ')' => { chars.next(); tokens.push(Token::RParen); }
            '"' => {
                chars.next(); // consume opening quote
                let mut s = String::new();
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some(c) => s.push(c),
                        None => return Err("unterminated string".to_string()),
                    }
                }
                tokens.push(Token::Str(s));
            }
            _ => {
                let mut sym = String::new();
                while let Some(&c) = chars.peek() {
                    if c == ' ' || c == '\t' || c == '\n' || c == '\r' || c == '(' || c == ')' {
                        break;
                    }
                    sym.push(c);
                    chars.next();
                }
                tokens.push(Token::Symbol(sym));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expr() {
        let tokens = tokenize("(category \"Vegetables\")").unwrap();
        assert_eq!(tokens, vec![
            Token::LParen,
            Token::Symbol("category".into()),
            Token::Str("Vegetables".into()),
            Token::RParen,
        ]);
    }

    #[test]
    fn test_nested() {
        let tokens = tokenize("(and (organic) (in-plan))").unwrap();
        assert_eq!(tokens, vec![
            Token::LParen, Token::Symbol("and".into()),
            Token::LParen, Token::Symbol("organic".into()), Token::RParen,
            Token::LParen, Token::Symbol("in-plan".into()), Token::RParen,
            Token::RParen,
        ]);
    }

    #[test]
    fn test_symbol_with_hyphens() {
        let tokens = tokenize("(in-plan)").unwrap();
        assert_eq!(tokens, vec![
            Token::LParen, Token::Symbol("in-plan".into()), Token::RParen,
        ]);
    }

    #[test]
    fn test_unterminated_string() {
        assert!(tokenize("(title \"hello)").is_err());
    }

    #[test]
    fn test_bare_symbol() {
        let tokens = tokenize("(start now)").unwrap();
        assert_eq!(tokens, vec![
            Token::LParen, Token::Symbol("start".into()), Token::Symbol("now".into()), Token::RParen,
        ]);
    }
}
