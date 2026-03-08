mod db;
mod error;
mod routes;
mod scraper;
mod templates;
mod viability;

use axum::Router;
use axum::routing::{get, post};
use sqlx::SqlitePool;
use tower_http::services::ServeDir;

use db::models::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Create data directories if they don't exist
    std::fs::create_dir_all("data/images").expect("Failed to create data/images directory");

    let pool = SqlitePool::connect("sqlite:data/seeds.db?mode=rwc")
        .await
        .expect("Failed to connect to database");

    // Enable WAL mode for better concurrent read/write performance
    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(&pool)
        .await
        .expect("Failed to set WAL journal mode");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    let state = AppState {
        db: pool,
        data_dir: std::path::PathBuf::from("data"),
    };

    let app = Router::new()
        .route("/", get(routes::home::home))
        .route("/seeds/{id}", get(routes::seeds::seed_detail)
                              .put(routes::seeds::update_seed_handler)
                              .delete(routes::seeds::delete_seed_handler))
        .route("/seeds/{id}/edit", get(routes::seeds::edit_seed_form))
        .route("/seeds/{id}/info", get(routes::seeds::seed_info_fragment))
        .route("/seeds/add", post(routes::seeds::add_seed))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/images", ServeDir::new("data/images"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind to port 3000");

    tracing::info!("Seeds app listening on http://localhost:3000");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
