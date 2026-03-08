use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    DbError(sqlx::Error),
    ScraperError(String),
    NotFound(String),
    DuplicateSeed { existing_id: i64 },
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DbError(e) => write!(f, "Database error: {}", e),
            AppError::ScraperError(msg) => write!(f, "Scraper error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::DuplicateSeed { existing_id } => {
                write!(f, "This seed is already in your collection (id: {})", existing_id)
            }
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DbError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::DbError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "A database error occurred.",
            ),
            AppError::ScraperError(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.as_str()),
            AppError::DuplicateSeed { .. } => (
                StatusCode::CONFLICT,
                "This seed is already in your collection.",
            ),
        };

        let body = format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Error | Seeds</title>
    <link rel="stylesheet" href="/static/style.css">
</head>
<body>
    <header class="app-header">
        <h1 class="logo">Seeds</h1>
        <nav class="main-nav">
            <a class="nav-link active" href="/">Seeds</a>
            <span class="nav-link disabled">Inventory</span>
            <span class="nav-link disabled">Schedule</span>
        </nav>
    </header>
    <main class="content">
        <div class="error-message">
            <h2>Error {}</h2>
            <p>{}</p>
            <a href="/">Back to Seeds</a>
        </div>
    </main>
</body>
</html>"#,
            status.as_u16(),
            message,
        );

        (status, Html(body)).into_response()
    }
}
