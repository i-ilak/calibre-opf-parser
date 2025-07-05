use std::io;

use quick_xml::events::attributes::AttrError;
use thiserror::Error;

/// Defines all possible errors that can occur during OPF file parsing.
#[derive(Debug, Error)]
pub enum OpfParseError {
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("XML parsing error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("XML attribute error: {0}")]
    XmlAttr(#[from] AttrError),

    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    /// Corrected error type for URL decoding.
    #[error("URL decoding error (UTF-8): {0}")]
    UrlDecode(#[from] std::str::Utf8Error),

    #[error("File not found at path: {0}")]
    CoverNotFound(String),
}
