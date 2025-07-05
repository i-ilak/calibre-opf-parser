//! A library for parsing Calibre OPF (`.opf`) files.
//!
//! This crate provides a high-level API to extract book metadata
//! from the XML-based OPF format used by Calibre and in EPUBs.
//!
//! # Example
//!
//! ```no_run
//! use std::path::Path;
//! use calibre_opf_parser::OpfMetadataExtractor;
//!
//! let path = Path::new("path/to/your/metadata.opf");
//! let metadata = OpfAdapter::new(path);
//!
//! println!("Title: {}", metadata.get_title());
//! println!("Author: {}", metadata.get_author_name());
//! if let Some(isbns) = metadata.get_isbns().first() {
//!     println!("ISBN: {}", isbns);
//! }
//! ```

mod adapter;
mod error;
mod models;
mod parser;

pub use adapter::OpfMetadataExtractor;
pub use error::OpfParseError;
pub use models::{BookMetadataExtractor, GuideReference, Identifier, OpfDoc};
