use std::sync::Arc;

use crate::dictionary::database::*;
use crate::import::parse::*;
use axum::extract::State;
use rusqlite::Result;
use tracing::{error, info};

pub fn create_term_to_insert(term: &TermLanguageSet) -> TermLanguageSet {
    TermLanguageSet {
        term: term.term.clone(),
        language: term.language.clone(),
        term_type: term.term_type.clone(),
        creator_id: term.creator_id.clone(),
        creation_timestamp: term.creation_timestamp.clone(),
        updater_id: term.updater_id.clone(),
        update_timestamp: term.update_timestamp.clone(),
        subject: term.subject.clone(),
        source: term.source.clone(),
        user: term.user.clone(),
        attributes: term.attributes.clone(),
        remark: term.remark.clone(),
        url: term.url.clone(),
        context: term.context.clone(),
        definition: term.definition.clone(),
    }
}
pub async fn import_dictionary_data(
    State(app_state): State<Arc<AppState>>,
    filename: &str,
) -> Result<(), String> {
    info!("Importing dictionary from file: {}", filename);

    let mut dictionary = Dictionary::new();
    dictionary.import_from_xml(filename);

    let _ = dictionary.serialize_to_json("processed_dictionary.json");

    create_terms_table(State(app_state.clone()));

    for dt in &dictionary.entries {
        if let Err(err) = process_term_set(State(app_state.clone()), dt) {
            error!("Error processing term set: {}", err);
            return Err(err.to_string());
        }
    }

    let unique_values_result = extract_and_insert_unique_values(State(app_state.clone()));
    handle_insert_unique_values_result(unique_values_result);

    info!("Dictionary import completed");
    Ok(())
}

pub fn process_term_set(
    State(app_state): State<Arc<AppState>>,
    dt: &DictionaryEntry,
) -> Result<()> {
    info!("Processing term set: {:?}", dt.id);

    match dt.language_sets.len() {
        2 => process_two_terms(
            State(app_state.clone()),
            &dt.language_sets[0],
            &dt.language_sets[1],
        ),
        1 => process_single_term(State(app_state.clone()), &dt.language_sets[0]),
        n if n > 2 => process_three_or_more_terms(State(app_state.clone()), &dt.language_sets),
        _ => {
            error!("Unexpected number of terms in term set");
            Ok(())
        }
    }
}

pub fn process_single_term(
    State(app_state): State<Arc<AppState>>,
    term: &TermLanguageSet,
) -> Result<()> {
    let term_to_insert = create_term_to_insert(term);
    info!("Inserting single term: {:?}", term_to_insert);

    if let Err(err) = add_term(State(app_state.clone()), &term_to_insert) {
        error!("Failed to insert single term: {}", err);
        return Err(err);
    }
    Ok(())
}
pub fn process_two_terms(
    State(app_state): State<Arc<AppState>>,
    first_term: &TermLanguageSet,
    second_term: &TermLanguageSet,
) -> Result<()> {
    let first_term_to_insert = create_term_to_insert(first_term);
    let second_term_to_insert = create_term_to_insert(second_term);

    let (primary_term, secondary_term) = if first_term_to_insert.term.is_some() {
        (first_term_to_insert, second_term_to_insert)
    } else if second_term_to_insert.term.is_some() {
        (second_term_to_insert, first_term_to_insert)
    } else {
        error!(
            "Both terms are missing 'term' values: {:?}, {:?}",
            first_term_to_insert, second_term_to_insert
        );
        return Err(rusqlite::Error::InvalidQuery);
    };

    info!("Inserting primary term: {:?}", primary_term);
    if let Err(err) = add_term(State(app_state.clone()), &primary_term) {
        error!("Failed to insert primary term: {}", err);
        return Err(err);
    }

    let term_set_id = get_term_set_id(
        State(app_state.clone()),
        primary_term.term.as_ref().unwrap(),
        primary_term.language.as_ref().unwrap(),
    )?;

    if let Some(id) = term_set_id {
        info!("Term set ID: {:?}", id);
        info!(
            "Inserting secondary term into term set: {:?}",
            secondary_term
        );
        if let Err(err) = add_term_to_term_set(State(app_state.clone()), id, &secondary_term) {
            error!("Failed to add secondary term to set: {}", err);
            return Err(err);
        }
    } else {
        error!(
            "Failed to retrieve term set ID for primary term {:?}",
            primary_term.term
        );
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }

    Ok(())
}

pub fn process_three_or_more_terms(
    State(app_state): State<Arc<AppState>>,
    terms: &[TermLanguageSet],
) -> Result<()> {
    let mut term_set_id: Option<i32> = None;
    let mut primary_term_to_insert: Option<TermLanguageSet> = None;

    for term in terms {
        let term_to_insert = create_term_to_insert(term);

        if term_to_insert.term.is_some() {
            info!("Inserting primary term: {:?}", term_to_insert);

            add_term(State(app_state.clone()), &term_to_insert)?;

            term_set_id = get_term_set_id(
                State(app_state.clone()),
                term_to_insert.term.as_ref().unwrap(),
                term_to_insert.language.as_ref().unwrap(),
            )?;

            if term_set_id.is_none() {
                error!(
                    "Failed to retrieve term set ID for primary term {:?}",
                    term_to_insert.term
                );
                return Err(rusqlite::Error::InvalidQuery);
            }

            primary_term_to_insert = Some(term_to_insert);
            break;
        }
    }

    let term_set_id = match term_set_id {
        Some(id) => id,
        None => {
            error!("No primary term found to establish a term set ID");
            return Err(rusqlite::Error::InvalidQuery);
        }
    };

    for (i, term) in terms.iter().enumerate() {
        let term_to_insert = create_term_to_insert(term);

        if let Some(primary_term) = &primary_term_to_insert {
            if term_to_insert.term == primary_term.term
                && term_to_insert.language == primary_term.language
            {
                continue;
            }
        }

        info!(
            "Adding term {} to set ID {}: {:?}",
            i, term_set_id, term_to_insert
        );
        add_term_to_term_set(State(app_state.clone()), term_set_id, &term_to_insert)?;
    }

    Ok(())
}
