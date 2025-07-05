use crate::error::OpfParseError;
use crate::models::{BookMetadataExtractor, OpfDoc};
use chrono::{DateTime, NaiveDate};
use std::io;

/// An adapter that wraps an `OpfDoc` to provide a clean, high-level
/// implementation of the `BookMetadataExtractor` trait.
pub struct OpfMetadataExtractor {
    doc: OpfDoc,
}

impl OpfMetadataExtractor {
    /// Creates a new adapter from a parsed `OpfDoc`.
    pub fn new(doc: OpfDoc) -> Self {
        Self { doc }
    }
}

impl BookMetadataExtractor for OpfMetadataExtractor {
    fn get_title(&self) -> String {
        self.doc
            .mdata("title")
            .unwrap_or_else(|| "Unknown Title".into())
    }

    fn get_author_name(&self) -> String {
        self.doc
            .mdata("creator")
            .unwrap_or_else(|| "Unknown Author".into())
    }

    fn get_language_code(&self) -> String {
        self.doc.mdata("language").unwrap_or_else(|| "und".into())
    }

    fn get_publisher_name(&self) -> Option<String> {
        self.doc.mdata("publisher")
    }

    fn get_publication_date(&self) -> Option<NaiveDate> {
        self.doc.mdata("date").and_then(|d| {
            DateTime::parse_from_rfc3339(&d)
                .ok()
                .map(|dt| dt.date_naive())
                .or_else(|| NaiveDate::parse_from_str(&d, "%Y-%m-%d").ok())
                .or_else(|| NaiveDate::parse_from_str(&d, "%Y-%m").ok())
                .or_else(|| NaiveDate::parse_from_str(&d, "%Y").ok())
        })
    }

    fn get_isbns(&self) -> Vec<String> {
        self.doc
            .identifiers
            .iter()
            .filter(|id| {
                id.scheme
                    .as_deref()
                    .map_or(false, |s| s.eq_ignore_ascii_case("ISBN"))
            })
            .map(|id| id.value.replace('-', "").trim().to_string())
            .collect()
    }

    fn get_subjects(&self) -> Vec<String> {
        self.doc
            .mdata_all("subject")
            .map(|subjects_vec| {
                subjects_vec
                    .iter()
                    .flat_map(|s| s.split(';'))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn get_description(&self) -> Option<String> {
        self.doc.mdata("description")
    }

    fn get_cover_image_data(&self) -> Result<Option<Vec<u8>>, OpfParseError> {
        let Some(cover_href) = self.doc.cover_href() else {
            return Ok(None);
        };
        let Some(base_path) = self.doc.file_path.parent() else {
            return Ok(None);
        };

        let decoded_href = urlencoding::decode(&cover_href)?;
        let cover_path = base_path.join(decoded_href.as_ref());

        match std::fs::read(cover_path) {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(OpfParseError::Io(e)),
        }
    }
}
