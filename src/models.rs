use crate::error::OpfParseError;
use chrono::NaiveDate;
use std::path::PathBuf;

/// A trait for extracting common book metadata from a source.
/// This allows for multiple implementations (e.g., from OPF, EPUB, etc.).
pub trait BookMetadataExtractor {
    fn get_title(&self) -> String;
    fn get_author_name(&self) -> String;
    fn get_language_code(&self) -> String;
    fn get_publisher_name(&self) -> Option<String>;
    fn get_publication_date(&self) -> Option<NaiveDate>;
    fn get_isbns(&self) -> Vec<String>;
    fn get_subjects(&self) -> Vec<String>;
    fn get_description(&self) -> Option<String>;
    fn get_cover_image_data(&self) -> Result<Option<Vec<u8>>, OpfParseError>;
}

/// Represents a parsed OPF document's raw data.
pub struct OpfDoc {
    pub metadata: std::collections::HashMap<String, Vec<String>>,
    pub unique_identifier_id: Option<String>,
    pub identifiers: Vec<Identifier>,
    pub guide: Vec<GuideReference>,
    pub file_path: PathBuf,
}

/// Represents a `<dc:identifier>` element.
#[derive(Debug, Default, Clone)]
pub struct Identifier {
    pub scheme: Option<String>,
    pub value: String,
}

/// Represents a `<guide>` `<reference>` element.
#[derive(Debug, Default)]
pub struct GuideReference {
    pub r#type: String,
    pub title: Option<String>,
    pub href: String,
}
