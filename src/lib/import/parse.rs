use chrono::format::ParseError;
use chrono::{DateTime, NaiveDateTime};
use elementtree::Element;
use serde::{self, Deserialize, Serialize};
use std::{fmt, fs, io};

#[derive(Debug, Serialize)]
pub struct Dictionary {
    pub entries: Vec<DictionaryEntry>,
}

#[derive(Debug, Serialize)]
pub struct DictionaryEntry {
    pub id: i32,
    pub language_sets: Vec<TermLanguageSet>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TermLanguageSet {
    pub language: Option<String>,
    pub term: Option<String>,
    pub term_type: Option<String>,
    pub creator_id: Option<String>,
    pub creation_timestamp: Option<i64>,
    pub updater_id: Option<String>,
    pub update_timestamp: Option<i64>,
    pub subject: Option<String>,
    pub source: Option<String>,
    pub user: Option<String>,
    pub attributes: Option<String>,
    pub remark: Option<String>,
    pub url: Option<String>,
    pub context: Option<String>,
    pub definition: Option<String>,
}

impl Default for TermLanguageSet {
    fn default() -> Self {
        TermLanguageSet {
            language: None,
            term: None,
            term_type: None,
            creator_id: None,
            creation_timestamp: None,
            updater_id: None,
            update_timestamp: None,
            subject: None,
            source: None,
            user: None,
            attributes: None,
            remark: None,
            url: None,
            context: None,
            definition: None,
        }
    }
}

impl fmt::Display for Dictionary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for entry in &self.entries {
            write!(f, "ID: {}\n", entry.id)?;
            write!(f, "Language Sets:\n")?;
            for lang_set in &entry.language_sets {
                write!(
                    f,
                    "\tLanguage: {}\n",
                    lang_set.language.as_deref().unwrap_or("")
                )?;
                write!(f, "\tTerm: {}\n", lang_set.term.as_deref().unwrap_or(""))?;
                write!(
                    f,
                    "\tTerm Type: {}\n",
                    lang_set.term_type.as_deref().unwrap_or("")
                )?;
                write!(
                    f,
                    "\tCreator ID: {}\n",
                    lang_set.creator_id.as_deref().unwrap_or("")
                )?;
                write!(
                    f,
                    "\tCreation Date: {}\n",
                    format_timestamp(lang_set.creation_timestamp)
                )?;
                write!(
                    f,
                    "\tUpdater ID: {}\n",
                    lang_set.updater_id.as_deref().unwrap_or("")
                )?;
                write!(
                    f,
                    "\tUpdate Date: {}\n",
                    format_timestamp(lang_set.update_timestamp)
                )?;
                write!(
                    f,
                    "\tSubject: {}\n",
                    lang_set.subject.as_deref().unwrap_or("")
                )?;
                write!(
                    f,
                    "\tSource: {}\n",
                    lang_set.source.as_deref().unwrap_or("")
                )?;
                write!(f, "\tUser: {}\n", lang_set.user.as_deref().unwrap_or(""))?;
                write!(
                    f,
                    "\tAttributes: {}\n",
                    lang_set.attributes.as_deref().unwrap_or("")
                )?;
                write!(
                    f,
                    "\tRemark: {}\n",
                    lang_set.remark.as_deref().unwrap_or("")
                )?;
                write!(f, "\tURL: {}\n", lang_set.url.as_deref().unwrap_or(""))?;
                write!(
                    f,
                    "\tContext: {}\n",
                    lang_set.context.as_deref().unwrap_or("")
                )?;
                write!(
                    f,
                    "\tDefinition: {}\n",
                    lang_set.definition.as_deref().unwrap_or("")
                )?;
                write!(f, "\n")?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

pub trait New {
    fn new() -> Self;
}

impl Dictionary {
    pub fn new() -> Dictionary {
        Dictionary {
            entries: Vec::new(),
        }
    }

    pub fn add_entry(&mut self, entry: DictionaryEntry) {
        self.entries.push(entry);
    }

    pub fn import_from_xml(&mut self, file_path: &str) {
        let root_element = load_xml(file_path);
        self.process_entries(root_element);
    }

    pub fn process_entries(&mut self, root: Element) {
        let terms = root
            .find("text")
            .unwrap()
            .find("body")
            .unwrap()
            .children()
            .into_iter();

        for term in terms {
            if term.tag().name() != "termEntry" {
                continue;
            }
            let entry_id: i32 = term.get_attr("id").unwrap().parse().unwrap();

            let mut entry = DictionaryEntry {
                id: entry_id,
                language_sets: Vec::new(),
            };

            let lang_sets = term.find_all("langSet");

            for lang_set in lang_sets {
                let mut lang_set_obj = TermLanguageSet::default();

                let language = lang_set.get_attr("lang").clone().unwrap().to_string();
                lang_set_obj.language = Some(language);

                let term_group = lang_set.find("ntig").unwrap().find("termGrp").unwrap();

                if let Some(term_elem) = term_group.find("term") {
                    lang_set_obj.term = Some(term_elem.text().to_string());
                }

                let term_notes = term_group.find_all("termNote");
                for term_note in term_notes {
                    let note_type = term_note.get_attr("type").unwrap();
                    match note_type {
                        "termType" => lang_set_obj.term_type = Some(term_note.text().to_string()),
                        "TS_CreateId" => {
                            lang_set_obj.creator_id = Some(term_note.text().to_string())
                        }
                        "TS_UpdateId" => {
                            lang_set_obj.updater_id = Some(term_note.text().to_string())
                        }
                        "TS_Subject" => lang_set_obj.subject = Some(term_note.text().to_string()),
                        "TS_Source" => lang_set_obj.source = Some(term_note.text().to_string()),
                        "TS_User1" => lang_set_obj.user = Some(term_note.text().to_string()),
                        "TS_Attributes" => {
                            lang_set_obj.attributes = Some(term_note.text().to_string())
                        }
                        "TS_Remark" => lang_set_obj.remark = Some(term_note.text().to_string()),
                        "TS_Hyperlink" => lang_set_obj.url = Some(term_note.text().to_string()),
                        _ => {}
                    }
                }

                let term_dates = term_group.find_all("date");
                for date in term_dates {
                    let date_type = date.get_attr("type").unwrap();
                    let date_str = date.text();
                    match date_type {
                        "origination" => {
                            lang_set_obj.creation_timestamp = parse_timestamp(date_str)
                        }
                        "modification" => lang_set_obj.update_timestamp = parse_timestamp(date_str),
                        _ => {}
                    }
                }

                let term_descriptions = term_group.find_all("descrip");
                for description in term_descriptions {
                    let description_type = description.get_attr("type").unwrap();
                    match description_type {
                        "context" => lang_set_obj.context = Some(description.text().to_string()),
                        "definition" => {
                            lang_set_obj.definition = Some(description.text().to_string())
                        }
                        _ => {}
                    }
                }

                entry.language_sets.push(lang_set_obj);
            }
            self.add_entry(entry);
        }
    }

    pub fn serialize_to_json(&self, file_path: &str) -> Result<(), io::Error> {
        let serialized = serde_json::to_string_pretty(&self)?;
        fs::write(file_path, serialized)?;
        Ok(())
    }
}

fn format_timestamp(timestamp: Option<i64>) -> String {
    match timestamp {
        Some(ts) => {
            let datetime = DateTime::from_timestamp(ts, 0);
            match datetime {
                Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
                None => "Invalid Date".to_string(),
            }
        }
        None => "".to_string(),
    }
}

pub fn load_xml(file_path: &str) -> Element {
    let xml_content = fs::read_to_string(file_path).unwrap();
    Element::from_reader(xml_content.as_bytes()).unwrap()
}

fn parse_timestamp(date_str: &str) -> Option<i64> {
    parse_timestamp_string(date_str)
        .ok()
        .map(|dt| dt.and_utc().timestamp())
}

pub fn parse_timestamp_string(date_str: &str) -> Result<NaiveDateTime, ParseError> {
    NaiveDateTime::parse_from_str(date_str, "%Y%m%dT%H%M%SZ")
}

pub fn timestamp_for_human_readable(date_str: &str) -> std::string::String {
    match parse_timestamp_string(&date_str) {
        Ok(datetime) => datetime.to_string(),
        Err(_error) => "error".to_string(),
    }
}
