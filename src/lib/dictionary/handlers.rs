use askama::Template;
use axum::{
    extract::{Multipart, Query},
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    Form,
};
use lazy_static::lazy_static;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, sync::RwLock};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::{error, info};

use crate::{
    dictionary::database::{
        add_term, add_term_to_term_set, current_epoch, delete_term, extract_and_insert_unique_values, get_all_terms, get_term_by_id, search_terms, search_terms_by_term_set_id, update_term, TermsList
    },
    import::{process::import_dictionary_data, parse::TermLanguageSet},
};

lazy_static! {
    static ref SEARCH_CACHE: RwLock<HashMap<String, Vec<TermsList>>> = RwLock::new(HashMap::new());
}

pub fn clear_cache() {
    SEARCH_CACHE.write().unwrap().clear();
}

const CURRENT_DB_NAME: &str = "term-squire.db";

// add term set to term
#[derive(Debug, Deserialize)]
pub struct AddTermSetRequest {
    existing_term_set_id: i32,
    term_language_set: TermLanguageSet,
}

pub async fn handle_add_term_set(Json(payload): Json<AddTermSetRequest>) -> impl IntoResponse {
    let db_name = CURRENT_DB_NAME;
    let existing_term_set_id = payload.existing_term_set_id;
    let mut term_set = payload.term_language_set;

    let now = current_epoch();
    term_set.creation_timestamp = Some(now);
    term_set.update_timestamp = Some(now);

    info!(
        "Adding term set to existing term ID: {}",
        existing_term_set_id
    );

    match add_term_to_term_set(db_name, existing_term_set_id, &term_set) {
        Ok(_) => {
            info!("Term set added successfully.");
            clear_cache();
            let _unique_values_result = extract_and_insert_unique_values(db_name);
            (
                StatusCode::OK,
                "Term set added to existing term successfully",
            )
                .into_response()
        }
        Err(err) => {
            error!("Failed to add term set: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to add term set to existing term: {}", err),
            )
                .into_response()
        }
    }
}

// delete term
#[derive(Debug, Deserialize)]
pub struct DeleteTermRequest {
    term_id: i32,
}

pub async fn handle_delete_term(Query(params): Query<DeleteTermRequest>) -> impl IntoResponse {
    let db_name = CURRENT_DB_NAME;
    let term_id = params.term_id;

    info!("Deleting term with ID: {}", term_id);

    match delete_term(db_name, term_id) {
        Ok(_) => {
            info!("Term deleted successfully.");
            clear_cache();
            let _unique_values_result = extract_and_insert_unique_values(db_name);
            (StatusCode::OK, "Term deleted successfully").into_response()
        }
        Err(rusqlite::Error::ExecuteReturnedResults) => {
            info!("Term not found or not deleted.");
            (
                StatusCode::NOT_FOUND,
                "Term not found or not deleted".to_string(),
            )
                .into_response()
        }
        Err(err) => {
            error!("Failed to delete term: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to delete term: {}", err),
            )
                .into_response()
        }
    }
}

// insert term
#[derive(Debug, Deserialize)]
pub struct InsertTermRequest {
    term_language_set: TermLanguageSet,
}

pub async fn handle_insert_term(Json(payload): Json<InsertTermRequest>) -> impl IntoResponse {
    let db_name = CURRENT_DB_NAME;
    let mut term_set = payload.term_language_set;

    let now = current_epoch();
    term_set.creation_timestamp = Some(now);
    term_set.update_timestamp = Some(now);

    info!("Inserting new term into database.");

    match add_term(db_name, &term_set) {
        Ok(_) => {
            info!("Term inserted successfully.");
            clear_cache();
            let _unique_values_result = extract_and_insert_unique_values(db_name);
            (StatusCode::OK, "Term inserted successfully").into_response()
        }
        Err(err) => {
            error!("Failed to insert term: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to insert term: {}", err),
            )
                .into_response()
        }
    }
}

// import dictionary data
pub async fn handle_import_dictionary_data(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(mut field) = match multipart.next_field().await {
        Ok(Some(field)) => Some(field),
        Ok(None) => None,
        Err(err) => {
            error!("Failed to read field: {}", err);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read field: {}", err),
            )
                .into_response();
        }
    } {
        let name = match field.name() {
            Some(name) => name.to_string(),
            None => {
                error!("Field does not have a name");
                return (
                    StatusCode::BAD_REQUEST,
                    "Field does not have a name".to_string(),
                )
                    .into_response();
            }
        };

        info!("Processing field: {}", name);

        if name == "dictionaryFile" {
            info!("Receiving dictionary file {}", name);

            let file_path = PathBuf::from("uploaded_dictionary.mtf");
            let mut file = match File::create(&file_path).await {
                Ok(file) => file,
                Err(err) => {
                    error!("Failed to create file: {}", err);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to create file: {}", err),
                    )
                        .into_response();
                }
            };

            while let Some(chunk) = match field.chunk().await {
                Ok(Some(chunk)) => Some(chunk),
                Ok(None) => {
                    info!("No more chunks available");
                    None
                }
                Err(err) => {
                    error!("Failed to read chunk: {}", err);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to read chunk: {}", err),
                    )
                        .into_response();
                }
            } {
                info!("Writing chunk of size {}", chunk.len());
                if let Err(err) = file.write_all(&chunk).await {
                    error!("Failed to write to file: {}", err);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to write to file: {}", err),
                    )
                        .into_response();
                }
            }

            if let Err(err) = file.sync_all().await {
                error!("Failed to flush file: {}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to flush file: {}", err),
                )
                    .into_response();
            }

            info!("Dictionary file {} uploaded successfully.", name);

            if let Err(err) = import_dictionary_data(file_path.to_string_lossy().as_ref()).await {
                error!("Failed to import dictionary: {}", err);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to import dictionary: {}", err),
                )
                    .into_response();
            }
            clear_cache();
            return (StatusCode::OK, "Dictionary imported successfully").into_response();
        }
    }

    (StatusCode::BAD_REQUEST, "No file was uploaded".to_string()).into_response()
}

// import dictionary file
#[derive(Template)]
#[template(path = "import_form.html")]
struct ImportFormTemplate;

pub async fn handle_import_form() -> Html<String> {
    info!("Serving import form.");
    let template = ImportFormTemplate;
    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template rendering error".to_string()),
    )
}

#[derive(Template)]
#[template(path = "insert_form.html")]
pub struct InsertFormTemplate;

pub async fn handle_insert_form() -> impl IntoResponse {
    info!("Serving insert form.");
    let template = InsertFormTemplate;
    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template rendering error".to_string()),
    )
}

// settings
#[derive(Deserialize)]
pub struct SettingsForm {
    columns: Vec<String>,
}

#[derive(Template)]
#[template(path = "settings.html")]
pub struct SettingsTemplate {
    pub columns: Vec<String>,
}

pub async fn handle_settings_update(Form(form): Form<SettingsForm>) -> impl IntoResponse {
    let selected_columns = form.columns;

    info!(
        "Processing settings update with columns: {:?}",
        selected_columns
    );

    let template = SettingsTemplate {
        columns: selected_columns,
    };

    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template rendering error".to_string()),
    )
}

pub async fn handle_get_settings() -> impl IntoResponse {
    let stored_columns = vec![];

    info!("Fetching stored settings.");

    let template = SettingsTemplate {
        columns: stored_columns,
    };
    Html(
        template
            .render()
            .unwrap_or_else(|_| "Template rendering error".to_string()),
    )
}

// terms
#[derive(Template)]
#[template(path = "terms.html")]
pub struct TermsTemplate {
    pub terms: Vec<TermsList>,
}

pub async fn handle_terms() -> Html<String> {
    let db_name = CURRENT_DB_NAME;

    info!("Fetching terms.");

    match get_all_terms(db_name) {
        Ok(terms) => {
            info!("Terms fetched successfully.");
            let template = TermsTemplate { terms };
            Html(
                template
                    .render()
                    .unwrap_or_else(|_| "Template rendering error".to_string()),
            )
        }
        Err(err) => {
            error!("Failed to get data: {}", err);
            Html(format!("<h1>Failed to get data: {}</h1>", err))
        }
    }
}

// term details
#[derive(Debug, Deserialize)]
pub struct TermDetailRequest {
    term_id: i32,
}

#[derive(Template)]
#[template(path = "term_detail.html")]
pub struct TermDetailTemplate {
    pub term: TermsList,
}

pub async fn handle_get_term_details(Query(params): Query<TermDetailRequest>) -> impl IntoResponse {
    let db_name = CURRENT_DB_NAME;
    let term_id = params.term_id;

    info!("Fetching details for term ID: {}", term_id);

    match get_term_by_id(db_name, term_id) {
        Ok(Some(term)) => {
            info!("Term details fetched successfully.");
            let template = TermDetailTemplate { term };
            Html(
                template
                    .render()
                    .unwrap_or_else(|_| "Template rendering error".to_string()),
            )
        }
        Ok(None) => {
            info!("Term not found for ID: {}", term_id);
            Html("<h1>Term not found</h1>".to_string())
        }
        Err(err) => {
            error!("Failed to get term details: {}", err);
            Html(format!("<h1>Failed to get term details: {}</h1>", err))
        }
    }
}

// search terms list
#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    term: String,
    language: String,
}

#[derive(Template)]
#[template(path = "search_form.html")]
pub struct SearchResultsTemplate {
    pub terms: Vec<TermsList>,
    pub count: usize,
}

pub async fn handle_search_terms(Query(params): Query<SearchRequest>) -> impl IntoResponse {
    let db_name = CURRENT_DB_NAME;
    let term_select = params.term.clone();
    let language_select = params.language.clone();
    let cache_key = format!("{}:{}", term_select, language_select);

    // Check the cache first
    if let Some(cached_results) = SEARCH_CACHE.read().unwrap().get(&cache_key) {
        info!(
            "Cache hit for search term: {} and language: {}",
            term_select, language_select
        );
        return Json(cached_results.clone());
    }

    info!(
        "Cache miss for search term: {} and language: {}",
        term_select, language_select
    );

    // Perform the database search if cache miss
    match search_terms(db_name, &term_select, &language_select) {
        Ok(terms) => {
            info!("Search completed successfully.");
            // Update the cache
            SEARCH_CACHE
                .write()
                .unwrap()
                .insert(cache_key, terms.clone());
            Json(terms)
        }
        Err(err) => {
            error!("Failed to search terms: {}", err);
            Json(vec![]) 
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchByTermSetIdRequest {
    term_set_id: i32,
}

pub async fn handle_search_terms_by_term_set_id(Query(params): Query<SearchByTermSetIdRequest>) -> impl IntoResponse {
    let db_name = CURRENT_DB_NAME;
    let term_set_id = params.term_set_id;

    info!("Searching for terms with term_set_id: {}", term_set_id);

    match search_terms_by_term_set_id(db_name, term_set_id) {
        Ok(terms) => {
            info!("Search completed successfully.");
            Json(terms)
        }
        Err(err) => {
            error!("Failed to search terms: {}", err);
            Json(vec![]) 
        }
    }
}

// update term
#[derive(Debug, Deserialize)]
pub struct UpdateTermRequest {
    term_id: i32,
    term_language_set: TermLanguageSet,
}

pub async fn handle_update_term(Json(payload): Json<UpdateTermRequest>) -> impl IntoResponse {
    let db_name = CURRENT_DB_NAME;
    let term_id = payload.term_id;
    let mut term_set = payload.term_language_set;

    let now = current_epoch();
    term_set.update_timestamp = Some(now);

    info!("Updating term ID: {}", term_id);

    match update_term(db_name, term_id, &term_set) {
        Ok(_) => {
            info!("Term updated successfully.");
            clear_cache();
            let _unique_values_result = extract_and_insert_unique_values(db_name);
            (StatusCode::OK, "Term updated successfully").into_response()
        }
        Err(err) => {
            error!("Failed to update term: {}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to update term: {}", err),
            )
                .into_response()
        }
    }
}
