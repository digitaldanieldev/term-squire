use axum::extract::DefaultBodyLimit;
use axum::routing::{delete, get, post};
use axum::Router;
use clap::{arg, command, Parser};
use term_squire::dictionary::{database::*, handlers::*};
use term_squire::logging::*;
use tracing::info;

const CURRENT_DB_NAME: &str = "term-squire.db";

/// simple dictionary
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port number used for server
    #[arg(short, long, default_value_t = 1234)]
    port: u64,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let args = Args::parse();

    init_db(CURRENT_DB_NAME);

    let app = Router::new()
        .route("/add_term_set", post(handle_add_term_set))
        .route("/delete_term", delete(handle_delete_term))
        .route("/import_dictionary", post(handle_import_dictionary_data))
        .route("/import_form", get(handle_import_form))
        .route("/insert_form", get(handle_insert_form))
        .route("/insert_term", post(handle_insert_term))
        .route("/search", get(handle_search_terms))
        .route("/search_terms_by_term_set_id", get(handle_search_terms_by_term_set_id))
        .route("/settings", get(handle_get_settings))
        .route("/terms", get(handle_terms))
        .route("/term_detail", get(handle_get_term_details))
        .route("/update_term", post(handle_update_term))
        .layer(DefaultBodyLimit::max(10 * 1024 * 1024))
        .into_make_service();

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port))
        .await
        .unwrap();
    info!("Starting server on port {}", args.port);
    axum::serve(listener, app).await.unwrap();
}
