pub mod ast;
pub mod eval;
pub mod parser;
pub mod token;

pub use ast::Filter;
pub use eval::{SeedContext, matches};

pub enum SearchQuery {
    Plaintext(String),
    SExp(Filter),
}

pub fn parse_query(input: &str) -> Result<SearchQuery, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(SearchQuery::Plaintext(String::new()));
    }
    if trimmed.starts_with('(') {
        let tokens = token::tokenize(trimmed)?;
        let filter = parser::parse(&tokens)?;
        Ok(SearchQuery::SExp(filter))
    } else {
        Ok(SearchQuery::Plaintext(trimmed.to_lowercase()))
    }
}
