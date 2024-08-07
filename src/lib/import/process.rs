use crate::dictionary::database::*;
use crate::import::parse::*;
use rusqlite::Result;
use tracing::{error, info};

const CURRENT_DB_NAME: &str = "term-squire.db";

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

pub async fn import_dictionary_data(filename: &str) -> Result<(), String> {
    let db_name = CURRENT_DB_NAME;

    info!("Importing dictionary from file: {}", filename);

    let mut dictionary = Dictionary::new();
    dictionary.import_from_xml(filename);

    let _ = dictionary.serialize_to_json("processed_dictionary.json");

    create_terms_table(CURRENT_DB_NAME);

    for dt in dictionary.entries {
        if let Err(err) = process_term_set(&dt) {
            error!("Error processing term set: {}", err);
            return Err(err.to_string());
        }
    }
    let unique_values_result = extract_and_insert_unique_values(db_name);
    handle_insert_unique_values_result(unique_values_result);
    info!("Dictionary import completed");
    Ok(())
}

pub fn process_term_set(dt: &DictionaryEntry) -> Result<()> {
    info!("Processing term set: {:?}", dt.id);

    match dt.language_sets.len() {
        2 => process_two_terms(&dt.language_sets[0], &dt.language_sets[1]),
        1 => process_single_term(&dt.language_sets[0]),
        n if n > 2 => process_three_or_more_terms(&dt.language_sets),
        _ => {
            error!("Unexpected number of terms in term set");
            Ok(())
        }
    }
}

pub fn process_single_term(term: &TermLanguageSet) -> Result<()> {
    let term_to_insert = create_term_to_insert(term);
    info!("Inserting single term: {:?}", term_to_insert);

    if let Err(err) = add_term(CURRENT_DB_NAME, &term_to_insert) {
        error!("Failed to insert single term: {}", err);
        return Err(err);
    }
    Ok(())
}

pub fn process_two_terms(
    first_term: &TermLanguageSet,
    second_term: &TermLanguageSet,
) -> Result<()> {
    let first_term_to_insert = create_term_to_insert(first_term);
    info!("Inserting first of two terms: {:?}", first_term_to_insert);

    if let Err(err) = add_term(CURRENT_DB_NAME, &first_term_to_insert) {
        error!("Failed to insert first term: {}", err);
        return Err(err);
    }

    let term_set_id = get_term_set_id(
        CURRENT_DB_NAME,
        &first_term_to_insert.term.as_ref().unwrap(),
        &first_term_to_insert.language.as_ref().unwrap(),
    )?;

    if let Some(id) = term_set_id {
        info!("Term set ID: {:?}", id);
        let second_term_to_insert = create_term_to_insert(second_term);
        info!("Inserting second of two terms: {:?}", second_term_to_insert);
        if let Err(err) = add_term_to_term_set(CURRENT_DB_NAME, id, &second_term_to_insert) {
            error!("Failed to add term to set: {}", err);
            return Err(err);
        }
    } else {
        error!(
            "Failed to retrieve term set ID for term {:?}",
            first_term_to_insert.term
        );
    }

    Ok(())
}

pub fn process_three_or_more_terms(terms: &[TermLanguageSet]) -> Result<()> {
    let mut term_set_id: Option<i32> = None;

    for (i, term) in terms.iter().enumerate() {
        let term_to_insert = create_term_to_insert(term);
        info!("Processing term {}: {:?}", i, term_to_insert);

        if i == 0 {
            if let Err(err) = add_term(CURRENT_DB_NAME, &term_to_insert) {
                error!("Failed to insert first term: {}", err);
                return Err(err);
            }

            term_set_id = get_term_set_id(
                CURRENT_DB_NAME,
                &term_to_insert.term.as_ref().unwrap(),
                &term_to_insert.language.as_ref().unwrap(),
            )?;

            if term_set_id.is_none() {
                error!(
                    "Failed to retrieve term set ID for term {:?}",
                    term_to_insert.term
                );
            }
        } else if let Some(id) = term_set_id {
            if let Err(err) = add_term_to_term_set(CURRENT_DB_NAME, id, &term_to_insert) {
                error!("Failed to add term to set: {}", err);
                return Err(err);
            }
        } else {
            error!(
                "Term set ID is None, cannot add term {:?}",
                term_to_insert.term
            );
        }
    }

    Ok(())
}
