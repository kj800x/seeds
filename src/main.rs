mod db;
mod error;
mod routes;
mod schedule;
mod scraper;
mod search;
mod templates;
mod viability;

use axum::Router;
use axum::routing::{delete, get, post};
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
        .route("/search", get(routes::home::search))
        .route("/seeds/{id}", get(routes::seeds::seed_detail)
                              .delete(routes::seeds::delete_seed_handler))
        .route("/seeds/{id}/purchases", get(routes::seeds::purchases_fragment)
                                        .post(routes::seeds::add_purchase_handler))
        .route("/seeds/{id}/purchases/{purchase_id}", delete(routes::seeds::delete_purchase_handler)
                                                      .put(routes::seeds::update_purchase_handler))
        .route("/seeds/{id}/purchases/{purchase_id}/edit", get(routes::seeds::edit_purchase_form))
        .route("/seeds/{id}/events", get(routes::seeds::events_fragment)
                                     .post(routes::seeds::add_event_handler))
        .route("/seeds/{id}/events/{event_id}", delete(routes::seeds::delete_event_handler)
                                                 .put(routes::seeds::update_event_handler))
        .route("/seeds/{id}/events/{event_id}/edit", get(routes::seeds::edit_event_form))
        .route("/seeds/add", post(routes::seeds::add_seed))
        .route("/seeds/reparse", post(routes::seeds::reparse_all))
        .route("/schedule", get(routes::schedule::schedule_page))
        .route("/schedule/list", get(routes::schedule::schedule_list))
        .route("/schedule/week", get(routes::schedule::this_week))
        .route("/plan/toggle/{seed_id}", post(routes::seeds::toggle_plan))
        .route("/plan/{seed_id}/start-method", post(routes::seeds::set_start_method))
        .route("/settings", get(routes::settings::settings_page))
        .route("/settings/reset-all-data", post(routes::settings::reset_all_data))
        .nest_service("/static", ServeDir::new("static"))
        .nest_service("/images", ServeDir::new("data/images"))
        .with_state(state);

    panic!("OH NO");

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{port}");

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|_| panic!("Failed to bind to {addr}"));

    tracing::info!("Seeds app listening on http://{addr}");

    axum::serve(listener, app)
        .await
        .expect("Server error");
}
