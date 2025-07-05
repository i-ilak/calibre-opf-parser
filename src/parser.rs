use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use quick_xml::events::{BytesStart, Event};
use quick_xml::reader::Reader;

use crate::error::OpfParseError;
use crate::models::{GuideReference, Identifier, OpfDoc};

impl OpfDoc {
    pub fn from_path(path: &Path) -> Result<Self, OpfParseError> {
        let file = File::open(path)?;
        let buf_reader = BufReader::new(file);
        let mut reader = Reader::from_reader(buf_reader);

        let mut metadata = HashMap::new();
        let mut unique_identifier_id = None;
        let mut identifiers = Vec::new();
        let mut guide = Vec::new();
        let mut buf = Vec::new();

        let mut in_metadata = false;
        let mut in_guide = false;

        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Start(ref e) => match e.name().as_ref() {
                    b"package" => {
                        if let Some(attr) = e.attributes().find(|a| {
                            a.as_ref()
                                .map_or(false, |a| a.key.as_ref() == b"unique-identifier")
                        }) {
                            unique_identifier_id = Some(String::from_utf8(attr?.value.to_vec())?);
                        }
                    }
                    b"metadata" => in_metadata = true,
                    b"guide" => in_guide = true,
                    _ if in_metadata => {
                        Self::parse_metadata_element(
                            &mut reader,
                            e,
                            &mut metadata,
                            &mut identifiers,
                        )?;
                    }
                    _ if in_guide => {
                        Self::parse_guide_element(e, &mut guide)?;
                    }
                    _ => {}
                },
                Event::Empty(ref e) => {
                    if in_metadata && e.name().as_ref() == b"meta" {
                        Self::parse_metadata_element(
                            &mut reader,
                            e,
                            &mut metadata,
                            &mut identifiers,
                        )?;
                    } else if in_guide {
                        Self::parse_guide_element(e, &mut guide)?;
                    }
                }
                Event::End(ref e) => match e.name().as_ref() {
                    b"metadata" => in_metadata = false,
                    b"guide" => in_guide = false,
                    _ => {}
                },
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(OpfDoc {
            metadata,
            unique_identifier_id,
            identifiers,
            guide,
            file_path: path.to_path_buf(),
        })
    }

    pub fn mdata(&self, key: &str) -> Option<String> {
        self.metadata
            .get(key.to_lowercase().as_str())
            .and_then(|values| values.first().cloned())
    }

    pub fn mdata_all(&self, key: &str) -> Option<&Vec<String>> {
        self.metadata.get(key.to_lowercase().as_str())
    }

    pub fn cover_href(&self) -> Option<String> {
        self.guide
            .iter()
            .find(|r| r.r#type.eq_ignore_ascii_case("cover"))
            .map(|r| r.href.clone())
    }

    fn parse_metadata_element(
        reader: &mut Reader<BufReader<File>>,
        start_element: &BytesStart,
        metadata: &mut HashMap<String, Vec<String>>,
        identifiers: &mut Vec<Identifier>,
    ) -> Result<(), OpfParseError> {
        match start_element.name().as_ref() {
            name if name.starts_with(b"dc:") => {
                let key = String::from_utf8(name[3..].to_vec())?.to_lowercase();
                let text_content = if !start_element.is_empty() {
                    Self::read_text_content(reader)?
                } else {
                    String::new()
                };

                if key == "identifier" {
                    let mut identifier = Identifier {
                        value: text_content,
                        scheme: None,
                    };
                    for attr in start_element.attributes() {
                        let attr = attr?;
                        if attr.key.as_ref() == b"opf:scheme" || attr.key.as_ref() == b"scheme" {
                            identifier.scheme = Some(String::from_utf8(attr.value.to_vec())?);
                        }
                    }
                    identifiers.push(identifier);
                } else {
                    metadata.entry(key).or_default().push(text_content);
                }
            }
            b"meta" => {
                let mut name_attr = None;
                let mut content_attr = None;
                for attr in start_element.attributes() {
                    let attr = attr?;
                    match attr.key.as_ref() {
                        b"name" => name_attr = Some(String::from_utf8(attr.value.to_vec())?),
                        b"content" => content_attr = Some(String::from_utf8(attr.value.to_vec())?),
                        _ => {}
                    }
                }
                if let (Some(name), Some(content)) = (name_attr, content_attr) {
                    metadata
                        .entry(name.to_lowercase())
                        .or_default()
                        .push(content);
                }
            }
            _ => {
                if !start_element.is_empty() {
                    Self::skip_element(reader)?;
                }
            }
        }
        Ok(())
    }

    fn parse_guide_element(
        start_element: &BytesStart,
        guide: &mut Vec<GuideReference>,
    ) -> Result<(), OpfParseError> {
        if start_element.name().as_ref() == b"reference" {
            let mut guide_ref = GuideReference::default();
            for attr in start_element.attributes() {
                let attr = attr?;
                match attr.key.as_ref() {
                    b"type" => guide_ref.r#type = String::from_utf8(attr.value.to_vec())?,
                    b"title" => guide_ref.title = Some(String::from_utf8(attr.value.to_vec())?),
                    b"href" => guide_ref.href = String::from_utf8(attr.value.to_vec())?,
                    _ => {}
                }
            }
            guide.push(guide_ref);
        }
        Ok(())
    }

    fn read_text_content(reader: &mut Reader<BufReader<File>>) -> Result<String, quick_xml::Error> {
        let mut buf = Vec::new();
        let mut text = String::new();
        loop {
            match reader.read_event_into(&mut buf)? {
                Event::Text(e) => text.push_str(&e.unescape()?),
                Event::End(_) => break,
                _ => {}
            }
            buf.clear();
        }
        Ok(text.trim().to_string())
    }

    fn skip_element(reader: &mut Reader<BufReader<File>>) -> Result<(), quick_xml::Error> {
        let mut buf = Vec::new();
        let mut depth = 1;
        while depth > 0 {
            match reader.read_event_into(&mut buf)? {
                Event::Start(_) => depth += 1,
                Event::End(_) => depth -= 1,
                Event::Eof => break,
                _ => {}
            }
            buf.clear();
        }
        Ok(())
    }
}
